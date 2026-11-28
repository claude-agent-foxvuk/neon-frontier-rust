use bevy::prelude::*;

const ROAD_SPACING: f32 = 48.0;
const BUILDING_HALF: f32 = 14.0;
const PARK_TILE_SIZE: f32 = 4.2;
const PARK_CORNER_OFFSET: f32 = 7.75;
const PARK_PATH_WIDTH: f32 = 0.34;
const PARK_BENCH_OFFSET: f32 = 0.68;
const PARK_TREE_COLLISION_RADIUS: f32 = 0.32;
const PARK_BENCH_HALF_LENGTH: f32 = 0.72;
const PARK_BENCH_HALF_WIDTH: f32 = 0.23;

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

pub(super) fn central_park_collision_bodies() -> [crate::physics::Body; 12] {
    std::array::from_fn(|index| {
        let centers = central_park_centers();
        if index < centers.len() {
            crate::physics::Body::new(
                centers[index],
                PARK_TREE_COLLISION_RADIUS,
                PARK_TREE_COLLISION_RADIUS,
                0.0,
            )
        } else {
            let bench_index = index - centers.len();
            let park = centers[bench_index / 2];
            let side = if bench_index % 2 == 0 { -1.0 } else { 1.0 };
            let yaw = if side < 0.0 {
                std::f32::consts::FRAC_PI_2
            } else {
                -std::f32::consts::FRAC_PI_2
            };
            crate::physics::Body::new(
                park + Vec2::new(side * PARK_BENCH_OFFSET, 0.0),
                PARK_BENCH_HALF_LENGTH,
                PARK_BENCH_HALF_WIDTH,
                yaw,
            )
        }
    })
}

fn spawn_central_park(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let lawn_mesh = meshes.add(Cuboid::new(PARK_TILE_SIZE, 0.12, PARK_TILE_SIZE));
    let curb_mesh = meshes.add(Cuboid::new(PARK_TILE_SIZE, 0.18, 0.16));
    let path_x_mesh = meshes.add(Cuboid::new(PARK_TILE_SIZE - 0.72, 0.06, PARK_PATH_WIDTH));
    let path_z_mesh = meshes.add(Cuboid::new(PARK_PATH_WIDTH, 0.06, PARK_TILE_SIZE - 0.72));
    let trunk_mesh = meshes.add(Cylinder::new(0.22, 1.8));
    let canopy_mesh = meshes.add(Sphere::new(1.0));
    let bench_seat_mesh = meshes.add(Cuboid::new(1.35, 0.26, 0.42));
    let bench_back_mesh = meshes.add(Cuboid::new(1.35, 0.7, 0.14));
    let flower_mesh = meshes.add(Sphere::new(0.16));
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
    let path_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.58, 0.57, 0.49),
        perceptual_roughness: 0.9,
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
    let flower_materials = [
        materials.add(Color::srgb(0.92, 0.30, 0.39)),
        materials.add(Color::srgb(0.98, 0.68, 0.20)),
        materials.add(Color::srgb(0.55, 0.34, 0.82)),
    ];

    for (park_index, center) in central_park_centers().into_iter().enumerate() {
        commands.spawn((
            Mesh3d(lawn_mesh.clone()),
            MeshMaterial3d(lawn_material.clone()),
            Transform::from_xyz(center.x, 0.12, center.y),
        ));
        commands.spawn((
            Mesh3d(path_x_mesh.clone()),
            MeshMaterial3d(path_material.clone()),
            Transform::from_xyz(center.x, 0.22, center.y - 1.52),
        ));
        commands.spawn((
            Mesh3d(path_x_mesh.clone()),
            MeshMaterial3d(path_material.clone()),
            Transform::from_xyz(center.x, 0.22, center.y + 1.52),
        ));
        commands.spawn((
            Mesh3d(path_z_mesh.clone()),
            MeshMaterial3d(path_material.clone()),
            Transform::from_xyz(center.x - 1.52, 0.22, center.y),
        ));
        commands.spawn((
            Mesh3d(path_z_mesh.clone()),
            MeshMaterial3d(path_material.clone()),
            Transform::from_xyz(center.x + 1.52, 0.22, center.y),
        ));
        for (offset, rotation) in [
            (Vec3::new(0.0, 0.24, -2.02), Quat::IDENTITY),
            (Vec3::new(0.0, 0.24, 2.02), Quat::IDENTITY),
            (
                Vec3::new(-2.02, 0.24, 0.0),
                Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
            ),
            (
                Vec3::new(2.02, 0.24, 0.0),
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
        for (side, yaw) in [
            (-1.0, std::f32::consts::FRAC_PI_2),
            (1.0, -std::f32::consts::FRAC_PI_2),
        ] {
            let bench_position = Vec3::new(center.x + side * PARK_BENCH_OFFSET, 0.34, center.y);
            let bench_rotation = Quat::from_rotation_y(yaw);
            commands.spawn((
                Mesh3d(bench_seat_mesh.clone()),
                MeshMaterial3d(bench_material.clone()),
                Transform::from_translation(bench_position).with_rotation(bench_rotation),
            ));
            commands.spawn((
                Mesh3d(bench_back_mesh.clone()),
                MeshMaterial3d(bench_material.clone()),
                Transform::from_translation(bench_position + Vec3::new(0.0, 0.44, 0.0))
                    .with_rotation(bench_rotation),
            ));
        }
        for (flower_index, (x, z)) in [
            (-1.10, -1.10),
            (-1.10, 1.10),
            (1.10, -1.10),
            (1.10, 1.10),
        ]
        .into_iter()
        .enumerate()
        {
            let color = (park_index + flower_index) % flower_materials.len();
            commands.spawn((
                Mesh3d(flower_mesh.clone()),
                MeshMaterial3d(flower_materials[color].clone()),
                Transform::from_xyz(center.x + x, 0.36, center.y + z)
                    .with_scale(Vec3::new(1.0, 0.55, 1.0)),
            ));
        }
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
    use crate::physics::{bodies_overlap, Body};

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
    fn park_paths_and_lawns_fit_in_the_open_corners_of_the_crossroads() {
        let road_half_width = 5.5;
        let building_edge = 10.0;
        let half_tile = PARK_TILE_SIZE * 0.5;
        let centers = central_park_centers();

        assert_eq!(centers.len(), 4);
        assert!(PARK_PATH_WIDTH > 0.0 && PARK_PATH_WIDTH < PARK_TILE_SIZE);
        for center in centers {
            assert!(center.x.abs() - half_tile > road_half_width);
            assert!(center.y.abs() - half_tile > road_half_width);
            assert!(center.x.abs() + half_tile < building_edge);
            assert!(center.y.abs() + half_tile < building_edge);
            assert!(1.52 + PARK_PATH_WIDTH * 0.5 < half_tile);
            assert!((PARK_TILE_SIZE - 0.72) * 0.5 < half_tile);
            assert!(2.02 + 0.08 <= half_tile + 1.0e-5);
        }
    }

    #[test]
    fn park_trees_and_benches_block_walkers_while_cross_paths_stay_clear() {
        let centers = central_park_centers();
        let obstacles = central_park_collision_bodies();
        assert_eq!(obstacles.len(), 12);

        for (park_index, center) in centers.into_iter().enumerate() {
            let walker = |position: Vec2| Body::new(position, 0.45, 0.45, 0.0);
            assert!(bodies_overlap(&walker(center), &obstacles[park_index]));

            for side in [-1.0, 1.0] {
                let bench_index = centers.len() + park_index * 2 + usize::from(side > 0.0);
                let bench_position = center + Vec2::new(side * PARK_BENCH_OFFSET, 0.0);
                assert!(bodies_overlap(&walker(bench_position), &obstacles[bench_index]));
            }

            for offset in [-1.52, 1.52] {
                assert!(!obstacles
                    .iter()
                    .any(|body| bodies_overlap(&walker(center + Vec2::new(offset, 0.0)), body)));
                assert!(!obstacles
                    .iter()
                    .any(|body| bodies_overlap(&walker(center + Vec2::new(0.0, offset)), body)));
            }
        }
    }
}
