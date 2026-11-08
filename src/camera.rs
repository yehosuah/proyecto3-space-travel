use crate::{
    simulation::{look_rotation, Flight},
    world,
};
use glam::{Mat4, Quat, Vec3, Vec4};

pub struct Camera {
    pub position: Vec3,
    pub rotation: Quat,
    pub fov: f32,
    pub front: bool,
}
impl Camera {
    pub fn new(ship: &Flight) -> Self {
        Self {
            position: ship.position - ship.forward() * 9.0 + ship.up() * 2.8,
            rotation: ship.rotation,
            fov: 62.0_f32.to_radians(),
            front: false,
        }
    }
    pub fn update(&mut self, ship: &Flight, dt: f32) {
        let target = if self.front {
            ship.position + ship.forward() * 2.4 + ship.up() * 0.3
        } else {
            ship.position - ship.forward() * (9.0 + ship.boost_amount * 2.0) + ship.up() * 2.8
        };
        let smoothing = if self.front { 35.0 } else { 8.0 };
        self.position = self.position.lerp(target, 1.0 - (-smoothing * dt).exp());
        for b in world::collision_bodies(ship.time) {
            let p = self.position - b.center;
            if p.length() < b.radius + 0.8 {
                self.position = b.center + p.normalize_or_zero() * (b.radius + 0.8);
            }
        }
        let desired = if self.front {
            ship.rotation
        } else {
            look_rotation(
                ship.position + ship.forward() * 22.0 - self.position,
                ship.up(),
            )
        };
        self.rotation = self.rotation.slerp(desired, 1.0 - (-12.0 * dt).exp());
        self.fov = (62.0 + (ship.velocity.length() / 195.0).min(1.0) * 11.0).to_radians();
    }
    pub fn matrix(&self, aspect: f32) -> Mat4 {
        let (near, far) = (0.4, 20000.0);
        let focal = 1.0 / (self.fov * 0.5).tan();
        // Proyección propia: mano derecha, profundidad NDC entre -1 y +1.
        let projection = Mat4::from_cols(
            Vec4::new(focal / aspect, 0.0, 0.0, 0.0),
            Vec4::new(0.0, focal, 0.0, 0.0),
            Vec4::new(0.0, 0.0, (far + near) / (near - far), -1.0),
            Vec4::new(0.0, 0.0, 2.0 * far * near / (near - far), 0.0),
        );
        let view =
            Mat4::from_quat(self.rotation.conjugate()) * Mat4::from_translation(-self.position);
        projection * view
    }
}
