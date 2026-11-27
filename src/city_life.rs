use bevy::prelude::*;

mod city_events;
mod city_neon;
mod menu_clock;

mod base {
    use super::*;

    #[path = "helpers.rs"]
    mod helpers;
    use helpers::*;

    #[path = "traffic_avoidance.rs"]
    mod traffic_avoidance;
    use traffic_avoidance::choose_real_width_traffic_offset as choose_safe_traffic_offset;
    pub(super) use traffic_avoidance::turn_traffic_at_world_edge;

    #[path = "sim.rs"]
    mod sim;
    pub(super) use sim::simulate_city_life;

    include!("city_life/core.rs");

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
            .add_systems(PreUpdate, menu_clock::pause_menu_clock)
            .add_systems(Startup, city_neon::spawn_city_neon)
            .add_systems(
                Update,
                city_events::city_event_system
                    .after(base::simulate_city_life)
                    .after(crate::controls),
            )
            .add_systems(
                Update,
                base::turn_traffic_at_world_edge.after(base::simulate_city_life),
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
