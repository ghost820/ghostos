pub use pc_keyboard::KeyCode;

pub const KEYBOARD_STATE_WORD_COUNT: usize = 2;

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InputState {
    keyboard: KeyboardState,
    mouse: MouseState,
}

impl InputState {
    pub const fn new(keyboard: KeyboardState, mouse: MouseState) -> Self {
        Self { keyboard, mouse }
    }

    pub const fn keyboard(self) -> KeyboardState {
        self.keyboard
    }

    pub const fn mouse(self) -> MouseState {
        self.mouse
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyboardState {
    words: [u64; KEYBOARD_STATE_WORD_COUNT],
}

impl KeyboardState {
    pub const fn new() -> Self {
        Self {
            words: [0; KEYBOARD_STATE_WORD_COUNT],
        }
    }

    pub const fn from_words(words: [u64; KEYBOARD_STATE_WORD_COUNT]) -> Self {
        Self { words }
    }

    pub const fn words(self) -> [u64; KEYBOARD_STATE_WORD_COUNT] {
        self.words
    }

    pub const fn is_down(self, key: KeyCode) -> bool {
        let index = key as usize;
        let word = index / 64;
        let bit = index % 64;

        self.words[word] & (1u64 << bit) != 0
    }

    pub const fn set_key(&mut self, key: KeyCode, down: bool) {
        let index = key as usize;
        let word = index / 64;
        let bit = index % 64;
        let mask = 1u64 << bit;

        if down {
            self.words[word] |= mask;
        } else {
            self.words[word] &= !mask;
        }
    }
}

impl Default for KeyboardState {
    fn default() -> Self {
        Self::new()
    }
}

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MouseButton {
    Left = 0,
    Right = 1,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MouseState {
    x: u16,
    y: u16,
    buttons: u8,
}

impl MouseState {
    pub const fn new() -> Self {
        Self {
            x: 0,
            y: 0,
            buttons: 0,
        }
    }

    pub const fn from_word(word: u64) -> Self {
        Self {
            x: word as u16,
            y: (word >> 16) as u16,
            buttons: (word >> 32) as u8,
        }
    }

    pub const fn word(self) -> u64 {
        self.x as u64 | ((self.y as u64) << 16) | ((self.buttons as u64) << 32)
    }

    pub const fn x(self) -> u16 {
        self.x
    }

    pub const fn y(self) -> u16 {
        self.y
    }

    pub const fn is_down(self, button: MouseButton) -> bool {
        self.buttons & (1 << button as u8) != 0
    }

    pub const fn set_position(&mut self, x: u16, y: u16) {
        self.x = x;
        self.y = y;
    }

    pub const fn set_button(&mut self, button: MouseButton, down: bool) {
        let mask = 1 << button as u8;

        if down {
            self.buttons |= mask;
        } else {
            self.buttons &= !mask;
        }
    }
}

impl Default for MouseState {
    fn default() -> Self {
        Self::new()
    }
}
