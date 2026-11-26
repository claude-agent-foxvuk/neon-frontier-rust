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

#[test]
fn live_traffic_collision_knocks_down_pedestrian_without_wanted_for_player() {
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
            speed: 0.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 1.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(1.0, 0.82, 0.0),
        Citizen {
            origin: Vec3::new(1.0, 0.82, 0.0),
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
        .expect("live city simulation resolves traffic-to-pedestrian contact");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.citizens.len(), 1);
    assert!(snapshot.citizens[0].dead);
    assert_eq!(snapshot.citizens[0].health, 0.0);
    assert!(snapshot.citizens[0].knocked > 0.0);
    assert_eq!(snapshot.wanted_level, 0);
    assert!(snapshot.traffic[0].stop_timer > 0.0);
}

#[test]
fn live_player_vehicle_collision_knocks_down_pedestrian_and_raises_wanted_level() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(0.0, 0.65, 0.0),
        vehicle_position: Vec3::new(0.0, 0.65, 0.0),
        forward: Vec3::NEG_Z,
        in_vehicle: true,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    world.spawn((
        Transform::from_xyz(0.0, 0.82, 1.0),
        Citizen {
            origin: Vec3::new(0.0, 0.82, 1.0),
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
        .expect("live city simulation resolves player-vehicle-to-pedestrian contact");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.citizens.len(), 1);
    assert!(snapshot.citizens[0].dead);
    assert_eq!(snapshot.citizens[0].health, 0.0);
    assert!(snapshot.citizens[0].knocked > 0.0);
    assert_eq!(snapshot.wanted_level, 1);
}

#[test]
fn live_walking_player_and_pedestrian_separate_without_wanted_level() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(0.0, 0.9, 0.0),
        vehicle_position: Vec3::new(100.0, 0.65, 100.0),
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
        Transform::from_xyz(0.2, 0.82, 0.0),
        Citizen {
            origin: Vec3::new(0.2, 0.82, 0.0),
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
        .expect("walking contact separates the player and pedestrian by OBB");

    let mut query = world.query_filtered::<(&Transform, &Citizen), With<Citizen>>();
    let (transform, citizen) = query.single(&world).expect("one pedestrian remains");
    assert!(!citizen.dead);
    assert!(!bodies_overlap(
        &Body::person(Vec2::ZERO),
        &Body::person(Vec2::new(transform.translation.x, transform.translation.z)),
    ));
    assert_eq!(world.resource::<WorldState>().snapshot.wanted_level, 0);
}

#[test]
fn live_traffic_brakes_before_the_player_vehicle_obb() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(0.0, 0.65, 0.0),
        vehicle_position: Vec3::new(0.0, 0.65, 0.0),
        forward: Vec3::NEG_Z,
        in_vehicle: true,
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
        .expect("traffic detects the player vehicle using its oriented footprint");

    let mut query = world.query_filtered::<(&Transform, &TrafficCar), With<TrafficCar>>();
    let (transform, car) = query.single(&world).expect("one traffic car remains");
    assert_eq!(transform.translation.x, -9.0);
    assert!(car.stop_timer > 0.0);
    assert!(!bodies_overlap(
        &traffic_vehicle_body(transform, car.along_x, car.direction),
        &Body::vehicle(Vec2::ZERO, -std::f32::consts::FRAC_PI_2),
    ));
}

fn live_police_collision_world() -> World {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(30.0, 0.9, 0.0),
        vehicle_position: Vec3::new(32.0, 0.65, 0.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.resource_mut::<WantedState>().level = 2;
    world.resource_mut::<WantedState>().decay_timer = 120.0;
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();
    world
}

fn advance_live_police_collision_world(world: &mut World) {
    world
        .resource_mut::<Time<()>>()
        .advance_by(Duration::from_millis(16));
    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation resolves police OBB contacts");
}

fn stationary_traffic_car(world: &mut World, position: Vec3) {
    world.spawn((
        Transform::from_translation(position),
        TrafficCar {
            speed: 0.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 120.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
}

#[test]
fn live_police_patrol_stops_before_a_stationary_traffic_vehicle_obb() {
    let mut world = live_police_collision_world();
    stationary_traffic_car(&mut world, Vec3::new(6.0, 0.48, 0.0));
    world.spawn((
        Transform::from_xyz(0.0, 0.48, 0.0),
        PolicePatrol {
            home: Vec2::ZERO,
            phase: 0.0,
        },
    ));

    for _ in 0..80 {
        advance_live_police_collision_world(&mut world);
        let patrol_body = {
            let mut query =
                world.query_filtered::<(&Transform, &PolicePatrol), With<PolicePatrol>>();
            let (transform, _) = query.single(&world).expect("one police patrol");
            vehicle_body_from_forward(transform.translation, transform.rotation * Vec3::Z)
        };
        let traffic_body = {
            let mut query = world.query_filtered::<(&Transform, &TrafficCar), With<TrafficCar>>();
            let (transform, car) = query.single(&world).expect("one stationary traffic car");
            traffic_vehicle_body(transform, car.along_x, car.direction)
        };
        assert!(!bodies_overlap(&patrol_body, &traffic_body));
    }

    let mut query = world.query_filtered::<(&Transform, &PolicePatrol), With<PolicePatrol>>();
    let (transform, _) = query.single(&world).expect("one police patrol");
    assert!(transform.translation.x > 0.2);
    assert!(transform.translation.x < 6.0);
}

#[test]
fn live_foot_officer_stops_before_a_stationary_traffic_vehicle_obb() {
    let mut world = live_police_collision_world();
    stationary_traffic_car(&mut world, Vec3::new(6.0, 0.48, 0.0));
    world.spawn((
        Transform::from_xyz(0.0, 0.48, 20.0),
        PolicePatrol {
            home: Vec2::new(0.0, 20.0),
            phase: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(0.0, 0.82, 0.0),
        FootOfficer {
            patrol_index: 0,
            active: true,
            shot_timer: 120.0,
        },
        Visibility::Visible,
    ));

    for _ in 0..80 {
        advance_live_police_collision_world(&mut world);
        let officer_position = {
            let mut query = world.query_filtered::<(&Transform, &FootOfficer), With<FootOfficer>>();
            let (transform, _) = query.single(&world).expect("one active foot officer");
            Vec2::new(transform.translation.x, transform.translation.z)
        };
        let traffic_body = {
            let mut query = world.query_filtered::<(&Transform, &TrafficCar), With<TrafficCar>>();
            let (transform, car) = query.single(&world).expect("one stationary traffic car");
            traffic_vehicle_body(transform, car.along_x, car.direction)
        };
        assert!(!bodies_overlap(&Body::person(officer_position), &traffic_body));
    }

    let mut query = world.query_filtered::<(&Transform, &FootOfficer), With<FootOfficer>>();
    let (transform, officer) = query.single(&world).expect("one active foot officer");
    assert!(officer.active);
    assert!(transform.translation.x > 0.2);
    assert!(transform.translation.x < 6.0);
}
