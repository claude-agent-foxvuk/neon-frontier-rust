use bevy::prelude::*;

mod city_events;
mod city_neon;
mod graphics_settings;
mod menu_clock;
mod police_reserve;

mod base {
    use super::*;

    #[path = "../helpers.rs"]
    mod helpers;
    use helpers::*;

    #[path = "../traffic_avoidance.rs"]
    mod traffic_avoidance;
    use traffic_avoidance::choose_real_width_traffic_offset as choose_safe_traffic_offset;
    pub(super) use traffic_avoidance::turn_traffic_at_world_edge;

    #[path = "../sim.rs"]
    mod sim;
    pub(super) use sim::simulate_city_life;

    include!("city_life/core.rs");

    const OFFICER_RETURN_DISTANCE: f32 = 42.0;
    const OFFICER_BOARD_DISTANCE: f32 = 3.0;
    const OFFICER_RETURN_SPEED: f32 = 8.0;

    fn officer_should_return_to_patrol(
        wanted_level: u8,
        player_position: Vec2,
        patrol_position: Vec2,
    ) -> bool {
        wanted_level >= 2
            && player_position.is_finite()
            && patrol_position.is_finite()
            && player_position.distance_squared(patrol_position)
                >= OFFICER_RETURN_DISTANCE * OFFICER_RETURN_DISTANCE
    }

    pub(super) fn return_foot_officers(
        time: Res<Time>,
        probe: Res<PlayerProbe>,
        wanted: Res<WantedState>,
        mut officers: Query<
            (&mut Transform, &mut FootOfficer, &mut Visibility),
            (
                Without<TrafficCar>,
                Without<Citizen>,
                Without<PolicePatrol>,
                Without<PoliceSirenLight>,
            ),
        >,
        patrols: Query<
            &Transform,
            (
                With<PolicePatrol>,
                Without<TrafficCar>,
                Without<Citizen>,
                Without<FootOfficer>,
            ),
        >,
    ) {
        let dt = time.delta_secs().clamp(0.0, 0.05);
        if dt <= 0.0 || wanted.level < 2 {
            return;
        }

        let player_position = Vec2::new(probe.position.x, probe.position.z);
        let patrol_positions: Vec<_> = patrols
            .iter()
            .map(|transform| Vec2::new(transform.translation.x, transform.translation.z))
            .collect();

        for (mut transform, mut officer, mut visibility) in &mut officers {
            if !officer.active {
                continue;
            }
            let Some(&patrol_position) = patrol_positions.get(officer.patrol_index) else {
                continue;
            };
            if !officer_should_return_to_patrol(wanted.level, player_position, patrol_position) {
                continue;
            }

            let position = Vec2::new(transform.translation.x, transform.translation.z);
            let to_patrol = patrol_position - position;
            let distance = to_patrol.length();
            if distance <= OFFICER_BOARD_DISTANCE {
                officer.active = false;
                *visibility = Visibility::Hidden;
                continue;
            }

            let step = (OFFICER_RETURN_SPEED * dt).min(distance - OFFICER_BOARD_DISTANCE);
            let movement = to_patrol.normalize_or_zero() * step;
            let next = crate::physics::move_with_slide(position, movement, |point| {
                city_collision(point, OFFICER_BODY_RADIUS)
            });
            transform.translation.x = next.x;
            transform.translation.z = next.y;
            let facing = patrol_position - next;
            if facing.length_squared() > 1.0e-5 {
                transform.rotation = Quat::from_rotation_y(facing.x.atan2(facing.y));
            }
            if next.distance(patrol_position) <= OFFICER_BOARD_DISTANCE {
                officer.active = false;
                *visibility = Visibility::Hidden;
            }
        }
    }

    #[cfg(test)]
    mod police_return_tests {
        use super::*;

        #[test]
        fn officers_return_only_when_a_wanted_player_escapes_the_patrol() {
            assert!(!officer_should_return_to_patrol(
                1,
                Vec2::new(OFFICER_RETURN_DISTANCE + 1.0, 0.0),
                Vec2::ZERO,
            ));
            assert!(!officer_should_return_to_patrol(
                2,
                Vec2::new(OFFICER_RETURN_DISTANCE - 0.01, 0.0),
                Vec2::ZERO,
            ));
            assert!(officer_should_return_to_patrol(
                2,
                Vec2::new(OFFICER_RETURN_DISTANCE, 0.0),
                Vec2::ZERO,
            ));
            assert!(!officer_should_return_to_patrol(
                6,
                Vec2::new(f32::INFINITY, 0.0),
                Vec2::ZERO,
            ));
        }

        #[test]
        fn fleeing_officer_returns_to_car_and_hides_when_boarding() {
            use bevy::ecs::system::RunSystemOnce;
            use std::time::Duration;

            let mut world = World::new();
            let mut time = Time::<()>::default();
            time.advance_by(Duration::from_millis(50));
            world.insert_resource(time);
            world.insert_resource(PlayerProbe {
                position: Vec3::new(100.0, 0.9, 0.0),
                vehicle_position: Vec3::new(100.0, 0.65, 0.0),
                forward: Vec3::NEG_Z,
                in_vehicle: true,
                punch: false,
                health: 100.0,
            });
            let mut wanted = WantedState::default();
            wanted.level = 3;
            world.insert_resource(wanted);

            world.spawn((
                Transform::from_xyz(0.0, 0.48, 0.0),
                PolicePatrol {
                    home: Vec2::ZERO,
                    phase: 0.0,
                },
            ));
            let officer = world
                .spawn((
                    Transform::from_xyz(16.0, 0.82, 0.0),
                    FootOfficer {
                        patrol_index: 0,
                        active: true,
                        shot_timer: 0.5,
                    },
                    Visibility::Visible,
                ))
                .id();

            world
                .run_system_once(return_foot_officers)
                .expect("officer returns when the player escapes");
            assert!(world.get::<Transform>(officer).unwrap().translation.x < 16.0);
            assert!(world.get::<FootOfficer>(officer).unwrap().active);
            assert_eq!(*world.get::<Visibility>(officer).unwrap(), Visibility::Visible);

            world.get_mut::<Transform>(officer).unwrap().translation.x = 2.9;
            world
                .run_system_once(return_foot_officers)
                .expect("officer boards the patrol car after reaching it");
            assert!(!world.get::<FootOfficer>(officer).unwrap().active);
            assert_eq!(*world.get::<Visibility>(officer).unwrap(), Visibility::Hidden);
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        include!("city_life/tests.rs");
    }

    #[cfg(test)]
    mod restore_tests {
        include!("city_life/restore_tests.rs");
    }

    #[cfg(test)]
    mod traffic_regression_tests {
        include!("city_life/traffic_regression_tests.rs");
    }

    #[cfg(test)]
    mod traffic_avoidance_runtime_tests {
        include!("city_life/traffic_avoidance_runtime_tests.rs");
    }

    #[cfg(test)]
    mod player_collision_tests {
        include!("city_life/player_collision_tests.rs");
    }

    #[cfg(test)]
    mod crowd_collision_tests {
        include!("city_life/crowd_collision_tests.rs");
    }

    #[cfg(test)]
    mod police_progression_tests {
        include!("city_life/police_progression_tests.rs");
    }
}

pub use base::{
    CitizenSave, OfficerSave, PatrolSave, PlayerProbe, TrafficBodies, TrafficSave, WantedState,
    WorldRestore, WorldSnapshot, WorldState,
};

pub struct CityLifePlugin;

impl Plugin for CityLifePlugin {
    fn build(&self, app: &mut App) {
        <base::CityLifePlugin as Plugin>::build(&base::CityLifePlugin, app);
        app.init_resource::<city_events::CityEvents>()
            .init_resource::<graphics_settings::GraphicsProfile>()
            .add_systems(PreUpdate, menu_clock::pause_menu_clock)
            .add_systems(
                Startup,
                (
                    city_neon::spawn_city_neon,
                    police_reserve::spawn_reserve_cruisers,
                ),
            )
            .add_systems(
                Update,
                graphics_settings::toggle_graphics_profile
                    .after(crate::controls)
                    .before(crate::update_world),
            )
            .add_systems(
                Update,
                city_events::city_event_system
                    .after(base::simulate_city_life)
                    .after(crate::controls),
            )
            .add_systems(
                Update,
                base::turn_traffic_at_world_edge.after(base::simulate_city_life),
            )
            .add_systems(
                Update,
                base::return_foot_officers.after(base::simulate_city_life),
            )
            .add_systems(
                Update,
                police_reserve::update_reserve_cruiser_visibility
                    .after(base::simulate_city_life)
                    .after(crate::controls),
            );
    }
}

#[cfg(test)]
mod plugin_tests {
    use super::*;
    use bevy::time::Virtual;
    use std::time::Duration;

    #[test]
    fn city_life_plugin_pauses_menu_clock_and_resumes_after_start() {
        let mut app = App::new();
        app.insert_resource(Time::<Virtual>::default());
        app.insert_resource(Time::<()>::default());
        app.insert_resource(crate::Game::default());
        app.add_plugins(CityLifePlugin);

        app.world_mut()
            .resource_mut::<Time<Virtual>>()
            .advance_by(Duration::from_millis(16));
        app.world_mut()
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(16));
        app.world_mut().run_schedule(PreUpdate);

        assert!(app.world().resource::<Time<Virtual>>().is_paused());
        assert_eq!(app.world().resource::<Time<Virtual>>().delta(), Duration::ZERO);
        assert_eq!(app.world().resource::<Time<()>>().delta(), Duration::ZERO);

        app.world_mut().resource_mut::<crate::Game>().started = true;
        app.world_mut().run_schedule(PreUpdate);
        assert!(!app.world().resource::<Time<Virtual>>().is_paused());
    }
}
