use core::fmt;

use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

use crate::io::{self, PortAddress, ReadWrite};
use crate::libs::graphics::colors::Rgb;
use crate::threading::with_lock_no_interrupts;

#[rustfmt::skip]
pub const MODE_320X200X256: [u8; 61] = [
    // Miscellaneous Output Register
    0x63,

    // Sequencer: 0x00..0x04
    0x03, 0x01, 0x0F, 0x00, 0x0E,

    // CRT Controller: 0x00..0x18
    0x5F, 0x4F, 0x50, 0x82, 0x54,
    0x80, 0xBF, 0x1F, 0x00, 0x41,
    0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x9C, 0x0E, 0x8F, 0x28,
    0x40, 0x96, 0xB9, 0xA3, 0xFF,

    // Graphics Controller: 0x00..0x08
    0x00, 0x00, 0x00, 0x00, 0x00,
    0x40, 0x05, 0x0F, 0xFF,

    // Attribute Controller: 0x00..0x14
    0x00, 0x01, 0x02, 0x03, 0x04,
    0x05, 0x06, 0x07, 0x08, 0x09,
    0x0A, 0x0B, 0x0C, 0x0D, 0x0E,
    0x0F, 0x41, 0x00, 0x0F, 0x00,
    0x00,
];

const BUFFER_WIDTH: usize = 80;
const BUFFER_HEIGHT: usize = 25;

type Buffer = [[Volatile<u16>; BUFFER_WIDTH]; BUFFER_HEIGHT];

const PORT_03C0: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C0) };
const PORT_03C2: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C2) };
const PORT_03C4: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C4) };
const PORT_03C5: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C5) };
const PORT_03C8: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C8) };
const PORT_03C9: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3C9) };
const PORT_03CE: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3CE) };
const PORT_03CF: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3CF) };
const PORT_03D4: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3D4) };
const PORT_03D5: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3D5) };
const PORT_03DA: PortAddress<u8, ReadWrite> = unsafe { PortAddress::new(0x3DA) };

lazy_static! {
    pub static ref VGA_TEXT_BUFFER: Mutex<VGATextBuffer> = Mutex::new(VGATextBuffer {
        col_pos: 0,
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

static LOCK: Mutex<()> = Mutex::new(());

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

        #[cfg(debug_assertions)]
        panic!("warning log triggered");
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
    ($($arg:tt)*) => ($crate::vga::_print(format_args!($($arg)*)));
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GraphicsMode {
    width: usize,
    height: usize,
    stride_bytes: usize,
    framebuffer_address: usize,
    pixel_format: PixelFormat,
}

impl GraphicsMode {
    pub const fn new(
        width: usize,
        height: usize,
        stride_bytes: usize,
        framebuffer_address: usize,
        pixel_format: PixelFormat,
    ) -> Self {
        Self {
            width,
            height,
            stride_bytes,
            framebuffer_address,
            pixel_format,
        }
    }

    pub fn set_pixel(self, x: usize, y: usize, color: Rgb) {
        if x >= self.width || y >= self.height {
            return;
        }

        let Some(row_offset) = y.checked_mul(self.stride_bytes) else {
            return;
        };

        let Some(pixel_offset) = x.checked_mul(self.pixel_format.bytes_per_pixel()) else {
            return;
        };

        let Some(offset) = row_offset.checked_add(pixel_offset) else {
            return;
        };

        let Some(address) = self.framebuffer_address.checked_add(offset) else {
            return;
        };

        self.pixel_format.write_pixel(address, color);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PixelFormat {
    IndexedRgb666 { palette_start_index: u8 },
}

impl PixelFormat {
    pub const fn bytes_per_pixel(self) -> usize {
        match self {
            Self::IndexedRgb666 { .. } => 1,
        }
    }

    fn write_pixel(self, address: usize, color: Rgb) {
        match self {
            Self::IndexedRgb666 {
                palette_start_index,
            } => {
                let red = ((color.red as u16 * 5 + 127) / 255) as u8;
                let green = ((color.green as u16 * 5 + 127) / 255) as u8;
                let blue = ((color.blue as u16 * 5 + 127) / 255) as u8;
                let color_index = palette_start_index + red * 36 + green * 6 + blue;
                unsafe {
                    (address as *mut u8).write_volatile(color_index);
                }
            }
        }
    }
}

pub fn set_mode_320x200x256() -> GraphicsMode {
    const PALETTE_START_INDEX: u8 = 16;

    with_lock_no_interrupts(&LOCK, || {
        write_mode(&MODE_320X200X256);

        io::write(PORT_03C8, PALETTE_START_INDEX);
        for red in 0..6 {
            for green in 0..6 {
                for blue in 0..6 {
                    io::write(PORT_03C9, ((red as u16) * 63 / 5) as u8);
                    io::write(PORT_03C9, ((green as u16) * 63 / 5) as u8);
                    io::write(PORT_03C9, ((blue as u16) * 63 / 5) as u8);
                }
            }
        }
    });

    GraphicsMode::new(
        320,
        200,
        320,
        0xA0000,
        PixelFormat::IndexedRgb666 {
            palette_start_index: PALETTE_START_INDEX,
        },
    )
}

fn write_mode(registers: &[u8; 61]) {
    io::write(PORT_03C2, registers[0]);
    write_indexed_registers(PORT_03C4, PORT_03C5, &registers[1..6]);
    unlock_crt_controller_registers();
    write_indexed_registers(PORT_03D4, PORT_03D5, &registers[6..31]);
    write_indexed_registers(PORT_03CE, PORT_03CF, &registers[31..40]);
    write_attribute_controller_registers(&registers[40..61]);
    enable_attribute_controller_display();
}

fn write_indexed_registers(
    address_port: PortAddress<u8, ReadWrite>,
    data_port: PortAddress<u8, ReadWrite>,
    values: &[u8],
) {
    for (index, value) in values.iter().copied().enumerate() {
        io::write(address_port, index as u8);
        io::write(data_port, value);
    }
}

fn write_attribute_controller_registers(values: &[u8]) {
    for (index, value) in values.iter().copied().enumerate() {
        reset_attribute_controller_flip_flop();
        io::write(PORT_03C0, index as u8);
        io::write(PORT_03C0, value);
    }
}

fn unlock_crt_controller_registers() {
    io::write(PORT_03D4, 0x03);
    io::write(PORT_03D5, io::read(PORT_03D5) | 0x80);

    io::write(PORT_03D4, 0x11);
    io::write(PORT_03D5, io::read(PORT_03D5) & !0x80u8);
}

fn enable_attribute_controller_display() {
    reset_attribute_controller_flip_flop();
    io::write(PORT_03C0, 0x20);
}

fn reset_attribute_controller_flip_flop() {
    let _ = io::read(PORT_03DA);
}
