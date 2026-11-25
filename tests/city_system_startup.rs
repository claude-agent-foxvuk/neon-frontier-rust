#![allow(dead_code)]

mod physics {
    include!("../src/physics.rs");
}

mod city_life {
    include!("../src/city_life.rs");

    #[cfg(test)]
    mod startup_tests {
        use super::*;
        use bevy::ecs::system::RunSystemOnce;
        use std::time::Duration;

        #[test]
        fn city_update_initializes_queries_with_officers_and_lights() {
            let mut world = World::new();
            let mut time = Time::<()>::default();
            time.advance_by(Duration::from_millis(16));
            world.insert_resource(time);
            world.init_resource::<PlayerProbe>();
            world.init_resource::<TrafficBodies>();
            world.insert_resource(WantedState {
                level: 1,
                decay_timer: 10.0,
            });
            world.init_resource::<WorldState>();
            world.init_resource::<WorldRestore>();

            world.spawn((
                Transform::from_xyz(0.0, 0.48, 0.0),
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
                Transform::from_xyz(8.0, 0.82, 0.0),
                Citizen {
                    origin: Vec3::new(8.0, 0.82, 0.0),
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
                Transform::from_xyz(-48.0, 0.48, 0.0),
                PolicePatrol {
                    home: Vec2::new(-48.0, 0.0),
                    phase: 0.0,
                },
            ));
            world.spawn((
                Transform::from_xyz(-45.0, 0.82, 0.0),
                Visibility::Visible,
                FootOfficer {
                    patrol_index: 0,
                    active: false,
                    shot_timer: 1.0,
                },
            ));
            let siren = world
                .spawn((
                    Visibility::Hidden,
                    PoliceSirenLight { phase: 0.0 },
                ))
                .id();
            let flash = world
                .spawn((
                    Visibility::Visible,
                    OfficerMuzzleFlash {
                        patrol_index: 0,
                        remaining: 0.0,
                    },
                ))
                .id();

            world
                .run_system_once(simulate_city_life)
                .expect("city simulation must initialize without ECS query conflicts");

            assert_eq!(*world.get::<Visibility>(siren).unwrap(), Visibility::Visible);
            assert_eq!(*world.get::<Visibility>(flash).unwrap(), Visibility::Hidden);
            assert_eq!(world.resource::<WorldState>().snapshot.traffic.len(), 1);
            assert_eq!(world.resource::<WorldState>().snapshot.citizens.len(), 1);
            assert_eq!(world.resource::<WorldState>().snapshot.patrols.len(), 1);
            assert_eq!(world.resource::<WorldState>().snapshot.officers.len(), 1);
        }
    }
}

mod save {
    include!("../src/save.rs");
}
