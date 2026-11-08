//! Modelos propios compuestos en CPU y distribuidos mediante el ruido del curso.
use crate::{
    mesh::{surface_radius, Mesh, Vertex},
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

struct Builder(Mesh);
impl Builder {
    fn new() -> Self {
        Self(Mesh {
            vertices: Vec::new(),
            faces: Vec::new(),
            colors: Vec::new(),
            flat: true,
        })
    }
    fn triangle(&mut self, a: Vec3, b: Vec3, c: Vec3, color: Vec3) {
        let id = self.0.vertices.len();
        for p in [a, b, c] {
            self.0.vertices.push(Vertex {
                position: p,
                normal: Vec3::Y,
            });
        }
        self.0.faces.push([id, id + 1, id + 2]);
        self.0.colors.push(color);
    }
    fn block(&mut self, center: Vec3, size: Vec3, color: Vec3) {
        let v = [
            [-1., -1., -1.],
            [1., -1., -1.],
            [1., 1., -1.],
            [-1., 1., -1.],
            [-1., -1., 1.],
            [1., -1., 1.],
            [1., 1., 1.],
            [-1., 1., 1.],
        ]
        .map(|p| center + Vec3::from_array(p) * size * 0.5);
        for [a, b, c] in [
            [0, 2, 1],
            [0, 3, 2],
            [4, 5, 6],
            [4, 6, 7],
            [0, 4, 7],
            [0, 7, 3],
            [1, 2, 6],
            [1, 6, 5],
            [0, 1, 5],
            [0, 5, 4],
            [3, 7, 6],
            [3, 6, 2],
        ] {
            self.triangle(v[a], v[b], v[c], color);
        }
    }
    fn cylinder(&mut self, base: Vec3, radius: f32, height: f32, color: Vec3) {
        for i in 0..12 {
            let point = |j| {
                let a = TAU * j as f32 / 12.0;
                base + Vec3::new(a.cos() * radius, 0.0, a.sin() * radius)
            };
            let a = point(i);
            let b = point(i + 1);
            let up = Vec3::Y * height;
            self.triangle(a, a + up, b, color);
            self.triangle(a + up, b + up, b, color);
            self.triangle(base + up, b + up, a + up, color);
            self.triangle(base, a, b, color);
        }
    }
}

pub fn model(kind: usize, low: bool) -> Mesh {
    let mut b = Builder::new();
    let ivory = Vec3::new(0.7, 0.72, 0.7);
    let dark = Vec3::new(0.09, 0.13, 0.17);
    let red = Vec3::new(0.72, 0.06, 0.07);
    let glass = Vec3::new(0.05, 0.34, 0.41);
    if low {
        b.block(Vec3::Y * 2.0, Vec3::new(4.0, 4.0, 3.0), ivory);
        return b.0;
    }
    match kind {
        0 => {
            // Hangar con marquesina, puertas, contrafuertes y pista de acceso.
            b.block(Vec3::new(0.0, 0.12, 1.0), Vec3::new(9.0, 0.3, 10.0), dark);
            b.block(Vec3::new(0.0, 1.5, 0.0), Vec3::new(7.0, 3.0, 5.0), ivory);
            b.block(Vec3::new(0.0, 3.15, 0.2), Vec3::new(8.0, 0.4, 6.0), red);
            b.block(Vec3::new(0.0, 1.35, 2.53), Vec3::new(5.4, 2.5, 0.08), dark);
            for x in [-2.0, 0.0, 2.0] {
                b.block(Vec3::new(x, 1.3, 2.6), Vec3::new(0.1, 2.4, 0.12), glass);
            }
            for x in [-3.6, 3.6] {
                b.block(Vec3::new(x, 1.0, 0.0), Vec3::new(0.5, 2.0, 5.7), dark);
            }
        }
        1 => {
            // Torre de control con cabina acristalada.
            b.block(Vec3::new(0.0, 0.25, 0.0), Vec3::new(4.0, 0.5, 4.0), dark);
            b.block(Vec3::new(0.0, 3.0, 0.0), Vec3::new(1.8, 6.0, 1.8), ivory);
            b.block(Vec3::new(0.0, 5.8, 0.0), Vec3::new(4.6, 0.5, 4.6), red);
            b.block(Vec3::new(0.0, 6.7, 0.0), Vec3::new(3.9, 1.4, 3.9), glass);
            b.block(Vec3::new(0.0, 7.5, 0.0), Vec3::new(4.4, 0.3, 4.4), ivory);
            b.cylinder(Vec3::new(0.0, 7.5, 0.0), 0.12, 2.5, red);
        }
        2 => {
            // Antena de comunicaciones con reflector parabólico.
            b.cylinder(Vec3::ZERO, 1.9, 0.6, dark);
            b.cylinder(Vec3::Y * 0.6, 0.25, 3.7, ivory);
            for ring in 0..4 {
                for i in 0..16 {
                    let point = |r: usize, j: usize| {
                        let radius = r as f32 * 0.65;
                        let a = TAU * j as f32 / 16.0;
                        Vec3::new(
                            radius * a.cos(),
                            4.0 + radius * radius * 0.18,
                            radius * a.sin(),
                        )
                    };
                    let a = point(ring, i);
                    let c = point(ring + 1, i);
                    let d = point(ring + 1, i + 1);
                    let e = point(ring, i + 1);
                    b.triangle(a, d, c, ivory);
                    b.triangle(a, e, d, ivory);
                    b.triangle(c, d, a, dark);
                    b.triangle(d, e, a, dark);
                }
            }
            b.cylinder(Vec3::Y * 4.0, 0.09, 2.6, red);
        }
        _ => {
            // Baliza de ruta con alas y cabezal luminoso.
            b.block(Vec3::Y * 0.2, Vec3::new(2.4, 0.4, 2.4), dark);
            b.block(Vec3::Y * 2.0, Vec3::new(0.65, 4.0, 0.65), ivory);
            b.block(Vec3::Y * 3.8, Vec3::new(2.8, 0.35, 0.5), red);
            b.cylinder(Vec3::Y * 4.0, 0.55, 0.65, glass);
        }
    }
    b.0
}

pub fn monaco_instances(seed: u32) -> (Vec<Mesh>, Vec<Mesh>, Vec<Instance>) {
    (
        (0..4).map(|i| model(i, false)).collect(),
        (0..4).map(|i| model(i, true)).collect(),
        distribute(seed),
    )
}

pub fn world_instance(instance: Instance, time: f64) -> BodyState {
    instance.body(world::planet(0, time))
}
