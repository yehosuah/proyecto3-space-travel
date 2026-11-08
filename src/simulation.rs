use crate::world;
use glam::{Mat3, Quat, Vec3};

#[derive(Clone, Copy, Default)]
pub struct Input {
    pub thrust: f32,
    pub strafe: f32,
    pub lift: f32,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
    pub boost: bool,
    pub brake: bool,
    pub drift: bool,
}

pub struct Flight {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    pub angular: Vec3,
    pub time: f64,
    pub assist: bool,
    pub local: Option<usize>,
    pub gravity: bool,
    pub energy: f32,
    pub boost_amount: f32,
    pub thrust_amount: f32,
    pub impacts: u64,
    accumulator: f64,
}

impl Default for Flight {
    fn default() -> Self {
        Self::new()
    }
}
impl Flight {
    pub fn new() -> Self {
        let target = world::planet(0, 0.0).center;
        let position = target + Vec3::new(-46.0, 20.0, 58.0) * world::SCALE;
        Self {
            position,
            velocity: Vec3::ZERO,
            rotation: look_rotation(target - position, Vec3::Y),
            angular: Vec3::ZERO,
            time: 0.0,
            assist: true,
            local: None,
            gravity: true,
            energy: 1.0,
            boost_amount: 0.0,
            thrust_amount: 0.0,
            impacts: 0,
            accumulator: 0.0,
        }
    }
    pub fn forward(&self) -> Vec3 {
        self.rotation * -Vec3::Z
    }
    pub fn up(&self) -> Vec3 {
        self.rotation * Vec3::Y
    }
    pub fn update(&mut self, input: Input, elapsed: f64) {
        if !elapsed.is_finite() || elapsed <= 0.0 {
            return;
        }
        self.accumulator += elapsed;
        const STEP: f64 = 1.0 / 120.0;
        while self.accumulator + 1e-10 >= STEP {
            self.step(input, STEP as f32);
            self.accumulator -= STEP;
        }
    }
    pub fn toggle_local(&mut self) -> bool {
        if self.local.is_some() {
            self.local = None;
            return true;
        }
        let (i, altitude) = world::closest(self.position, self.time);
        if altitude < world::planet(i, self.time).radius * 2.5 {
            self.local = Some(i);
            true
        } else {
            false
        }
    }
    fn step(&mut self, input: Input, dt: f32) {
        let previous_time = self.time;
        self.time += 1.0 / 120.0;
        if let Some(i) = self.local {
            self.position +=
                world::planet(i, self.time).center - world::planet(i, previous_time).center;
            let normal = (self.position - world::planet(i, self.time).center).normalize();
            let correction = Quat::from_rotation_arc(self.up(), normal);
            self.rotation =
                Quat::IDENTITY.slerp(correction, 1.0 - (-2.2 * dt).exp()) * self.rotation;
        }
        let target_angular = Vec3::new(input.pitch, input.yaw, input.roll) * 1.6;
        self.angular = self.angular.lerp(target_angular, 1.0 - (-7.0 * dt).exp());
        self.rotation = (self.rotation * Quat::from_scaled_axis(self.angular * dt)).normalize();
        let boosting = input.boost && input.thrust > 0.0 && self.energy > 0.01;
        self.energy = (self.energy + if boosting { -dt * 0.14 } else { dt * 0.11 }).clamp(0.0, 1.0);
        self.boost_amount +=
            ((boosting as u8 as f32) - self.boost_amount) * (1.0 - (-8.0 * dt).exp());
        self.thrust_amount += (input.thrust.abs() - self.thrust_amount) * (1.0 - (-8.0 * dt).exp());
        let acceleration = if boosting { 110.0 } else { 23.0 };
        let local_input = Vec3::new(input.strafe * 0.7, input.lift * 0.7, -input.thrust);
        let mut thrust = self.rotation * local_input * acceleration;
        if let Some(i) = self.local {
            let relative = self.position - world::planet(i, self.time).center;
            let radial = relative.normalize();
            let lateral = self.velocity - radial * self.velocity.dot(radial);
            // Aceleración centrípeta y amortiguación radial; R/F siguen controlando altura.
            thrust -= radial * thrust.dot(radial);
            thrust += radial * input.lift * 16.0;
            thrust -= radial * lateral.length_squared() / relative.length();
            self.velocity -= radial * self.velocity.dot(radial) * (1.0 - (-3.5 * dt).exp());
        } else if self.gravity {
            for i in 0..2 {
                let p = world::planet(i, self.time);
                let delta = p.center - self.position;
                if delta.length() < p.radius * 6.0 {
                    thrust += delta.normalize_or_zero()
                        * (p.radius * p.radius * 1.2
                            / delta.length_squared().max(p.radius * p.radius));
                }
            }
        }
        self.velocity += thrust * dt;
        if input.brake {
            self.velocity *= (-4.5 * dt).exp();
        } else if self.assist && !input.drift {
            let f = self.forward();
            let longitudinal = f * self.velocity.dot(f);
            self.velocity = longitudinal * (-0.25 * dt).exp()
                + (self.velocity - longitudinal) * (-1.8 * dt).exp();
        }
        let limit = if boosting {
            195.0
        } else if input.drift || !self.assist {
            210.0
        } else {
            76.0
        };
        if self.velocity.length() > limit {
            self.velocity = self.velocity.normalize() * limit;
        }
        self.position += self.velocity * dt;
        for b in world::collision_bodies(self.time) {
            let relative = self.position - b.center;
            let minimum = b.radius + 1.1;
            if relative.length_squared() < minimum * minimum {
                let n = relative.try_normalize().unwrap_or(Vec3::Y);
                self.position = b.center + n * minimum;
                let into = self.velocity.dot(n);
                if into < 0.0 {
                    self.velocity -= n * into * 1.08;
                    if into < -2.0 {
                        self.impacts += 1;
                    }
                }
            }
        }
        if let Some(i) = self.local {
            if self.position.distance(world::planet(i, self.time).center)
                > world::planet(i, self.time).radius * 5.0
            {
                self.local = None;
            }
        }
    }
}

pub fn look_rotation(direction: Vec3, up: Vec3) -> Quat {
    let back = -direction.normalize_or_zero();
    let right = up.cross(back).try_normalize().unwrap_or(Vec3::X);
    let corrected_up = back.cross(right).normalize_or_zero();
    Quat::from_mat3(&Mat3::from_cols(right, corrected_up, back)).normalize()
}
