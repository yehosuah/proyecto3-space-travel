//! Campos de ruido estacionarios precalculados con el mismo algoritmo propio.
//! La iluminación y el mapa de normales se evalúan todavía por fragmento.
use crate::noise::fbm;
use glam::{Vec2, Vec3};
use std::{
    f32::consts::{PI, TAU},
    sync::OnceLock,
};
const W: usize = 2048;
const H: usize = 1024;
static FIELD: OnceLock<Vec<Vec2>> = OnceLock::new();
pub fn initialize() {
    FIELD.get_or_init(|| {
        let mut data = vec![Vec2::ZERO; W * H];
        std::thread::scope(|scope| {
            for (band, chunk) in data.chunks_mut(W * 128).enumerate() {
                scope.spawn(move || {
                    for (i, value) in chunk.iter_mut().enumerate() {
                        let y = band * 128 + i / W;
                        let x = i % W;
                        let longitude = x as f32 / W as f32 * TAU - PI;
                        let latitude = (0.5 - y as f32 / (H - 1) as f32) * PI;
                        let p = Vec3::new(
                            latitude.cos() * longitude.cos(),
                            latitude.sin(),
                            latitude.cos() * longitude.sin(),
                        );
                        *value = Vec2::new(fbm(p * 4.5 + Vec3::splat(12.3), 4), fbm(p * 28.0, 3));
                    }
                });
            }
        });
        data
    });
}
pub fn sample(p: Vec3) -> Vec2 {
    let data = FIELD.get().expect("material field initialized");
    let x = (p.z.atan2(p.x) / TAU + 0.5) * W as f32;
    let y = (0.5 - p.y.clamp(-1.0, 1.0).asin() / PI) * (H - 1) as f32;
    let ix = x.floor() as usize % W;
    let iy = (y.floor() as usize).min(H - 1);
    let a = data[iy * W + ix].lerp(data[iy * W + (ix + 1) % W], x.fract());
    let b = data[(iy + 1).min(H - 1) * W + ix]
        .lerp(data[(iy + 1).min(H - 1) * W + (ix + 1) % W], x.fract());
    a.lerp(b, y.fract())
}
