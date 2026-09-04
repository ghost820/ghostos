use crate::PIXELS_PER_METER;
use crate::math::Vec2;

#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct Particle {
    pos: Vec2,
    vel: Vec2,
    acc: Vec2,
    mass: f32,
    mass_inv: f32,
    force: Vec2,
}

impl Particle {
    pub const fn new(pos: Vec2, mass: f32) -> Self {
        debug_assert!(mass.is_normal());

        let mass_inv = 1.0 / mass;

        Self {
            pos,
            vel: Vec2::ZERO,
            acc: Vec2::ZERO,
            mass,
            mass_inv,
            force: Vec2::ZERO,
        }
    }

    pub const fn from_raw_coords(x: f32, y: f32, mass: f32) -> Self {
        debug_assert!(mass.is_normal());

        let mass_inv = 1.0 / mass;

        Self {
            pos: Vec2::new(x, y),
            vel: Vec2::ZERO,
            acc: Vec2::ZERO,
            mass,
            mass_inv,
            force: Vec2::ZERO,
        }
    }

    pub const fn mass(&self) -> f32 {
        self.mass
    }

    pub const fn set_pos(&mut self, pos: Vec2) {
        self.pos = pos;
    }

    pub const fn set_pos_xy(&mut self, x: f32, y: f32) {
        self.pos = Vec2::new(x, y);
    }

    pub const fn apply_force(&mut self, force: Vec2) {
        self.force.addi(force.scaled(PIXELS_PER_METER as f32));
    }

    pub const fn update(&mut self, dt: f32) {
        self.acc = self.force.scaled(self.mass_inv);

        self.vel.addi(self.acc.scaled(dt));

        self.pos.addi(self.vel.scaled(dt));

        self.force = Vec2::ZERO;
    }

    pub fn draw(&self, framebuffer: &mut crate::Framebuffer, c: crate::Rgba) {
        use crate::draw::draw_rect;

        draw_rect(
            framebuffer,
            self.pos.x() as usize,
            self.pos.y() as usize,
            3,
            3,
            c,
        );
    }
}
