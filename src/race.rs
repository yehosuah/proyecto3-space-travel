//! Contrarreloj secuencial; se cruza el plano de cada puerta en el sentido correcto.
use crate::{
    simulation::{look_rotation, Flight},
    world,
};
use glam::Vec3;
use std::f32::consts::TAU;

pub const GATE_COUNT: usize = 24;
#[derive(Clone, Copy, Debug)]
pub struct Gate {
    pub center: Vec3,
    pub normal: Vec3,
    pub radius: f32,
}
fn route(t: f32) -> Vec3 {
    let a = 2.4 + t * TAU;
    let latitude = 0.13 * (a * 2.0).sin() + 0.055 * (a * 5.0).sin();
    let radius = world::planet(0, 0.0).radius + 68.0 + 18.0 * (a * 3.0).sin();
    Vec3::new(
        a.cos() * latitude.cos(),
        latitude.sin(),
        a.sin() * latitude.cos(),
    ) * radius
}
pub fn gate(index: usize) -> Gate {
    let t = index as f32 / GATE_COUNT as f32;
    Gate {
        center: route(t),
        normal: (route(t + 0.001) - route(t - 0.001)).normalize(),
        radius: if index < 5 {
            23.0
        } else if index < 14 {
            18.0
        } else {
            14.0
        },
    }
}
pub fn world_gate(index: usize, time: f64) -> Gate {
    let p = world::planet(0, time);
    let g = gate(index);
    Gate {
        center: p.center + p.rotation * g.center,
        normal: p.rotation * g.normal,
        radius: g.radius,
    }
}
pub fn crossing(a: Vec3, b: Vec3, g: Gate) -> Option<f32> {
    let da = (a - g.center).dot(g.normal);
    let db = (b - g.center).dot(g.normal);
    if da > 0.0 || db <= 0.0 || db - da < 1e-6 {
        return None;
    }
    let fraction = -da / (db - da);
    ((a.lerp(b, fraction) - g.center).length() <= g.radius).then_some(fraction)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Countdown,
    Running,
    Finished,
}
#[derive(Clone, Copy, Default, Debug)]
pub struct View {
    pub active: bool,
    pub running: bool,
    pub finished: bool,
    pub countdown: f32,
    pub next: usize,
    pub elapsed: f64,
    pub best: Option<f64>,
    pub flash: f32,
}
pub struct Race {
    pub phase: Phase,
    pub next: usize,
    pub elapsed: f64,
    pub best: Option<f64>,
    pub countdown: f32,
    pub flash: f32,
    pub new_best: bool,
}
impl Default for Race {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            next: 0,
            elapsed: 0.0,
            best: None,
            countdown: 0.0,
            flash: 0.0,
            new_best: false,
        }
    }
}
impl Race {
    pub fn prepare(&mut self, ship: &mut Flight) {
        self.phase = Phase::Countdown;
        self.countdown = 3.0;
        self.elapsed = 0.0;
        self.next = 0;
        self.flash = 0.0;
        self.new_best = false;
        ship.local = Some(0);
        ship.assist = true;
        ship.energy = 1.0;
        self.place_start(ship);
    }
    fn place_start(&self, ship: &mut Flight) {
        let p = world::planet(0, ship.time);
        let g = world_gate(0, ship.time);
        ship.position = g.center - g.normal * 65.0;
        ship.velocity = Vec3::ZERO;
        ship.angular = Vec3::ZERO;
        ship.rotation = look_rotation(g.normal, (ship.position - p.center).normalize());
    }
    pub fn cancel(&mut self) {
        self.phase = Phase::Idle;
    }
    /// Returns a cue number: 1 gate, 2 finish, 3 green lights.
    pub fn update(&mut self, ship: &mut Flight, previous: Vec3, previous_time: f64, dt: f64) -> u8 {
        self.flash = (self.flash - dt as f32).max(0.0);
        if self.phase == Phase::Countdown {
            self.countdown -= dt as f32;
            self.place_start(ship);
            if self.countdown <= 0.0 {
                self.phase = Phase::Running;
                return 3;
            }
            return 0;
        }
        if self.phase != Phase::Running {
            return 0;
        }
        let a = world::planet(0, previous_time);
        let b = world::planet(0, ship.time);
        let from = a.rotation.conjugate() * (previous - a.center);
        let to = b.rotation.conjugate() * (ship.position - b.center);
        let mut cue = 0;
        if let Some(fraction) = crossing(from, to, gate(self.next % GATE_COUNT)) {
            self.next += 1;
            self.flash = 0.5;
            cue = 1;
            if self.next > GATE_COUNT {
                self.elapsed += dt * fraction as f64;
                self.phase = Phase::Finished;
                self.new_best = self.best.is_none_or(|best| self.elapsed < best);
                if self.new_best {
                    self.best = Some(self.elapsed);
                }
                return 2;
            }
        }
        self.elapsed += dt;
        cue
    }
    pub fn view(&self) -> View {
        View {
            active: self.phase != Phase::Idle,
            running: self.phase == Phase::Running,
            finished: self.phase == Phase::Finished,
            countdown: if self.phase == Phase::Countdown {
                self.countdown
            } else {
                0.0
            },
            next: self.next,
            elapsed: self.elapsed,
            best: self.best,
            flash: self.flash,
        }
    }
}
pub fn format_time(time: f64) -> String {
    let ms = (time.max(0.0) * 1000.0).round() as u64;
    format!("{:02}:{:02}.{:03}", ms / 60000, (ms / 1000) % 60, ms % 1000)
}
