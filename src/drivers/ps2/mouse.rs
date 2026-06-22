use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;

use heapless::spsc::{Consumer, Producer, Queue};
use spin::Mutex;
use static_cell::StaticCell;

use super::controller;
use crate::info;

static BYTE_QUEUE: StaticCell<Queue<u8, 257>> = StaticCell::new();
static BYTE_PRODUCER: Mutex<Option<Producer<'static, u8>>> = Mutex::new(None);
static BYTE_CONSUMER: Mutex<Option<Consumer<'static, u8>>> = Mutex::new(None);

static WAKER: AtomicWaker = AtomicWaker::new();

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    EnableDataReporting = 0xF4,
    DisableDataReporting = 0xF5,
    SetDefaults = 0xF6,
    Reset = 0xFF,
}

impl Command {
    fn value(self) -> u8 {
        self as u8
    }
}

const ACK: u8 = 0xFA;
const RESEND: u8 = 0xFE;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    Resend,
    UnexpectedResponse(u8),
}

pub fn init() -> Result<(), CommandError> {
    let queue = BYTE_QUEUE.init(Queue::new());
    let (producer, consumer) = queue.split();

    if BYTE_PRODUCER.lock().replace(producer).is_some() {
        panic!("PS/2 mouse byte producer already initialized");
    }

    if BYTE_CONSUMER.lock().replace(consumer).is_some() {
        panic!("PS/2 mouse byte consumer already initialized");
    }

    controller::enable_second_port();

    send_command(Command::SetDefaults)?;
    send_command(Command::EnableDataReporting)?;

    Ok(())
}

pub fn send_command(command: Command) -> Result<(), CommandError> {
    use controller::Command::WriteToSecondPort;

    controller::write_command(WriteToSecondPort);
    controller::write_data(command.value());

    // TODO: Improve error handling
    match controller::read_data() {
        ACK => Ok(()),
        RESEND => Err(CommandError::Resend),
        response => Err(CommandError::UnexpectedResponse(response)),
    }
}

// TODO: Move outside of this driver
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub x: i32,
    pub y: i32,
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
}

impl State {
    const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            left_button: false,
            right_button: false,
            middle_button: false,
        }
    }

    fn apply_packet(&mut self, packet: Packet) {
        self.left_button = packet.left_button;
        self.right_button = packet.right_button;
        self.middle_button = packet.middle_button;

        // TODO: Improve this
        if !packet.x_overflow {
            self.x = self.x.saturating_add(packet.x_movement as i32);
        }

        if !packet.y_overflow {
            self.y = self.y.saturating_add(packet.y_movement as i32);
        }
    }
}

static STATE: Mutex<State> = Mutex::new(State::new());

pub fn state() -> State {
    *STATE.lock()
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PacketParseError {
    InvalidFirstByte(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Packet {
    pub left_button: bool,
    pub right_button: bool,
    pub middle_button: bool,
    pub x_movement: i16,
    pub y_movement: i16,
    pub x_overflow: bool,
    pub y_overflow: bool,
}

impl Packet {
    pub fn from_bytes(bytes: [u8; 3]) -> Result<Self, PacketParseError> {
        const LEFT_BUTTON: u8 = 1 << 0;
        const RIGHT_BUTTON: u8 = 1 << 1;
        const MIDDLE_BUTTON: u8 = 1 << 2;
        const ALWAYS_ONE: u8 = 1 << 3;
        const X_SIGN: u8 = 1 << 4;
        const Y_SIGN: u8 = 1 << 5;
        const X_OVERFLOW: u8 = 1 << 6;
        const Y_OVERFLOW: u8 = 1 << 7;

        let flags = bytes[0];

        if flags & ALWAYS_ONE == 0 {
            return Err(PacketParseError::InvalidFirstByte(flags));
        }

        let mut x_movement = bytes[1] as i16;
        let mut y_movement = bytes[2] as i16;

        if flags & X_SIGN != 0 {
            x_movement -= 256;
        }

        if flags & Y_SIGN != 0 {
            y_movement -= 256;
        }

        Ok(Packet {
            left_button: flags & LEFT_BUTTON != 0,
            right_button: flags & RIGHT_BUTTON != 0,
            middle_button: flags & MIDDLE_BUTTON != 0,
            x_movement,
            y_movement,
            x_overflow: flags & X_OVERFLOW != 0,
            y_overflow: flags & Y_OVERFLOW != 0,
        })
    }
}

pub async fn task() {
    const FIRST_BYTE_MARKER: u8 = 1 << 3;

    let mut bytes = ByteStream::new();
    let mut packet_bytes = [0; 3];
    let mut packet_index = 0;

    info!("PS/2 Mouse task started");

    while let Some(byte) = bytes.next().await {
        // TODO: Improve this
        if packet_index == 0 && byte & FIRST_BYTE_MARKER == 0 {
            continue;
        }

        packet_bytes[packet_index] = byte;
        packet_index += 1;

        if packet_index == 3 {
            let packet =
                Packet::from_bytes(packet_bytes).expect("invalid PS/2 mouse packet first byte");

            STATE.lock().apply_packet(packet);

            packet_index = 0;
        }
    }
}

struct ByteStream {
    _private: (),
}

impl ByteStream {
    fn new() -> Self {
        ByteStream { _private: () }
    }
}

impl Stream for ByteStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(byte) = pop_byte() {
            return Poll::Ready(Some(byte));
        }

        WAKER.register(cx.waker());

        match pop_byte() {
            Some(byte) => {
                WAKER.take();
                Poll::Ready(Some(byte))
            }
            None => Poll::Pending,
        }
    }
}

pub(crate) fn push_byte(byte: u8) {
    let should_wake = {
        let mut producer = BYTE_PRODUCER.lock();

        let Some(producer) = producer.as_mut() else {
            panic!("PS/2 mouse byte producer uninitialized");
        };

        if producer.enqueue(byte).is_err() {
            // TODO: This should not panic
            panic!("PS/2 mouse byte queue full, dropped byte: {:#04x}", byte);
        }

        true
    };

    if should_wake {
        WAKER.wake();
    }
}

fn pop_byte() -> Option<u8> {
    let mut consumer = BYTE_CONSUMER.lock();

    consumer
        .as_mut()
        .expect("PS/2 mouse byte consumer uninitialized")
        .dequeue()
}
