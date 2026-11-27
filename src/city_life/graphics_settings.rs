use bevy::{post_process::bloom::Bloom, prelude::*};

#[derive(Default, Resource)]
pub(super) struct GraphicsProfile {
    performance: bool,
}

fn next_performance_mode(current: bool) -> bool {
    !current
}

pub(super) fn toggle_graphics_profile(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<crate::Game>,
    mut profile: ResMut<GraphicsProfile>,
    mut sun: Query<&mut DirectionalLight, With<crate::SunLight>>,
    cameras: Query<(Entity, Option<&Bloom>), With<crate::FollowCamera>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::F2) {
        return;
    }

    profile.performance = next_performance_mode(profile.performance);
    let quality_enabled = !profile.performance;
    game.bloom = quality_enabled;
    for mut light in &mut sun {
        light.shadows_enabled = quality_enabled;
    }
    for (camera, bloom) in &cameras {
        if quality_enabled && bloom.is_none() {
            commands.entity(camera).insert(Bloom::NATURAL);
        } else if !quality_enabled && bloom.is_some() {
            commands.entity(camera).remove::<Bloom>();
        }
    }

    game.notice = if quality_enabled {
        "Графика: качество, тени и bloom включены".to_string()
    } else {
        "Графика: производительность, тени и bloom выключены".to_string()
    };
    game.notice_timer = 3.0;
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn graphics_profile_toggles_between_quality_and_performance() {
        assert!(next_performance_mode(false));
        assert!(!next_performance_mode(true));
    }

    #[test]
    fn f2_changes_bloom_and_sun_shadows_in_both_directions() {
        for initial_performance in [false, true] {
            let mut world = World::new();
            let mut keys = ButtonInput::<KeyCode>::default();
            keys.press(KeyCode::F2);
            world.insert_resource(keys);
            world.insert_resource(GraphicsProfile {
                performance: initial_performance,
            });

            let mut game = crate::Game::default();
            game.bloom = !initial_performance;
            world.insert_resource(game);

            let sun = world
                .spawn((
                    crate::SunLight,
                    DirectionalLight {
                        shadows_enabled: !initial_performance,
                        ..default()
                    },
                ))
                .id();
            let camera = world.spawn(crate::FollowCamera).id();
            if !initial_performance {
                world.entity_mut(camera).insert(Bloom::NATURAL);
            }

            world
                .run_system_once(toggle_graphics_profile)
                .expect("F2 applies the selected graphics profile");
            world.flush();

            let expected_quality = initial_performance;
            assert_eq!(
                world.resource::<GraphicsProfile>().performance,
                !initial_performance
            );
            assert_eq!(world.resource::<crate::Game>().bloom, expected_quality);
            assert_eq!(
                world.get::<DirectionalLight>(sun).unwrap().shadows_enabled,
                expected_quality
            );
            assert_eq!(world.get::<Bloom>(camera).is_some(), expected_quality);
            assert_eq!(world.resource::<crate::Game>().notice_timer, 3.0);
        }
    }
}
