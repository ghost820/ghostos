use core::f32::consts::{PI, TAU};

use libm::cosf as cos;
use libm::sinf as sin;
use libm::sqrtf as sqrt;

#[repr(transparent)]
#[derive(Debug, Default, Clone, Copy, PartialEq, PartialOrd)]
pub struct Rad(f32);

impl Rad {
    pub const fn new(value: f32) -> Self {
        debug_assert!(value >= 0.0 && value < TAU);

        Self(value)
    }

    pub const fn raw(self) -> f32 {
        self.0
    }
}

#[repr(C)]
#[derive(Debug, Default, Clone, Copy)]
pub struct Vec2 {
    x: f32,
    y: f32,
}

impl Vec2 {
    pub const fn new(x: f32, y: f32) -> Self {
        debug_assert!(x.is_finite());
        debug_assert!(y.is_finite());

        Self { x, y }
    }

    pub const fn x(self) -> f32 {
        self.x
    }

    pub const fn y(self) -> f32 {
        self.y
    }

    pub const fn approx_eq(self, other: Vec2) -> bool {
        const EPSILON: f32 = 1e-5;

        (self.x - other.x).abs() <= EPSILON && (self.y - other.y).abs() <= EPSILON
    }

    pub const fn approx_neq(self, other: Vec2) -> bool {
        !self.approx_eq(other)
    }

    pub fn mag(self) -> f32 {
        let result = sqrt(self.x * self.x + self.y * self.y);

        debug_assert!(result.is_finite());

        result
    }

    pub const fn mag_sq(self) -> f32 {
        let result = self.x * self.x + self.y * self.y;

        debug_assert!(result.is_finite());

        result
    }

    pub fn normal(self) -> Vec2 {
        Vec2::new(self.y, -self.x).normalized()
    }

    pub const fn dot(self, other: Vec2) -> f32 {
        let result = self.x * other.x + self.y * other.y;

        debug_assert!(result.is_finite());

        result
    }

    pub const fn cross(self, other: Vec2) -> f32 {
        let result = self.x * other.y - self.y * other.x;

        debug_assert!(result.is_finite());

        result
    }

    pub fn normalize(&mut self) {
        let mag = self.mag();

        debug_assert!(mag > 0.0);

        self.x /= mag;
        self.y /= mag;
    }

    pub fn normalized(self) -> Self {
        let mut result = self;

        result.normalize();

        result
    }

    pub const fn addi(&mut self, other: Vec2) {
        self.x += other.x;
        self.y += other.y;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub const fn addi_xy(&mut self, x: f32, y: f32) {
        self.x += x;
        self.y += y;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub const fn addi_x(&mut self, x: f32) {
        self.x += x;

        debug_assert!(self.x.is_finite());
    }

    pub const fn addi_y(&mut self, y: f32) {
        self.y += y;

        debug_assert!(self.y.is_finite());
    }

    pub const fn add(self, other: Vec2) -> Self {
        let mut result = self;

        result.addi(other);

        result
    }

    pub const fn add_xy(self, x: f32, y: f32) -> Self {
        let mut result = self;

        result.addi_xy(x, y);

        result
    }

    pub const fn add_x(self, x: f32) -> Self {
        let mut result = self;

        result.addi_x(x);

        result
    }

    pub const fn add_y(self, y: f32) -> Self {
        let mut result = self;

        result.addi_y(y);

        result
    }

    pub const fn subi(&mut self, other: Vec2) {
        self.x -= other.x;
        self.y -= other.y;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub const fn subi_xy(&mut self, x: f32, y: f32) {
        self.x -= x;
        self.y -= y;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub const fn subi_x(&mut self, x: f32) {
        self.x -= x;

        debug_assert!(self.x.is_finite());
    }

    pub const fn subi_y(&mut self, y: f32) {
        self.y -= y;

        debug_assert!(self.y.is_finite());
    }

    pub const fn sub(self, other: Vec2) -> Self {
        let mut result = self;

        result.subi(other);

        result
    }

    pub const fn sub_xy(self, x: f32, y: f32) -> Self {
        let mut result = self;

        result.subi_xy(x, y);

        result
    }

    pub const fn sub_x(self, x: f32) -> Self {
        let mut result = self;

        result.subi_x(x);

        result
    }

    pub const fn sub_y(self, y: f32) -> Self {
        let mut result = self;

        result.subi_y(y);

        result
    }

    pub const fn negate(&mut self) {
        self.scale(-1.0);
    }

    pub const fn negated(self) -> Self {
        self.scaled(-1.0)
    }

    pub const fn scale(&mut self, value: f32) {
        self.x *= value;
        self.y *= value;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub const fn scaled(self, value: f32) -> Self {
        let mut result = self;

        result.scale(value);

        result
    }

    pub fn rotate(&mut self, value: Rad) {
        let x = self.x;
        let y = self.y;
        let angle = value.raw();
        let sinv = sin(angle);
        let cosv = cos(angle);

        self.x = x * cosv - y * sinv;
        self.y = x * sinv + y * cosv;

        debug_assert!(self.x.is_finite() && self.y.is_finite());
    }

    pub fn rotated(self, value: Rad) -> Self {
        let mut result = self;

        result.rotate(value);

        result
    }
}
