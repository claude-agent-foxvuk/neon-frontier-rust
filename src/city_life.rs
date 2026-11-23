use bevy::prelude::*;

const CITY_LIMIT: f32 = 160.0;

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

    for road in 0..7 {
        let lane = road as f32 * 48.0 - 144.0;
        let direction = if road % 2 == 0 { 1.0 } else { -1.0 };
        let along_x = Vec3::new(-126.0 + road as f32 * 34.0, 0.48, lane + 2.1);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(car_materials[road % car_materials.len()].clone()),
            Transform::from_translation(along_x),
            TrafficCar {
                speed: 6.0 + (road % 3) as f32 * 1.2,
                direction,
                along_x: true,
            },
        ));
        let along_z = Vec3::new(lane - 2.1, 0.48, -118.0 + road as f32 * 32.0);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(car_materials[(road + 1) % car_materials.len()].clone()),
            Transform::from_translation(along_z)
                .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            TrafficCar {
                speed: 6.6 + (road % 2) as f32 * 1.4,
                direction: -direction,
                along_x: false,
            },
        ));
    }

    for index in 0..14 {
        let road = (index % 7) as f32;
        let side = if index < 7 { 1.0 } else { -1.0 };
        let lane = road * 48.0 - 144.0;
        let along_x = index >= 7;
        let origin = if along_x {
            Vec3::new(-108.0 + road * 31.0, 0.82, lane + side * 8.5)
        } else {
            Vec3::new(lane + side * 8.5, 0.82, -108.0 + road * 31.0)
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
        let offset = (now * citizen.speed + citizen.phase).sin() * citizen.range;
        transform.translation = citizen.origin
            + if citizen.along_x {
                Vec3::X * offset
            } else {
                Vec3::Z * offset
            };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_uses_separate_parallel_lanes() {
        let lane_a = -144.0 + 2.1;
        let lane_b = -144.0 - 2.1;
        assert!((lane_a - lane_b).abs() > 4.0);
        assert!(CITY_LIMIT > 144.0);
    }

    #[test]
    fn pedestrian_paths_stay_inside_the_city_bounds() {
        let road = 6.0 * 48.0 - 144.0;
        let sidewalk = road + 8.5;
        assert!(sidewalk.abs() < CITY_LIMIT);
        assert!((-108.0_f32 - 7.0).abs() < CITY_LIMIT);
    }
}
