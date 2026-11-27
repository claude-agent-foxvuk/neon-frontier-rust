use bevy::prelude::*;

mod helpers;
use helpers::*;
mod menu_clock;
mod sim;
use sim::simulate_city_life;
mod traffic_avoidance;
use traffic_avoidance::choose_real_width_traffic_offset as choose_safe_traffic_offset;

mod base {
    use super::*;

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
        app.add_systems(PreUpdate, menu_clock::pause_menu_clock);
    }
}
