use crate::{
    framebuffer::Fragment,
    noise::{smooth},
};
use glam::{Quat, Vec3};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
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
fn lit(color: Vec3, f: Fragment) -> Vec3 {
    let diffuse = smooth(-0.15, 0.8, f.normal.dot(light(f.world)));
    color * (0.15 + 0.85 * diffuse)
}

pub fn shade(f: Fragment, u: &Uniforms) -> Option<Vec3> {
    Some(match u.body {
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
