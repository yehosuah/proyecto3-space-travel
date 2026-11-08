//! Reproducible demonstration driven by the game's real flight inputs.
//! It never assigns the ship's position or velocity after initialization.
use cosmos_gp::{
    audio::Synth,
    camera::Camera,
    framebuffer::Framebuffer,
    hud,
    render::{Options, Renderer},
    simulation::{Flight, Input},
    world,
};
use glam::Vec3;
use std::{
    error::Error,
    f32::consts::{PI, TAU},
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
    process::{Command, Stdio},
};

#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    Intro,
    Approach,
    Orbit,
    Depart,
    End,
}
struct Pilot {
    stage: Stage,
    stage_time: f64,
    axis: Vec3,
    start: Vec3,
    last_angle: f32,
    travel: f32,
}
impl Pilot {
    fn new() -> Self {
        Self {
            stage: Stage::Intro,
            stage_time: 0.0,
            axis: Vec3::Y,
            start: Vec3::X,
            last_angle: 0.0,
            travel: 0.0,
        }
    }
    fn transition(&mut self, stage: Stage) {
        self.stage = stage;
        self.stage_time = 0.0;
    }
    fn input(&mut self, ship: &mut Flight, dt: f64) -> Input {
        self.stage_time += dt;
        let p = world::planet(0, ship.time);
        let relative = ship.position - p.center;
        let radial = relative.normalize();
        let altitude = relative.length() - p.radius;
        let steer = |direction: Vec3| {
            let local = ship.rotation.conjugate() * direction.normalize();
            Input {
                yaw: (-local.x.atan2(-local.z) * 2.5).clamp(-1.0, 1.0),
                pitch: (local
                    .y
                    .atan2((local.x * local.x + local.z * local.z).sqrt())
                    * 2.5)
                    .clamp(-1.0, 1.0),
                ..Input::default()
            }
        };
        match self.stage {
            Stage::Intro => {
                if self.stage_time >= 3.0 {
                    self.transition(Stage::Approach);
                }
                Input {
                    brake: true,
                    ..Input::default()
                }
            }
            Stage::Approach => {
                let mut input = steer(-radial);
                let speed = ship.velocity.dot(-radial);
                let target = if altitude > 190.0 { 65.0 } else { 28.0 };
                input.thrust = (target * 0.25 / 23.0 + (target - speed) * 0.10).clamp(0.0, 1.0);
                input.brake = speed > target + 8.0;
                if altitude < 64.0 {
                    ship.toggle_local();
                    self.start = radial;
                    let tangent = Vec3::Y.cross(radial).normalize();
                    self.axis = radial.cross(tangent).normalize();
                    self.last_angle = 0.0;
                    self.travel = 0.0;
                    self.transition(Stage::Orbit);
                }
                input
            }
            Stage::Orbit => {
                let tangent = self.axis.cross(radial).normalize();
                let mut input = steer(tangent - radial * 0.22);
                let speed = ship.velocity.dot(tangent);
                input.thrust = (62.0 * 0.25 / 23.0 + (62.0 - speed) * 0.10).clamp(0.0, 1.0);
                input.lift =
                    ((42.0 - altitude) * 0.28 - ship.velocity.dot(radial) * 0.7).clamp(-1.0, 1.0);
                let angle = self
                    .axis
                    .dot(self.start.cross(radial))
                    .atan2(self.start.dot(radial));
                let delta = (angle - self.last_angle + PI).rem_euclid(TAU) - PI;
                self.travel += delta;
                self.last_angle = angle;
                if self.travel > TAU + 0.04 {
                    self.transition(Stage::Depart);
                }
                input
            }
            Stage::Depart => {
                let mut input = steer(-radial);
                if self.stage_time < 2.5 {
                    input.brake = true;
                } else {
                    if ship.local.is_some() {
                        ship.toggle_local();
                    }
                    input.thrust = -1.0;
                }
                if altitude > 690.0 && self.stage_time > 10.0 {
                    self.transition(Stage::End);
                }
                input
            }
            Stage::End => Input {
                brake: true,
                ..Input::default()
            },
        }
    }
    fn caption(&self) -> (&'static str, &'static str) {
        match self.stage {
            Stage::Intro => ("COSMOS / GP", "ASTER R  /  EL ESPACIO ES LA PISTA"),
            Stage::Approach => ("01 / APROXIMACION", "ACELERACION Y FRENADO PROGRESIVOS"),
            Stage::Orbit => (
                "02 / VUELTA A MONACO",
                "SUPERFICIE PROCEDURAL / SIN TRANSICIONES",
            ),
            Stage::Depart => ("03 / ALEJAMIENTO", "VUELO LIBRE / ORIENTACION CONTINUA"),
            Stage::End => (
                "COSMOS / GP",
                "RECORRIDO COMPLETO  /  RENDERIZADOR PROPIO EN CPU",
            ),
        }
    }
}

fn wav(path: &std::path::Path, samples: &[i16]) -> std::io::Result<()> {
    let bytes = (samples.len() * 2) as u32;
    let mut f = BufWriter::new(File::create(path)?);
    f.write_all(b"RIFF")?;
    f.write_all(&(36 + bytes).to_le_bytes())?;
    f.write_all(b"WAVEfmt ")?;
    f.write_all(&16u32.to_le_bytes())?;
    for n in [1u16, 1u16] {
        f.write_all(&n.to_le_bytes())?;
    }
    f.write_all(&48000u32.to_le_bytes())?;
    f.write_all(&96000u32.to_le_bytes())?;
    f.write_all(&2u16.to_le_bytes())?;
    f.write_all(&16u16.to_le_bytes())?;
    f.write_all(b"data")?;
    f.write_all(&bytes.to_le_bytes())?;
    for s in samples {
        f.write_all(&s.to_le_bytes())?;
    }
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    let dry = std::env::args().any(|a| a == "--simulate-only");
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let output = root.join("deliverable");
    fs::create_dir_all(&output)?;
    let mut ship = Flight::new();
    let mut camera = Camera::new(&ship);
    let mut pilot = Pilot::new();
    let mut renderer = if dry {
        None
    } else {
        Some(Renderer::new(&root)?)
    };
    let mut frame = Framebuffer::new(1440, 900);
    let fps = 30usize;
    let dt = 1.0 / fps as f64;
    let mut encoder = if dry {
        None
    } else {
        Some(
            Command::new(std::env::var("COSMOS_FFMPEG").unwrap_or_else(|_| "ffmpeg".into()))
                .args([
                    "-hide_banner",
                    "-loglevel",
                    "error",
                    "-y",
                    "-f",
                    "rawvideo",
                    "-pix_fmt",
                    "rgb24",
                    "-s",
                    "1440x900",
                    "-r",
                    "30",
                    "-i",
                    "-",
                    "-an",
                    "-c:v",
                    "libx264",
                    "-preset",
                    "fast",
                    "-crf",
                    "19",
                    "-pix_fmt",
                    "yuv420p",
                    "-movflags",
                    "+faststart",
                ])
                .arg(output.join("vuelo-sin-audio.mp4"))
                .stdin(Stdio::piped())
                .spawn()?,
        )
    };
    let mut pipe = encoder.as_mut().map(|e| e.stdin.take().unwrap());
    let mut rgb = vec![0u8; 1440 * 900 * 3];
    let mut trace = BufWriter::new(File::create(output.join("trayectoria.csv"))?);
    writeln!(
        trace,
        "frame,time,stage,x,y,z,altitude,speed,orbit_radians,impacts,thrust,yaw,pitch,lift"
    )?;
    let mut min_alt = f32::INFINITY;
    let mut max_step = 0.0f32;
    let mut frames = 0;
    let mut synth = Synth::default();
    let mut audio = Vec::new();
    let mut pcm = vec![0.0; 48000 / fps];
    let mut last_stage = Stage::Intro;
    for index in 0..fps * 100 {
        let previous = ship.position;
        let input = pilot.input(&mut ship, dt);
        ship.update(input, dt);
        camera.update(&ship, dt as f32);
        let altitude = ship.position.distance(world::planet(0, ship.time).center)
            - world::planet(0, ship.time).radius;
        min_alt = min_alt.min(altitude);
        max_step = max_step.max(previous.distance(ship.position));
        frames = index + 1;
        if pilot.stage != last_stage {
            println!(
                "{:.2}s {:?} altitude={altitude:.2} orbit={:.3}",
                ship.time, pilot.stage, pilot.travel
            );
            last_stage = pilot.stage;
        }
        writeln!(
            trace,
            "{index},{:.5},{:?},{:.4},{:.4},{:.4},{:.3},{:.3},{:.6},{},{:.3},{:.3},{:.3},{:.3}",
            ship.time,
            pilot.stage,
            ship.position.x,
            ship.position.y,
            ship.position.z,
            altitude,
            ship.velocity.length(),
            pilot.travel,
            ship.impacts,
            input.thrust,
            input.yaw,
            input.pitch,
            input.lift
        )?;
        if let Some(renderer) = &mut renderer {
            renderer.render(&mut frame, &camera, &ship, Options::default());
            let (title, subtitle) = pilot.caption();
            if pilot.stage == Stage::Intro || pilot.stage == Stage::End {
                let x = frame.width / 2 - 330;
                let y = frame.height / 2 - 70;
                hud::panel(&mut frame, x, y, 660, 125, 0.82);
                hud::rect(&mut frame, x, y, 4, 125, 0xe84149);
                hud::text(&mut frame, title, x + 60, y + 25, 5, 0xe1e9ec);
                hud::text(&mut frame, subtitle, x + 60, y + 87, 2, 0x81cfc3);
            } else {
                let x = frame.width / 2 - 350;
                let y = frame.height - 145;
                hud::panel(&mut frame, x, y, 700, 76, 0.82);
                hud::text(&mut frame, title, x + 20, y + 13, 2, 0xe1e9ec);
                hud::text(&mut frame, subtitle, x + 20, y + 46, 2, 0x81cfc3);
                if pilot.stage == Stage::Orbit {
                    hud::text(
                        &mut frame,
                        &format!(
                            "ARCO {:03} / 360",
                            (pilot.travel.to_degrees()).min(360.0) as u32
                        ),
                        x + 495,
                        y + 18,
                        1,
                        0xdcb470,
                    );
                }
            }
            for (color, out) in frame.pixels.iter().zip(rgb.chunks_exact_mut(3)) {
                out[0] = (color >> 16) as u8;
                out[1] = (color >> 8) as u8;
                out[2] = *color as u8;
            }
            pipe.as_mut().unwrap().write_all(&rgb)?;
            synth.fill(
                &mut pcm,
                ship.velocity.length(),
                ship.thrust_amount,
                ship.boost_amount,
                true,
            );
            audio.extend(pcm.iter().map(|v| (v * 32767.0) as i16));
            if index % 300 == 0 {
                println!("captured {index} frames");
            }
        }
        if pilot.stage == Stage::End && pilot.stage_time >= 3.0 {
            break;
        }
    }
    drop(pipe);
    if let Some(mut e) = encoder {
        if !e.wait()?.success() {
            return Err("No se pudo codificar el video".into());
        }
    }
    let final_alt = ship.position.distance(world::planet(0, ship.time).center)
        - world::planet(0, ship.time).radius;
    println!("RESULT duration={:.2} min_altitude={min_alt:.2} final_altitude={final_alt:.2} orbit_degrees={:.2} max_step={max_step:.3} impacts={}",frames as f64/fps as f64,pilot.travel.to_degrees(),ship.impacts);
    if pilot.stage != Stage::End
        || pilot.travel < TAU
        || min_alt > 90.0
        || final_alt < 600.0
        || max_step > 8.0
        || ship.impacts > 0
    {
        return Err("Recorrido incompleto; revisar trayectoria.csv".into());
    }
    fs::write(output.join("video-evidencia.json"),format!("{{\n  \"duration_seconds\": {:.3},\n  \"frames\": {frames},\n  \"fps\": {fps},\n  \"minimum_altitude\": {min_alt:.4},\n  \"final_altitude\": {final_alt:.4},\n  \"orbit_degrees\": {:.4},\n  \"maximum_step_distance\": {max_step:.4},\n  \"impacts\": {},\n  \"capture\": \"deterministic inputs through the same flight simulation\"\n}}\n",frames as f64/fps as f64,pilot.travel.to_degrees(),ship.impacts))?;
    if !dry {
        wav(&output.join("propulsores.wav"), &audio)?;
    }
    Ok(())
}
