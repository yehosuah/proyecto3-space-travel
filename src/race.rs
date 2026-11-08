//! Contrarreloj secuencial; se cruza el plano de cada puerta en el sentido correcto.
use crate::world;
use glam::Vec3;
use std::f32::consts::TAU;

pub const GATE_COUNT: usize = 24;
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub center: Vec3,
    pub normal: Vec3,
    pub radius: f32,
}
fn route(t: f32) -> Vec3 {
    let a = 2.4 + t * TAU;
    let latitude = 0.13 * (a * 2.0).sin() + 0.055 * (a * 5.0).sin();
    let radius = world::planet(0, 0.0).radius + 68.0 + 18.0 * (a * 3.0).sin();
    Vec3::new(
        a.cos() * latitude.cos(),
        latitude.sin(),
        a.sin() * latitude.cos(),
    ) * radius
}
pub fn gate(index: usize) -> Gate {
    let t = index as f32 / GATE_COUNT as f32;
    Gate {
        center: route(t),
        normal: (route(t + 0.001) - route(t - 0.001)).normalize(),
        radius: if index < 5 {
            23.0
        } else if index < 14 {
            18.0
        } else {
            14.0
        },
    }
}
pub fn world_gate(index: usize, time: f64) -> Gate {
    let p = world::planet(0, time);
    let g = gate(index);
    Gate {
        center: p.center + p.rotation * g.center,
        normal: p.rotation * g.normal,
        radius: g.radius,
    }
}
pub fn crossing(a: Vec3, b: Vec3, g: Gate) -> Option<f32> {
    let da = (a - g.center).dot(g.normal);
    let db = (b - g.center).dot(g.normal);
    if da > 0.0 || db <= 0.0 || db - da < 1e-6 {
        return None;
    }
    let fraction = -da / (db - da);
    ((a.lerp(b, fraction) - g.center).length() <= g.radius).then_some(fraction)
}
