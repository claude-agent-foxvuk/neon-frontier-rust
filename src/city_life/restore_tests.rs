use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use std::time::Duration;

use super::*;

fn saved_world() -> WorldSnapshot {
    WorldSnapshot {
        health: 63.0,
        wanted_level: 3,
        wanted_timer: 7.5,
        traffic: vec![TrafficSave {
            position: Vec3::new(0.0, 0.48, -120.0),
            speed: 6.2,
            direction: 1.0,
            along_x: true,
            lane_offset: -1.2,
            avoid_offset: -3.5,
            stop_timer: 1.4,
        }],
        citizens: vec![CitizenSave {
            origin: Vec3::new(-20.0, 0.82, 60.0),
            phase: 1.2,
            speed: 0.5,
            range: 3.0,
            along_x: true,
            displacement: Vec3::ZERO,
            knock_velocity: Vec3::ZERO,
            knocked: 0.0,
            health: 0.0,
            dead: true,
        }],
        patrols: vec![PatrolSave {
            position: Vec3::new(-48.0, 0.48, 2.0),
            phase: 0.7,
        }],
        officers: vec![OfficerSave {
            position: Vec3::new(50.0, 0.82, -20.0),
            active: true,
            shot_timer: 0.85,
        }],
    }
}

#[test]
fn v1_restore_round_trips_traffic_citizens_patrols_and_officers() {
    let saved = saved_world();

    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe::default());
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.insert_resource(WorldRestore(Some(saved.clone())));

    world.spawn((
        Transform::from_xyz(0.0, 0.48, -120.0),
        TrafficCar {
            speed: 0.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(-20.0, 0.82, 60.0),
        Citizen {
            origin: Vec3::new(-20.0, 0.82, 60.0),
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
    world.spawn((
        Transform::from_xyz(-48.0, 0.48, 2.0),
        PolicePatrol {
            home: Vec2::new(-48.0, 2.0),
            phase: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(50.0, 0.82, -20.0),
        Visibility::Visible,
        FootOfficer {
            patrol_index: 0,
            active: true,
            shot_timer: 0.0,
        },
    ));
    world.spawn((Visibility::Hidden, PoliceSirenLight { phase: 0.0 }));
    world.spawn((
        Visibility::Hidden,
        OfficerMuzzleFlash {
            patrol_index: 0,
            remaining: 0.0,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("city simulation must initialize every query without an ECS conflict");

    let snapshot = world.resource::<WorldState>().snapshot.clone();
    assert_eq!(snapshot.wanted_level, 3);
    assert!((snapshot.health - 63.0).abs() < 1.0e-4);
    assert!(snapshot.wanted_timer < 7.5 && snapshot.wanted_timer > 7.4);

    assert_eq!(snapshot.traffic.len(), 1);
    assert!((snapshot.traffic[0].speed - 6.2).abs() < 1.0e-4);
    assert!(snapshot.traffic[0].stop_timer < 1.4 && snapshot.traffic[0].stop_timer > 1.3);

    assert_eq!(snapshot.citizens.len(), 1);
    assert!(snapshot.citizens[0].dead);
    assert_eq!(snapshot.citizens[0].health, 0.0);

    assert_eq!(snapshot.patrols.len(), 1);
    assert!((snapshot.patrols[0].phase - 0.7).abs() < 1.0e-4);

    assert_eq!(snapshot.officers.len(), 1);
    assert!(snapshot.officers[0].active);
    assert!(snapshot.officers[0].shot_timer < 0.85);
}

#[test]
fn mismatched_entity_counts_reject_the_saved_world() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe::default());
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.insert_resource(WorldRestore(Some(saved_world())));

    world.spawn((
        Transform::from_xyz(0.0, 0.48, -120.0),
        TrafficCar {
            speed: 0.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(-20.0, 0.82, 60.0),
        Citizen {
            origin: Vec3::new(-20.0, 0.82, 60.0),
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
    world.spawn((
        Transform::from_xyz(-48.0, 0.48, 2.0),
        PolicePatrol {
            home: Vec2::new(-48.0, 2.0),
            phase: 0.0,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("simulation still runs when the saved world cannot be restored");

    assert_eq!(world.resource::<WorldState>().snapshot.officers.len(), 0);
    assert!((world.resource::<PlayerProbe>().health - 100.0).abs() < 1.0e-4);
}

#[test]
fn main_update_world_queries_initialize_without_conflicts() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(crate::Game::default());
    world.spawn((crate::SunLight, DirectionalLight::default()));
    world.spawn((
        crate::FollowCamera,
        Transform::default(),
        DistanceFog {
            color: Color::srgb(0.035, 0.06, 0.10),
            directional_light_color: Color::srgba(0.35, 0.55, 0.80, 0.18),
            directional_light_exponent: 12.0,
            falloff: FogFalloff::from_visibility_colors(
                190.0,
                Color::srgb(0.04, 0.06, 0.10),
                Color::srgb(0.10, 0.15, 0.22),
            ),
        },
    ));

    world
        .run_system_once(crate::update_world)
        .expect("main world update must initialize its ECS queries without conflicts");
}

#[test]
fn live_traffic_starts_a_safe_pass_around_ahead_player_vehicle() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(10.0, 0.65, 0.0),
        vehicle_position: Vec3::new(10.0, 0.65, 0.0),
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
        Transform::from_xyz(6.0, 0.48, 0.0),
        TrafficCar {
            speed: 6.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("live city simulation initializes and advances its collision queries");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.traffic.len(), 1);
    assert!(snapshot.traffic[0].avoid_offset.abs() > 0.1);
    assert!(snapshot.traffic[0].lane_offset.abs() > 0.0);
    assert!((snapshot.traffic[0].position.x - 6.0).abs() < 1.0e-4);
    let traffic_transform = world
        .query_filtered::<&Transform, With<TrafficCar>>()
        .single(&world)
        .expect("one traffic vehicle was spawned");
    let traffic_body = vehicle_body_from_forward(traffic_transform.translation, Vec3::X);
    let player = world.resource::<PlayerProbe>();
    let player_body = vehicle_body_from_forward(player.vehicle_position, player.forward);
    assert!(
        !crate::physics::bodies_overlap(&traffic_body, &player_body),
        "traffic must shift into a clear lane without advancing into the player's oriented vehicle footprint"
    );
}

#[test]
fn live_traffic_obb_contact_knocks_down_and_damages_a_pedestrian() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(100.0, 0.9, 100.0),
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
        Transform::from_xyz(-2.45, 0.48, 0.0),
        TrafficCar {
            speed: 6.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));
    world.spawn((
        Transform::from_xyz(0.0, 0.82, 0.0),
        Citizen {
            origin: Vec3::new(0.0, 0.82, 0.0),
            phase: 0.0,
            speed: 0.0,
            range: 0.0,
            along_x: false,
            displacement: Vec3::ZERO,
            knock_velocity: Vec3::ZERO,
            knocked: 0.0,
            health: CITIZEN_HEALTH,
            dead: false,
        },
    ));

    world
        .run_system_once(simulate_city_life)
        .expect("live simulation must resolve OBB contact between traffic and a pedestrian");

    let snapshot = &world.resource::<WorldState>().snapshot;
    assert_eq!(snapshot.traffic.len(), 1);
    assert!(snapshot.traffic[0].stop_timer >= CAR_STOP_SECONDS - 1.0e-4);
    assert_eq!(snapshot.citizens.len(), 1);
    assert!(snapshot.citizens[0].dead);
    assert_eq!(snapshot.citizens[0].health, 0.0);
    assert!(snapshot.citizens[0].knocked > 0.0);
}
