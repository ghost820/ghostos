use pc_keyboard::{KeyState, ScancodeSet, ScancodeSet1};
use spin::Mutex;

use super::controller;
use crate::interrupts;
use ghostos_syscall::input::KeyboardState;

static KEYBOARD: Mutex<Keyboard> = Mutex::new(Keyboard::new());

struct Keyboard {
    decoder: ScancodeSet1,
    state: KeyboardState,
}

impl Keyboard {
    const fn new() -> Self {
        Self {
            decoder: ScancodeSet1::new(),
            state: KeyboardState::new(),
        }
    }

    // TODO: Better error handling
    fn handle_scancode(&mut self, scancode: u8) {
        let event = self.decoder.advance_state(scancode);

        debug_assert!(
            event.is_ok(),
            "failed to decode keyboard scancode {:#04x}: {:?}",
            scancode,
            event
        );

        let Ok(Some(event)) = event else {
            return;
        };

        match event.state {
            KeyState::Down => self.state.set_key(event.code, true),
            KeyState::Up => self.state.set_key(event.code, false),
            KeyState::SingleShot => {}
        }
    }
}

pub(crate) fn handle_scancode(scancode: u8) {
    KEYBOARD.lock().handle_scancode(scancode);
}

pub fn init() {
    controller::enable_first_port();
}

pub fn snapshot() -> KeyboardState {
    interrupts::without_interrupts(|| KEYBOARD.lock().state)
}
