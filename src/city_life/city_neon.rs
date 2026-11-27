use bevy::prelude::*;

const ROAD_SPACING: f32 = 48.0;
const BUILDING_HALF: f32 = 14.0;

#[derive(Clone, Copy)]
struct NeonDetail {
    position: Vec3,
    rotation: Quat,
    color: usize,
    sign: bool,
}

fn city_neon_details() -> Vec<NeonDetail> {
    let mut details = Vec::with_capacity(216);
    for grid_x in -3_i32..=2 {
        for grid_z in -3_i32..=2 {
            let x = grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let z = grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let height = 8.0 + (((grid_x + 3) * 7 + (grid_z + 3) * 11) % 6) as f32 * 4.0;
            let color = ((grid_x + 3 + (grid_z + 3) * 2) % 3) as usize;
            for side in [-1.0_f32, 1.0] {
                details.push(NeonDetail {
                    position: Vec3::new(x, height + 0.18, z + side * BUILDING_HALF),
                    rotation: Quat::IDENTITY,
                    color,
                    sign: false,
                });
                details.push(NeonDetail {
                    position: Vec3::new(x + side * BUILDING_HALF, height + 0.18, z),
                    rotation: Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    color,
                    sign: false,
                });
                details.push(NeonDetail {
                    position: Vec3::new(x, 5.1, z + side * (BUILDING_HALF + 0.17)),
                    rotation: Quat::IDENTITY,
                    color: (color + 1) % 3,
                    sign: true,
                });
            }
        }
    }
    details
}

pub(super) fn spawn_city_neon(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let roof_strip = meshes.add(Cuboid::new(28.0, 0.22, 0.22));
    let sign_strip = meshes.add(Cuboid::new(8.0, 1.0, 0.18));
    let colors = [
        (Color::srgb(0.05, 0.72, 0.95), LinearRgba::rgb(0.0, 2.4, 4.2)),
        (Color::srgb(0.92, 0.16, 0.55), LinearRgba::rgb(4.0, 0.15, 1.7)),
        (Color::srgb(1.0, 0.62, 0.13), LinearRgba::rgb(4.0, 1.7, 0.1)),
    ];
    let materials: Vec<_> = colors
        .into_iter()
        .map(|(base_color, emissive)| {
            materials.add(StandardMaterial {
                base_color,
                emissive,
                metallic: 0.2,
                perceptual_roughness: 0.4,
                ..default()
            })
        })
        .collect();
    for detail in city_neon_details() {
        let mesh = if detail.sign { &sign_strip } else { &roof_strip };
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(materials[detail.color].clone()),
            Transform::from_translation(detail.position).with_rotation(detail.rotation),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_building_has_four_roof_strips_and_two_facade_signs() {
        let details = city_neon_details();
        assert_eq!(details.len(), 36 * 6);
        assert_eq!(details.iter().filter(|detail| detail.sign).count(), 36 * 2);
        assert_eq!(details.iter().filter(|detail| !detail.sign).count(), 36 * 4);
        assert!(details.iter().all(|detail| detail.color < 3));
        assert!(details.iter().all(|detail| detail.position.is_finite()));
    }

    #[test]
    fn signs_are_above_street_level_and_roof_trims_above_each_roof() {
        for detail in city_neon_details() {
            if detail.sign {
                assert_eq!(detail.position.y, 5.1);
            } else {
                assert!(detail.position.y > 8.0);
            }
        }
    }
}
