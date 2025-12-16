use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

type Buffer = [[Volatile<u16>; BUFFER_WIDTH]; BUFFER_HEIGHT];

lazy_static! {
    pub static ref VGA_TEXT_BUFFER: Mutex<VGATextBuffer> = Mutex::new(VGATextBuffer {
        col_pos: 0,
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_text_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    VGA_TEXT_BUFFER.lock().write_fmt(args).unwrap();
}

pub struct VGATextBuffer {
    col_pos: usize,
    buffer: &'static mut Buffer,
}

impl VGATextBuffer {
    pub fn write_str(&mut self, s: &str, fg: Color, bg: Color) {
        for byte in s.bytes() {
            self.write(byte, fg, bg)
        }
    }

    pub fn write(&mut self, ch: u8, fg: Color, bg: Color) {
        match ch {
            b'\n' => self.new_line(),
            ch => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.col_pos;
                let color = ((bg as u16) << 4) | (fg as u16);
                let entry = (color << 8) | (ch as u16);
                self.buffer[row][col].write(entry);

                self.col_pos += 1;
            }
        }
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.buffer[row][col].read();
                self.buffer[row - 1][col].write(ch);
            }
        }

        for col in 0..BUFFER_WIDTH {
            self.buffer[BUFFER_HEIGHT - 1][col].write(b' ' as u16);
        }

        self.col_pos = 0;
    }
}

impl fmt::Write for VGATextBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s, Color::White, Color::Black);
        Ok(())
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}
