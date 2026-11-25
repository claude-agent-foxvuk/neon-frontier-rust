#![allow(dead_code)]

mod physics {
    include!("../src/physics.rs");
}

mod city_life {
    include!("../src/city_life.rs");

    #[cfg(test)]
    mod runtime_restore_tests {
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
            let saved = restore.0.take().expect("loaded V1 world is pending");
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
        fn v1_decode_and_runtime_restore_preserve_city_phases_and_timers() {
            let saved_world = WorldSnapshot {
                health: 63.0,
                wanted_level: 2,
                wanted_timer: 8.0,
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
                    displacement: Vec3::new(0.5, 0.25, 0.2),
                    knock_velocity: Vec3::new(0.3, 0.4, -0.2),
                    knocked: 0.8,
                    health: 1.5,
                    dead: false,
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
            };
            let encoded = crate::save::encode_snapshot(&crate::save::Snapshot {
                player: Vec3::new(90.0, 0.9, 90.0),
                car: Vec3::new(92.0, 0.65, 90.0),
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
                world_time: 0.42,
                world: saved_world.clone(),
            });
            let snapshot = crate::save::decode_snapshot(&encoded).expect("V1 save decodes");
            assert_eq!(snapshot.world_time, 0.42);
            assert_eq!(snapshot.world, saved_world);

            let mut world = World::new();
            world.insert_resource(PlayerProbe::default());
            world.init_resource::<WantedState>();
            world.insert_resource(WorldRestore(Some(snapshot.world.clone())));
            world.spawn((
                Transform::IDENTITY,
                TrafficCar {
                    speed: 0.0,
                    direction: 1.0,
                    along_x: false,
                    stop_timer: 0.0,
                    lane_offset: 0.0,
                    avoid_offset: 0.0,
                },
            ));
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
                PolicePatrol {
                    home: Vec2::ZERO,
                    phase: 0.0,
                },
            ));
            world.spawn((
                Transform::IDENTITY,
                Visibility::Hidden,
                FootOfficer {
                    patrol_index: 0,
                    active: false,
                    shot_timer: 0.0,
                },
            ));

            world
                .run_system_once(restore_saved_world)
                .expect("restore system runs");

            let probe = world.resource::<PlayerProbe>();
            assert_eq!(probe.health, 63.0);
            let wanted = world.resource::<WantedState>();
            assert_eq!(wanted.level, 2);
            assert_eq!(wanted.decay_timer, 8.0);
            assert_eq!(world.resource::<WorldRestore>().0, None);

            let mut traffic = world.query::<(&Transform, &TrafficCar)>();
            let (traffic_transform, traffic_car) = traffic.iter(&world).next().unwrap();
            assert_eq!(traffic_transform.translation, snapshot.world.traffic[0].position);
            assert_eq!(traffic_car.avoid_offset, -3.5);
            assert_eq!(traffic_car.stop_timer, 1.4);

            let mut citizen = world.query::<(&Transform, &Citizen)>();
            let (citizen_transform, citizen_state) = citizen.iter(&world).next().unwrap();
            assert_eq!(citizen_transform.translation, Vec3::new(-19.5, 1.07, 60.2));
            assert_eq!(citizen_state.phase, 1.2);
            assert_eq!(citizen_state.knocked, 0.8);
            assert_eq!(citizen_state.health, 1.5);
            assert!(!citizen_state.dead);

            let mut patrol = world.query::<(&Transform, &PolicePatrol)>();
            let (patrol_transform, patrol_state) = patrol.iter(&world).next().unwrap();
            assert_eq!(patrol_transform.translation, snapshot.world.patrols[0].position);
            assert_eq!(patrol_state.phase, 0.7);

            let mut officer = world.query::<(&Transform, &FootOfficer, &Visibility)>();
            let (officer_transform, officer_state, visibility) =
                officer.iter(&world).next().unwrap();
            assert_eq!(officer_transform.translation, snapshot.world.officers[0].position);
            assert!(officer_state.active);
            assert_eq!(officer_state.shot_timer, 0.85);
            assert_eq!(*visibility, Visibility::Visible);
        }
    }
}

mod save {
    include!("../src/save.rs");
}
