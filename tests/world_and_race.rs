use cosmos_gp::{
    race::{self, Phase, Race},
    simulation::Flight,
    world,
};
use glam::Vec3;

#[test]
fn gate_requires_forward_crossing_inside_aperture() {
    let g = race::gate(3);
    let left = g.normal.cross(g.center.normalize()).normalize();
    assert!(race::crossing(g.center - g.normal * 10.0, g.center + g.normal * 10.0, g).is_some());
    assert!(race::crossing(g.center + g.normal * 10.0, g.center - g.normal * 10.0, g).is_none());
    assert!(race::crossing(
        g.center + left * (g.radius + 2.0) - g.normal * 10.0,
        g.center + left * (g.radius + 2.0) + g.normal * 10.0,
        g
    )
    .is_none());
}

#[test]
fn all_gates_are_above_the_collision_surface() {
    let surface = world::collision_bodies(0.0)[1].radius + 1.1;
    for i in 0..race::GATE_COUNT {
        let g = race::gate(i);
        assert!(g.center.length() - g.radius > surface + 0.5, "gate {i}");
    }
}

#[test]
fn race_needs_ordered_full_lap_and_persists_best() {
    let mut r = Race::default();
    let mut ship = Flight::new();
    r.prepare(&mut ship);
    let previous = ship.position;
    r.update(&mut ship, previous, 0.0, 3.1);
    assert_eq!(r.phase, Phase::Running);
    // Crossing an unrelated gate cannot advance the checkpoint sequence.
    let wrong = race::world_gate(4, ship.time);
    ship.position = wrong.center + wrong.normal * 4.0;
    let time = ship.time;
    assert_eq!(
        r.update(&mut ship, wrong.center - wrong.normal * 4.0, time, 0.1),
        0
    );
    assert_eq!(r.next, 0);
    for i in 0..=race::GATE_COUNT {
        let previous_time = ship.time;
        let before = race::world_gate(i % race::GATE_COUNT, previous_time);
        ship.time += 3.0;
        let after = race::world_gate(i % race::GATE_COUNT, ship.time);
        ship.position = after.center + after.normal * 4.0;
        r.update(
            &mut ship,
            before.center - before.normal * 4.0,
            previous_time,
            3.0,
        );
        if i < race::GATE_COUNT {
            assert_eq!(r.phase, Phase::Running);
        }
    }
    assert_eq!(r.phase, Phase::Finished);
    assert!(r.new_best);
    assert!(r.best.unwrap() > 60.0);
    let path = std::env::temp_dir().join(format!("cosmos-test-best-{}.txt", std::process::id()));
    r.save(&path).unwrap();
    let loaded = Race::load(&path);
    assert!((loaded.best.unwrap() - r.best.unwrap()).abs() < 0.00001);
    std::fs::remove_file(path).unwrap();
}
