mod helpers;
use helpers::*;
mod menu_clock;
use menu_clock::pause_menu_clock;
mod sim;
use sim::simulate_city_life;
mod traffic_avoidance;
use traffic_avoidance::choose_real_width_traffic_offset as choose_safe_traffic_offset;

include!("city_life/core.rs");

#[cfg(test)]
mod tests {
    use super::*;
    include!("city_life/tests.rs");
}

#[cfg(test)]
mod restore_tests;

#[cfg(test)]
mod traffic_regression_tests;

#[cfg(test)]
mod traffic_avoidance_runtime_tests;

#[cfg(test)]
mod player_collision_tests;

#[cfg(test)]
mod crowd_collision_tests;

#[cfg(test)]
mod police_progression_tests;
