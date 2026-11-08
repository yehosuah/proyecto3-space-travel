use cosmos_gp::{
    simulation::{look_rotation, Flight, Input},
    world,
};
use glam::Vec3;

#[test]
fn flight_is_independent_of_render_frame_rate() {
    fn simulate(fps: usize) -> Flight {
        let mut s = Flight::new();
        for _ in 0..fps * 5 {
            s.update(
                Input {
                    thrust: 0.8,
                    yaw: 0.4,
                    pitch: 0.12,
                    boost: true,
                    ..Input::default()
                },
                1.0 / fps as f64,
            );
        }
        s
    }
    let a = simulate(30);
    let b = simulate(144);
    assert!((a.time - b.time).abs() < 1e-8);
    assert!(
        a.position.distance(b.position) < 0.001,
        "{}",
        a.position.distance(b.position)
    );
    assert!(a.velocity.distance(b.velocity) < 0.001);
}

#[test]
fn boost_cannot_tunnel_through_a_planet() {
    let mut s = Flight::new();
    let p = world::planet(0, 0.0);
    s.position = p.center + Vec3::Z * (p.radius * 1.075 + 20.0);
    s.rotation = look_rotation(-Vec3::Z, Vec3::Y);
    s.velocity = -Vec3::Z * 195.0;
    s.update(
        Input {
            thrust: 1.0,
            boost: true,
            ..Default::default()
        },
        0.7,
    );
    let p = world::collision_bodies(s.time)[1];
    assert!(s.position.distance(p.center) >= p.radius + 1.09);
    assert!(s.impacts > 0);
}

#[test]
fn braking_stops_and_unassisted_drift_preserves_momentum() {
    let mut s = Flight::new();
    s.gravity = false;
    s.position = Vec3::splat(1500.0);
    s.velocity = Vec3::X * 60.0;
    s.assist = false;
    s.update(Input::default(), 1.0);
    assert!((s.velocity.length() - 60.0).abs() < 0.01);
    s.update(
        Input {
            brake: true,
            ..Default::default()
        },
        2.0,
    );
    assert!(s.velocity.length() < 0.02);
}

#[test]
fn gravity_curves_an_unpowered_unassisted_flight() {
    let mut active = Flight::new();
    let mut disabled = Flight::new();
    let p = world::planet(0, 0.0);
    for ship in [&mut active, &mut disabled] {
        ship.position = p.center + Vec3::Z * (p.radius * 1.6);
        ship.assist = false;
    }
    disabled.gravity = false;
    active.update(Input::default(), 5.0);
    disabled.update(Input::default(), 5.0);
    let direction = (world::planet(0, active.time).center - active.position).normalize();
    assert!(active.velocity.dot(direction) > 2.0);
    assert!(active.position.distance(disabled.position) > 4.0);
    assert_eq!(disabled.velocity, Vec3::ZERO);
}

#[test]
fn orbit_mode_tracks_the_moving_planet() {
    let mut s = Flight::new();
    let orbit_radius = world::planet(0, 0.0).radius * 1.5;
    s.position = world::planet(0, 0.0).center + Vec3::Y * orbit_radius;
    s.rotation = look_rotation(-Vec3::Z, Vec3::Y);
    s.velocity = -Vec3::Z * 15.0;
    s.assist = false;
    s.local = Some(0);
    for _ in 0..600 {
        s.update(Input::default(), 1.0 / 60.0);
    }
    let distance = s.position.distance(world::planet(0, s.time).center);
    assert!((distance - orbit_radius).abs() < 1.0, "distance={distance}");
    assert!(s.position.is_finite() && s.rotation.is_finite());
}
