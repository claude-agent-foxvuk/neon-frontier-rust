use bevy::prelude::*;

const CITY_LIMIT: f32 = 160.0;
const ROAD_SPACING: f32 = 48.0;
const ROAD_HALF_WIDTH: f32 = 5.5;
const SIDEWALK_OFFSET: f32 = 7.0;
const BUILDING_HALF: f32 = 14.0;
const BUILDING_EDGE_FROM_ROAD: f32 = ROAD_SPACING * 0.5 - BUILDING_HALF;

#[derive(Component)]
struct TrafficCar {
    speed: f32,
    direction: f32,
    along_x: bool,
}

#[derive(Component)]
struct Citizen {
    origin: Vec3,
    phase: f32,
    speed: f32,
    range: f32,
    along_x: bool,
}

pub struct CityLifePlugin;

impl Plugin for CityLifePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_city_life)
            .add_systems(Update, animate_city_life);
    }
}

fn traffic_rotation(along_x: bool, direction: f32) -> Quat {
    let forward = (if along_x { Vec3::X } else { Vec3::Z }) * direction;
    Quat::from_rotation_y(forward.x.atan2(forward.z))
}

fn pedestrian_rotation(along_x: bool, direction: f32) -> Quat {
    let forward = (if along_x { Vec3::X } else { Vec3::Z }) * direction;
    Quat::from_rotation_y(forward.x.atan2(forward.z))
}

fn streetlight_positions() -> Vec<Vec3> {
    let mut positions = Vec::with_capacity(36);
    for road_x in [-96.0, 0.0, 96.0] {
        for road_z in [-96.0, 0.0, 96.0] {
            for side_x in [-1.0, 1.0] {
                for side_z in [-1.0, 1.0] {
                    positions.push(Vec3::new(
                        road_x + side_x * 7.4,
                        0.0,
                        road_z + side_z * 7.4,
                    ));
                }
            }
        }
    }
    positions
}

fn spawn_city_life(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let car_mesh = meshes.add(Cuboid::new(1.9, 0.85, 3.6));
    let citizen_mesh = meshes.add(Cuboid::new(0.48, 1.55, 0.42));
    let car_colors = [
        Color::srgb(0.78, 0.20, 0.12),
        Color::srgb(0.12, 0.44, 0.72),
        Color::srgb(0.77, 0.60, 0.16),
        Color::srgb(0.34, 0.68, 0.53),
    ];
    let car_materials: Vec<_> = car_colors
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                metallic: 0.48,
                perceptual_roughness: 0.34,
                ..default()
            })
        })
        .collect();
    let citizen_materials = [
        materials.add(Color::srgb(0.86, 0.40, 0.24)),
        materials.add(Color::srgb(0.24, 0.67, 0.78)),
        materials.add(Color::srgb(0.79, 0.70, 0.48)),
        materials.add(Color::srgb(0.49, 0.39, 0.73)),
    ];
    let beacon_materials: Vec<_> = [
        (
            Color::srgb(0.05, 0.55, 0.72),
            LinearRgba::rgb(0.0, 2.2, 4.0),
        ),
        (
            Color::srgb(0.72, 0.36, 0.08),
            LinearRgba::rgb(4.0, 1.4, 0.1),
        ),
        (
            Color::srgb(0.58, 0.16, 0.62),
            LinearRgba::rgb(2.8, 0.2, 3.5),
        ),
        (
            Color::srgb(0.18, 0.58, 0.32),
            LinearRgba::rgb(0.2, 3.2, 0.6),
        ),
    ]
    .into_iter()
    .map(|(base_color, emissive)| {
        materials.add(StandardMaterial {
            base_color,
            emissive,
            metallic: 0.35,
            perceptual_roughness: 0.25,
            ..default()
        })
    })
    .collect();

    for road in 0..7 {
        let lane = road as f32 * ROAD_SPACING - 144.0;
        let direction = if road % 2 == 0 { 1.0 } else { -1.0 };
        let along_x = Vec3::new(-126.0 + road as f32 * 34.0, 0.48, lane + 2.1);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(car_materials[road % car_materials.len()].clone()),
            Transform::from_translation(along_x).with_rotation(traffic_rotation(true, direction)),
            TrafficCar {
                speed: 6.0 + (road % 3) as f32 * 1.2,
                direction,
                along_x: true,
            },
        ));

        let direction_z = -direction;
        let along_z = Vec3::new(lane - 2.1, 0.48, -118.0 + road as f32 * 32.0);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(car_materials[(road + 1) % car_materials.len()].clone()),
            Transform::from_translation(along_z)
                .with_rotation(traffic_rotation(false, direction_z)),
            TrafficCar {
                speed: 6.6 + (road % 2) as f32 * 1.4,
                direction: direction_z,
                along_x: false,
            },
        ));
    }

    for index in 0..14 {
        let road = (index % 7) as f32;
        let side = if index < 7 { 1.0 } else { -1.0 };
        let lane = road * ROAD_SPACING - 144.0;
        let along_x = index >= 7;
        let origin = if along_x {
            Vec3::new(-108.0 + road * 31.0, 0.82, lane + side * SIDEWALK_OFFSET)
        } else {
            Vec3::new(lane + side * SIDEWALK_OFFSET, 0.82, -108.0 + road * 31.0)
        };
        commands.spawn((
            Mesh3d(citizen_mesh.clone()),
            MeshMaterial3d(citizen_materials[index % citizen_materials.len()].clone()),
            Transform::from_translation(origin),
            Citizen {
                origin,
                phase: index as f32 * 0.71,
                speed: 0.35 + (index % 4) as f32 * 0.08,
                range: 7.0,
                along_x,
            },
        ));
    }

    let landmark_mesh = meshes.add(Cuboid::default());
    for (index, (grid_x, grid_z)) in [(-1, -1), (-1, 1), (1, -1), (1, 1)].into_iter().enumerate() {
        let x = grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
        let z = grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
        let height = 8.0 + (((grid_x + 3) * 7 + (grid_z + 3) * 11) % 6) as f32 * 4.0;
        commands.spawn((
            Mesh3d(landmark_mesh.clone()),
            MeshMaterial3d(beacon_materials[index].clone()),
            Transform::from_xyz(x, height + 2.0, z).with_scale(Vec3::new(1.5, 4.0, 1.5)),
        ));
    }

    let lamp_post_mesh = meshes.add(Cuboid::new(0.18, 6.2, 0.18));
    let lamp_arm_mesh = meshes.add(Cuboid::new(1.2, 0.12, 0.12));
    let lamp_head_mesh = meshes.add(Cuboid::new(0.42, 0.18, 0.36));
    let lamp_post_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.12, 0.17),
        metallic: 0.78,
        perceptual_roughness: 0.32,
        ..default()
    });
    let lamp_arm_material = lamp_post_material.clone();
    let lamp_glow_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.52, 0.72),
            emissive: LinearRgba::rgb(0.05, 2.0, 4.2),
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.66, 0.18, 0.62),
            emissive: LinearRgba::rgb(2.5, 0.1, 2.2),
            ..default()
        }),
    ];
    for (index, position) in streetlight_positions().into_iter().enumerate() {
        let side_x = if position.x < 0.0 { -1.0 } else { 1.0 };
        let inward_arm = Vec3::new(-side_x * 0.42, 0.0, 0.0);
        let head_offset = Vec3::new(-side_x * 0.95, 0.0, 0.0);
        let bulb_material = lamp_glow_materials[index % lamp_glow_materials.len()].clone();
        commands.spawn((
            Mesh3d(lamp_post_mesh.clone()),
            MeshMaterial3d(lamp_post_material.clone()),
            Transform::from_translation(position + Vec3::Y * 3.1),
        ));
        commands.spawn((
            Mesh3d(lamp_arm_mesh.clone()),
            MeshMaterial3d(lamp_arm_material.clone()),
            Transform::from_translation(position + Vec3::Y * 6.0 + inward_arm),
        ));
        commands.spawn((
            Mesh3d(lamp_head_mesh.clone()),
            MeshMaterial3d(bulb_material),
            Transform::from_translation(position + Vec3::Y * 5.95 + head_offset),
        ));
    }
}

fn animate_city_life(
    time: Res<Time>,
    mut traffic: Query<(&mut Transform, &TrafficCar)>,
    mut citizens: Query<(&mut Transform, &Citizen)>,
) {
    let dt = time.delta_secs().min(0.05);
    for (mut transform, car) in &mut traffic {
        if car.along_x {
            transform.translation.x += car.speed * car.direction * dt;
            if transform.translation.x > CITY_LIMIT {
                transform.translation.x = -CITY_LIMIT;
            } else if transform.translation.x < -CITY_LIMIT {
                transform.translation.x = CITY_LIMIT;
            }
        } else {
            transform.translation.z += car.speed * car.direction * dt;
            if transform.translation.z > CITY_LIMIT {
                transform.translation.z = -CITY_LIMIT;
            } else if transform.translation.z < -CITY_LIMIT {
                transform.translation.z = CITY_LIMIT;
            }
        }
    }
    let now = time.elapsed_secs();
    for (mut transform, citizen) in &mut citizens {
        let phase = now * citizen.speed + citizen.phase;
        let offset = phase.sin() * citizen.range;
        let direction = if phase.cos() >= 0.0 { 1.0 } else { -1.0 };
        transform.translation = citizen.origin
            + if citizen.along_x {
                Vec3::X * offset
            } else {
                Vec3::Z * offset
            };
        transform.rotation = pedestrian_rotation(citizen.along_x, direction);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_models_face_the_direction_they_drive() {
        for (along_x, direction, expected) in [
            (true, 1.0, Vec3::X),
            (true, -1.0, Vec3::new(-1.0, 0.0, 0.0)),
            (false, 1.0, Vec3::Z),
            (false, -1.0, Vec3::new(0.0, 0.0, -1.0)),
        ] {
            let forward = traffic_rotation(along_x, direction) * Vec3::Z;
            assert!(forward.distance(expected) < 1.0e-5);
        }
    }

    #[test]
    fn pedestrians_face_the_direction_they_walk() {
        let forward = pedestrian_rotation(true, 1.0) * Vec3::Z;
        let backward = pedestrian_rotation(true, -1.0) * Vec3::Z;
        assert!(forward.distance(Vec3::X) < 1.0e-5);
        assert!(backward.distance(Vec3::new(-1.0, 0.0, 0.0)) < 1.0e-5);
    }

    #[test]
    fn pedestrian_paths_fit_between_roads_and_buildings() {
        assert!(SIDEWALK_OFFSET > ROAD_HALF_WIDTH);
        assert!(SIDEWALK_OFFSET < BUILDING_EDGE_FROM_ROAD);
        let road = 6.0 * ROAD_SPACING - 144.0;
        let sidewalk = road + SIDEWALK_OFFSET;
        assert!(sidewalk.abs() < CITY_LIMIT);
        assert!((-108.0_f32 - 7.0).abs() < CITY_LIMIT);
    }

    #[test]
    fn streetlights_cover_selected_intersections_without_leaving_the_city() {
        let positions = streetlight_positions();
        assert_eq!(positions.len(), 36);
        assert!(positions.iter().all(|position| {
            position.x.abs() < CITY_LIMIT && position.z.abs() < CITY_LIMIT
        }));
    }
}
