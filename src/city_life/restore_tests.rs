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
            active: false,
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
