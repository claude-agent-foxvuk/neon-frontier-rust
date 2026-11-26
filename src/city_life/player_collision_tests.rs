use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Time, Transform, Vec2, Vec3, World};
use std::time::Duration;

use super::*;
use crate::physics::{bodies_overlap, Body};

fn simulate_vehicle_pedestrian_contact(position: Vec3, forward: Vec3) -> WorldSnapshot {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(0.0, 0.9, 0.0),
        vehicle_position: Vec3::new(0.0, 0.65, 0.0),
        forward,
        in_vehicle: true,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    world.spawn((
        Transform::from_translation(position),
        Citizen {
            origin: position,
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
        .expect("live city simulation resolves player vehicle and pedestrian contact");
    world.resource::<WorldState>().snapshot.clone()
}

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

#[test]
fn live_traffic_stops_before_walking_player_obb_without_wanted() {
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

    world.spawn((
        Transform::from_xyz(-9.0, 0.48, 0.0),
        TrafficCar {
            speed: 8.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation brakes traffic before a walking player");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.traffic.len(), 1);
    assert!(snapshot.traffic[0].stop_timer > 0.0);
    assert_eq!(snapshot.wanted_level, 0);

    let mut cars = world.query_filtered::<(&Transform, &TrafficCar), With<TrafficCar>>();
    let (transform, car) = cars.single(&world).expect("one traffic car remains");
    assert!((transform.translation.x + 9.0).abs() < 1.0e-4);
    let traffic_body = traffic_vehicle_body(transform, car.along_x, car.direction);
    let player_body = Body::person(Vec2::new(player_position.x, player_position.z));
    assert!(!bodies_overlap(&traffic_body, &player_body));
}

#[test]
fn live_player_vehicle_obb_knocks_down_overlapping_pedestrian() {
    let snapshot = simulate_vehicle_pedestrian_contact(
        Vec3::new(1.5, 0.82, 0.0),
        Vec3::X,
    );

    assert_eq!(snapshot.citizens.len(), 1);
    assert!(snapshot.citizens[0].dead);
    assert!(snapshot.citizens[0].displacement.x > 0.0);
    assert_eq!(snapshot.wanted_level, 1);
}

#[test]
fn live_player_vehicle_obb_avoids_circle_only_false_positive() {
    let pedestrian = Vec2::new(0.0, 1.4);
    assert!(pedestrian.length() < CAR_BODY_RADIUS + CITIZEN_BODY_RADIUS);

    let snapshot = simulate_vehicle_pedestrian_contact(
        Vec3::new(pedestrian.x, 0.82, pedestrian.y),
        Vec3::X,
    );

    assert_eq!(snapshot.citizens.len(), 1);
    assert!(!snapshot.citizens[0].dead);
    assert_eq!(snapshot.citizens[0].health, CITIZEN_HEALTH);
    assert_eq!(snapshot.wanted_level, 0);
}

#[test]
fn live_city_publishes_oriented_vehicle_and_pedestrian_bodies() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe::default());
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    world.spawn((
        Transform::from_xyz(16.0, 0.48, 0.0),
        TrafficCar {
            speed: 0.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    let pedestrian = Vec3::new(32.0, 0.82, 12.0);
    world.spawn((
        Transform::from_translation(pedestrian),
        Citizen {
            origin: pedestrian,
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
        .expect("live city simulation publishes collision bodies for player movement");

    let bodies = &world.resource::<TrafficBodies>().oriented;
    assert_eq!(bodies.len(), 2);
    assert!(bodies
        .iter()
        .any(|body| body.half_x > crate::physics::PERSON_HALF_EXTENT));
    assert!(bodies
        .iter()
        .any(|body| body.half_x == crate::physics::PERSON_HALF_EXTENT));
}
