mod helpers;
use helpers::*;
mod sim;
use sim::simulate_city_life;

include!("city_life/core.rs");

#[cfg(test)]
mod tests {
    use super::*;
    include!("city_life/tests.rs");
}

#[cfg(test)]
mod restore_tests;
