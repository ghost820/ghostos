use libm::roundf as round;

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Rgba {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Rgba {
    pub const RED: Self = Self::new(1.0, 0.0, 0.0, 1.0);
    pub const GREEN: Self = Self::new(0.0, 1.0, 0.0, 1.0);
    pub const BLUE: Self = Self::new(0.0, 0.0, 1.0, 1.0);
    pub const WHITE: Self = Self::new(1.0, 1.0, 1.0, 1.0);
    pub const BLACK: Self = Self::new(0.0, 0.0, 0.0, 1.0);

    pub const fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        debug_assert!(r >= 0.0 && r <= 1.0);
        debug_assert!(g >= 0.0 && g <= 1.0);
        debug_assert!(b >= 0.0 && b <= 1.0);
        debug_assert!(a >= 0.0 && a <= 1.0);

        Self { r, g, b, a }
    }

    pub const fn r(self) -> f32 {
        self.r
    }

    pub const fn g(self) -> f32 {
        self.g
    }

    pub const fn b(self) -> f32 {
        self.b
    }

    pub const fn a(self) -> f32 {
        self.a
    }

    pub fn r255(self) -> u8 {
        round(self.r * 255.0) as u8
    }

    pub fn g255(self) -> u8 {
        round(self.g * 255.0) as u8
    }

    pub fn b255(self) -> u8 {
        round(self.b * 255.0) as u8
    }

    pub fn a255(self) -> u8 {
        round(self.a * 255.0) as u8
    }
}
