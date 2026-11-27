use bevy::prelude::*;

const ROAD_SPACING: f32 = 48.0;
const BUILDING_HALF: f32 = 14.0;
const PARK_TILE_SIZE: f32 = 3.8;
const PARK_CORNER_OFFSET: f32 = 8.0;

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

fn central_park_centers() -> [Vec2; 4] {
    [
        Vec2::new(-PARK_CORNER_OFFSET, -PARK_CORNER_OFFSET),
        Vec2::new(-PARK_CORNER_OFFSET, PARK_CORNER_OFFSET),
        Vec2::new(PARK_CORNER_OFFSET, -PARK_CORNER_OFFSET),
        Vec2::new(PARK_CORNER_OFFSET, PARK_CORNER_OFFSET),
    ]
}

fn spawn_central_park(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let lawn_mesh = meshes.add(Cuboid::new(PARK_TILE_SIZE, 0.12, PARK_TILE_SIZE));
    let curb_mesh = meshes.add(Cuboid::new(PARK_TILE_SIZE, 0.18, 0.16));
    let trunk_mesh = meshes.add(Cylinder::new(0.22, 1.8));
    let canopy_mesh = meshes.add(Sphere::new(1.0));
    let bench_seat_mesh = meshes.add(Cuboid::new(1.4, 0.26, 0.42));
    let bench_back_mesh = meshes.add(Cuboid::new(1.4, 0.7, 0.14));
    let lawn_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.34, 0.20),
        perceptual_roughness: 0.94,
        ..default()
    });
    let curb_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.48, 0.48),
        perceptual_roughness: 0.82,
        ..default()
    });
    let trunk_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.24, 0.13, 0.07),
        perceptual_roughness: 0.9,
        ..default()
    });
    let canopy_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.38, 0.21),
        perceptual_roughness: 0.88,
        ..default()
    });
    let bench_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.27, 0.12),
        perceptual_roughness: 0.78,
        ..default()
    });

    for center in central_park_centers() {
        commands.spawn((
            Mesh3d(lawn_mesh.clone()),
            MeshMaterial3d(lawn_material.clone()),
            Transform::from_xyz(center.x, 0.12, center.y),
        ));
        for (offset, rotation) in [
            (Vec3::new(0.0, 0.24, -1.82), Quat::IDENTITY),
            (Vec3::new(0.0, 0.24, 1.82), Quat::IDENTITY),
            (
                Vec3::new(-1.82, 0.24, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ),
            (
                Vec3::new(1.82, 0.24, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ),
        ] {
            commands.spawn((
                Mesh3d(curb_mesh.clone()),
                MeshMaterial3d(curb_material.clone()),
                Transform::from_translation(Vec3::new(center.x, 0.0, center.y) + offset)
                    .with_rotation(rotation),
            ));
        }
        commands.spawn((
            Mesh3d(trunk_mesh.clone()),
            MeshMaterial3d(trunk_material.clone()),
            Transform::from_xyz(center.x, 1.08, center.y),
        ));
        commands.spawn((
            Mesh3d(canopy_mesh.clone()),
            MeshMaterial3d(canopy_material.clone()),
            Transform::from_xyz(center.x, 2.45, center.y),
        ));
        commands.spawn((
            Mesh3d(bench_seat_mesh.clone()),
            MeshMaterial3d(bench_material.clone()),
            Transform::from_xyz(center.x + 1.0, 0.34, center.y),
        ));
        commands.spawn((
            Mesh3d(bench_back_mesh.clone()),
            MeshMaterial3d(bench_material.clone()),
            Transform::from_xyz(center.x + 1.0, 0.78, center.y + 0.2),
        ));
    }
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
    let neon_materials: Vec<_> = colors
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
            MeshMaterial3d(neon_materials[detail.color].clone()),
            Transform::from_translation(detail.position).with_rotation(detail.rotation),
        ));
    }
    spawn_central_park(&mut commands, &mut meshes, &mut materials);
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

    #[test]
    fn central_park_planters_fit_in_the_open_corners_of_the_crossroads() {
        let road_half_width = 5.5;
        let building_edge = 10.0;
        let half_tile = PARK_TILE_SIZE * 0.5;
        let centers = central_park_centers();

        assert_eq!(centers.len(), 4);
        for center in centers {
            assert!((center.x.abs() - half_tile) > road_half_width);
            assert!((center.y.abs() - half_tile) > road_half_width);
            assert!(center.x.abs() + half_tile < building_edge);
            assert!(center.y.abs() + half_tile < building_edge);
        }
    }
}
