#![no_std]

pub mod color;
pub mod draw;
pub mod math;
pub mod particle;
pub mod physics;

use crate::color::Rgba;
use crate::draw::draw_rect;

pub const PIXELS_PER_METER: usize = 50;

#[repr(C)]
#[derive(Default)]
pub struct State {
    initialized: bool,
}

pub struct Framebuffer<'a> {
    width: usize,
    height: usize,
    bpp: usize,
    size: usize,
    byte_len: usize,
    data: &'a mut [u8],
}

impl<'a> Framebuffer<'a> {
    pub fn new(width: usize, height: usize, bpp: usize, data: &'a mut [u8]) -> Self {
        Self {
            width,
            height,
            bpp,
            size: width * height,
            byte_len: width * height * bpp,
            data,
        }
    }

    fn width(&self) -> usize {
        self.width
    }

    fn height(&self) -> usize {
        self.height
    }

    fn bpp(&self) -> usize {
        self.bpp
    }

    fn size(&self) -> usize {
        self.size
    }

    fn byte_len(&self) -> usize {
        self.byte_len
    }
}

pub fn update_and_render(state: &mut State, mut buffer: Framebuffer, dt: f32) {
    if !state.initialized {
        // ...

        state.initialized = true;
    }

    let bw = buffer.width();
    let bh = buffer.height();

    draw_rect(&mut buffer, 0, 0, bw, bh, Rgba::BLACK);
}

//
// TODO: Temporary debug
//

use core::fmt::{self, Write};

struct DebugWriter;

impl Write for DebugWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            ghostos_syscall::debug_log_byte(byte);
        }

        Ok(())
    }
}

pub fn _print(args: fmt::Arguments) {
    DebugWriter.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => {
        $crate::_print(format_args!($($arg)*))
    };
}

#[macro_export]
macro_rules! println {
    () => {
        $crate::print!("\n")
    };
    ($($arg:tt)*) => {
        $crate::print!("{}\n", format_args!($($arg)*))
    };
}
