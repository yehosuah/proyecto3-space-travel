use glam::{Vec3, Vec4};

#[derive(Clone, Copy, Debug)]
pub struct ClipVertex {
    pub clip: Vec4,
    pub object: Vec3,
    pub world: Vec3,
    pub normal: Vec3,
}
impl ClipVertex {
    fn lerp(self, other: Self, t: f32) -> Self {
        Self {
            clip: self.clip.lerp(other.clip, t),
            object: self.object.lerp(other.object, t),
            world: self.world.lerp(other.world, t),
            normal: self.normal.lerp(other.normal, t),
        }
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Fragment {
    pub object: Vec3,
    pub world: Vec3,
    pub normal: Vec3,
}
#[derive(Clone, Copy)]
struct ScreenVertex {
    xy: Vec3,
    inverse_w: f32,
    object: Vec3,
    world: Vec3,
    normal: Vec3,
}

#[derive(Clone, Copy)]
struct Prepared {
    vertices: [ScreenVertex; 3],
    min: Vec3,
    max: Vec3,
    dx: Vec3,
    dy: Vec3,
    row: Vec3,
    depths: Vec3,
    inverse_w: Vec3,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct Stats {
    pub objects: u64,
    pub objects_culled: u64,
    pub triangles_available: u64,
    pub triangles_submitted: u64,
    pub triangles_rasterized: u64,
    pub fragments: u64,
}

pub struct Framebuffer {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u32>,
    pub depth: Vec<f32>,
    pub stats: Stats,
}
impl Framebuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            pixels: vec![0; width * height],
            depth: vec![f32::INFINITY; width * height],
            stats: Stats::default(),
        }
    }
    pub fn clear_depth(&mut self) {
        self.depth.fill(f32::INFINITY);
        self.stats = Stats::default();
    }
    pub fn triangle(
        &mut self,
        vertices: [ClipVertex; 3],
        two_sided: bool,
        shade: impl Fn(Fragment) -> Option<Vec3>,
    ) {
        self.stats.triangles_submitted += 1;
        clip_triangles(vertices, |vertices| {
            if let Some(t) = prepare(vertices, two_sided, self.width, self.height) {
                self.stats.triangles_rasterized += 1;
                self.stats.fragments +=
                    raster_region(&t, &mut self.pixels, &mut self.depth, self.width, 0, &shade);
            }
        });
    }
}
fn clip_triangles(vertices: [ClipVertex; 3], mut emit: impl FnMut([ClipVertex; 3])) {
    if vertices.iter().any(|v| !v.clip.is_finite()) {
        return;
    }
    let codes = vertices.map(|v| {
        (0..6).fold(0u8, |bits, plane| {
            bits | if distance(v.clip, plane) < 0.0 {
                1 << plane
            } else {
                0
            }
        })
    });
    if codes[0] & codes[1] & codes[2] != 0 {
        return;
    }
    let union = codes[0] | codes[1] | codes[2];
    if union == 0 {
        emit(vertices);
        return;
    }
    // Sutherland-Hodgman en coordenadas homogéneas. Buffers acotados en la pila.
    let mut polygon = [vertices[0]; 12];
    let mut next = polygon;
    polygon[..3].copy_from_slice(&vertices);
    let mut length = 3;
    for plane in 0..6 {
        if union & (1 << plane) == 0 {
            continue;
        }
        if length < 3 {
            return;
        }
        let mut count = 0;
        let mut a = polygon[length - 1];
        let mut da = distance(a.clip, plane);
        for &b in &polygon[..length] {
            let db = distance(b.clip, plane);
            if (da >= 0.0) != (db >= 0.0) {
                next[count] = a.lerp(b, da / (da - db));
                count += 1;
            }
            if db >= 0.0 {
                next[count] = b;
                count += 1;
            }
            a = b;
            da = db;
        }
        std::mem::swap(&mut polygon, &mut next);
        length = count;
    }
    for i in 1..length.saturating_sub(1) {
        emit([polygon[0], polygon[i], polygon[i + 1]]);
    }
}
fn prepare(
    vertices: [ClipVertex; 3],
    two_sided: bool,
    width: usize,
    height: usize,
) -> Option<Prepared> {
    if vertices.iter().any(|v| v.clip.w <= 0.0) {
        return None;
    }
    let v = vertices.map(|v| {
        let iw = v.clip.w.recip();
        let ndc = v.clip.truncate() * iw;
        ScreenVertex {
            xy: Vec3::new(
                (ndc.x * 0.5 + 0.5) * width as f32,
                (0.5 - ndc.y * 0.5) * height as f32,
                ndc.z * 0.5 + 0.5,
            ),
            inverse_w: iw,
            object: v.object * iw,
            world: v.world * iw,
            normal: v.normal * iw,
        }
    });
    let [a, b, c] = v.map(|v| v.xy);
    let area = edge(a, b, c);
    if area.abs() < 1e-5 || (!two_sided && area >= 0.0) {
        return None;
    }
    let min = a.min(b).min(c).floor().max(Vec3::ZERO);
    let max = a
        .max(b)
        .max(c)
        .ceil()
        .min(Vec3::new(width as f32 - 1.0, height as f32 - 1.0, 1.0));
    if min.x > max.x || min.y > max.y {
        return None;
    }
    let inv_area = area.recip();
    let start = Vec3::new(min.x + 0.5, min.y + 0.5, 0.0);
    Some(Prepared {
        vertices: v,
        min,
        max,
        dx: Vec3::new(b.y - c.y, c.y - a.y, a.y - b.y) * inv_area,
        dy: Vec3::new(c.x - b.x, a.x - c.x, b.x - a.x) * inv_area,
        row: Vec3::new(edge(b, c, start), edge(c, a, start), edge(a, b, start)) * inv_area,
        depths: Vec3::new(a.z, b.z, c.z),
        inverse_w: Vec3::new(v[0].inverse_w, v[1].inverse_w, v[2].inverse_w),
    })
}
fn raster_region(
    t: &Prepared,
    pixels: &mut [u32],
    depth_buffer: &mut [f32],
    width: usize,
    row_offset: usize,
    shade: &impl Fn(Fragment) -> Option<Vec3>,
) -> u64 {
    let low = (t.min.y as usize).max(row_offset);
    let high = (t.max.y as usize).min(row_offset + pixels.len() / width - 1);
    if low > high {
        return 0;
    }
    let v = &t.vertices;
    let mut fragments = 0;
    for y in low..=high {
        let mut weights = t.row + t.dy * (y as f32 - t.min.y);
        for x in t.min.x as usize..=t.max.x as usize {
            if weights.min_element() >= -0.00002 {
                let depth = weights.dot(t.depths);
                let index = (y - row_offset) * width + x;
                if depth >= 0.0 && depth < depth_buffer[index] {
                    let correct = weights.dot(t.inverse_w).recip();
                    let object = (v[0].object * weights.x
                        + v[1].object * weights.y
                        + v[2].object * weights.z)
                        * correct;
                    let world =
                        (v[0].world * weights.x + v[1].world * weights.y + v[2].world * weights.z)
                            * correct;
                    let normal = (v[0].normal * weights.x
                        + v[1].normal * weights.y
                        + v[2].normal * weights.z)
                        .normalize_or_zero();
                    if let Some(color) = shade(Fragment {
                        object,
                        world,
                        normal,
                    }) {
                        depth_buffer[index] = depth;
                        pixels[index] = to_rgb(color);
                        fragments += 1;
                    }
                }
            }
            weights += t.dx;
        }
    }
    fragments
}

fn edge(a: Vec3, b: Vec3, p: Vec3) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (b.y - a.y) * (p.x - a.x)
}
fn distance(p: Vec4, plane: usize) -> f32 {
    match plane {
        0 => p.x + p.w,
        1 => p.w - p.x,
        2 => p.y + p.w,
        3 => p.w - p.y,
        4 => p.z + p.w,
        _ => p.w - p.z,
    }
}
pub fn to_rgb(c: Vec3) -> u32 {
    let [r, g, b] = (c.clamp(Vec3::ZERO, Vec3::ONE) * 255.0)
        .round()
        .as_uvec3()
        .to_array();
    (r << 16) | (g << 8) | b
}
pub fn from_rgb(c: u32) -> Vec3 {
    Vec3::new(
        ((c >> 16) & 255) as f32,
        ((c >> 8) & 255) as f32,
        (c & 255) as f32,
    ) / 255.0
}
