use bevy::prelude::*;

use crate::physics::Body;

use super::{TrafficBodies, WantedState};

#[derive(Component)]
pub(super) struct ReserveCruiser {
    index: usize,
}

fn reserve_vehicle_count(wanted_level: u8) -> usize {
    wanted_level.saturating_sub(2).min(4) as usize
}

pub(super) fn spawn_reserve_cruisers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_mesh = meshes.add(Cuboid::new(1.9, 0.85, 3.6));
    let cabin_mesh = meshes.add(Cuboid::new(1.45, 0.52, 1.8));
    let lightbar_mesh = meshes.add(Cuboid::new(0.95, 0.12, 0.24));
    let lamp_mesh = meshes.add(Cuboid::new(0.30, 0.14, 0.22));
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.025, 0.07, 0.15),
        metallic: 0.62,
        perceptual_roughness: 0.28,
        ..default()
    });
    let glass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.12, 0.18),
        metallic: 0.7,
        perceptual_roughness: 0.12,
        ..default()
    });
    let bar_material = materials.add(Color::srgb(0.84, 0.88, 0.92));
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.08, 0.06),
        emissive: LinearRgba::rgb(4.0, 0.12, 0.05),
        ..default()
    });
    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.22, 0.95),
        emissive: LinearRgba::rgb(0.06, 0.36, 4.2),
        ..default()
    });
    let posts = [
        (Vec3::new(-96.0, 0.48, 8.8), 0.0),
        (Vec3::new(96.0, 0.48, -8.8), 0.0),
        (Vec3::new(8.8, 0.48, 96.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(-8.8, 0.48, -96.0), std::f32::consts::FRAC_PI_2),
    ];

    for (index, (position, yaw)) in posts.into_iter().enumerate() {
        let cruiser = commands
            .spawn((
                Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::Hidden,
                ReserveCruiser { index },
            ))
            .id();
        commands.entity(cruiser).with_children(|parent| {
            parent.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(body_material.clone()),
                Transform::default(),
            ));
            parent.spawn((
                Mesh3d(cabin_mesh.clone()),
                MeshMaterial3d(glass_material.clone()),
                Transform::from_xyz(0.0, 0.56, 0.05),
            ));
            parent.spawn((
                Mesh3d(lightbar_mesh.clone()),
                MeshMaterial3d(bar_material.clone()),
                Transform::from_xyz(0.0, 0.86, -0.05),
            ));
            parent.spawn((
                Mesh3d(lamp_mesh.clone()),
                MeshMaterial3d(red_material.clone()),
                Transform::from_xyz(-0.29, 0.90, -0.05),
            ));
            parent.spawn((
                Mesh3d(lamp_mesh.clone()),
                MeshMaterial3d(blue_material.clone()),
                Transform::from_xyz(0.29, 0.90, -0.05),
            ));
        });
    }
}

pub(super) fn update_reserve_cruiser_visibility(
    game: Res<crate::Game>,
    wanted: Res<WantedState>,
    mut traffic_bodies: ResMut<TrafficBodies>,
    mut cruisers: Query<(&ReserveCruiser, &Transform, &mut Visibility)>,
) {
    let visible_count = if game.started {
        reserve_vehicle_count(wanted.level)
    } else {
        0
    };
    let mut active_bodies = Vec::with_capacity(visible_count);
    for (cruiser, transform, mut visibility) in &mut cruisers {
        let active = cruiser.index < visible_count;
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if active {
            let forward = transform.rotation * Vec3::Z;
            let forward = Vec2::new(forward.x, forward.z).normalize_or_zero();
            active_bodies.push(Body::vehicle(
                Vec2::new(transform.translation.x, transform.translation.z),
                forward.y.atan2(forward.x),
            ));
        }
    }
    traffic_bodies.oriented.extend(active_bodies);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn extra_cruisers_and_their_collision_bodies_scale_from_three_to_six_stars() {
        for wanted_level in 0_u8..=6 {
            let mut world = World::new();
            let mut game = crate::Game::default();
            game.started = true;
            world.insert_resource(game);
            let mut wanted = WantedState::default();
            wanted.level = wanted_level;
            world.insert_resource(wanted);
            world.insert_resource(TrafficBodies::default());
            for index in 0..4 {
                world.spawn((
                    ReserveCruiser { index },
                    Transform::from_xyz(index as f32 * 10.0, 0.48, 0.0),
                    Visibility::Hidden,
                ));
            }

            world
                .run_system_once(update_reserve_cruiser_visibility)
                .expect("reserve cruisers follow the wanted-level response");

            let mut query = world.query_filtered::<&Visibility, With<ReserveCruiser>>();
            let visible = query
                .iter(&world)
                .filter(|visibility| matches!(**visibility, Visibility::Visible))
                .count();
            assert_eq!(visible, reserve_vehicle_count(wanted_level));
            assert_eq!(
                world.resource::<TrafficBodies>().oriented.len(),
                reserve_vehicle_count(wanted_level)
            );
        }
    }
}
