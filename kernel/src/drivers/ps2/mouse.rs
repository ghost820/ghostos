use spin::Mutex;

use super::controller;
use crate::interrupts;
use ghostos_syscall::input::{MouseButton, MouseState};

static MOUSE: Mutex<Mouse> = Mutex::new(Mouse::new());

struct Mouse {
    packet_bytes: [u8; 3],
    packet_index: usize,
    state: MouseState,
}

impl Mouse {
    const fn new() -> Self {
        Self {
            packet_bytes: [0; 3],
            packet_index: 0,
            state: MouseState::new(),
        }
    }

    fn handle_byte(&mut self, byte: u8) {
        const FIRST_BYTE_MARKER: u8 = 1 << 3;

        // TODO: Improve this
        if self.packet_index == 0 && byte & FIRST_BYTE_MARKER == 0 {
            return;
        }

        self.packet_bytes[self.packet_index] = byte;
        self.packet_index += 1;

        if self.packet_index != self.packet_bytes.len() {
            return;
        }

        self.packet_index = 0;

        let packet = Packet::from_bytes(self.packet_bytes).unwrap();

        // TODO: Improve this
        if !packet.x_overflow {
            let x = self.state.x().saturating_add_signed(packet.x_movement);
            self.state.set_position(x, self.state.y());
        }

        if !packet.y_overflow {
            let y = self.state.y().saturating_add_signed(-packet.y_movement);
            self.state.set_position(self.state.x(), y);
        }

        self.state.set_button(MouseButton::Left, packet.left_button);
        self.state
            .set_button(MouseButton::Right, packet.right_button);
    }
}

pub(crate) fn handle_byte(byte: u8) {
    MOUSE.lock().handle_byte(byte);
}

pub fn snapshot() -> MouseState {
    interrupts::without_interrupts(|| MOUSE.lock().state)
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
    controller::disable_first_port();

    controller::drain_output_buffer();

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
