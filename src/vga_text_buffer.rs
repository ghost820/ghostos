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
macro_rules! debug {
    ($($arg:tt)+) => {{
        $crate::println!("[<darkgray>debug</>] {}", format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {{
        $crate::println!("[<lightcyan>info</>] {}", format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! warning {
    ($($arg:tt)+) => {{
        $crate::println!("[<yellow>warning</>] {}", format_args!($($arg)+));
    }};
}

#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {{
        $crate::println!("[<lightred>error</>] {}", format_args!($($arg)+));

        #[cfg(debug_assertions)]
        panic!("error log triggered");
    }};
}

#[macro_export]
macro_rules! critical {
    ($($arg:tt)+) => {{
        $crate::println!("[<red>critical</>] {}", format_args!($($arg)+));
        panic!("critical log triggered");
    }};
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

    let result = {
        let mut buffer = VGA_TEXT_BUFFER.lock();
        let mut writer = ColorWriter::new(&mut buffer);
        let result = writer.write_fmt(args);
        writer.flush();
        result
    };

    #[cfg(debug_assertions)]
    result.expect("failed to format text for VGA output");

    #[cfg(not(debug_assertions))]
    let _ = result;
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

struct ColorWriter<'a> {
    buffer: &'a mut VGATextBuffer,
    foreground: Color,
    tag_buffer: [u8; 16],
    tag_len: usize,
    in_tag: bool,
}

impl<'a> ColorWriter<'a> {
    fn new(buffer: &'a mut VGATextBuffer) -> Self {
        Self {
            buffer,
            foreground: Color::White,
            tag_buffer: [0; 16],
            tag_len: 0,
            in_tag: false,
        }
    }

    fn flush(&mut self) {
        if self.in_tag {
            self.write_byte(b'<');

            for i in 0..self.tag_len {
                let byte = self.tag_buffer[i];
                self.write_byte(byte);
            }

            self.in_tag = false;
            self.tag_len = 0;
        }
    }

    fn write_byte(&mut self, byte: u8) {
        self.buffer.write(byte, self.foreground, Color::Black);
    }

    fn write_rich_byte(&mut self, byte: u8) {
        if self.in_tag {
            match byte {
                b'>' => self.finish_tag(),
                byte => self.push_tag_byte(byte),
            }

            return;
        }

        match byte {
            b'<' => {
                self.in_tag = true;
                self.tag_len = 0;
            }
            byte => self.write_byte(byte),
        }
    }

    fn push_tag_byte(&mut self, byte: u8) {
        if self.tag_len < self.tag_buffer.len() {
            self.tag_buffer[self.tag_len] = byte;
            self.tag_len += 1;
            return;
        }

        self.write_tag_as_text(false);
        self.write_byte(byte);
    }

    fn finish_tag(&mut self) {
        if let Some(color) = color_from_tag(&self.tag_buffer[..self.tag_len]) {
            self.foreground = color;
            self.in_tag = false;
            self.tag_len = 0;
            return;
        }

        self.write_tag_as_text(true);
    }

    fn write_tag_as_text(&mut self, has_end: bool) {
        self.write_byte(b'<');

        for i in 0..self.tag_len {
            let byte = self.tag_buffer[i];
            self.write_byte(byte);
        }

        if has_end {
            self.write_byte(b'>');
        }

        self.in_tag = false;
        self.tag_len = 0;
    }
}

impl fmt::Write for ColorWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_rich_byte(byte);
        }

        Ok(())
    }
}

fn color_from_tag(tag: &[u8]) -> Option<Color> {
    match tag {
        b"/" => Some(Color::White),
        b"black" => Some(Color::Black),
        b"blue" => Some(Color::Blue),
        b"green" => Some(Color::Green),
        b"cyan" => Some(Color::Cyan),
        b"red" => Some(Color::Red),
        b"magenta" => Some(Color::Magenta),
        b"brown" => Some(Color::Brown),
        b"lightgray" => Some(Color::LightGray),
        b"darkgray" => Some(Color::DarkGray),
        b"lightblue" => Some(Color::LightBlue),
        b"lightgreen" => Some(Color::LightGreen),
        b"lightcyan" => Some(Color::LightCyan),
        b"lightred" => Some(Color::LightRed),
        b"pink" => Some(Color::Pink),
        b"yellow" => Some(Color::Yellow),
        b"white" => Some(Color::White),
        _ => None,
    }
}
