use crate::{
    framebuffer::Fragment,
    noise::{fbm, smooth},
};
use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Star,
    Ship,
    Prop,
    Gate,
}

#[derive(Clone, Copy)]
pub struct Uniforms {
    pub body: Body,
    pub time: f32,
    pub layers: u8,
    pub center: Vec3,
    pub radius: f32,
    pub ring_normal: Vec3,
    pub camera: Vec3,
    pub tint: Vec3,
    pub rotation: Quat,
    pub normal_mapping: bool,
}

pub fn light(position: Vec3) -> Vec3 {
    (-position).normalize_or_zero()
}
fn rgb(r: f32, g: f32, b: f32) -> Vec3 {
    Vec3::new(r, g, b)
}
fn lit(color: Vec3, f: Fragment) -> Vec3 {
    let diffuse = smooth(-0.15, 0.8, f.normal.dot(light(f.world)));
    color * (0.15 + 0.85 * diffuse)
}
fn rim(f: Fragment, u: &Uniforms) -> f32 {
    let view = (u.camera - f.world).normalize_or_zero();
    (1.0 - f.normal.dot(view).abs()).max(0.0).powf(3.0)
}

pub fn shade(f: Fragment, u: &Uniforms) -> Option<Vec3> {
    Some(match u.body {
        Body::Star => star(f, u),
        Body::Ship => {
            let diffuse = lit(u.tint, f);
            let view = (u.camera - f.world).normalize_or_zero();
            let half = (light(f.world) + view).normalize_or_zero();
            let specular = f.normal.dot(half).max(0.0).powf(48.0) * 0.45;
            diffuse + Vec3::splat(specular) + u.tint * 0.10
        }
        Body::Prop => {
            let emission = if u.tint.z > u.tint.x * 3.0 {
                0.28
            } else {
                0.04
            };
            lit(u.tint, f) + u.tint * emission
        }
        Body::Gate => lit(u.tint, f) * 0.35 + u.tint * 1.3,
    })
}

fn star(f: Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object.normalize();
    let drift = Vec3::new(0.0, u.time * 0.035, -u.time * 0.025);
    // 1. Campo continuo de temperatura que evoluciona con el tiempo.
    let heat = fbm(p * 3.8 + drift, 4);
    let mut c = rgb(0.62, 0.025, 0.008).lerp(rgb(1.0, 0.65, 0.12), smooth(0.22, 0.8, heat));
    // 2. Pliegues de plasma con dominio deformado y zonas oscuras de caucho quemado.
    if u.layers >= 2 {
        let warp = fbm(p * 6.0 - drift, 3);
        let folds = fbm(p * 12.0 + Vec3::splat(warp * 3.0) + drift, 3);
        c *= 0.42 + 0.85 * smooth(0.29, 0.68, folds);
        c += rgb(0.75, 0.15, 0.015) * smooth(0.56, 0.72, folds);
    }
    // 3. Granulación brillante sobre las estructuras grandes del plasma.
    if u.layers >= 3 {
        let cells = fbm(p * 55.0 + drift * 2.0, 3);
        let hot = smooth(0.52, 0.74, cells);
        c = c.lerp(rgb(1.0, 0.87, 0.39), hot * 0.78);
        c *= 0.8 + 0.28 * cells;
    }
    // 4. Cinco focos magnéticos rojos con borde caliente, anclados a la estrella.
    if u.layers >= 4 {
        for i in 0..5 {
            let x = (i as f32 - 2.0) * 0.30;
            let center = Vec3::new(x, 0.13, (1.0 - x * x - 0.13 * 0.13).sqrt());
            let d = p.distance(center);
            let spot = 1.0 - smooth(0.056, 0.115, d);
            let border = (1.0 - smooth(0.11, 0.15, d)) * smooth(0.075, 0.11, d);
            c = c.lerp(rgb(0.53, 0.003, 0.012), spot * 0.88);
            c += rgb(0.5, 0.12, 0.025) * border;
        }
        c += rgb(0.6, 0.08, 0.006) * rim(f, u) * 0.28;
    }
    // Estrella emisiva, sin lado nocturno lambertiano.
    c * (0.67
        + 0.33
            * f.normal
                .dot((u.camera - f.world).normalize_or_zero())
                .abs()
                .sqrt())
}
