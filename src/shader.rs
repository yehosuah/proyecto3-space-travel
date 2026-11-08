use crate::{
    framebuffer::Fragment,
    noise::{fbm, noise, smooth},
};
use glam::{Quat, Vec3};
use std::f32::consts::TAU;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Body {
    Star,
    Rocky,
    Gas,
    Ring,
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
        Body::Rocky => rocky(f, u),
        Body::Gas => gas(f, u),
        Body::Ring => return ring(f, u),
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

fn rocky(f: Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object.normalize();
    let longitude = p.z.atan2(p.x);
    let field = crate::material_field::sample(p);
    let terrain = field.x;
    // 1. La paleta rojo y marfil de Mónaco forma dos hemisferios geológicos.
    let red = smooth(-0.08, 0.11, p.y + (terrain - 0.5) * 0.22);
    let mut c = rgb(0.73, 0.71, 0.64).lerp(rgb(0.48, 0.082, 0.075), red);
    // 2. Mármol fracturado, regolito oscuro y cuencas de impacto.
    if u.layers >= 2 {
        let mineral = field.y;
        let veins = 1.0 - smooth(0.018, 0.07, (terrain - 0.48).abs());
        c *= 0.56 + 0.54 * mineral;
        c = c.lerp(rgb(0.16, 0.11, 0.13), veins * 0.45);
        for (center, size) in [
            (Vec3::new(-0.4, 0.5, 0.77).normalize(), 0.19),
            (Vec3::new(0.4, -0.4, 0.82).normalize(), 0.14),
            (Vec3::new(0.15, 0.7, 0.7).normalize(), 0.09),
        ] {
            let d = p.distance(center) / size;
            c *= 1.0 - (1.0 - smooth(0.55, 0.97, d)) * 0.5;
            c += rgb(0.18, 0.13, 0.10) * (1.0 - smooth(0.08, 0.24, (d - 1.0).abs()));
        }
    }
    let mut emission = Vec3::ZERO;
    // 3. Circuito de asfalto sinuoso con pianos alternados en rojo y blanco.
    if u.layers >= 3 {
        let route = p.y + 0.19 * (longitude * 3.0).sin() + 0.075 * (longitude * 7.0 + 0.4).sin();
        let d = route.abs();
        let track = 1.0 - smooth(0.025, 0.033, d);
        let curb = smooth(0.03, 0.036, d) * (1.0 - smooth(0.047, 0.053, d));
        let paint = if (longitude * 42.0 / TAU).floor() as i32 % 2 == 0 {
            rgb(0.95, 0.93, 0.83)
        } else {
            rgb(0.9, 0.018, 0.03)
        };
        c = c.lerp(rgb(0.045, 0.055, 0.068), track);
        c = c.lerp(paint, curb);
        let lane = 1.0 - smooth(0.0015, 0.004, d);
        c = c.lerp(rgb(0.9, 0.72, 0.39), lane * 0.8);
    }
    // 4. Puertos turquesa y brillo costero en un mundo imaginario de Mónaco.
    if u.layers >= 4 {
        let ocean = (1.0 - smooth(0.33, 0.42, terrain)) * smooth(-0.15, -0.5, p.y);
        c = c.lerp(rgb(0.018, 0.30, 0.36), ocean);
        emission += rgb(0.01, 0.25, 0.32) * ocean * 0.20;
        emission += rgb(0.11, 0.39, 0.53) * rim(f, u) * 0.48;
    }
    let f = if u.normal_mapping {
        let mut mapped = crate::normal_map::mapped(f, u.rotation);
        let proximity = (1.0 - u.camera.distance(f.world) / (u.radius * 2.0)).clamp(0.0, 1.0);
        mapped.normal = f
            .normal
            .lerp(mapped.normal, 0.15 + proximity * 0.65)
            .normalize();
        mapped
    } else {
        f
    };
    lit(c, f) + emission
}

fn gas(f: Fragment, u: &Uniforms) -> Vec3 {
    let p = f.object.normalize();
    let longitude = p.z.atan2(p.x);
    // 1. Bandas latitudinales en verde de carreras y plata.
    let broad = (p.y * 15.0 + 0.8 * (longitude * 2.0).sin()).sin() * 0.5 + 0.5;
    let mut c = rgb(0.018, 0.15, 0.18).lerp(rgb(0.34, 0.81, 0.73), smooth(0.05, 0.92, broad));
    // 2. Ruido 3D y variación sinusoidal deforman las bandas con el tiempo.
    if u.layers >= 2 {
        let drift = Vec3::new(u.time * 0.022, 0.0, u.time * 0.013);
        let turbulence = fbm(p * Vec3::new(5.0, 13.0, 5.0) + drift, 4);
        let fine =
            (p.y * 94.0 + turbulence * 16.0 + (longitude * 6.0 + u.time * 0.05).sin() * 1.8).sin();
        c = c.lerp(rgb(0.73, 0.87, 0.75), smooth(0.4, 0.92, fine) * 0.68);
        c *= 0.7 + turbulence * 0.42;
    }
    // 3. Huracán cobrizo con ojo elíptico y brazos espirales calculados analíticamente.
    if u.layers >= 3 {
        let q = Vec3::new(
            (p.x - 0.25) / 0.34,
            (p.y - 0.37) / 0.18,
            (p.z - 0.87) / 0.65,
        );
        let d = (q.x * q.x + q.y * q.y).sqrt();
        let front = smooth(0.45, 0.75, p.z);
        let storm = (1.0 - smooth(0.8, 1.4, d)) * front;
        let spiral = (q.y.atan2(q.x) * 3.0 + d * 16.0 - u.time * 0.16).sin() * 0.5 + 0.5;
        let hurricane = rgb(0.25, 0.05, 0.028).lerp(rgb(1.0, 0.55, 0.22), spiral);
        c = c.lerp(hurricane, storm * 0.94);
        c = c.lerp(
            rgb(0.11, 0.025, 0.014),
            (1.0 - smooth(0.08, 0.2, d)) * front,
        );
    }
    // 4. Franja de meta a cuadros y borde atmosférico.
    let mut atmosphere = Vec3::ZERO;
    if u.layers >= 4 {
        let band = smooth(0.46, 0.475, p.y) * (1.0 - smooth(0.59, 0.605, p.y));
        let checker = ((longitude * 36.0 / TAU).floor() as i32
            + ((p.y - 0.46) / 0.065).floor() as i32)
            .rem_euclid(2);
        let flag = if checker == 0 {
            rgb(0.82, 0.91, 0.86)
        } else {
            rgb(0.015, 0.055, 0.065)
        };
        c = c.lerp(flag, band * 0.9);
        atmosphere = rgb(0.09, 0.65, 0.56) * rim(f, u) * 0.45;
    }
    let mut result = lit(c, f);
    // Sombra del plano independiente de anillos proyectada hacia la luz.
    let denom = light(f.world).dot(u.ring_normal);
    if denom.abs() > 1e-4 {
        let relative = (f.world - u.center) / u.radius;
        let t = -relative.dot(u.ring_normal) / denom;
        let hit = (relative + light(f.world) * t).length();
        if t > 0.0 && ring_density(hit) > 0.0 {
            result *= 0.58;
        }
    }
    result + atmosphere
}

pub fn ring_density(r: f32) -> f32 {
    if !(1.28..=1.98).contains(&r) || (1.52..1.59).contains(&r) || (1.82..1.855).contains(&r) {
        0.0
    } else {
        1.0
    }
}

fn ring(f: Fragment, u: &Uniforms) -> Option<Vec3> {
    let r = f.object.length();
    if ring_density(r) == 0.0 {
        return None;
    }
    // Surcos de neumático, carril cobrizo y franja exterior a cuadros.
    let grooves = (r * 330.0 + noise(f.object * 22.0) * 2.0).sin() * 0.5 + 0.5;
    let mut c = rgb(0.10, 0.23, 0.24).lerp(rgb(0.49, 0.76, 0.67), grooves);
    if r > 1.60 && r < 1.70 {
        c = rgb(0.61, 0.30, 0.13) * (0.65 + grooves * 0.4);
    }
    if r > 1.90 {
        let angle = f.object.z.atan2(f.object.x);
        let checker = ((angle * 64.0 / TAU).floor() as i32 + ((r - 1.90) * 50.0).floor() as i32)
            .rem_euclid(2);
        c = if checker == 0 {
            rgb(0.79, 0.83, 0.73)
        } else {
            rgb(0.035, 0.075, 0.09)
        };
    }
    // La esfera proyecta su sombra sobre la malla de anillos.
    let relative = (f.world - u.center) / u.radius;
    let along = -relative.dot(light(f.world));
    if along > 0.0 && (relative + light(f.world) * along).length_squared() < 1.0 {
        c *= 0.17;
    }
    Some(c * (0.65 + 0.35 * f.normal.dot(light(f.world)).abs()))
}
