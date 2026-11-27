use bevy::{post_process::bloom::Bloom, prelude::*};

fn next_quality_enabled(bloom_enabled: bool, shadows_enabled: bool) -> bool {
    !bloom_enabled && !shadows_enabled
}

pub(super) fn toggle_graphics_profile(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<crate::Game>,
    mut sun: Query<&mut DirectionalLight, With<crate::SunLight>>,
    cameras: Query<(Entity, Option<&Bloom>), With<crate::FollowCamera>>,
    mut commands: Commands,
) {
    if !keys.just_pressed(KeyCode::F2) {
        return;
    }

    let quality_enabled = next_quality_enabled(
        game.bloom,
        sun.iter().any(|light| light.shadows_enabled),
    );
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

    #[test]
    fn graphics_profile_toggles_between_quality_and_performance() {
        assert!(!next_quality_enabled(true, true));
        assert!(!next_quality_enabled(false, true));
        assert!(next_quality_enabled(false, false));
    }
}
