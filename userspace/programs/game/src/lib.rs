#![no_std]

#[repr(C)]
#[derive(Default)]
pub struct State {}

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

pub fn update_and_render(state: &mut State, buffer: Framebuffer) {
    let f1 = buffer.size() / 3;
    let f2 = buffer.size() * 2 / 3;
    let f3 = buffer.size();

    for i in 0..f1 {
        let i = i * 3;
        buffer.data[i + 0] = 255;
        buffer.data[i + 1] = 0;
        buffer.data[i + 2] = 0;
    }

    for i in f1..f2 {
        let i = i * 3;
        buffer.data[i + 0] = 0;
        buffer.data[i + 1] = 255;
        buffer.data[i + 2] = 0;
    }

    for i in f2..f3 {
        let i = i * 3;
        buffer.data[i + 0] = 0;
        buffer.data[i + 1] = 0;
        buffer.data[i + 2] = 255;
    }
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
