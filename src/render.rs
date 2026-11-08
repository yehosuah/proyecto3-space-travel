use crate::{
    camera::Camera,
    framebuffer::{from_rgb, to_rgb, ClipVertex, Framebuffer},
    hud,
    mesh::{self, Mesh},
    race,
    shader::{Body, Uniforms},
    simulation::Flight,
    sky::Sky,
    surface,
    world::{self, BodyState},
};
use glam::{Mat4, Vec3};
use std::path::Path;

pub struct Renderer {
    spheres: Vec<Mesh>,
    rocky: Vec<Mesh>,
    rings: Mesh,
    ship: Mesh,
    sky: Sky,
    props: Vec<Mesh>,
    low_props: Vec<Mesh>,
    instances: Vec<surface::Instance>,
    gates: [Mesh; 2],
}
#[derive(Clone, Copy)]
pub struct Options {
    pub lod: bool,
    pub culling: bool,
    pub ui: bool,
    pub help: bool,
    pub debug: bool,
    pub target: usize,
    pub fps: f32,
    pub paused: bool,
    pub normal_mapping: bool,
    pub race: race::View,
    pub music_volume: f32,
    pub fx_volume: f32,
    pub muted: bool,
    pub track_loaded: bool,
}
impl Default for Options {
    fn default() -> Self {
        Self {
            lod: true,
            culling: true,
            ui: true,
            help: false,
            debug: false,
            target: 0,
            fps: 0.0,
            paused: false,
            normal_mapping: true,
            race: race::View::default(),
            music_volume: 0.28,
            fx_volume: 0.55,
            muted: false,
            track_loaded: false,
        }
    }
}
impl Renderer {
    pub fn new(root: &Path) -> Result<Self, String> {
        crate::normal_map::initialize();
        crate::material_field::initialize();
        let (props, low_props, instances) = surface::monaco_instances(world::SURFACE_SEED);
        let gates =
            [Vec3::new(0.04, 0.55, 0.38), Vec3::new(0.09, 0.25, 0.31)].map(crate::hoop::mesh);
        let levels = [(20, 12), (40, 24), (80, 48), (144, 88)];
        Ok(Self {
            spheres: levels
                .iter()
                .map(|&(a, b)| mesh::sphere(a, b, false))
                .collect(),
            rocky: levels
                .iter()
                .map(|&(a, b)| mesh::sphere(a, b, true))
                .collect(),
            rings: mesh::rings(192, 1.28, 1.98),
            ship: mesh::load_obj(&root.join("assets/models/Nave_Aster.obj"))?,
            sky: Sky::new(),
            props,
            low_props,
            instances,
            gates,
        })
    }
    pub fn render(
        &self,
        frame: &mut Framebuffer,
        camera: &Camera,
        ship: &Flight,
        options: Options,
    ) {
        frame.clear_depth();
        self.sky.draw(frame, camera);
        let aspect = frame.width as f32 / frame.height as f32;
        let vp = camera.matrix(aspect);
        self.halo(frame, camera, vp);
        let sun = world::star(ship.time);
        let mut objects = vec![
            (sun, Body::Star),
            (world::planet(0, ship.time), Body::Rocky),
            (world::planet(1, ship.time), Body::Gas),
            (world::moon(ship.time), Body::Moon),
        ];
        objects.sort_by(|a, b| {
            camera
                .position
                .distance_squared(a.0.center)
                .total_cmp(&camera.position.distance_squared(b.0.center))
        });
        for (body, kind) in objects {
            let meshes = if kind == Body::Rocky {
                &self.rocky
            } else {
                &self.spheres
            };
            let distance = camera.position.distance(body.center).max(0.1);
            let pixels = body.radius / distance * frame.height as f32 / (camera.fov * 0.5).tan();
            let lod = if !options.lod || pixels > 350.0 {
                3
            } else if pixels > 130.0 {
                2
            } else if pixels > 45.0 {
                1
            } else {
                0
            };
            frame.stats.triangles_available += meshes.last().unwrap().faces.len() as u64;
            if options.culling && !camera.sphere_visible(body.center, body.radius * 1.08, aspect) {
                frame.stats.objects_culled += 1;
                continue;
            }
            self.draw(
                frame,
                &meshes[lod],
                body,
                kind,
                vp,
                camera.position,
                ship.time as f32,
                false,
                options.normal_mapping,
            );
        }
        let gas = world::planet(1, ship.time);
        let ring_body = BodyState {
            rotation: world::ring_tilt(),
            ..gas
        };
        frame.stats.triangles_available += self.rings.faces.len() as u64;
        if !options.culling || camera.sphere_visible(gas.center, gas.radius * 2.0, aspect) {
            self.draw(
                frame,
                &self.rings,
                ring_body,
                Body::Ring,
                vp,
                camera.position,
                ship.time as f32,
                true,
                false,
            );
        } else {
            frame.stats.objects_culled += 1;
        }
        if !camera.front {
            frame.stats.triangles_available += self.ship.faces.len() as u64;
            self.draw(
                frame,
                &self.ship,
                BodyState {
                    center: ship.position,
                    radius: 1.0,
                    rotation: ship.rotation,
                },
                Body::Ship,
                vp,
                camera.position,
                ship.time as f32,
                false,
                false,
            );
        }
        let planet = world::planet(0, ship.time);
        let camera_distance = camera.position.distance(planet.center);
        for instance in &self.instances {
            let body = instance.body(planet);
            let bound = 12.0 * body.radius;
            frame.stats.triangles_available += self.props[instance.kind].faces.len() as u64;
            let facing = (body.center - planet.center)
                .normalize()
                .dot((camera.position - planet.center).normalize());
            if options.culling
                && (!camera.sphere_visible(body.center, bound, aspect)
                    || facing < planet.radius / camera_distance - 0.12)
            {
                frame.stats.objects_culled += 1;
                continue;
            }
            let projected = bound / camera.position.distance(body.center) * frame.height as f32;
            let m = if options.lod && projected < 7.0 {
                &self.low_props[instance.kind]
            } else {
                &self.props[instance.kind]
            };
            self.draw(
                frame,
                m,
                body,
                Body::Prop,
                vp,
                camera.position,
                ship.time as f32,
                false,
                false,
            );
        }
        if options.race.active && !options.race.finished {
            for offset in 0..6 {
                let index = (options.race.next + offset) % race::GATE_COUNT;
                let g = race::world_gate(index, ship.time);
                let mesh = &self.gates[usize::from(offset != 0)];
                frame.stats.triangles_available += mesh.faces.len() as u64;
                if options.culling && !camera.sphere_visible(g.center, g.radius, aspect) {
                    frame.stats.objects_culled += 1;
                    continue;
                }
                self.draw(
                    frame,
                    mesh,
                    BodyState {
                        center: g.center,
                        radius: g.radius,
                        rotation: glam::Quat::from_rotation_arc(Vec3::Y, g.normal),
                    },
                    Body::Gate,
                    vp,
                    camera.position,
                    ship.time as f32,
                    true,
                    false,
                );
            }
        }
        frame.flush();
        if !camera.front {
            self.exhaust(frame, ship, vp);
        }
        if options.ui {
            overlay(frame, camera, ship, options, vp);
        }
    }
    #[allow(clippy::too_many_arguments)]
    fn draw(
        &self,
        frame: &mut Framebuffer,
        mesh: &Mesh,
        body: BodyState,
        kind: Body,
        vp: Mat4,
        camera: Vec3,
        time: f32,
        two_sided: bool,
        normal_mapping: bool,
    ) {
        frame.stats.objects += 1;
        let vertices: Vec<_> = mesh
            .vertices
            .iter()
            .map(|v| {
                let world = body.rotation * v.position * body.radius + body.center;
                ClipVertex {
                    clip: vp * world.extend(1.0),
                    object: v.position,
                    world,
                    normal: body.rotation * v.normal,
                }
            })
            .collect();
        let mut u = Uniforms {
            body: kind,
            time,
            layers: 4,
            center: body.center,
            radius: body.radius,
            ring_normal: world::ring_tilt() * Vec3::Y,
            camera,
            tint: mesh.colors.first().copied().unwrap_or(Vec3::ONE),
            rotation: body.rotation,
            normal_mapping,
        };
        for (i, face) in mesh.faces.iter().enumerate() {
            let mut triangle = face.map(|id| vertices[id]);
            if mesh.flat {
                let n = (triangle[1].world - triangle[0].world)
                    .cross(triangle[2].world - triangle[0].world)
                    .normalize_or_zero();
                for v in &mut triangle {
                    v.normal = n;
                }
                u.tint = mesh.colors.get(i).copied().unwrap_or(Vec3::splat(0.7));
            }
            frame.submit(triangle, two_sided, u);
        }
    }
    fn halo(&self, frame: &mut Framebuffer, camera: &Camera, vp: Mat4) {
        let Some((x, y, _)) = project(Vec3::ZERO, vp, frame.width, frame.height) else {
            return;
        };
        let radius = world::star(0.0).radius / camera.position.length() * frame.height as f32 * 0.5
            / (camera.fov * 0.5).tan();
        let outer = (radius * 3.3).min(frame.width as f32 * 2.0);
        let minx = (x - outer).max(0.0) as usize;
        let maxx = (x + outer).min(frame.width as f32) as usize;
        let miny = (y - outer).max(0.0) as usize;
        let maxy = (y + outer).min(frame.height as f32) as usize;
        for py in miny..maxy {
            for px in minx..maxx {
                let dx = px as f32 - x;
                let dy = py as f32 - y;
                let d = (dx * dx + dy * dy).sqrt() / radius;
                if d < 0.95 {
                    continue;
                }
                let glow = (-(d - 1.0) * 7.0).exp() * 0.42 + (-(d - 1.0) * 2.3).exp() * 0.12;
                let i = py * frame.width + px;
                frame.pixels[i] =
                    to_rgb(from_rgb(frame.pixels[i]) + Vec3::new(1.0, 0.29, 0.05) * glow);
            }
        }
    }
    fn exhaust(&self, frame: &mut Framebuffer, ship: &Flight, vp: Mat4) {
        let power = 0.2 + ship.thrust_amount * 0.8 + ship.boost_amount;
        for side in [-0.65, 0.65] {
            for i in 0..10 {
                let t = i as f32 / 10.0;
                let position =
                    ship.position + ship.rotation * Vec3::new(side, -0.12, 1.8 + t * power * 2.8);
                if let Some((x, y, z)) = project(position, vp, frame.width, frame.height) {
                    let radius = (1.0 - t) * 2.3 + 0.5;
                    for dy in -4..=4 {
                        for dx in -4..=4 {
                            let (px, py) = (x as i32 + dx, y as i32 + dy);
                            if px < 0
                                || py < 0
                                || px >= frame.width as i32
                                || py >= frame.height as i32
                            {
                                continue;
                            }
                            let at = py as usize * frame.width + px as usize;
                            if z > frame.depth[at] + 0.000001 {
                                continue;
                            }
                            let strength = (-(dx * dx + dy * dy) as f32 / (radius * radius)).exp()
                                * (1.0 - t)
                                * 0.55;
                            frame.pixels[at] = to_rgb(
                                from_rgb(frame.pixels[at]) + Vec3::new(0.16, 0.48, 0.9) * strength,
                            );
                        }
                    }
                }
            }
        }
    }
}

pub fn project(point: Vec3, vp: Mat4, width: usize, height: usize) -> Option<(f32, f32, f32)> {
    let p = vp * point.extend(1.0);
    if p.w <= 0.01 {
        return None;
    }
    let n = p.truncate() / p.w;
    Some((
        (n.x * 0.5 + 0.5) * width as f32,
        (0.5 - n.y * 0.5) * height as f32,
        n.z * 0.5 + 0.5,
    ))
}

fn overlay(f: &mut Framebuffer, camera: &Camera, ship: &Flight, o: Options, vp: Mat4) {
    let white = 0xd9e4e9;
    let muted = 0x7d98aa;
    let mint = 0x70e1ca;
    let red = 0xe84149;
    let margin = (f.width / 32).max(12);
    let unit = if f.width >= 1400 { 2 } else { 1 };
    hud::rect(f, margin, margin, 3, 20, red);
    hud::text(f, "COSMOS / GP", margin + 13, margin, 2 * unit, white);
    hud::text(
        f,
        "ASTER R   /   FLIGHT DIVISION",
        margin + 14,
        margin + 23 * unit,
        unit,
        muted,
    );
    hud::text(
        f,
        &if o.muted {
            "AUDIO SILENCIADO  /  M".into()
        } else {
            format!(
                "MUS {:02}  FX {:02}{}",
                (o.music_volume * 100.0) as u32,
                (o.fx_volume * 100.0) as u32,
                if o.track_loaded { "  F1" } else { "" }
            )
        },
        margin + 14,
        margin + 39 * unit,
        unit,
        muted,
    );
    let (near, altitude) = world::closest(ship.position, ship.time);
    let status = if ship.local.is_some() {
        "ORBITAL"
    } else if ship.assist {
        "ASISTIDO"
    } else {
        "INERCIAL"
    };
    let top_right = f.width.saturating_sub(margin + 138 * unit);
    hud::text(f, "SISTEMAS NOMINALES", top_right, margin, unit, mint);
    hud::text(
        f,
        &format!("MODO {}", status),
        top_right,
        margin + 16 * unit,
        unit,
        muted,
    );
    let bottom = f.height.saturating_sub(margin + 80 * unit);
    hud::text(f, "VELOCIDAD", margin, bottom, unit, muted);
    hud::text(
        f,
        &format!("{:03}", ship.velocity.length().round() as u32),
        margin,
        bottom + 15 * unit,
        4 * unit,
        white,
    );
    hud::text(
        f,
        "M/S",
        margin + 80 * unit,
        bottom + 34 * unit,
        unit,
        muted,
    );
    hud::text(f, "IMPULSO", margin, bottom + 55 * unit, unit, muted);
    for i in 0..20 {
        hud::rect(
            f,
            margin + i * 6 * unit,
            bottom + 69 * unit,
            4 * unit,
            3 * unit,
            if (i as f32) < ship.energy * 20.0 {
                mint
            } else {
                0x1b303d
            },
        );
    }
    let navx = f.width.saturating_sub(margin + 155 * unit);
    hud::text(
        f,
        world::NAMES[near],
        navx,
        bottom + 13 * unit,
        2 * unit,
        white,
    );
    hud::text(
        f,
        &format!("ALTITUD {:06.1} M", altitude.max(0.0)),
        navx,
        bottom + 38 * unit,
        unit,
        muted,
    );
    hud::text(
        f,
        if ship.local.is_some() {
            "O  SALIR DE ORBITAL"
        } else if altitude >= world::planet(near, ship.time).radius * 2.5 {
            "ACERCATE PARA ORBITAR"
        } else {
            "O  ASISTENCIA ORBITAL"
        },
        navx,
        bottom + 58 * unit,
        unit,
        mint,
    );
    let (cx, cy) = (f.width as f32 * 0.5, f.height as f32 * 0.5);
    for (a, b) in [
        ((cx - 13.0, cy), (cx - 6.0, cy)),
        ((cx + 6.0, cy), (cx + 13.0, cy)),
        ((cx, cy - 13.0), (cx, cy - 6.0)),
        ((cx, cy + 6.0), (cx, cy + 13.0)),
    ] {
        f.line(a, b, muted, 0.6);
    }
    let target = if o.race.active && !o.race.finished {
        race::world_gate(o.race.next % race::GATE_COUNT, ship.time).center
    } else {
        world::planet(o.target, ship.time).center
    };
    let target_name = if o.race.active && !o.race.finished {
        format!("PUERTA {:02}", o.race.next % race::GATE_COUNT + 1)
    } else {
        world::NAMES[o.target].into()
    };
    let projected = project(target, vp, f.width, f.height).unwrap_or_else(|| {
        let local = camera.rotation.conjugate() * (target - camera.position);
        (
            if local.x >= 0.0 { f.width as f32 } else { 0.0 },
            f.height as f32 * 0.5,
            1.0,
        )
    });
    {
        let (x, y, _) = projected;
        let x = x.clamp(70.0, f.width as f32 - 110.0);
        let y = y.clamp(80.0, f.height as f32 - 130.0);
        for k in 0..4 {
            let t = k as f32 * std::f32::consts::FRAC_PI_2;
            let t2 = t + std::f32::consts::FRAC_PI_2;
            f.line(
                (x + t.cos() * 6.0, y + t.sin() * 6.0),
                (x + t2.cos() * 6.0, y + t2.sin() * 6.0),
                mint,
                0.5,
            );
        }
        hud::text(
            f,
            &target_name,
            (x + 13.0) as usize,
            (y - 4.0) as usize,
            unit,
            white,
        );
        hud::text(
            f,
            &format!("{:04.0} M", ship.position.distance(target)),
            (x + 13.0) as usize,
            (y + 9.0) as usize,
            unit,
            muted,
        );
    }
    {
        let text = if o.race.active {
            "W/S MOTOR  FLECHAS GIRAR  ESPACIO FRENO  N REINTENTAR  BORRAR VUELO LIBRE  H AYUDA"
        } else {
            "W/S MOTOR  FLECHAS GIRAR  ESPACIO FRENO  N CONTRARRELOJ  H CONTROLES"
        };
        let width = text.len() * 6 * unit;
        if width + 20 < f.width {
            hud::text(
                f,
                text,
                (f.width - width) / 2,
                f.height - 17 * unit,
                unit,
                muted,
            );
        }
    }
    if o.debug {
        let s = f.stats;
        for (i, line) in [
            format!("FPS {:.1}  {}X{}", o.fps, f.width, f.height),
            format!("TRIS {} / {}", s.triangles_submitted, s.triangles_available),
            format!("RASTER {} PIX {}", s.triangles_rasterized, s.fragments),
            format!(
                "CULL {} OBJ {}  LOD {}",
                s.objects_culled,
                s.objects,
                if o.lod { "ON" } else { "OFF" }
            ),
            format!("T {:.2}  IMPACTOS {}", ship.time, ship.impacts),
        ]
        .iter()
        .enumerate()
        {
            hud::text(f, line, margin, margin + 65 + i * 14, unit, 0xd6b66f);
        }
    }
    if o.race.active {
        let x = f.width.saturating_sub(252) / 2;
        hud::panel(f, x, 20, 252, 82, 0.82);
        hud::rect(
            f,
            x,
            20,
            252,
            2,
            if o.race.flash > 0.0 { mint } else { red },
        );
        hud::text(
            f,
            if o.race.finished {
                "VUELTA COMPLETA"
            } else {
                "MONACO / GRAND PRIX"
            },
            x + 16,
            32,
            1,
            muted,
        );
        hud::text(f, &race::format_time(o.race.elapsed), x + 16, 48, 3, white);
        hud::text(
            f,
            &format!("{:02}/24", o.race.next.saturating_sub(1).min(24)),
            x + 202,
            55,
            1,
            mint,
        );
        for i in 0..24 {
            hud::rect(
                f,
                x + 16 + i * 9,
                82,
                6,
                3,
                if i < o.race.next.saturating_sub(1) {
                    mint
                } else {
                    0x223b47
                },
            );
        }
        if let Some(best) = o.race.best {
            hud::text(
                f,
                &format!("MEJOR {}", race::format_time(best)),
                x + 16,
                92,
                1,
                muted,
            );
        }
        if o.race.countdown > 0.0 {
            let y = f.height as f32 * 0.36;
            hud::panel(f, x + 36, y as usize - 20, 180, 68, 0.8);
            for i in 0..5 {
                hud::circle(
                    f,
                    x as f32 + 66.0 + i as f32 * 30.0,
                    y,
                    8.0,
                    if (i as f32) < 5.0 - o.race.countdown * 1.6667 {
                        red
                    } else {
                        0x3a252b
                    },
                );
            }
            hud::text(
                f,
                "PREPARATE / W ACELERA",
                x + 65,
                y as usize + 22,
                1,
                white,
            );
        }
        if o.race.finished {
            hud::panel(f, x - 12, f.height / 2 - 30, 276, 74, 0.88);
            hud::text(f, "META", x + 105, f.height / 2 - 18, 3, mint);
            hud::text(
                f,
                "N OTRA VUELTA / BORRAR EXPLORAR",
                x + 32,
                f.height / 2 + 18,
                1,
                white,
            );
        }
    } else if !o.debug {
        let x = f.width.saturating_sub(margin + 174);
        hud::panel(f, x - 10, margin + 52, 184, 61, 0.52);
        hud::text(f, "MONACO GP", x, margin + 62, 1, white);
        hud::text(f, "N  IR A PARRILLA", x, margin + 80, 1, mint);
        hud::text(f, "24 PUERTAS / UNA VUELTA", x, margin + 96, 1, muted);
    }
    if o.help {
        let x = f.width.saturating_sub(650) / 2;
        let y = f.height.saturating_sub(330) / 2;
        hud::panel(f, x, y, 650, 330, 0.97);
        hud::rect(f, x, y, 4, 330, red);
        hud::text(f, "GUIA DE VUELO", x + 28, y + 24, 2, white);
        let lines = [
            "W / S        ACELERAR / RETROPROPULSION",
            "FLECHAS      ORIENTAR  /  RATON + BOTON DERECHO",
            "A D / R F    DESPLAZAR / SUBIR Y BAJAR",
            "Q / E        ALABEO",
            "SHIFT        IMPULSO     ESPACIO  FRENO",
            "ALT          DERRAPE     T  ASISTENCIA / INERCIA",
            "O            ORBITAL CERCA DE UN PLANETA",
            "C / TAB      CAMARA / DESTINO",
            "N / BORRAR   PARRILLA / EXPLORACION LIBRE",
            "[ ] / - +    VOLUMEN MUSICA / EFECTOS   M SILENCIO",
            "F3 F4 F5 F6  DATOS / CULLING / LOD / MAPA NORMAL",
            "G / F8       GRAVEDAD / TEMA OFICIAL EN NAVEGADOR",
            "P / U / F12  PAUSA / INSTRUMENTOS / CAPTURA",
        ];
        for (i, line) in lines.iter().enumerate() {
            hud::text(
                f,
                line,
                x + 28,
                y + 64 + i * 17,
                1,
                if i % 2 == 0 { white } else { muted },
            );
        }
        hud::text(f, "H PARA VOLVER AL VUELO", x + 28, y + 304, 1, mint);
    }
    if o.paused && !o.help {
        let x = f.width.saturating_sub(160) / 2;
        let y = f.height / 2 + 52;
        hud::rect(f, x - 12, y - 12, 184, 48, 0x0b1720);
        hud::text(f, "PAUSA", x + 50, y, 2, white);
        hud::text(f, "P PARA CONTINUAR", x + 35, y + 22, 1, muted);
    }
}
