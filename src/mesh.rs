use crate::noise::fbm;
use glam::Vec3;
use std::{
    collections::HashMap,
    f32::consts::{PI, TAU},
    fs,
    path::Path,
};

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

/// Intersección radial con los mismos triángulos del LOD cercano, para apoyar bases.
pub fn surface_radius(p: Vec3) -> f32 {
    let phi = p.y.clamp(-1.0, 1.0).acos();
    let theta = p.z.atan2(p.x).rem_euclid(TAU);
    let j = (phi / PI * 88.0).floor().min(87.0);
    let i = (theta / TAU * 144.0).floor();
    let vertex = |x: f32, y: f32| {
        let a = TAU * x / 144.0;
        let b = PI * y / 88.0;
        let d = Vec3::new(b.sin() * a.cos(), b.cos(), b.sin() * a.sin());
        d * terrain_height(d)
    };
    let v = [
        vertex(i, j),
        vertex(i + 1.0, j),
        vertex(i, j + 1.0),
        vertex(i + 1.0, j + 1.0),
    ];
    for [a, b, c] in [[v[0], v[1], v[2]], [v[1], v[3], v[2]]] {
        let e1 = b - a;
        let e2 = c - a;
        let h = p.cross(e2);
        let determinant = e1.dot(h);
        if determinant.abs() < 1e-9 {
            continue;
        }
        let inv = 1.0 / determinant;
        let q = (-a).cross(e1);
        let u = (-a).dot(h) * inv;
        let w = p.dot(q) * inv;
        if u >= -0.00001 && w >= -0.00001 && u + w <= 1.00001 {
            return e2.dot(q) * inv;
        }
    }
    terrain_height(p)
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

pub fn rings(segments: usize, inner: f32, outer: f32) -> Mesh {
    let mut vertices = Vec::new();
    let mut faces = Vec::new();
    for i in 0..segments {
        let t = TAU * i as f32 / segments as f32;
        for radius in [inner, outer] {
            vertices.push(Vertex {
                position: Vec3::new(t.cos() * radius, 0.0, t.sin() * radius),
                normal: Vec3::Y,
            });
        }
        let a = 2 * i;
        let b = 2 * ((i + 1) % segments);
        faces.push([a, a + 1, b]);
        faces.push([a + 1, b + 1, b]);
    }
    Mesh {
        vertices,
        faces,
        colors: vec![],
        flat: false,
    }
}

/// Lectura de posiciones, caras trianguladas y materiales Kd del OBJ propio.
pub fn load_obj(path: &Path) -> Result<Mesh, String> {
    let source = fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    let mut positions = Vec::new();
    let mut faces = Vec::new();
    let mut materials = HashMap::new();
    let mut face_materials = Vec::new();
    let mut active = String::new();
    for line in source.lines() {
        let f: Vec<_> = line
            .split('#')
            .next()
            .unwrap_or("")
            .split_whitespace()
            .collect();
        match f.first().copied() {
            Some("mtllib") if f.len() > 1 => {
                let text = fs::read_to_string(path.parent().unwrap_or(Path::new(".")).join(f[1]))
                    .map_err(|e| e.to_string())?;
                let mut name = String::new();
                for line in text.lines() {
                    let a: Vec<_> = line.split_whitespace().collect();
                    if a.first() == Some(&"newmtl") && a.len() > 1 {
                        name = a[1].to_string();
                    }
                    if a.first() == Some(&"Kd") && a.len() >= 4 {
                        materials.insert(name.clone(), read_vec(&a[1..])?);
                    }
                }
            }
            Some("v") => positions.push(read_vec(&f[1..])?),
            Some("usemtl") if f.len() > 1 => active = f[1].to_string(),
            Some("f") => {
                if f.len() != 4 {
                    return Err("El OBJ debe estar triangulado".into());
                }
                let mut ids = [0usize; 3];
                for (i, token) in f[1..].iter().enumerate() {
                    let n = token
                        .split('/')
                        .next()
                        .unwrap_or("")
                        .parse::<isize>()
                        .map_err(|_| "Índice OBJ inválido")?;
                    let id = if n > 0 {
                        n - 1
                    } else {
                        positions.len() as isize + n
                    };
                    if n == 0 || id < 0 || id >= positions.len() as isize {
                        return Err("Índice OBJ fuera de rango".into());
                    }
                    ids[i] = id as usize;
                }
                faces.push(ids);
                face_materials.push(active.clone());
            }
            _ => {}
        }
    }
    if positions.is_empty() || faces.is_empty() {
        return Err("OBJ vacío".into());
    }
    let min = positions
        .iter()
        .copied()
        .fold(Vec3::splat(f32::INFINITY), Vec3::min);
    let max = positions
        .iter()
        .copied()
        .fold(Vec3::splat(f32::NEG_INFINITY), Vec3::max);
    let center = (min + max) * 0.5;
    let scale = 3.8 / (max - min).max_element();
    let vertices = positions
        .into_iter()
        .map(|p| Vertex {
            position: (p - center) * Vec3::new(-1.0, 1.0, -1.0) * scale,
            normal: Vec3::Y,
        })
        .collect();
    let colors = face_materials
        .iter()
        .map(|n| materials.get(n).copied().unwrap_or(Vec3::splat(0.7)))
        .collect();
    Ok(Mesh {
        vertices,
        faces,
        colors,
        flat: true,
    })
}

fn read_vec(f: &[&str]) -> Result<Vec3, String> {
    if f.len() < 3 {
        return Err("Faltan coordenadas".into());
    }
    let mut a = [0.0; 3];
    for i in 0..3 {
        a[i] = f[i].parse::<f32>().map_err(|_| "Coordenada inválida")?;
    }
    let p = Vec3::from_array(a);
    if !p.is_finite() {
        return Err("Coordenada no finita".into());
    }
    Ok(p)
}
