use bevy::prelude::*;

const CITY_LIMIT: f32 = 160.0;
const ROAD_SPACING: f32 = 48.0;
const ROAD_HALF_WIDTH: f32 = 5.5;
const SIDEWALK_OFFSET: f32 = 7.0;
const BUILDING_HALF: f32 = 14.0;
const BUILDING_EDGE_FROM_ROAD: f32 = ROAD_SPACING * 0.5 - BUILDING_HALF;

const CAR_BODY_RADIUS: f32 = 2.2;
const CAR_BRAKE_DISTANCE: f32 = 7.5;
const CAR_LANE_GAP: f32 = 3.4;
const CAR_HIT_DISTANCE: f32 = 3.2;
const CAR_STOP_SECONDS: f32 = 1.1;
const CITIZEN_BODY_RADIUS: f32 = 0.42;
const BODY_PUSH: f32 = 1.2;
const KNOCK_SPEED: f32 = 7.5;
const KNOCK_LIFT: f32 = 2.8;
const KNOCK_GRAVITY: f32 = 20.0;
const KNOCK_SECONDS: f32 = 1.6;
const MAX_KNOCK_OFFSET: f32 = 10.0;
const PUNCH_RANGE: f32 = 2.8;
const PUNCH_CONE: f32 = 0.35;
const PUNCH_KNOCK: f32 = 9.5;
const FALL_TILT: f32 = 0.55;

#[derive(Component)]
struct TrafficCar {
    speed: f32,
    direction: f32,
    along_x: bool,
    stop_timer: f32,
}

#[derive(Component)]
struct Citizen {
    origin: Vec3,
    phase: f32,
    speed: f32,
    range: f32,
    along_x: bool,
    displacement: Vec3,
    knock_velocity: Vec3,
    knocked: f32,
}

/// Where the player is and what they are doing, read once per frame by the city simulation.
#[derive(Resource)]
pub struct PlayerProbe {
    pub position: Vec3,
    pub forward: Vec3,
    pub in_vehicle: bool,
    pub punch: bool,
}

impl Default for PlayerProbe {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            in_vehicle: false,
            punch: false,
        }
    }
}

/// Flat positions and radii of the moving traffic cars, so the player bumps into them.
#[derive(Resource, Default)]
pub struct TrafficBodies {
    pub cars: Vec<(Vec2, f32)>,
}

pub struct CityLifePlugin;

impl Plugin for CityLifePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerProbe>()
            .init_resource::<TrafficBodies>()
            .add_systems(Startup, spawn_city_life)
            .add_systems(Update, simulate_city_life);
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
                    positions.push(Vec3::new(road_x + side_x * 7.4, 0.0, road_z + side_z * 7.4));
                }
            }
        }
    }
    positions
}

fn intersection_light_positions() -> Vec<Vec3> {
    let mut positions = Vec::with_capacity(9);
    for road_x in [-96.0, 0.0, 96.0] {
        for road_z in [-96.0, 0.0, 96.0] {
            positions.push(Vec3::new(road_x, 5.5, road_z));
        }
    }
    positions
}

fn facade_window_transforms() -> Vec<(Vec3, Quat)> {
    let mut windows = Vec::with_capacity(1_008);
    for grid_x in -3_i32..=2 {
        for grid_z in -3_i32..=2 {
            let center_x = grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let center_z = grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let height = 8.0 + (((grid_x + 3) * 7 + (grid_z + 3) * 11) % 6) as f32 * 4.0;
            let mut y = 3.4;
            while y < height - 1.0 {
                for column in [-1.0_f32, 1.0] {
                    windows.push((
                        Vec3::new(center_x + column * 6.0, y, center_z - BUILDING_HALF - 0.08),
                        Quat::IDENTITY,
                    ));
                    windows.push((
                        Vec3::new(center_x + column * 6.0, y, center_z + BUILDING_HALF + 0.08),
                        Quat::IDENTITY,
                    ));
                    windows.push((
                        Vec3::new(center_x - BUILDING_HALF - 0.08, y, center_z + column * 6.0),
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ));
                    windows.push((
                        Vec3::new(center_x + BUILDING_HALF + 0.08, y, center_z + column * 6.0),
                        Quat::from_rotation_y(std::f32::consts::FRAC_PI_2),
                    ));
                }
                y += 4.6;
            }
        }
    }
    windows
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
                stop_timer: 0.0,
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
                stop_timer: 0.0,
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
                displacement: Vec3::ZERO,
                knock_velocity: Vec3::ZERO,
                knocked: 0.0,
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

    // Nine unshadowed, short-range lights keep the neon intersections readable
    // without adding a shadow map for every decorative lamp.
    for (index, position) in intersection_light_positions().into_iter().enumerate() {
        let color = if index % 2 == 0 {
            Color::srgb(0.28, 0.72, 0.92)
        } else {
            Color::srgb(0.88, 0.30, 0.70)
        };
        commands.spawn((
            PointLight {
                color,
                intensity: 1_000.0,
                range: 26.0,
                shadow_maps_enabled: false,
                ..default()
            },
            Transform::from_translation(position),
        ));
    }

    // Shared meshes and three emissive materials keep the procedural facade detail light.
    let window_mesh = meshes.add(Cuboid::new(1.3, 1.2, 0.12));
    let window_materials = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.82, 0.59, 0.25),
            emissive: LinearRgba::rgb(1.5, 0.78, 0.22),
            metallic: 0.12,
            perceptual_roughness: 0.48,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.08, 0.48, 0.62),
            emissive: LinearRgba::rgb(0.08, 1.25, 1.75),
            metallic: 0.18,
            perceptual_roughness: 0.38,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.58, 0.22, 0.52),
            emissive: LinearRgba::rgb(1.1, 0.16, 0.9),
            metallic: 0.15,
            perceptual_roughness: 0.4,
            ..default()
        }),
    ];
    for (index, (position, rotation)) in facade_window_transforms().into_iter().enumerate() {
        commands.spawn((
            Mesh3d(window_mesh.clone()),
            MeshMaterial3d(window_materials[index % window_materials.len()].clone()),
            Transform::from_translation(position).with_rotation(rotation),
        ));
    }
}

fn resolve_knock(displacement: &mut Vec3, velocity: &mut Vec3, knocked: &mut f32, dt: f32) {
    *displacement += *velocity * dt;
    velocity.y -= KNOCK_GRAVITY * dt;
    let damping = (1.0 - 4.0 * dt).max(0.0);
    velocity.x *= damping;
    velocity.z *= damping;
    if displacement.y < 0.0 {
        displacement.y = 0.0;
        velocity.y = 0.0;
    }
    if displacement.length() > MAX_KNOCK_OFFSET {
        *displacement = displacement.normalize() * MAX_KNOCK_OFFSET;
    }
    *knocked = (*knocked - dt).max(0.0);
}

fn simulate_city_life(
    time: Res<Time>,
    mut probe: ResMut<PlayerProbe>,
    mut bodies: ResMut<TrafficBodies>,
    mut traffic: Query<(&mut Transform, &mut TrafficCar), Without<Citizen>>,
    mut citizens: Query<(&mut Transform, &mut Citizen), Without<TrafficCar>>,
) {
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    let now = time.elapsed_secs();
    let player = probe.position;
    let punching = probe.punch;
    let player_forward = if probe.forward.length_squared() > 1.0e-6 {
        probe.forward.normalize()
    } else {
        Vec3::NEG_Z
    };

    let mut car_positions: Vec<Vec3> = Vec::with_capacity(16);
    for (transform, _) in traffic.iter() {
        car_positions.push(transform.translation);
    }
    let mut citizen_positions: Vec<Vec3> = Vec::with_capacity(16);
    for (transform, _) in citizens.iter() {
        citizen_positions.push(transform.translation);
    }

    let mut knock_events: Vec<(usize, Vec3)> = Vec::with_capacity(4);
    let mut car_bodies: Vec<(Vec2, f32)> = Vec::with_capacity(car_positions.len());

    for (index, (mut transform, mut car)) in traffic.iter_mut().enumerate() {
        if car.stop_timer > 0.0 {
            car.stop_timer = (car.stop_timer - dt).max(0.0);
        }
        let forward = if car.along_x { Vec3::X } else { Vec3::Z } * car.direction;

        let mut blocked = false;
        for (other, position) in car_positions.iter().enumerate() {
            if other == index {
                continue;
            }
            let to_other = *position - transform.translation;
            let ahead = to_other.dot(forward);
            let sideways = Vec2::new(to_other.x, to_other.z).length();
            if ahead > 0.0 && ahead < CAR_BRAKE_DISTANCE && sideways < CAR_LANE_GAP {
                blocked = true;
            }
        }
        if blocked {
            car.stop_timer = CAR_STOP_SECONDS;
        }

        if car.stop_timer <= 0.0 {
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

        for (citizen_index, position) in citizen_positions.iter().enumerate() {
            let flat = Vec2::new(
                transform.translation.x - position.x,
                transform.translation.z - position.z,
            );
            if flat.length() < CAR_HIT_DISTANCE {
                let push = if flat.length() > 1.0e-4 {
                    flat.normalize()
                } else {
                    Vec2::Y
                };
                let velocity = Vec3::new(push.x, 0.0, push.y) * (KNOCK_SPEED + car.speed * 0.35)
                    + Vec3::Y * KNOCK_LIFT;
                knock_events.push((citizen_index, velocity));
                car.stop_timer = CAR_STOP_SECONDS;
            }
        }

        car_bodies.push((
            Vec2::new(transform.translation.x, transform.translation.z),
            CAR_BODY_RADIUS,
        ));
    }

    for (index, (mut transform, mut citizen)) in citizens.iter_mut().enumerate() {
        let phase = now * citizen.speed + citizen.phase;
        let offset = phase.sin() * citizen.range;
        let direction = if phase.cos() >= 0.0 { 1.0 } else { -1.0 };
        let along = if citizen.along_x { Vec3::X } else { Vec3::Z };
        let base = citizen.origin + along * offset;

        if citizen.knocked > 0.0 {
            let mut displacement = citizen.displacement;
            let mut velocity = citizen.knock_velocity;
            let mut knocked = citizen.knocked;
            resolve_knock(&mut displacement, &mut velocity, &mut knocked, dt);
            citizen.displacement = displacement;
            citizen.knock_velocity = velocity;
            citizen.knocked = knocked;
        } else {
            citizen.displacement = citizen.displacement.lerp(Vec3::ZERO, (dt * 3.0).min(1.0));
            citizen.knock_velocity = Vec3::ZERO;
        }

        let mut knock_velocity = Vec3::ZERO;
        let mut hit = false;

        let current = base + citizen.displacement;

        for (other, position) in citizen_positions.iter().enumerate() {
            if other == index {
                continue;
            }
            let flat = Vec2::new(current.x - position.x, current.z - position.z);
            let distance = flat.length();
            if distance < CITIZEN_BODY_RADIUS * 2.0 && distance > 1.0e-4 {
                let push = flat.normalize();
                let overlap = CITIZEN_BODY_RADIUS * 2.0 - distance;
                citizen.displacement += Vec3::new(push.x, 0.0, push.y) * overlap * 0.5;
            }
        }

        let to_player = Vec2::new(current.x - player.x, current.z - player.z);
        if !probe.in_vehicle && to_player.length() < BODY_PUSH && to_player.length() > 1.0e-4 {
            let push = to_player.normalize();
            let overlap = BODY_PUSH - to_player.length();
            citizen.displacement += Vec3::new(push.x, 0.0, push.y) * overlap;
        }

        if probe.in_vehicle && to_player.length() < CAR_HIT_DISTANCE {
            let push = if to_player.length() > 1.0e-4 {
                to_player.normalize()
            } else {
                Vec2::Y
            };
            knock_velocity = Vec3::new(push.x, 0.0, push.y) * KNOCK_SPEED + Vec3::Y * KNOCK_LIFT;
            hit = true;
        }

        if punching {
            let to_citizen = Vec3::new(current.x - player.x, 0.0, current.z - player.z);
            let flat_distance = to_citizen.length();
            if flat_distance < PUNCH_RANGE && flat_distance > 1.0e-4 {
                let aim = to_citizen.normalize().dot(player_forward);
                if aim > PUNCH_CONE || flat_distance < BODY_PUSH {
                    knock_velocity =
                        to_citizen.normalize() * PUNCH_KNOCK + Vec3::Y * KNOCK_LIFT;
                    hit = true;
                }
            }
        }

        for (target, velocity) in &knock_events {
            if *target == index {
                knock_velocity = *velocity;
                hit = true;
            }
        }

        if hit {
            citizen.knocked = KNOCK_SECONDS;
            citizen.knock_velocity = knock_velocity;
        }

        let fallen = if citizen.knocked > 0.0 {
            (citizen.knocked / KNOCK_SECONDS).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let path_rotation = pedestrian_rotation(citizen.along_x, direction);
        transform.translation =
            base + citizen.displacement - Vec3::Y * (FALL_TILT * fallen);
        transform.rotation = path_rotation * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * fallen);
    }

    bodies.cars = car_bodies;
    probe.punch = false;
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
        assert!(
            positions
                .iter()
                .all(|position| { position.x.abs() < CITY_LIMIT && position.z.abs() < CITY_LIMIT })
        );
    }

    #[test]
    fn intersection_lights_are_limited_and_inside_the_city() {
        let positions = intersection_light_positions();
        assert_eq!(positions.len(), 9);
        assert!(positions.iter().all(|position| {
            position.x.abs() < CITY_LIMIT && position.z.abs() < CITY_LIMIT && position.y == 5.5
        }));
    }

    #[test]
    fn facade_windows_cover_every_building_and_stay_inside_the_city() {
        let windows = facade_window_transforms();
        assert_eq!(windows.len(), 1_008);
        assert!(windows.iter().all(|(position, _)| {
            position.x.abs() < CITY_LIMIT
                && position.z.abs() < CITY_LIMIT
                && position.y > 0.0
                && position.y < 28.0
        }));
        assert!(
            windows
                .iter()
                .any(|(_, rotation)| *rotation == Quat::IDENTITY)
        );
        assert!(
            windows
                .iter()
                .any(|(_, rotation)| { (rotation * Vec3::Z).distance(Vec3::X) < 1.0e-5 })
        );
    }

    #[test]
    fn knock_moves_the_body_and_settles_on_the_ground() {
        let mut displacement = Vec3::ZERO;
        let mut velocity = Vec3::new(6.0, KNOCK_LIFT, 0.0);
        let mut knocked = KNOCK_SECONDS;
        let mut steps = 0;
        while knocked > 0.0 && steps < 500 {
            resolve_knock(&mut displacement, &mut velocity, &mut knocked, 0.02);
            steps += 1;
        }
        assert_eq!(knocked, 0.0);
        assert!(displacement.x > 0.5);
        assert!(displacement.y >= 0.0);
        assert!(displacement.length() <= MAX_KNOCK_OFFSET + 1.0e-3);
    }

    #[test]
    fn punch_reach_stays_shorter_than_a_car_body() {
        assert!(PUNCH_RANGE > BODY_PUSH);
        assert!(PUNCH_RANGE < CAR_HIT_DISTANCE + CAR_BODY_RADIUS);
    }
}
