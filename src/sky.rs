use crate::{
    camera::Camera,
    framebuffer::{to_rgb, Framebuffer},
    noise::{fbm, smooth},
};
use glam::Vec3;

const SIZE: usize = 192;
pub struct Sky {
    maps: Vec<Vec<u32>>,
    stars: Vec<(Vec3, f32, u32)>,
}
impl Default for Sky {
    fn default() -> Self {
        Self::new()
    }
}
impl Sky {
    pub fn new() -> Self {
        let mut maps = Vec::new();
        for side in 0..6 {
            let mut pixels = Vec::with_capacity(SIZE * SIZE);
            for y in 0..SIZE {
                for x in 0..SIZE {
                    let p = direction(
                        side,
                        2.0 * (x as f32 + 0.5) / SIZE as f32 - 1.0,
                        2.0 * (y as f32 + 0.5) / SIZE as f32 - 1.0,
                    )
                    .normalize();
                    let band = (-(p.dot(Vec3::new(0.22, 0.86, 0.46).normalize()) + 0.14).powi(2)
                        * 15.0)
                        .exp();
                    let n = fbm(p * 5.5 + Vec3::splat(8.0), 5);
                    let dust = fbm(p * 16.0 + Vec3::new(1.0, 8.0, 12.0), 3);
                    let luminosity = band * smooth(0.29, 0.72, n) * (0.4 + dust * 0.7);
                    let color = Vec3::new(0.035, 0.074, 0.12)
                        .lerp(Vec3::new(0.15, 0.065, 0.12), smooth(-0.6, 0.7, p.x));
                    pixels.push(to_rgb(Vec3::new(0.005, 0.009, 0.02) + color * luminosity));
                }
            }
            maps.push(pixels);
        }
        let mut state = 0x8a15bf21u32;
        let mut rand = || {
            state ^= state << 13;
            state ^= state >> 17;
            state ^= state << 5;
            state as f32 / u32::MAX as f32
        };
        let mut stars = Vec::new();
        for _ in 0..3400 {
            let y = rand() * 2.0 - 1.0;
            let angle = rand() * std::f32::consts::TAU;
            let r = (1.0 - y * y).sqrt();
            let brightness = 0.13 + rand().powi(7) * 0.85;
            let color = if rand() > 0.75 {
                0x9ec5ee
            } else if rand() > 0.82 {
                0xe8c8ad
            } else {
                0xd9e4ed
            };
            stars.push((
                Vec3::new(r * angle.cos(), y, r * angle.sin()),
                brightness,
                color,
            ));
        }
        Self { maps, stars }
    }
    pub fn draw(&self, frame: &mut Framebuffer, camera: &Camera) {
        let aspect = frame.width as f32 / frame.height as f32;
        let tan = (camera.fov * 0.5).tan();
        let right = camera.rotation * Vec3::X;
        let up = camera.rotation * Vec3::Y;
        let forward = camera.rotation * -Vec3::Z;
        let dx = right * (2.0 * tan * aspect / frame.width as f32);
        for y in 0..frame.height {
            let mut p = forward + up * ((1.0 - (y as f32 + 0.5) * 2.0 / frame.height as f32) * tan)
                - right * (tan * aspect)
                + dx * 0.5;
            for x in 0..frame.width {
                let (side, u, v) = lookup(p);
                let sx =
                    ((u * 0.5 + 0.5) * (SIZE - 1) as f32).clamp(0.0, (SIZE - 1) as f32) as usize;
                let sy =
                    ((v * 0.5 + 0.5) * (SIZE - 1) as f32).clamp(0.0, (SIZE - 1) as f32) as usize;
                frame.pixels[y * frame.width + x] = self.maps[side][sy * SIZE + sx];
                p += dx;
            }
        }
        for &(world, brightness, color) in &self.stars {
            let p = camera.rotation.conjugate() * world;
            if p.z >= -0.001 {
                continue;
            }
            let x = (0.5 + p.x / (-p.z * tan * aspect) * 0.5) * frame.width as f32;
            let y = (0.5 - p.y / (-p.z * tan) * 0.5) * frame.height as f32;
            let (ix, iy) = (x.floor() as i32, y.floor() as i32);
            let (fx, fy) = (x.fract(), y.fract());
            for (ox, oy, weight) in [
                (0, 0, (1.0 - fx) * (1.0 - fy)),
                (1, 0, fx * (1.0 - fy)),
                (0, 1, (1.0 - fx) * fy),
                (1, 1, fx * fy),
            ] {
                frame.blend(ix + ox, iy + oy, color, brightness * weight);
            }
            if brightness > 0.85 {
                for d in [-2, -1, 1, 2] {
                    frame.blend(ix + d, iy, color, 0.12 / d.abs() as f32);
                    frame.blend(ix, iy + d, color, 0.12 / d.abs() as f32);
                }
            }
        }
    }
}
fn direction(face: usize, u: f32, v: f32) -> Vec3 {
    match face {
        0 => Vec3::new(1.0, -v, -u),
        1 => Vec3::new(-1.0, -v, u),
        2 => Vec3::new(u, 1.0, v),
        3 => Vec3::new(u, -1.0, -v),
        4 => Vec3::new(u, -v, 1.0),
        _ => Vec3::new(-u, -v, -1.0),
    }
}
fn lookup(p: Vec3) -> (usize, f32, f32) {
    let a = p.abs();
    if a.x >= a.y && a.x >= a.z {
        if p.x > 0.0 {
            (0, -p.z / a.x, -p.y / a.x)
        } else {
            (1, p.z / a.x, -p.y / a.x)
        }
    } else if a.y >= a.z {
        if p.y > 0.0 {
            (2, p.x / a.y, p.z / a.y)
        } else {
            (3, p.x / a.y, -p.z / a.y)
        }
    } else if p.z > 0.0 {
        (4, p.x / a.z, -p.y / a.z)
    } else {
        (5, -p.x / a.z, -p.y / a.z)
    }
}
