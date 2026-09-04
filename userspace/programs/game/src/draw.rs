use crate::Framebuffer;
use crate::color::Rgba;

pub fn draw_rect(framebuffer: &mut Framebuffer, x: usize, y: usize, w: usize, h: usize, c: Rgba) {
    debug_assert!(x + w <= framebuffer.width() && y + h <= framebuffer.height());

    let bpp = framebuffer.bpp();
    let buffer_width = framebuffer.width() * bpp;
    let buffer = &mut framebuffer.data;
    let x = x * bpp;
    let w = w * bpp;

    for yi in y..y + h {
        for xi in (x..x + w).step_by(bpp) {
            buffer[yi * buffer_width + xi + 0] = c.b255();
            buffer[yi * buffer_width + xi + 1] = c.g255();
            buffer[yi * buffer_width + xi + 2] = c.r255();
        }
    }
}
