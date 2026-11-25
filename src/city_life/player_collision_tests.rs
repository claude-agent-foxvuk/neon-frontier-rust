use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Time, Transform, Vec2, Vec3, World};
use std::time::Duration;

use super::*;
use crate::physics::{bodies_overlap, Body};

#[test]
fn live_walking_player_and_citizen_separate_by_obb_without_wanted() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);

    let player_position = Vec3::new(0.0, 0.9, 0.0);
    world.insert_resource(PlayerProbe {
        position: player_position,
        vehicle_position: Vec3::new(120.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    let origin = Vec3::new(0.5, 0.82, 0.0);
    world.spawn((
        Transform::from_translation(origin),
        Citizen {
            origin,
            phase: 0.0,
            speed: 0.0,
            range: 0.0,
            along_x: true,
            displacement: Vec3::ZERO,
            knock_velocity: Vec3::ZERO,
            knocked: 0.0,
            health: CITIZEN_HEALTH,
            dead: false,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation separates a walking player from a citizen");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.citizens.len(), 1);
    let saved = snapshot.citizens[0];
    let player_body = Body::person(Vec2::new(player_position.x, player_position.z));
    let citizen_body = Body::person(Vec2::new(
        saved.origin.x + saved.displacement.x,
        saved.origin.z + saved.displacement.z,
    ));
    assert!(!bodies_overlap(&player_body, &citizen_body));
    assert_eq!(snapshot.wanted_level, 0);
}
