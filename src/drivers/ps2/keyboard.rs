use core::pin::Pin;
use core::task::{Context, Poll};
use futures_util::Stream;
use futures_util::stream::StreamExt;
use futures_util::task::AtomicWaker;

use heapless::spsc::{Consumer, Producer, Queue};
use pc_keyboard::{DecodedKey, HandleControl, Keyboard, ScancodeSet1, layouts};
use spin::Mutex;
use static_cell::StaticCell;

use crate::info;

static SCANCODE_QUEUE: StaticCell<Queue<u8, 101>> = StaticCell::new();
static SCANCODE_PRODUCER: Mutex<Option<Producer<'static, u8>>> = Mutex::new(None);
static SCANCODE_CONSUMER: Mutex<Option<Consumer<'static, u8>>> = Mutex::new(None);

static WAKER: AtomicWaker = AtomicWaker::new();

pub fn init() {
    let queue = SCANCODE_QUEUE.init(Queue::new());
    let (producer, consumer) = queue.split();

    if SCANCODE_PRODUCER.lock().replace(producer).is_some() {
        panic!("PS/2 keyboard scancode producer already initialized");
    }

    if SCANCODE_CONSUMER.lock().replace(consumer).is_some() {
        panic!("PS/2 keyboard scancode consumer already initialized");
    }
}

pub async fn task() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    info!("PS/2 Keyboard task started");

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode)
            && let Some(key) = keyboard.process_keyevent(key_event)
        {
            match key {
                DecodedKey::Unicode(_c) => {}
                DecodedKey::RawKey(_key) => {}
            }
        }
    }
}

struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    fn new() -> Self {
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if let Some(scancode) = pop_scancode() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());

        match pop_scancode() {
            Some(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            None => Poll::Pending,
        }
    }
}

pub(crate) fn push_scancode(scancode: u8) {
    let should_wake = {
        let mut producer = SCANCODE_PRODUCER.lock();

        let Some(producer) = producer.as_mut() else {
            panic!("PS/2 keyboard scancode producer uninitialized");
        };

        if producer.enqueue(scancode).is_err() {
            // TODO: This should not panic
            panic!(
                "PS/2 keyboard scancode queue full, dropped scancode: {:#04x}",
                scancode
            );
        }

        true
    };

    if should_wake {
        WAKER.wake();
    }
}

fn pop_scancode() -> Option<u8> {
    let mut consumer = SCANCODE_CONSUMER.lock();

    consumer
        .as_mut()
        .expect("PS/2 keyboard scancode consumer uninitialized")
        .dequeue()
}
