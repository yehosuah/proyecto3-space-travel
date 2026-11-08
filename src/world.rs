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
pub fn star(time: f64) -> BodyState {
    BodyState {
        center: Vec3::ZERO,
        radius: 42.0 * SCALE,
        rotation: Quat::from_rotation_y(time as f32 * 0.04),
    }
}
pub fn moon(time: f64) -> BodyState {
    let p = planet(0, time);
    let a = time as f32 * 0.035 + 1.4;
    BodyState {
        center: p.center + Vec3::new(a.cos() * 61.0, a.sin() * 19.0, a.sin() * 61.0) * SCALE,
        radius: 5.5 * SCALE,
        rotation: Quat::from_rotation_y(time as f32 * 0.09),
    }
}

pub fn closest(position: Vec3, time: f64) -> (usize, f32) {
    (0..2)
        .map(|i| {
            (
                i,
                position.distance(planet(i, time).center) - planet(i, time).radius,
            )
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .unwrap()
}

pub fn collision_bodies(time: f64) -> [BodyState; 4] {
    let mut monaco = planet(0, time);
    monaco.radius *= 1.075;
    [star(time), monaco, planet(1, time), moon(time)]
}
