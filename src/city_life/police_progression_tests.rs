use super::*;

#[test]
fn police_tactics_escalate_across_wanted_levels_three_to_six() {
    let levels: Vec<_> = (3..=6)
        .map(|level| police_tactics(level, 0, 1.0))
        .collect();

    for pair in levels.windows(2) {
        assert!(pair[1].speed > pair[0].speed);
        assert!(pair[1].stop_distance > pair[0].stop_distance);
        assert!(pair[1].flank_distance > pair[0].flank_distance);
        assert!(pair[1].search_radius > pair[0].search_radius);
    }
}

#[test]
fn high_wanted_levels_use_more_aggressive_tactics_than_low_levels() {
    let low = police_tactics(2, 0, 1.0);
    let high = police_tactics(6, 0, 1.0);

    assert!(high.speed > low.speed);
    assert!(high.flank_distance > low.flank_distance);
    assert!(high.search_radius > low.search_radius);
}

#[test]
fn reserve_cruiser_keeps_closing_on_player_after_high_wanted_spawn() {
    use bevy::ecs::system::RunSystemOnce;
    use std::time::Duration;

    let mut world = World::new();
    let mut game = crate::Game::default();
    game.started = true;
    world.insert_resource(game);
    let mut wanted = WantedState::default();
    wanted.level = 4;
    wanted.decay_timer = 120.0;
    world.insert_resource(wanted);
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(50));
    world.insert_resource(time);
    let player = Vec2::new(100.0, 100.0);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(player.x, 0.9, player.y),
        vehicle_position: Vec3::new(120.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.insert_resource(TrafficBodies::default());
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();
    let cruiser = world
        .spawn((
            super::super::police_reserve::ReserveCruiser { index: 0 },
            Transform::from_xyz(-96.0, 0.48, 8.8),
            Visibility::Hidden,
        ))
        .id();

    world
        .run_system_once(simulate_city_life)
        .expect("the live city simulation publishes its current collision bodies");
    world
        .run_system_once(super::super::police_reserve::update_reserve_cruiser_visibility)
        .expect("a high-wanted cruiser spawns on a clear road near the player");
    assert_eq!(
        *world.get::<Visibility>(cruiser).unwrap(),
        Visibility::Visible
    );
    let spawn_position = world.get::<Transform>(cruiser).unwrap().translation;
    let spawn_distance = Vec2::new(spawn_position.x, spawn_position.z).distance(player);
    assert!(spawn_distance <= 56.0);
    assert!(spawn_distance > 12.0);

    for _ in 0..20 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        world
            .run_system_once(simulate_city_life)
            .expect("the live simulation refreshes non-reserve collision bodies each frame");
        world
            .run_system_once(super::super::police_reserve::update_reserve_cruiser_visibility)
            .expect("the reserve cruiser continues pursuing across frames");
    }

    let final_position = world.get::<Transform>(cruiser).unwrap().translation;
    let final_distance = Vec2::new(final_position.x, final_position.z).distance(player);
    assert!(
        final_distance < spawn_distance - 5.0,
        "reserve cruiser did not close the gap: {spawn_distance} m to {final_distance} m"
    );
    assert!(final_distance >= 12.0 - 1.0e-3);
}
