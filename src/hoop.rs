use crate::mesh::{Mesh, Vertex};
use glam::Vec3;
use std::f32::consts::TAU;
pub fn mesh(accent: Vec3) -> Mesh {
    let mut m = Mesh {
        vertices: Vec::new(),
        faces: Vec::new(),
        colors: Vec::new(),
        flat: true,
    };
    for i in 0..48 {
        let a = TAU * i as f32 / 48.0;
        for j in 0..6 {
            let b = TAU * j as f32 / 6.0;
            let normal = Vec3::new(a.cos() * b.cos(), b.sin(), a.sin() * b.cos());
            m.vertices.push(Vertex {
                position: Vec3::new(a.cos(), 0.0, a.sin()) * 0.98 + normal * 0.024,
                normal,
            });
        }
    }
    for i in 0..48 {
        for j in 0..6 {
            let a = i * 6 + j;
            let b = ((i + 1) % 48) * 6 + j;
            let c = i * 6 + (j + 1) % 6;
            let d = ((i + 1) % 48) * 6 + (j + 1) % 6;
            for face in [[a, c, b], [c, d, b]] {
                m.faces.push(face);
                m.colors.push(if i % 8 < 6 {
                    accent
                } else {
                    Vec3::new(0.11, 0.15, 0.18)
                });
            }
        }
    }
    m
}
