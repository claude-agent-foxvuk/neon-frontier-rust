use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::{Time, Transform, Vec2, Vec3, World};
use std::time::Duration;

use super::*;
use crate::physics::{bodies_overlap, move_with_slide, resolve_body_pairs, Body};

#[test]
fn separated_crossing_traffic_pair_stays_stable_on_the_next_tick() {
    let mut bodies = [
        Body::vehicle(Vec2::ZERO, 0.0),
        Body::vehicle(Vec2::new(0.0, 1.4), std::f32::consts::FRAC_PI_2),
    ];
    assert!(bodies_overlap(&bodies[0], &bodies[1]));

    resolve_body_pairs(&mut bodies);
    assert!(!bodies_overlap(&bodies[0], &bodies[1]));
    let settled = bodies;

    resolve_body_pairs(&mut bodies);
    assert_eq!(bodies, settled);
    assert!(bodies.iter().all(|body| body.center.is_finite()));
}

#[test]
fn swept_vehicle_motion_stops_before_a_rotated_obstacle_without_tunneling() {
    let wall = Body::new(
        Vec2::new(4.0, 0.0),
        5.0,
        0.25,
        std::f32::consts::FRAC_PI_2,
    );
    let start = Vec2::ZERO;
    let end = move_with_slide(start, Vec2::new(10.0, 0.0), |position| {
        bodies_overlap(&Body::vehicle(position, 0.0), &wall)
    });
    let stopped = Body::vehicle(end, 0.0);

    assert!(end.x > 0.0 && end.x < 4.0);
    assert!(!bodies_overlap(&stopped, &wall));
}

#[test]
fn dynamic_vehicle_person_pair_separates_and_remains_stable() {
    let mut bodies = [
        Body::vehicle(Vec2::ZERO, 0.4),
        Body::person(Vec2::new(1.0, 0.2)),
    ];
    assert!(bodies_overlap(&bodies[0], &bodies[1]));

    resolve_body_pairs(&mut bodies);
    assert!(!bodies_overlap(&bodies[0], &bodies[1]));
    let settled = bodies;

    resolve_body_pairs(&mut bodies);
    assert_eq!(bodies, settled);
}

#[test]
fn live_city_simulation_separates_crossing_traffic_vehicles_by_obb() {
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

    world.spawn((
        Transform::from_xyz(0.0, 0.48, 0.0),
        TrafficCar {
            speed: 6.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 1.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(0.0, 0.48, 1.4),
        TrafficCar {
            speed: 6.0,
            direction: 1.0,
            along_x: false,
            stop_timer: 1.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation resolves crossing traffic OBB contacts");

    let mut query = world.query_filtered::<(&Transform, &TrafficCar), With<TrafficCar>>();
    let bodies: Vec<_> = query
        .iter(&world)
        .map(|(transform, car)| traffic_vehicle_body(transform, car.along_x, car.direction))
        .collect();
    assert_eq!(bodies.len(), 2);
    assert!(bodies.iter().all(|body| body.center.is_finite()));
    assert!(!bodies_overlap(&bodies[0], &bodies[1]));
    assert_eq!(world.resource::<WorldState>().snapshot.traffic.len(), 2);
}
