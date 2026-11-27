use bevy::{prelude::*, time::Virtual};
use std::time::Duration;

pub(super) fn pause_menu_clock(
    game: Option<Res<crate::Game>>,
    mut virtual_time: ResMut<Time<Virtual>>,
    mut default_time: ResMut<Time>,
) {
    if game.as_ref().is_some_and(|game| !game.started) {
        if !virtual_time.is_paused() {
            virtual_time.pause();
        }
        virtual_time.update_with_duration(Duration::ZERO);
        default_time.update_with_duration(Duration::ZERO);
    } else if virtual_time.is_paused() {
        virtual_time.unpause();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn menu_pauses_both_game_clocks_and_new_game_resumes_virtual_time() {
        let mut world = World::new();
        let mut virtual_time = Time::<Virtual>::default();
        virtual_time.advance_by(Duration::from_millis(16));
        world.insert_resource(virtual_time);
        let mut default_time = Time::<()>::default();
        default_time.advance_by(Duration::from_millis(16));
        world.insert_resource(default_time);
        world.insert_resource(crate::Game::default());

        world
            .run_system_once(pause_menu_clock)
            .expect("main menu pauses the game clocks");

        assert!(world.resource::<Time<Virtual>>().is_paused());
        assert_eq!(world.resource::<Time<Virtual>>().delta(), Duration::ZERO);
        assert_eq!(world.resource::<Time<()>>().delta(), Duration::ZERO);

        world.resource_mut::<crate::Game>().started = true;
        world
            .run_system_once(pause_menu_clock)
            .expect("starting a new game resumes virtual time");

        assert!(!world.resource::<Time<Virtual>>().is_paused());
    }
}
