use glam::Vec3;

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
