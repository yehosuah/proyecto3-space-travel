use crate::noise::fbm;
use glam::Vec3;
use std::f32::consts::{PI, TAU};

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub position: Vec3,
    pub normal: Vec3,
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub faces: Vec<[usize; 3]>,
    pub colors: Vec<Vec3>,
    pub flat: bool,
}

pub fn terrain_height(p: Vec3) -> f32 {
    1.0 + (fbm(p * 4.5 + Vec3::splat(12.3), 4) - 0.48) * 0.14
}


/// La malla se genera en CPU; el relieve desplaza vértices antes de proyectar.
pub fn sphere(slices: usize, stacks: usize, deform: bool) -> Mesh {
    let point = |p: Vec3| if deform { p * terrain_height(p) } else { p };
    let normal = |p: Vec3| {
        if !deform {
            return p;
        }
        let axis = if p.y.abs() < 0.9 { Vec3::Y } else { Vec3::X };
        let tangent = axis.cross(p).normalize();
        let bitangent = p.cross(tangent);
        let e = 0.001;
        let a = point((p + tangent * e).normalize()) - point((p - tangent * e).normalize());
        let b = point((p + bitangent * e).normalize()) - point((p - bitangent * e).normalize());
        a.cross(b).normalize()
    };
    let mut vertices = vec![Vertex {
        position: point(Vec3::Y),
        normal: Vec3::Y,
    }];
    for j in 1..stacks {
        let phi = PI * j as f32 / stacks as f32;
        for i in 0..slices {
            let theta = TAU * i as f32 / slices as f32;
            let p = Vec3::new(phi.sin() * theta.cos(), phi.cos(), phi.sin() * theta.sin());
            vertices.push(Vertex {
                position: point(p),
                normal: normal(p),
            });
        }
    }
    let bottom = vertices.len();
    vertices.push(Vertex {
        position: point(-Vec3::Y),
        normal: -Vec3::Y,
    });
    let mut faces = Vec::new();
    for i in 0..slices {
        let k = (i + 1) % slices;
        faces.push([0, 1 + k, 1 + i]);
        for j in 0..stacks - 2 {
            let a = 1 + j * slices + i;
            let b = 1 + j * slices + k;
            faces.push([a, b, a + slices]);
            faces.push([b, b + slices, a + slices]);
        }
        faces.push([bottom, bottom - slices + i, bottom - slices + k]);
    }
    Mesh {
        vertices,
        faces,
        colors: vec![],
        flat: false,
    }
}
