use crate::math::Vec2;

use super::consts::GRAVITY_ACC;

pub const fn weight(mass: f32) -> Vec2 {
    Vec2::new(0.0, mass * GRAVITY_ACC)
}

// TODO: Replace k with real world constants?
// TODO: Handle drag > vel?
pub fn drag(vel: Vec2, k: f32) -> Vec2 {
    let dir = vel.normalized().negated();
    let mag = k * vel.mag_sq();
    Vec2::from_dir_mag(dir, mag)
}

// TODO: Replace k with real world constants, especially normal?
pub fn friction(vel: Vec2, k: f32) -> Vec2 {
    let dir = vel.normalized().negated();
    Vec2::from_dir_mag(dir, k)
}

// obja.apply_force(grav)
// objb.apply_force(-grav)
pub fn grav(pos_a: Vec2, pos_b: Vec2, mass_a: f32, mass_b: f32, g: f32) -> Vec2 {
    let d = pos_b.sub(pos_a);

    let dir = d.normalized();
    let mag = g * (mass_a * mass_b) / d.mag_sq();
    Vec2::from_dir_mag(dir, mag)
}

// Chain of particles:
//  f = spring(curr, prev)
//  curr.apply_force(f)
//  prev.apply_force(-f)
// Box:
//  a b
//  d c
//  f = spring(a, b); a.apply_force(f); b.apply_force(-f)
//  f = spring(b, c); b.apply_force(f); c.apply_force(-f)
//  f = spring(c, d); c.apply_force(f); d.apply_force(-f)
//  f = spring(d, a); d.apply_force(f); a.apply_force(-f)
//  f = spring(a, c); a.apply_force(f); c.apply_force(-f)
//  f = spring(b, d); b.apply_force(f); d.apply_force(-f)
pub fn spring(pos: Vec2, anchor_pos: Vec2, rest_len: f32, k: f32) -> Vec2 {
    let d = pos.sub(anchor_pos);
    let disp = d.mag() - rest_len;

    let dir = d.normalized();
    let mag = -k * disp;
    Vec2::from_dir_mag(dir, mag)
}
