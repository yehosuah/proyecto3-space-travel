use cosmos_gp::{
    camera::Camera,
    framebuffer::{ClipVertex, Framebuffer},
    simulation::{look_rotation, Flight, Input},
    world,
};
use glam::{Mat4, Vec3, Vec4};

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

fn vertex(p: Vec3, projection: Mat4) -> ClipVertex {
    ClipVertex {
        clip: projection * p.extend(1.0),
        world: p,
        object: p,
        normal: Vec3::Z,
    }
}

#[test]
fn perspective_interpolation_is_independent_of_quad_diagonal() {
    let matrix = Mat4::perspective_rh_gl(65.0f32.to_radians(), 1.0, 0.1, 100.0);
    let v = [
        Vec3::new(-1.0, -1.0, -2.0),
        Vec3::new(1.0, -1.0, -2.0),
        Vec3::new(1.0, 1.0, -5.0),
        Vec3::new(-1.0, 1.0, -5.0),
    ]
    .map(|p| vertex(p, matrix));
    let mut a = Framebuffer::new(80, 80);
    let mut b = Framebuffer::new(80, 80);
    let shade = |f: cosmos_gp::framebuffer::Fragment| {
        Some(Vec3::new(f.world.x * 0.3 + 0.5, -f.world.z * 0.15, 0.2))
    };
    for t in [[0, 1, 2], [0, 2, 3]] {
        a.triangle(t.map(|i| v[i]), true, shade);
    }
    for t in [[0, 1, 3], [1, 2, 3]] {
        b.triangle(t.map(|i| v[i]), true, shade);
    }
    let diffs = a
        .pixels
        .iter()
        .zip(&b.pixels)
        .filter(|(a, b)| {
            [0, 8, 16]
                .iter()
                .any(|s| (((*a >> s) & 255) as i32 - ((*b >> s) & 255) as i32).abs() > 1)
        })
        .count();
    assert!(diffs < 4, "pixels varying with triangulation: {diffs}");
}

#[test]
fn geometry_crossing_near_plane_is_clipped_and_remains_visible() {
    let matrix = Mat4::perspective_rh_gl(65.0f32.to_radians(), 1.0, 0.1, 100.0);
    let vertices = [
        Vec3::new(-0.5, -0.5, -1.0),
        Vec3::new(0.5, -0.5, -1.0),
        Vec3::new(0.0, 0.2, 0.05),
    ]
    .map(|p| vertex(p, matrix));
    let mut f = Framebuffer::new(64, 64);
    f.triangle(vertices, true, |_| Some(Vec3::ONE));
    assert!(f.stats.fragments > 100);
    assert!(f
        .depth
        .iter()
        .all(|d| d.is_infinite() || (*d >= 0.0 && *d <= 1.0)));
}

#[test]
fn depth_and_fragment_discard_do_not_depend_on_draw_order() {
    let triangle = |z| {
        [
            Vec4::new(-0.8, -0.8, z, 1.0),
            Vec4::new(0.8, -0.8, z, 1.0),
            Vec4::new(0.0, 0.8, z, 1.0),
        ]
        .map(|clip| ClipVertex {
            clip,
            object: Vec3::ZERO,
            world: Vec3::ZERO,
            normal: Vec3::Z,
        })
    };
    let mut a = Framebuffer::new(32, 32);
    let mut b = Framebuffer::new(32, 32);
    a.triangle(triangle(0.4), true, |_| Some(Vec3::X));
    a.triangle(triangle(-0.4), true, |_| Some(Vec3::Y));
    b.triangle(triangle(-0.4), true, |_| Some(Vec3::Y));
    b.triangle(triangle(0.4), true, |_| Some(Vec3::X));
    assert_eq!(a.pixels, b.pixels);
    let before = a.pixels.clone();
    a.triangle(triangle(-0.7), true, |_| None);
    assert_eq!(a.pixels, before);
}

#[test]
fn camera_culling_keeps_intersecting_spheres() {
    let camera = Camera {
        position: Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        fov: 60.0f32.to_radians(),
        front: true,
        anchor: Vec3::ZERO,
    };
    assert!(camera.sphere_visible(Vec3::new(0.0, 0.0, -10.0), 1.0, 1.6));
    assert!(!camera.sphere_visible(Vec3::new(0.0, 0.0, 10.0), 1.0, 1.6));
    assert!(camera.sphere_visible(Vec3::new(0.0, 0.0, 0.2), 2.0, 1.6));
    assert!(!camera.sphere_visible(Vec3::new(40.0, 0.0, -10.0), 1.0, 1.6));
}

#[test]
fn own_projection_maps_near_far_and_horizontal_edge() {
    let c = Camera {
        position: Vec3::ZERO,
        rotation: glam::Quat::IDENTITY,
        fov: 60.0f32.to_radians(),
        front: true,
        anchor: Vec3::ZERO,
    };
    let matrix = c.matrix(1.6);
    for (depth, expected) in [(0.4, -1.0), (20000.0, 1.0)] {
        let p = matrix * Vec4::new(0.0, 0.0, -depth, 1.0);
        assert!((p.z / p.w - expected).abs() < 0.00001);
    }
    let p = matrix * Vec4::new(10.0 * (c.fov / 2.0).tan() * 1.6, 0.0, -10.0, 1.0);
    assert!((p.x / p.w - 1.0).abs() < 0.00001);
}

#[test]
fn parallel_rasterization_matches_serial_pixels_and_depth() {
    use cosmos_gp::shader::{self, Body, Uniforms};
    let mut serial = Framebuffer::new(96, 80);
    let mut parallel = Framebuffer::new(96, 80);
    parallel.workers = 4;
    let matrix = Mat4::perspective_rh_gl(60.0f32.to_radians(), 1.2, 0.1, 100.0);
    let vertices = [
        Vec3::new(-2.0, -1.0, -3.0),
        Vec3::new(1.0, -1.0, -2.0),
        Vec3::new(0.0, 1.0, -1.0),
    ]
    .map(|p| vertex(p, matrix));
    let u = Uniforms {
        body: Body::Prop,
        time: 0.0,
        layers: 4,
        center: Vec3::ZERO,
        radius: 1.0,
        ring_normal: Vec3::Y,
        camera: Vec3::ZERO,
        tint: Vec3::new(0.7, 0.3, 0.2),
        rotation: glam::Quat::IDENTITY,
        normal_mapping: false,
    };
    serial.triangle(vertices, true, |f| shader::shade(f, &u));
    parallel.submit(vertices, true, u);
    parallel.flush();
    assert_eq!(serial.pixels, parallel.pixels);
    assert_eq!(serial.depth, parallel.depth);
}

#[test]
fn chase_camera_stays_behind_ship_during_fast_reverse() {
    let mut ship = Flight::new();
    ship.position = Vec3::splat(10000.0);
    ship.rotation = glam::Quat::IDENTITY;
    ship.velocity = Vec3::Z * 76.0;
    ship.assist = false;
    ship.gravity = false;
    let mut camera = Camera::new(&ship);
    for _ in 0..240 {
        ship.update(Input::default(), 1.0 / 60.0);
        camera.update(&ship, 1.0 / 60.0);
        let offset = camera.position - ship.position;
        assert!(offset.dot(ship.forward()) < -8.5);
        assert!(offset.length() > 9.0);
    }
}
