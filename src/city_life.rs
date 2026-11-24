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
const CAR_LANE_OFFSET: f32 = 2.1;
const CAR_PASSING_OFFSET: f32 = CAR_BODY_RADIUS * 2.0 + 0.2;
const CAR_HIT_DISTANCE: f32 = 3.2;
const CAR_STOP_SECONDS: f32 = 1.1;
const OFFICER_BODY_RADIUS: f32 = 0.34;
const OFFICER_SPEED: f32 = 5.2;
const OFFICER_STOP_DISTANCE: f32 = 12.0;
const OFFICER_RETREAT_DISTANCE: f32 = 9.0;
const POLICE_SHOT_MIN_DISTANCE: f32 = 8.0;
const POLICE_SHOT_MAX_DISTANCE: f32 = 18.0;
const POLICE_SHOT_COOLDOWN: f32 = 1.25;
const POLICE_SHOT_DAMAGE: f32 = 8.0;
const POLICE_MUZZLE_FLASH_SECONDS: f32 = 0.14;
const LINE_OF_SIGHT_SAMPLE_SPACING: f32 = 1.0;
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
const CITIZEN_HEALTH: f32 = 3.0;
const WANTED_DECAY_START: f32 = 18.0;
const WANTED_DECAY_STEP: f32 = 8.0;
pub const WANTED_LEVEL_MAX: u8 = 6;

#[derive(Component)]
struct TrafficCar {
    speed: f32,
    direction: f32,
    along_x: bool,
    stop_timer: f32,
    lane_offset: f32,
    avoid_offset: f32,
}

#[derive(Component)]
struct PolicePatrol {
    home: Vec2,
    phase: f32,
}

#[derive(Component)]
struct FootOfficer {
    patrol_index: usize,
    active: bool,
    shot_timer: f32,
}

#[derive(Component)]
struct OfficerMuzzleFlash {
    patrol_index: usize,
    remaining: f32,
}

#[derive(Clone, Copy)]
struct PoliceTactics {
    speed: f32,
    stop_distance: f32,
    flank_distance: f32,
    search_radius: f32,
}

fn police_tactics(level: u8, patrol_index: usize, now: f32) -> PoliceTactics {
    let search_scale = 0.75 + 0.25 * (now * 0.8 + patrol_index as f32 * 1.7).sin().abs();
    let (speed, stop_distance, flank_distance, search_radius) = match level {
        1 => (5.5, 4.0, 0.0, 0.0),
        2 => (7.0, 4.5, 1.5, 1.5),
        3 => (8.5, 5.0, 5.0, 2.5),
        4 => (10.0, 5.5, 8.0, 3.5),
        5 => (11.5, 6.0, 12.0, 5.0),
        _ => (13.0, 6.5, 16.0, 7.0),
    };
    PoliceTactics {
        speed,
        stop_distance,
        flank_distance,
        search_radius: search_radius * search_scale,
    }
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
    health: f32,
    dead: bool,
}

#[derive(Resource)]
pub struct PlayerProbe {
    pub position: Vec3,
    pub vehicle_position: Vec3,
    pub forward: Vec3,
    pub in_vehicle: bool,
    pub punch: bool,
    pub health: f32,
}

impl Default for PlayerProbe {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            vehicle_position: Vec3::new(0.0, 0.65, 6.0),
            forward: Vec3::NEG_Z,
            in_vehicle: false,
            punch: false,
            health: 100.0,
        }
    }
}

#[derive(Resource, Default)]
pub struct TrafficBodies {
    pub cars: Vec<(Vec2, f32)>,
}

#[derive(Resource, Default)]
pub struct WantedState {
    pub level: u8,
    decay_timer: f32,
}

impl WantedState {
    fn report_incident(&mut self) {
        self.level = (self.level + 1).min(WANTED_LEVEL_MAX);
        self.decay_timer = WANTED_DECAY_START;
    }

    fn tick(&mut self, dt: f32, incident: bool) {
        if incident {
            self.report_incident();
        } else if self.level > 0 {
            self.decay_timer -= dt;
            if self.decay_timer <= 0.0 {
                self.level -= 1;
                self.decay_timer = WANTED_DECAY_STEP;
            }
        }
    }
}

pub struct CityLifePlugin;

impl Plugin for CityLifePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerProbe>()
            .init_resource::<TrafficBodies>()
            .init_resource::<WantedState>()
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
        let along_x = Vec3::new(-126.0 + road as f32 * 34.0, 0.48, lane + CAR_LANE_OFFSET);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(car_materials[road % car_materials.len()].clone()),
            Transform::from_translation(along_x).with_rotation(traffic_rotation(true, direction)),
            TrafficCar {
                speed: 6.0 + (road % 3) as f32 * 1.2,
                direction,
                along_x: true,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ));
        let direction_z = -direction;
        let along_z = Vec3::new(lane - CAR_LANE_OFFSET, 0.48, -118.0 + road as f32 * 32.0);
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
                lane_offset: 0.0,
                avoid_offset: 0.0,
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
                health: CITIZEN_HEALTH,
                dead: false,
            },
        ));
    }

    let police_body = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.065, 0.11),
        metallic: 0.55,
        perceptual_roughness: 0.3,
        ..default()
    });
    let red_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.08, 0.08),
        emissive: LinearRgba::rgb(3.2, 0.05, 0.03),
        ..default()
    });
    let blue_light = materials.add(StandardMaterial {
        base_color: Color::srgb(0.04, 0.25, 0.95),
        emissive: LinearRgba::rgb(0.04, 0.35, 3.5),
        ..default()
    });
    let officer_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.12, 0.28),
        metallic: 0.15,
        perceptual_roughness: 0.62,
        ..default()
    });
    let officer_skin = materials.add(Color::srgb(0.78, 0.53, 0.36));
    let officer_gun_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.045, 0.055),
        metallic: 0.62,
        perceptual_roughness: 0.28,
        ..default()
    });
    let muzzle_flash_material = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.78, 0.18),
        emissive: LinearRgba::rgb(9.0, 4.2, 0.35),
        ..default()
    });
    let officer_mesh = meshes.add(Cuboid::new(0.56, 1.5, 0.44));
    let officer_head_mesh = meshes.add(Cuboid::new(0.36, 0.36, 0.36));
    let officer_gun_mesh = meshes.add(Cuboid::new(0.12, 0.12, 0.48));
    let muzzle_flash_mesh = meshes.add(Cuboid::new(0.34, 0.34, 0.34));
    let bar_mesh = meshes.add(Cuboid::new(0.45, 0.14, 0.32));
    let badge_mesh = meshes.add(Cuboid::new(0.08, 0.28, 0.68));
    for (index, home) in [Vec2::new(-48.0, -2.0), Vec2::new(48.0, 2.0)]
        .into_iter()
        .enumerate()
    {
        let center = Vec3::new(home.x, 0.48, home.y);
        commands.spawn((
            Mesh3d(car_mesh.clone()),
            MeshMaterial3d(police_body.clone()),
            Transform::from_translation(center),
            PolicePatrol {
                home,
                phase: index as f32 * 2.7,
            },
        ));
        commands.spawn((
            Mesh3d(bar_mesh.clone()),
            MeshMaterial3d(red_light.clone()),
            Transform::from_translation(center + Vec3::new(-0.28, 0.56, 0.0)),
        ));
        commands.spawn((
            Mesh3d(bar_mesh.clone()),
            MeshMaterial3d(blue_light.clone()),
            Transform::from_translation(center + Vec3::new(0.28, 0.56, 0.0)),
        ));
        commands.spawn((
            Mesh3d(badge_mesh.clone()),
            MeshMaterial3d(blue_light.clone()),
            Transform::from_translation(center + Vec3::new(0.97, 0.05, 0.0)),
        ));
        let officer = commands
            .spawn((
                Mesh3d(officer_mesh.clone()),
                MeshMaterial3d(officer_material.clone()),
                Transform::from_translation(Vec3::new(home.x + 2.8, 0.82, home.y)),
                Visibility::Hidden,
                FootOfficer {
                    patrol_index: index,
                    active: false,
                    shot_timer: 0.6 + index as f32 * 0.2,
                },
            ))
            .id();
        commands.entity(officer).with_children(|parent| {
            parent.spawn((
                Mesh3d(officer_head_mesh.clone()),
                MeshMaterial3d(officer_skin.clone()),
                Transform::from_translation(Vec3::new(0.0, 0.94, 0.0)),
            ));
            parent.spawn((
                Mesh3d(officer_gun_mesh.clone()),
                MeshMaterial3d(officer_gun_material.clone()),
                Transform::from_translation(Vec3::new(0.2, 0.42, 0.35)),
            ));
            parent.spawn((
                Mesh3d(muzzle_flash_mesh.clone()),
                MeshMaterial3d(muzzle_flash_material.clone()),
                Transform::from_translation(Vec3::new(0.2, 0.42, 0.72)),
                Visibility::Hidden,
                OfficerMuzzleFlash {
                    patrol_index: index,
                    remaining: 0.0,
                },
            ));
        });
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
        let bulb = lamp_glow_materials[index % lamp_glow_materials.len()].clone();
        commands.spawn((
            Mesh3d(lamp_post_mesh.clone()),
            MeshMaterial3d(lamp_post_material.clone()),
            Transform::from_translation(position + Vec3::Y * 3.1),
        ));
        commands.spawn((
            Mesh3d(lamp_arm_mesh.clone()),
            MeshMaterial3d(lamp_post_material.clone()),
            Transform::from_translation(
                position + Vec3::Y * 6.0 + Vec3::new(-side_x * 0.42, 0.0, 0.0),
            ),
        ));
        commands.spawn((
            Mesh3d(lamp_head_mesh.clone()),
            MeshMaterial3d(bulb),
            Transform::from_translation(
                position + Vec3::Y * 5.95 + Vec3::new(-side_x * 0.95, 0.0, 0.0),
            ),
        ));
    }
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

fn damage_citizen(citizen: &mut Citizen, damage: f32) -> bool {
    if citizen.dead || damage <= 0.0 {
        return false;
    }
    citizen.health = (citizen.health - damage).max(0.0);
    if citizen.health == 0.0 {
        citizen.dead = true;
        return true;
    }
    false
}

fn city_collision(position: Vec2, radius: f32) -> bool {
    if position.x.abs() + radius > CITY_LIMIT || position.y.abs() + radius > CITY_LIMIT {
        return true;
    }
    (-3_i32..=2).any(|grid_x| {
        (-3_i32..=2).any(|grid_z| {
            let center = Vec2::new(
                grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
                grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
            );
            let dx = ((position.x - center.x).abs() - BUILDING_HALF).max(0.0);
            let dz = ((position.y - center.y).abs() - BUILDING_HALF).max(0.0);
            dx * dx + dz * dz < radius * radius
        })
    })
}

fn building_contains(position: Vec2) -> bool {
    (-3_i32..=2).any(|grid_x| {
        (-3_i32..=2).any(|grid_z| {
            let center = Vec2::new(
                grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
                grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
            );
            (position.x - center.x).abs() < BUILDING_HALF
                && (position.y - center.y).abs() < BUILDING_HALF
        })
    })
}

fn police_line_of_sight_clear(start: Vec2, end: Vec2) -> bool {
    let distance = start.distance(end);
    let sample_count = (distance / LINE_OF_SIGHT_SAMPLE_SPACING).ceil().max(1.0) as usize;
    (0..=sample_count).all(|sample| {
        let point = start.lerp(end, sample as f32 / sample_count as f32);
        !building_contains(point)
    })
}

fn can_police_shoot(distance: f32, line_of_sight_clear: bool) -> bool {
    line_of_sight_clear && (POLICE_SHOT_MIN_DISTANCE..=POLICE_SHOT_MAX_DISTANCE).contains(&distance)
}

fn damage_player(probe: &mut PlayerProbe, damage: f32) {
    if damage > 0.0 {
        probe.health = (probe.health - damage).max(0.0);
    }
}

fn traffic_obstacle_ahead(
    baseline: Vec2,
    forward: Vec2,
    player: Vec2,
    vehicle: Option<Vec2>,
    in_vehicle: bool,
    car_positions: &[Vec3],
    car_index: usize,
) -> bool {
    let side = Vec2::new(-forward.y, forward.x);
    let blocks_path = |position: Vec2| {
        let relative = position - baseline;
        let ahead = relative.dot(forward);
        ahead > 0.0 && ahead < CAR_BRAKE_DISTANCE && relative.dot(side).abs() < CAR_LANE_GAP
    };
    blocks_path(player)
        || (!in_vehicle && vehicle.map_or(false, |position| blocks_path(position)))
        || car_positions.iter().enumerate().any(|(other, position)| {
            other != car_index && blocks_path(Vec2::new(position.x, position.z))
        })
}

fn choose_safe_traffic_offset(
    current: Vec2,
    baseline: Vec2,
    side: Vec2,
    road_center: f32,
    road_half_width: f32,
    mut blocked: impl FnMut(Vec2) -> bool,
) -> Option<f32> {
    for direction in [1.0, -1.0] {
        let offset = direction * CAR_PASSING_OFFSET;
        let target = baseline + side * offset;
        if (target.dot(side) - road_center).abs() + CAR_BODY_RADIUS > road_half_width {
            continue;
        }
        let candidate = crate::physics::move_with_slide(current, target - current, |point| {
            city_collision(point, CAR_BODY_RADIUS) || blocked(point)
        });
        if candidate.distance(target) < 0.05 {
            return Some(offset);
        }
    }
    None
}

fn simulate_city_life(
    time: Res<Time>,
    mut probe: ResMut<PlayerProbe>,
    mut bodies: ResMut<TrafficBodies>,
    mut wanted: ResMut<WantedState>,
    mut traffic: Query<
        (&mut Transform, &mut TrafficCar),
        (
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    mut citizens: Query<
        (&mut Transform, &mut Citizen),
        (
            Without<TrafficCar>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    mut patrols: Query<
        (&mut Transform, &PolicePatrol),
        (Without<TrafficCar>, Without<Citizen>, Without<FootOfficer>),
    >,
    mut officers: Query<
        (&mut Transform, &mut FootOfficer, &mut Visibility),
        (Without<TrafficCar>, Without<Citizen>, Without<PolicePatrol>),
    >,
    mut muzzle_flashes: Query<
        (&mut Visibility, &mut OfficerMuzzleFlash),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
) {
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    let now = time.elapsed_secs();
    let player = probe.position;
    let player_flat = Vec2::new(player.x, player.z);
    let parked_vehicle = if probe.in_vehicle {
        None
    } else {
        Some(Vec2::new(probe.vehicle_position.x, probe.vehicle_position.z))
    };
    let player_radius = if probe.in_vehicle {
        CAR_BODY_RADIUS
    } else {
        0.5
    };
    let punching = probe.punch;
    let player_forward = if probe.forward.length_squared() > 1.0e-6 {
        probe.forward.normalize()
    } else {
        Vec3::NEG_Z
    };
    let mut player_incident = false;
    let mut car_positions: Vec<Vec3> = traffic
        .iter()
        .map(|(transform, _)| transform.translation)
        .collect();
    let citizen_positions: Vec<Vec3> = citizens
        .iter()
        .map(|(transform, _)| transform.translation)
        .collect();
    let mut knock_events: Vec<(usize, Vec3)> = Vec::with_capacity(4);
    let mut car_bodies: Vec<(Vec2, f32)> = Vec::with_capacity(car_positions.len() + 2);

    for (index, (mut transform, mut car)) in traffic.iter_mut().enumerate() {
        car.stop_timer = (car.stop_timer - dt).max(0.0);
        let forward = (if car.along_x { Vec3::X } else { Vec3::Z }) * car.direction;
        let forward = Vec2::new(forward.x, forward.z);
        let side = Vec2::new(-forward.y, forward.x);
        let current = Vec2::new(transform.translation.x, transform.translation.z);
        let baseline = current - side * car.lane_offset;
        let road_center = (baseline.dot(side) / ROAD_SPACING).round() * ROAD_SPACING;
        let blocked_ahead = traffic_obstacle_ahead(
            baseline,
            forward,
            player_flat,
            parked_vehicle,
            probe.in_vehicle,
            &car_positions,
            index,
        );
        let blocks_movement = |point: Vec2| {
            point.distance(player_flat) < CAR_BODY_RADIUS + player_radius
                || parked_vehicle.map_or(false, |vehicle| {
                    point.distance(vehicle) < CAR_BODY_RADIUS * 2.0
                })
                || car_positions.iter().enumerate().any(|(other, position)| {
                    if other == index {
                        return false;
                    }
                    Vec2::new(position.x, position.z).distance(point) < CAR_BODY_RADIUS * 2.0
                })
        };

        if blocked_ahead {
            match choose_safe_traffic_offset(
                current,
                baseline,
                side,
                road_center,
                ROAD_HALF_WIDTH,
                |point| blocks_movement(point),
            ) {
                Some(offset) => car.avoid_offset = offset,
                None => {
                    car.avoid_offset = car.lane_offset;
                    car.stop_timer = car.stop_timer.max(0.2);
                }
            }
        } else {
            car.avoid_offset = 0.0;
        }

        if car.stop_timer <= 0.0 {
            let lateral_step = (car.avoid_offset - car.lane_offset)
                .clamp(-car.speed.abs() * dt, car.speed.abs() * dt);
            let lateral_target = current + side * lateral_step;
            let shifted = crate::physics::move_with_slide(current, side * lateral_step, |point| {
                city_collision(point, CAR_BODY_RADIUS) || blocks_movement(point)
            });
            let lateral_clear = shifted.distance(lateral_target) < 0.05;
            car.lane_offset += (shifted - current).dot(side);
            let mut candidate = shifted;

            // Do not move forward into the obstacle while still crossing lanes.
            let has_passing_clearance = car.lane_offset.abs() >= CAR_PASSING_OFFSET - 0.1;
            if lateral_clear && (!blocked_ahead || has_passing_clearance) {
                let delta = forward * car.speed * dt;
                candidate = crate::physics::move_with_slide(candidate, delta, |point| {
                    city_collision(point, CAR_BODY_RADIUS) || blocks_movement(point)
                });
                if candidate.distance(shifted) < delta.length() * 0.5 && blocked_ahead {
                    car.stop_timer = 0.2;
                }
            }
            transform.translation.x = candidate.x;
            transform.translation.z = candidate.y;
            car_positions[index] = transform.translation;
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

    let mut citizen_index = 0;
    for (mut transform, mut citizen) in &mut citizens {
        let index = citizen_index;
        citizen_index += 1;
        if citizen.dead {
            continue;
        }
        let phase = now * citizen.speed + citizen.phase;
        let offset = phase.sin() * citizen.range;
        let direction = if phase.cos() >= 0.0 { 1.0 } else { -1.0 };
        let along = if citizen.along_x { Vec3::X } else { Vec3::Z };
        let base = citizen.origin + along * offset;
        let current = base + citizen.displacement;
        let mut impact_velocity = Vec3::ZERO;
        let mut impact_damage = 0.0;
        for (target, velocity) in &knock_events {
            if *target == index {
                impact_velocity = *velocity;
                impact_damage = CITIZEN_HEALTH;
            }
        }
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
        let player_distance = to_player.length();
        if probe.in_vehicle && player_distance < CAR_HIT_DISTANCE {
            let push = if player_distance > 1.0e-4 {
                to_player.normalize()
            } else {
                Vec2::Y
            };
            impact_velocity = Vec3::new(push.x, 0.0, push.y) * KNOCK_SPEED + Vec3::Y * KNOCK_LIFT;
            impact_damage = CITIZEN_HEALTH;
            player_incident = true;
        } else if punching && player_distance < PUNCH_RANGE && player_distance > 1.0e-4 {
            let to_citizen = Vec3::new(current.x - player.x, 0.0, current.z - player.z);
            let aim = to_citizen.normalize().dot(player_forward);
            if aim > PUNCH_CONE || player_distance < BODY_PUSH {
                impact_velocity = to_citizen.normalize() * PUNCH_KNOCK + Vec3::Y * KNOCK_LIFT;
                impact_damage = 1.0;
                player_incident = true;
            }
        }
        if impact_damage > 0.0 {
            damage_citizen(&mut citizen, impact_damage);
            citizen.knocked = KNOCK_SECONDS;
            citizen.knock_velocity = impact_velocity;
        }
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
        let fallen = if citizen.dead {
            1.0
        } else if citizen.knocked > 0.0 {
            (citizen.knocked / KNOCK_SECONDS).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let path_rotation = pedestrian_rotation(citizen.along_x, direction);
        transform.translation = base + citizen.displacement - Vec3::Y * (FALL_TILT * fallen);
        transform.rotation =
            path_rotation * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * fallen);
    }

    wanted.tick(dt, player_incident);
    let patrol_positions: Vec<Vec2> = patrols
        .iter()
        .map(|(transform, _)| Vec2::new(transform.translation.x, transform.translation.z))
        .collect();
    for (index, (mut transform, patrol)) in (&mut patrols).into_iter().enumerate() {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let tactics = if wanted.level > 0 {
            Some(police_tactics(wanted.level, index, now))
        } else {
            None
        };
        let target = if let Some(tactics) = tactics {
            if wanted.level >= 2 {
                player_flat
            } else {
                let to_player = player_flat - position;
                let side = Vec2::new(-to_player.y, to_player.x).normalize_or_zero();
                let flank_side = if index % 2 == 0 { 1.0 } else { -1.0 };
                let search_phase = now * (0.45 + wanted.level as f32 * 0.08) + patrol.phase;
                let search_offset =
                    Vec2::new(search_phase.cos(), search_phase.sin()) * tactics.search_radius;
                player_flat + side * (tactics.flank_distance * flank_side) + search_offset
            }
        } else {
            patrol.home
                + Vec2::new(
                    (now * 0.35 + patrol.phase).sin() * 8.0,
                    (now * 0.28 + patrol.phase).cos() * 5.0,
                )
        };
        let delta = target - position;
        let distance = delta.length();
        let stop_distance = if wanted.level >= 2 {
            9.0
        } else {
            tactics.map_or(5.5, |tactics| tactics.stop_distance)
        };
        if distance > stop_distance {
            let speed = tactics.map_or(2.4, |tactics| tactics.speed);
            let movement = delta.normalize() * (speed * dt).min(distance - stop_distance);
            let mut blockers: Vec<Vec2> = car_positions
                .iter()
                .map(|car| Vec2::new(car.x, car.z))
                .collect();
            blockers.extend(
                patrol_positions
                    .iter()
                    .enumerate()
                    .filter(|(other, _)| *other != index)
                    .map(|(_, position)| *position),
            );
            let next = crate::physics::move_with_slide(position, movement, |point| {
                city_collision(point, CAR_BODY_RADIUS)
                    || point.distance(player_flat) < CAR_BODY_RADIUS + player_radius
                    || blockers
                        .iter()
                        .any(|center| center.distance(point) < CAR_BODY_RADIUS * 2.0)
            });
            let step = next - position;
            transform.translation.x = next.x;
            transform.translation.z = next.y;
            if step.length_squared() > 1.0e-5 {
                transform.rotation = Quat::from_rotation_y(step.x.atan2(step.y));
            }
        }
        car_bodies.push((
            Vec2::new(transform.translation.x, transform.translation.z),
            CAR_BODY_RADIUS + 0.25,
        ));
    }

    let updated_patrol_positions: Vec<Vec2> = patrols
        .iter()
        .map(|(transform, _)| Vec2::new(transform.translation.x, transform.translation.z))
        .collect();
    let mut shots_fired = Vec::new();
    for (mut transform, mut officer, mut visibility) in &mut officers {
        if wanted.level < 2 {
            officer.active = false;
            *visibility = Visibility::Hidden;
            continue;
        }
        if !officer.active {
            let patrol_position = updated_patrol_positions
                .get(officer.patrol_index)
                .copied()
                .unwrap_or(player_flat);
            let side = if officer.patrol_index % 2 == 0 {
                Vec2::X
            } else {
                -Vec2::X
            };
            transform.translation = Vec3::new(
                patrol_position.x + side.x * (CAR_BODY_RADIUS + OFFICER_BODY_RADIUS + 0.35),
                0.82,
                patrol_position.y + side.y * (CAR_BODY_RADIUS + OFFICER_BODY_RADIUS + 0.35),
            );
            officer.active = true;
        }
        *visibility = Visibility::Visible;

        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let to_player = player_flat - position;
        let distance = to_player.length();
        let movement = if distance > OFFICER_STOP_DISTANCE {
            to_player.normalize() * (OFFICER_SPEED * dt).min(distance - OFFICER_STOP_DISTANCE)
        } else if distance < OFFICER_RETREAT_DISTANCE {
            -to_player.normalize() * (OFFICER_SPEED * dt).min(OFFICER_RETREAT_DISTANCE - distance)
        } else {
            Vec2::ZERO
        };
        let mut next = position;
        if movement.length_squared() > 0.0 {
            let mut blockers: Vec<Vec2> = car_positions
                .iter()
                .map(|car| Vec2::new(car.x, car.z))
                .collect();
            blockers.extend(updated_patrol_positions.iter().copied());
            next = crate::physics::move_with_slide(position, movement, |point| {
                city_collision(point, OFFICER_BODY_RADIUS)
                    || point.distance(player_flat) < OFFICER_BODY_RADIUS + player_radius
                    || blockers.iter().any(|center| {
                        center.distance(point) < CAR_BODY_RADIUS + OFFICER_BODY_RADIUS
                    })
            });
        }
        transform.translation.x = next.x;
        transform.translation.z = next.y;
        let facing = player_flat - next;
        if facing.length_squared() > 1.0e-5 {
            transform.rotation = Quat::from_rotation_y(facing.x.atan2(facing.y));
        }

        officer.shot_timer = (officer.shot_timer - dt).max(0.0);
        let officer_distance = next.distance(player_flat);
        let clear_line = police_line_of_sight_clear(next, player_flat);
        if officer.shot_timer <= 0.0 && can_police_shoot(officer_distance, clear_line) {
            damage_player(&mut probe, POLICE_SHOT_DAMAGE);
            officer.shot_timer = POLICE_SHOT_COOLDOWN;
            shots_fired.push(officer.patrol_index);
        }
    }
    for (mut visibility, mut flash) in &mut muzzle_flashes {
        if wanted.level < 2 {
            flash.remaining = 0.0;
        } else if shots_fired.contains(&flash.patrol_index) {
            flash.remaining = POLICE_MUZZLE_FLASH_SECONDS;
        } else {
            flash.remaining = (flash.remaining - dt).max(0.0);
        }
        *visibility = if flash.remaining > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    bodies.cars = car_bodies;
    probe.punch = false;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn citizen() -> Citizen {
        Citizen {
            origin: Vec3::ZERO,
            phase: 0.0,
            speed: 0.0,
            range: 0.0,
            along_x: true,
            displacement: Vec3::ZERO,
            knock_velocity: Vec3::ZERO,
            knocked: 0.0,
            health: CITIZEN_HEALTH,
            dead: false,
        }
    }

    #[test]
    fn traffic_models_face_the_direction_they_drive() {
        for (along_x, direction, expected) in [
            (true, 1.0, Vec3::X),
            (true, -1.0, Vec3::new(-1.0, 0.0, 0.0)),
            (false, 1.0, Vec3::Z),
            (false, -1.0, Vec3::new(0.0, 0.0, -1.0)),
        ] {
            assert!((traffic_rotation(along_x, direction) * Vec3::Z).distance(expected) < 1.0e-5);
        }
    }

    #[test]
    fn pedestrian_paths_fit_between_roads_and_buildings() {
        assert!(SIDEWALK_OFFSET > ROAD_HALF_WIDTH);
        assert!(SIDEWALK_OFFSET < BUILDING_EDGE_FROM_ROAD);
        assert!((6.0 * ROAD_SPACING - 144.0 + SIDEWALK_OFFSET).abs() < CITY_LIMIT);
    }

    #[test]
    fn streets_keep_lights_and_windows_inside_the_city() {
        assert_eq!(streetlight_positions().len(), 36);
        assert_eq!(intersection_light_positions().len(), 9);
        let windows = facade_window_transforms();
        assert_eq!(windows.len(), 1_008);
        assert!(
            windows
                .iter()
                .all(|(p, _)| p.x.abs() < CITY_LIMIT && p.z.abs() < CITY_LIMIT && p.y > 0.0)
        );
    }

    #[test]
    fn traffic_and_police_cannot_drive_through_buildings() {
        assert!(city_collision(Vec2::new(24.0, 24.0), CAR_BODY_RADIUS));
        assert!(!city_collision(Vec2::new(0.0, 0.0), CAR_BODY_RADIUS));
    }

    #[test]
    fn traffic_detects_parked_player_vehicle_after_exit() {
        let baseline = Vec2::ZERO;
        let forward = Vec2::Y;
        let player = Vec2::new(100.0, 0.0);
        let vehicle = Some(Vec2::new(0.0, 3.0));

        assert!(traffic_obstacle_ahead(
            baseline,
            forward,
            player,
            vehicle,
            false,
            &[],
            0,
        ));
        assert!(!traffic_obstacle_ahead(
            baseline,
            forward,
            player,
            vehicle,
            true,
            &[],
            0,
        ));
    }

    #[test]
    fn traffic_chooses_the_clear_adjacent_side() {
        let current = Vec2::new(0.0, CAR_LANE_OFFSET);
        let side = Vec2::Y;
        let offset = choose_safe_traffic_offset(current, current, side, 0.0, 10.0, |point| {
            point.y > current.y + 0.05
        });
        assert_eq!(offset, Some(-CAR_PASSING_OFFSET));
    }

    #[test]
    fn traffic_brakes_when_both_sides_are_blocked() {
        let current = Vec2::ZERO;
        let offset = choose_safe_traffic_offset(current, current, Vec2::Y, 0.0, 10.0, |point| {
            point.y.abs() > 0.05
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn traffic_avoidance_stays_inside_the_road_width() {
        let current = Vec2::new(0.0, CAR_LANE_OFFSET);
        let side = Vec2::Y;
        let offset =
            choose_safe_traffic_offset(current, current, side, 0.0, ROAD_HALF_WIDTH, |_| false)
                .expect("the inner passing lane should fit");
        let target = current + side * offset;
        assert!((target.dot(side)).abs() + CAR_BODY_RADIUS <= ROAD_HALF_WIDTH);
    }

    #[test]
    fn knock_moves_the_body_and_settles_on_the_ground() {
        let mut displacement = Vec3::ZERO;
        let mut velocity = Vec3::new(6.0, KNOCK_LIFT, 0.0);
        let mut knocked = KNOCK_SECONDS;
        for _ in 0..500 {
            if knocked <= 0.0 {
                break;
            }
            resolve_knock(&mut displacement, &mut velocity, &mut knocked, 0.02);
        }
        assert_eq!(knocked, 0.0);
        assert!(displacement.x > 0.5 && displacement.y >= 0.0);
        assert!(displacement.length() <= MAX_KNOCK_OFFSET + 1.0e-3);
    }

    #[test]
    fn repeated_hits_kill_and_dead_npc_stays_dead() {
        let mut person = citizen();
        assert!(!damage_citizen(&mut person, 1.0));
        assert!(!damage_citizen(&mut person, 1.0));
        assert!(damage_citizen(&mut person, 1.0));
        assert!(person.dead);
        assert!(!damage_citizen(&mut person, 1.0));
        assert_eq!(person.health, 0.0);
    }

    #[test]
    fn player_health_starts_full_and_never_drops_below_zero() {
        let mut probe = PlayerProbe::default();
        assert_eq!(probe.health, 100.0);
        damage_player(&mut probe, 120.0);
        assert_eq!(probe.health, 0.0);
    }

    #[test]
    fn building_blocks_police_line_of_sight() {
        assert!(!police_line_of_sight_clear(
            Vec2::new(0.0, 24.0),
            Vec2::new(48.0, 24.0),
        ));
        assert!(police_line_of_sight_clear(
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 20.0),
        ));
    }

    #[test]
    fn police_shooting_requires_clear_sight_and_valid_distance() {
        assert!(!can_police_shoot(7.9, true));
        assert!(can_police_shoot(POLICE_SHOT_MIN_DISTANCE, true));
        assert!(can_police_shoot(POLICE_SHOT_MAX_DISTANCE, true));
        assert!(!can_police_shoot(18.1, true));
        assert!(!can_police_shoot(12.0, false));
    }

    #[test]
    fn police_tactics_distinguish_all_wanted_levels() {
        let tactics: Vec<_> = (1..=WANTED_LEVEL_MAX)
            .map(|level| police_tactics(level, 0, 0.0))
            .collect();
        for index in 0..tactics.len() {
            assert!(
                tactics[..index]
                    .iter()
                    .all(|other| other.speed != tactics[index].speed)
            );
            assert!(
                tactics[..index]
                    .iter()
                    .all(|other| other.stop_distance != tactics[index].stop_distance)
            );
            assert!(
                tactics[..index]
                    .iter()
                    .all(|other| other.flank_distance != tactics[index].flank_distance)
            );
        }
        assert!(tactics[5].speed > tactics[0].speed);
    }

    #[test]
    fn wanted_caps_at_six_and_decays_in_stages() {
        let mut wanted = WantedState::default();
        for _ in 0..8 {
            wanted.report_incident();
        }
        assert_eq!(wanted.level, WANTED_LEVEL_MAX);
        for _ in 0..7 {
            wanted.tick(WANTED_DECAY_START + 0.1, false);
        }
        assert_eq!(wanted.level, 0);
    }
}
