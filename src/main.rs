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

fn run_window(
    _: Renderer,
    _: Framebuffer,
    _: Flight,
    _: Camera,
    _: Options,
) -> Result<(), Box<dyn Error>> {
    Err("Compila con window o usa --headless".into())
}
