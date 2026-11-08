use glam::Vec3;

fn hash(x: i32, y: i32, z: i32) -> f32 {
    let mut h = (x as u32).wrapping_mul(0x8da6b343)
        ^ (y as u32).wrapping_mul(0xd8163841)
        ^ (z as u32).wrapping_mul(0xcb1ab31f);
    h = (h ^ (h >> 13)).wrapping_mul(0x85ebca6b);
    (h ^ (h >> 16)) as f32 / u32::MAX as f32
}

/// Ruido 3D continuo con interpolación quíntica, independiente de la pantalla.
pub fn noise(p: Vec3) -> f32 {
    let base = p.floor();
    let f = p - base;
    let u = f * f * f * (f * (f * 6.0 - Vec3::splat(15.0)) + Vec3::splat(10.0));
    let [x, y, z] = base.as_ivec3().to_array();
    let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
    let plane = |k| {
        lerp(
            lerp(hash(x, y, k), hash(x + 1, y, k), u.x),
            lerp(hash(x, y + 1, k), hash(x + 1, y + 1, k), u.x),
            u.y,
        )
    };
    lerp(plane(z), plane(z + 1), u.z)
}

pub fn fbm(mut p: Vec3, octaves: usize) -> f32 {
    let mut sum = 0.0;
    let mut amplitude = 0.5;
    let mut weight = 0.0;
    for _ in 0..octaves {
        sum += amplitude * noise(p);
        weight += amplitude;
        p = p * 2.03 + Vec3::new(7.1, 1.7, 3.8);
        amplitude *= 0.5;
    }
    sum / weight
}

pub fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
