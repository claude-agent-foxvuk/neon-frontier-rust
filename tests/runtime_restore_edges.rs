#![allow(dead_code)]

mod physics {
    include!("../src/physics.rs");
}

mod city_life {
    include!("../src/city_life.rs");

    #[cfg(test)]
    mod restore_edge_tests {
        use super::*;
        use bevy::ecs::system::RunSystemOnce;

        fn restore_saved_world(
            mut probe: ResMut<PlayerProbe>,
            mut wanted: ResMut<WantedState>,
            mut restore: ResMut<WorldRestore>,
            mut traffic: Query<
                (&mut Transform, &mut TrafficCar),
                (
                    Without<Citizen>,
                    Without<PolicePatrol>,
                    Without<FootOfficer>,
                ),
            >,
            mut citizens: Query<
                (&mut Transform, &mut Citizen),
                (
                    Without<TrafficCar>,
                    Without<PolicePatrol>,
                    Without<FootOfficer>,
                ),
            >,
            mut patrols: Query<
                (&mut Transform, &mut PolicePatrol),
                (Without<TrafficCar>, Without<Citizen>, Without<FootOfficer>),
            >,
            mut officers: Query<
                (&mut Transform, &mut FootOfficer, &mut Visibility),
                (Without<TrafficCar>, Without<Citizen>, Without<PolicePatrol>),
            >,
        ) {
            let saved = restore.0.take().expect("V1 world is pending");
            assert!(restore_world(
                &saved,
                &mut probe,
                &mut wanted,
                &mut traffic,
                &mut citizens,
                &mut patrols,
                &mut officers,
            ));
        }

        #[test]
        fn v1_round_trip_preserves_dead_citizens_and_inactive_officers() {
            let saved_world = WorldSnapshot {
                health: 42.0,
                wanted_level: 1,
                wanted_timer: 6.25,
                citizens: vec![CitizenSave {
                    origin: Vec3::new(-18.0, 0.82, 24.0),
                    phase: 1.1,
                    speed: 0.4,
                    range: 5.0,
                    along_x: true,
                    displacement: Vec3::new(0.2, 0.0, -0.1),
                    knock_velocity: Vec3::new(0.3, 0.0, -0.2),
                    knocked: 1.2,
                    health: 0.0,
                    dead: true,
                }],
                officers: vec![OfficerSave {
                    position: Vec3::new(50.0, 0.82, -20.0),
                    active: false,
                    shot_timer: 2.5,
                }],
                ..Default::default()
            };
            let encoded = crate::save::encode_snapshot(&crate::save::Snapshot {
                player: Vec3::new(40.0, 0.9, 40.0),
                car: Vec3::new(42.0, 0.65, 40.0),
                yaw: 0.0,
                speed: 0.0,
                steering: 0.0,
                lateral_speed: 0.0,
                player_facing_yaw: 0.0,
                player_moving: false,
                player_running: false,
                player_vertical_speed: 0.0,
                player_grounded: true,
                punch_cooldown: 0.0,
                camera_yaw: 0.0,
                camera_orbit: 0.0,
                camera_pitch: 0.38,
                stage: 0,
                credits: 0,
                driving: false,
                bloom: true,
                world_time: 0.61,
                world: saved_world.clone(),
            });
            let snapshot = crate::save::decode_snapshot(&encoded).expect("V1 save decodes");
            assert_eq!(snapshot.world, saved_world);
            assert_eq!(snapshot.world_time, 0.61);

            let mut world = World::new();
            world.insert_resource(PlayerProbe::default());
            world.init_resource::<WantedState>();
            world.insert_resource(WorldRestore(Some(snapshot.world.clone())));
            world.spawn((
                Transform::IDENTITY,
                Citizen {
                    origin: Vec3::ZERO,
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
            world.spawn((
                Transform::IDENTITY,
                Visibility::Visible,
                FootOfficer {
                    patrol_index: 0,
                    active: true,
                    shot_timer: 0.0,
                },
            ));

            world
                .run_system_once(restore_saved_world)
                .expect("restore system runs");

            let mut citizens = world.query::<(&Transform, &Citizen)>();
            let (citizen_transform, citizen) = citizens.iter(&world).next().unwrap();
            assert_eq!(citizen_transform.translation, saved_world.citizens[0].origin + saved_world.citizens[0].displacement);
            assert!(citizen.dead);
            assert_eq!(citizen.health, 0.0);
            assert_eq!(citizen.knocked, 1.2);
            assert_eq!(citizen.knock_velocity, Vec3::new(0.3, 0.0, -0.2));

            let mut officers = world.query::<(&Transform, &FootOfficer, &Visibility)>();
            let (officer_transform, officer, visibility) = officers.iter(&world).next().unwrap();
            assert_eq!(officer_transform.translation, saved_world.officers[0].position);
            assert!(!officer.active);
            assert_eq!(officer.shot_timer, 2.5);
            assert_eq!(*visibility, Visibility::Hidden);
        }
    }
}

mod save {
    include!("../src/save.rs");
}
