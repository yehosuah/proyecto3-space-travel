use cosmos_gp::{
    camera::Camera,
    framebuffer::Framebuffer,
    race::{Phase, Race},
    render::{Options as RenderOptions, Renderer},
    simulation::{look_rotation, Flight},
    world,
};
use glam::Vec3;
use std::{
    error::Error,
    path::{Path, PathBuf},
    time::Instant,
};

struct Options {
    width: usize,
    height: usize,
    headless: bool,
    output: Option<PathBuf>,
    shot: String,
    benchmark: usize,
    frames: Option<usize>,
    ui: bool,
    music: Option<String>,
    lod: bool,
    culling: bool,
    normal_mapping: bool,
}
fn main() -> Result<(), Box<dyn Error>> {
    let Some(o) = parse()? else {
        return Ok(());
    };
    let renderer = Renderer::new(&resource_root()?)?;
    let mut ship = Flight::new();
    let mut camera = Camera::new(&ship);
    set_shot(&o.shot, &mut ship, &mut camera);
    let mut frame = Framebuffer::new(o.width, o.height);
    let mut options = RenderOptions {
        ui: o.ui,
        lod: o.lod,
        culling: o.culling,
        normal_mapping: o.normal_mapping,
        ..RenderOptions::default()
    };
    if o.shot == "circuito" {
        let mut race = Race::default();
        race.prepare(&mut ship);
        camera = Camera::new(&ship);
        camera.update(&ship, 1.0);
        options.race = race.view();
    }
    renderer.render(&mut frame, &camera, &ship, options);
    if let Some(path) = &o.output {
        save(&frame, path)?;
    }
    if o.benchmark > 0 {
        let mut durations = Vec::new();
        for i in 0..o.benchmark {
            ship.update(Default::default(), 1.0 / 60.0);
            camera.update(&ship, 1.0 / 60.0);
            let start = Instant::now();
            renderer.render(&mut frame, &camera, &ship, options);
            if i >= 3 {
                durations.push(start.elapsed().as_secs_f64() * 1000.0);
            }
        }
        durations.sort_by(f64::total_cmp);
        let mean = durations.iter().sum::<f64>() / durations.len() as f64;
        let p95 = durations[((durations.len() as f64 * 0.95) as usize).min(durations.len() - 1)];
        println!("BENCHMARK CPU {}x{} frames={} mean_ms={mean:.2} p95_ms={p95:.2} theoretical_render_fps={:.1} tris={} fragments={}",o.width,o.height,durations.len(),1000.0/mean,frame.stats.triangles_submitted,frame.stats.fragments);
    }
    if !o.headless {
        run_window(renderer, frame, ship, camera, o)?;
    }
    Ok(())
}
fn parse() -> Result<Option<Options>, Box<dyn Error>> {
    let mut o = Options {
        width: 960,
        height: 600,
        headless: false,
        output: None,
        shot: "inicio".into(),
        benchmark: 0,
        frames: None,
        ui: true,
        music: None,
        lod: true,
        culling: true,
        normal_mapping: true,
    };
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--headless" | "--solo-render" => o.headless = true,
            "--width" | "--ancho" => o.width = args.next().ok_or("Falta ancho")?.parse()?,
            "--height" | "--alto" => o.height = args.next().ok_or("Falta alto")?.parse()?,
            "--output" | "--salida" => o.output = Some(args.next().ok_or("Falta ruta")?.into()),
            "--shot" | "--vista" => o.shot = args.next().ok_or("Falta vista")?,
            "--benchmark" => o.benchmark = args.next().ok_or("Falta cantidad")?.parse()?,
            "--frames" => o.frames = Some(args.next().ok_or("Falta cantidad")?.parse()?),
            "--no-ui" => o.ui = false,
            "--no-lod" => o.lod = false,
            "--no-culling" => o.culling = false,
            "--no-normal-map" => o.normal_mapping = false,
            "--music" => o.music = Some(args.next().ok_or("Falta archivo de música")?),
            "--help" | "--ayuda" | "-h" => {
                println!("COSMOS / GP — ASTER R\n\ncargo run --release --locked\n\n--headless --shot inicio|monaco|superficie|puerto|rebufo|anillos|circuito\n--output captura.png --width 960 --height 600\n--benchmark 120     Medición de CPU, sin ventana\n--frames N          Cerrar después de N cuadros\n--no-ui             Ocultar instrumentos\n--no-lod --no-culling --no-normal-map  Comparar optimizaciones\n--music archivo     Música local opcional\n\nW/S motor | Flechas orientar | A/D desplazar | R/F subir/bajar\nQ/E roll | Shift impulso | Espacio freno | Alt derrape\nT asistencia | O modo orbital | C cámara | Tab destino\nH ayuda | P pausa | F3 diagnóstico | F4 culling | F5 LOD | F6 normales\nN parrilla | Borrar vuelo libre | [ ] música | - = efectos | M silencio\nF8 tema oficial F1 | F12 captura | Esc salir");
                return Ok(None);
            }
            _ => return Err(format!("Opción desconocida: {a}").into()),
        }
    }
    if !(320..=2560).contains(&o.width) || !(200..=1600).contains(&o.height) {
        return Err("Resolución permitida: 320..2560 por 200..1600".into());
    }
    if o.benchmark > 0 && o.benchmark < 5 {
        return Err("Benchmark requiere al menos 5 cuadros".into());
    }
    if ![
        "inicio",
        "monaco",
        "superficie",
        "rebufo",
        "anillos",
        "circuito",
        "puerto",
    ]
    .contains(&o.shot.as_str())
    {
        return Err("Vista desconocida; consulta --help".into());
    }
    Ok(Some(o))
}
fn set_shot(name: &str, ship: &mut Flight, camera: &mut Camera) {
    if name == "puerto" {
        let p = world::planet(0, ship.time);
        let i = cosmos_gp::surface::distribute(world::SURFACE_SEED)
            .into_iter()
            .filter(|i| i.kind == 1)
            .max_by(|a, b| {
                a.direction
                    .dot(Vec3::new(-0.8, -0.25, 0.4))
                    .total_cmp(&b.direction.dot(Vec3::new(-0.8, -0.25, 0.4)))
            })
            .unwrap();
        let body = i.body(p);
        let up = p.rotation * i.direction;
        let tangent = up.cross(Vec3::Y).normalize();
        ship.position = body.center + up * 48.0 - tangent * 85.0;
        ship.rotation = look_rotation(body.center + tangent * 30.0 - ship.position, up);
        *camera = Camera::new(ship);
        camera.update(ship, 1.0);
        return;
    }
    let p = world::planet(
        if name == "rebufo" || name == "anillos" {
            1
        } else {
            0
        },
        ship.time,
    );
    let offset = match name {
        "monaco" => Vec3::new(-46.0, 20.0, 58.0),
        "superficie" => Vec3::new(-34.0, 13.0, 17.0),
        "rebufo" => Vec3::new(-123.0, 65.0, 160.0),
        "anillos" => Vec3::new(-90.0, 18.0, 58.0),
        _ => return,
    };
    ship.position = p.center + offset * world::SCALE;
    ship.rotation = look_rotation(p.center - ship.position, Vec3::Y);
    *camera = Camera::new(ship);
    camera.rotation = look_rotation(p.center - camera.position, Vec3::Y);
}
fn save(frame: &Framebuffer, path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    frame.write_png(path)?;
    println!("Captura: {}", path.display());
    Ok(())
}

/// Prefer resources alongside the executable, so a copied .app is self-contained.
fn resource_root() -> Result<PathBuf, Box<dyn Error>> {
    let executable = std::env::current_exe()?;
    let directory = executable.parent().ok_or("Ejecutable sin directorio")?;
    if directory.ends_with("Contents/MacOS") {
        let resources = directory.join("../Resources");
        return resources
            .join("assets/models/Nave_Aster.obj")
            .is_file()
            .then_some(resources)
            .ok_or_else(|| {
                "Paquete incompleto: falta la Nave Aster en Contents/Resources/assets/models".into()
            });
    }
    let candidates = [
        directory.join("../Resources"),
        directory.to_path_buf(),
        PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    ];
    candidates
        .into_iter()
        .find(|root| root.join("assets/models/Nave_Aster.obj").is_file())
        .ok_or_else(|| "No se encuentran assets/models/Nave_Aster.obj junto al programa".into())
}

fn capture_path() -> Result<PathBuf, Box<dyn Error>> {
    let executable = std::env::current_exe()?;
    let directory = executable.parent().ok_or("Ejecutable sin directorio")?;
    let output = if directory.ends_with("Contents/MacOS") {
        directory.join("../../..")
    } else {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    };
    Ok(output.join("renders/captura.png"))
}

#[cfg(feature = "window")]
fn run_window(
    renderer: Renderer,
    mut frame: Framebuffer,
    mut ship: Flight,
    mut camera: Camera,
    o: Options,
) -> Result<(), Box<dyn Error>> {
    use cosmos_gp::simulation::Input;
    use raylib::prelude::{
        Color, Image, KeyboardKey, MouseButton, RaylibAudio, RaylibDraw, RaylibTexture2D,
        Rectangle, TextureFilter, Vector2,
    };
    let (mut window, thread) = raylib::init()
        .size(1440, 900)
        .title("COSMOS / GP — ASTER R")
        .resizable()
        .build();
    window.set_target_fps(60);
    let image = Image::gen_image_color(frame.width as i32, frame.height as i32, Color::BLACK);
    let mut texture = window.load_texture_from_image(&thread, &image)?;
    texture.set_texture_filter(&thread, TextureFilter::TEXTURE_FILTER_BILINEAR);
    let mut rgba = vec![255u8; frame.width * frame.height * 4];
    let mut options = RenderOptions {
        ui: o.ui,
        lod: o.lod,
        culling: o.culling,
        normal_mapping: o.normal_mapping,
        ..RenderOptions::default()
    };
    let mut previous = Instant::now();
    let mut count = 0;
    let mut paused = false;
    let mut fps = 60.0f32;
    let device = match RaylibAudio::init_audio_device() {
        Ok(device) => Some(device),
        Err(e) => {
            eprintln!("Audio no disponible: {e:?}");
            None
        }
    };
    let music_path = o
        .music
        .as_ref()
        .map(PathBuf::from)
        .unwrap_or(resource_root()?.join("assets/audio/local.mp3"));
    let mut audio = device.as_ref().map(|device| {
        cosmos_gp::audio::Audio::new(device, music_path.is_file().then_some(music_path.as_path()))
    });
    let best_path = capture_path()?
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("session-best.txt");
    let mut race = Race::load(&best_path);
    if o.shot == "circuito" {
        race.prepare(&mut ship);
        camera = Camera::new(&ship);
    }
    while !window.window_should_close() {
        let now = Instant::now();
        let dt = now.duration_since(previous).as_secs_f64();
        previous = now;
        fps = fps * 0.93 + (1.0 / dt.max(0.001)) as f32 * 0.07;
        options.fps = fps;
        if window.is_key_pressed(KeyboardKey::KEY_P) {
            paused = !paused;
        }
        if window.is_key_pressed(KeyboardKey::KEY_T) {
            ship.assist = !ship.assist;
        }
        if window.is_key_pressed(KeyboardKey::KEY_O) {
            ship.toggle_local();
        }
        if window.is_key_pressed(KeyboardKey::KEY_C) {
            camera.front = !camera.front;
        }
        if window.is_key_pressed(KeyboardKey::KEY_TAB) {
            options.target = (options.target + 1) % 2;
        }
        if window.is_key_pressed(KeyboardKey::KEY_H) {
            options.help = !options.help;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F3) {
            options.debug = !options.debug;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F4) {
            options.culling = !options.culling;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F5) {
            options.lod = !options.lod;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F6) {
            options.normal_mapping = !options.normal_mapping;
        }
        if window.is_key_pressed(KeyboardKey::KEY_G) {
            ship.gravity = !ship.gravity;
        }
        if window.is_key_pressed(KeyboardKey::KEY_U) {
            options.ui = !options.ui;
        }
        if window.is_key_pressed(KeyboardKey::KEY_N) {
            race.prepare(&mut ship);
            camera = Camera::new(&ship);
            paused = false;
            options.help = false;
        }
        if window.is_key_pressed(KeyboardKey::KEY_BACKSPACE) {
            race.cancel();
        }
        if let Some(a) = &mut audio {
            if window.is_key_pressed(KeyboardKey::KEY_M) {
                a.muted = !a.muted;
            }
            if window.is_key_pressed(KeyboardKey::KEY_LEFT_BRACKET) {
                a.music_volume = (a.music_volume - 0.07).max(0.0);
            }
            if window.is_key_pressed(KeyboardKey::KEY_RIGHT_BRACKET) {
                a.music_volume = (a.music_volume + 0.07).min(1.0);
            }
            if window.is_key_pressed(KeyboardKey::KEY_MINUS) {
                a.fx_volume = (a.fx_volume - 0.07).max(0.0);
            }
            if window.is_key_pressed(KeyboardKey::KEY_EQUAL) {
                a.fx_volume = (a.fx_volume + 0.07).min(1.0);
            }
            options.music_volume = a.music_volume;
            options.fx_volume = a.fx_volume;
            options.muted = a.muted;
            options.track_loaded = a.track_loaded;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F8) {
            std::process::Command::new("open")
                .arg("https://www.youtube.com/watch?v=8AYy-BcjRXg")
                .spawn()?;
        }
        if window.is_key_pressed(KeyboardKey::KEY_F12) {
            save(&frame, &capture_path()?)?;
        }
        options.paused = paused || !window.is_window_focused() || options.help;
        let mut cue = 0;
        if !options.paused {
            let axis = |positive, negative| {
                (window.is_key_down(positive) as i32 - window.is_key_down(negative) as i32) as f32
            };
            let mouse = window.get_mouse_delta();
            let looking = window.is_mouse_button_down(MouseButton::MOUSE_BUTTON_RIGHT);
            let mouse_scale = 0.00042 / dt.max(1.0 / 240.0) as f32;
            let input = Input {
                thrust: axis(KeyboardKey::KEY_W, KeyboardKey::KEY_S),
                strafe: axis(KeyboardKey::KEY_D, KeyboardKey::KEY_A),
                lift: axis(KeyboardKey::KEY_R, KeyboardKey::KEY_F),
                yaw: axis(KeyboardKey::KEY_LEFT, KeyboardKey::KEY_RIGHT)
                    - if looking { mouse.x * mouse_scale } else { 0.0 },
                pitch: axis(KeyboardKey::KEY_UP, KeyboardKey::KEY_DOWN)
                    - if looking { mouse.y * mouse_scale } else { 0.0 },
                roll: axis(KeyboardKey::KEY_Q, KeyboardKey::KEY_E),
                boost: window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT),
                brake: window.is_key_down(KeyboardKey::KEY_SPACE),
                drift: window.is_key_down(KeyboardKey::KEY_LEFT_ALT),
            };
            let previous_position = ship.position;
            let previous_time = ship.time;
            ship.update(
                if race.phase == Phase::Countdown {
                    Input::default()
                } else {
                    input
                },
                dt,
            );
            cue = race.update(&mut ship, previous_position, previous_time, dt);
            if cue == 2 {
                if let Err(e) = race.save(&best_path) {
                    eprintln!("No se pudo guardar la marca: {e}");
                }
            }
            camera.update(&ship, dt as f32);
        }
        options.race = race.view();
        if let Some(a) = &mut audio {
            a.tick(&ship, options.paused, cue);
        }
        renderer.render(&mut frame, &camera, &ship, options);
        for (pixel, rgba) in frame.pixels.iter().zip(rgba.chunks_exact_mut(4)) {
            rgba[0] = (pixel >> 16) as u8;
            rgba[1] = (pixel >> 8) as u8;
            rgba[2] = *pixel as u8;
        }
        texture.update_texture(&rgba)?;
        let width = window.get_screen_width() as f32;
        let height = window.get_screen_height() as f32;
        let scale = (width / frame.width as f32).min(height / frame.height as f32);
        let (w, h) = (frame.width as f32 * scale, frame.height as f32 * scale);
        {
            let mut drawing = window.begin_drawing(&thread);
            drawing.clear_background(Color::BLACK);
            drawing.draw_texture_pro(
                &texture,
                Rectangle::new(0.0, 0.0, frame.width as f32, frame.height as f32),
                Rectangle::new((width - w) * 0.5, (height - h) * 0.5, w, h),
                Vector2::zero(),
                0.0,
                Color::WHITE,
            );
        }
        count += 1;
        if o.frames.is_some_and(|n| count >= n) {
            break;
        }
    }
    Ok(())
}
#[cfg(not(feature = "window"))]
fn run_window(
    _: Renderer,
    _: Framebuffer,
    _: Flight,
    _: Camera,
    _: Options,
) -> Result<(), Box<dyn Error>> {
    Err("Compila con window o usa --headless".into())
}
