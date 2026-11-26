use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Time, Transform, Vec2, Vec3, World};
use std::time::Duration;

use super::*;
use crate::physics::{bodies_overlap, Body};

#[test]
fn live_overlapping_pedestrians_separate_by_obb_without_wanted_level() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(120.0, 0.9, 120.0),
        vehicle_position: Vec3::new(122.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    for x in [0.0, 0.2] {
        let origin = Vec3::new(x, 0.82, 0.0);
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
    }

    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation separates overlapping pedestrians");

    let mut query = world.query_filtered::<(&Transform, &Citizen), With<Citizen>>();
    let pedestrians: Vec<_> = query
        .iter(&world)
        .map(|(transform, citizen)| {
            assert!(!citizen.dead);
            Body::person(Vec2::new(transform.translation.x, transform.translation.z))
        })
        .collect();
    assert_eq!(pedestrians.len(), 2);
    assert!(!bodies_overlap(&pedestrians[0], &pedestrians[1]));
    assert_eq!(world.resource::<WorldState>().snapshot.wanted_level, 0);
}
