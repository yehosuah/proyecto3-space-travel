//! Modelos propios compuestos en CPU y distribuidos mediante el ruido del curso.
use crate::{
    mesh::surface_radius,
    noise::noise,
    world::{self, BodyState},
};
use glam::{Quat, Vec3};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Instance {
    pub direction: Vec3,
    pub heading: f32,
    pub scale: f32,
    pub kind: usize,
}
impl Instance {
    pub fn body(self, planet: BodyState) -> BodyState {
        BodyState {
            center: planet.center
                + planet.rotation
                    * self.direction
                    * (surface_radius(self.direction) * planet.radius - 0.12),
            rotation: planet.rotation
                * Quat::from_rotation_arc(Vec3::Y, self.direction)
                * Quat::from_rotation_y(self.heading),
            radius: self.scale,
        }
    }
}

pub fn distribute(seed: u32) -> Vec<Instance> {
    let offset = Vec3::new(
        (seed % 997) as f32,
        (seed % 479) as f32,
        (seed % 233) as f32,
    ) * 0.137;
    let mut instances = Vec::new();
    for i in 0..720 {
        let y = 1.0 - 2.0 * (i as f32 + 0.5) / 720.0;
        let angle = i as f32 * 2.399_963_1;
        let p = Vec3::new(
            (1.0 - y * y).sqrt() * angle.cos(),
            y,
            (1.0 - y * y).sqrt() * angle.sin(),
        );
        let density = noise(p * 6.0 + offset);
        // El ruido agrupa asentamientos y deja espacios abiertos entre ellos.
        if density < 0.57 {
            continue;
        }
        let lane = p.y + 0.19 * (angle * 3.0).sin() + 0.075 * (angle * 7.0 + 0.4).sin();
        if lane.abs() < 0.075 {
            continue;
        }
        let n = noise(p * 21.0 + offset + Vec3::splat(27.4));
        instances.push(Instance {
            direction: p,
            heading: n * TAU,
            scale: 0.85 + density * 1.1,
            kind: ((n * 11.0) as usize) % 4,
        });
    }
    instances
}

pub fn world_instance(instance: Instance, time: f64) -> BodyState {
    instance.body(world::planet(0, time))
}
