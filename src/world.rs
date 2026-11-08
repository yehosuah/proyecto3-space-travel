use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug)]
pub struct BodyState {
    pub center: Vec3,
    pub radius: f32,
    pub rotation: Quat,
}

pub const NAMES: [&str; 2] = ["MONACO", "REBUFO"];
pub const SCALE: f32 = 10.0;
pub const SURFACE_SEED: u32 = 2026;

pub fn planet(index: usize, time: f64) -> BodyState {
    let (orbit, radius, phase, speed, spin) = if index == 0 {
        (220.0, 30.0, 0.25, 0.006, 0.022)
    } else {
        (430.0, 48.0, -0.70, 0.0036, 0.028)
    };
    let a = phase + time as f32 * speed;
    let tilt = if index == 0 {
        Quat::from_rotation_z(-0.12)
    } else {
        ring_tilt()
    };
    BodyState {
        center: Vec3::new(
            orbit * a.cos(),
            if index == 0 { 0.0 } else { 22.0 * a.sin() },
            orbit * a.sin(),
        ) * SCALE,
        radius: radius * SCALE,
        rotation: tilt * Quat::from_rotation_y(time as f32 * spin),
    }
}

pub fn ring_tilt() -> Quat {
    Quat::from_rotation_z(-0.34) * Quat::from_rotation_x(0.53)
}
