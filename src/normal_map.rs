//! Mapa tangente RGB generado en CPU; muestreo bilineal y transformación TBN.
use crate::{framebuffer::Fragment, noise::fbm};
use glam::{Quat, Vec3};
use std::{
    f32::consts::{PI, TAU},
    sync::OnceLock,
};
const W: usize = 512;
const H: usize = 256;
static MAP: OnceLock<Vec<Vec3>> = OnceLock::new();
pub fn initialize() {
    MAP.get_or_init(|| {
        let mut height = vec![0.0; W * H];
        for y in 0..H {
            for x in 0..W {
                let longitude = x as f32 / W as f32 * TAU - PI;
                let latitude = (0.5 - y as f32 / (H - 1) as f32) * PI;
                let p = Vec3::new(
                    latitude.cos() * longitude.cos(),
                    latitude.sin(),
                    latitude.cos() * longitude.sin(),
                );
                height[y * W + x] = fbm(p * 95.0, 3);
            }
        }
        let mut rgb = vec![Vec3::ZERO; W * H];
        for y in 0..H {
            for x in 0..W {
                let dx = height[y * W + (x + W - 1) % W] - height[y * W + (x + 1) % W];
                let dy = height[y.saturating_sub(1) * W + x] - height[(y + 1).min(H - 1) * W + x];
                let n = Vec3::new(dx * 0.85, dy * 0.85, 1.0).normalize();
                rgb[y * W + x] = n * 0.5 + Vec3::splat(0.5);
            }
        }
        rgb
    });
}
pub fn sample(p: Vec3) -> Vec3 {
    let map = MAP.get().expect("normal map initialized before rendering");
    let x = (p.z.atan2(p.x) / TAU + 0.5) * W as f32;
    let y = (0.5 - p.y.clamp(-1.0, 1.0).asin() / PI) * (H - 1) as f32;
    let ix = x.floor() as usize % W;
    let iy = (y.floor() as usize).min(H - 1);
    let a = map[iy * W + ix].lerp(map[iy * W + (ix + 1) % W], x.fract());
    let b = map[(iy + 1).min(H - 1) * W + ix]
        .lerp(map[(iy + 1).min(H - 1) * W + (ix + 1) % W], x.fract());
    (a.lerp(b, y.fract()) * 2.0 - Vec3::ONE).normalize()
}
pub fn mapped(mut fragment: Fragment, rotation: Quat) -> Fragment {
    let p = fragment.object.normalize();
    let n = fragment.normal.normalize();
    let tangent = rotation * Vec3::new(-p.z, 0.0, p.x).try_normalize().unwrap_or(Vec3::X);
    let t = (tangent - n * tangent.dot(n)).normalize_or_zero();
    let b = n.cross(t);
    let s = sample(p);
    fragment.normal = (t * s.x + b * s.y + n * s.z).normalize();
    fragment
}
pub fn save(path: &std::path::Path) -> image::ImageResult<()> {
    initialize();
    let map = MAP.get().unwrap();
    image::RgbImage::from_fn(W as u32, H as u32, |x, y| {
        let c = map[y as usize * W + x as usize] * 255.0;
        image::Rgb([c.x as u8, c.y as u8, c.z as u8])
    })
    .save(path)
}
