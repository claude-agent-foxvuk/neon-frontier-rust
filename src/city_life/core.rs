use bevy::prelude::*;

use crate::physics::{bodies_overlap, Body};

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
struct PoliceSirenLight {
    phase: f32,
}

fn police_siren_visible(wanted_level: u8, phase: f32, time_seconds: f32) -> bool {
    wanted_level > 0 && (time_seconds * 12.0 + phase).cos() >= 0.0
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
    pub oriented: Vec<Body>,
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

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CitizenSave {
    pub origin: Vec3,
    pub phase: f32,
    pub speed: f32,
    pub range: f32,
    pub along_x: bool,
    pub displacement: Vec3,
    pub knock_velocity: Vec3,
    pub knocked: f32,
    pub health: f32,
    pub dead: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TrafficSave {
    pub position: Vec3,
    pub speed: f32,
    pub direction: f32,
    pub along_x: bool,
    pub lane_offset: f32,
    pub avoid_offset: f32,
    pub stop_timer: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatrolSave {
    pub position: Vec3,
    pub phase: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OfficerSave {
    pub position: Vec3,
    pub active: bool,
    pub shot_timer: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WorldSnapshot {
    pub health: f32,
    pub wanted_level: u8,
    pub wanted_timer: f32,
    pub traffic: Vec<TrafficSave>,
    pub citizens: Vec<CitizenSave>,
    pub patrols: Vec<PatrolSave>,
    pub officers: Vec<OfficerSave>,
}

#[derive(Resource, Default)]
pub struct WorldState {
    pub snapshot: WorldSnapshot,
}

#[derive(Resource, Default)]
pub struct WorldRestore(pub Option<WorldSnapshot>);

fn restore_world(
    snapshot: &WorldSnapshot,
    probe: &mut PlayerProbe,
    wanted: &mut WantedState,
    traffic: &mut Query<
        (&mut Transform, &mut TrafficCar),
        (
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    citizens: &mut Query<
        (&mut Transform, &mut Citizen),
        (
            Without<TrafficCar>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    patrols: &mut Query<
        (&mut Transform, &mut PolicePatrol),
        (Without<TrafficCar>, Without<Citizen>, Without<FootOfficer>),
    >,
    officers: &mut Query<
        (&mut Transform, &mut FootOfficer, &mut Visibility),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<PoliceSirenLight>,
        ),
    >,
) -> bool {
    if !crate::save::restore_counts_match(
        snapshot,
        traffic.iter().count(),
        citizens.iter().count(),
        patrols.iter().count(),
        officers.iter().count(),
    ) {
        return false;
    }
    probe.health = snapshot.health.clamp(0.0, 100.0);
    wanted.level = snapshot.wanted_level.min(WANTED_LEVEL_MAX);
    wanted.decay_timer = snapshot.wanted_timer.max(0.0);
    for ((mut transform, mut car), saved) in traffic.iter_mut().zip(snapshot.traffic.iter()) {
        transform.translation = saved.position;
        transform.rotation = traffic_rotation(saved.along_x, saved.direction);
        car.speed = saved.speed;
        car.direction = saved.direction;
        car.along_x = saved.along_x;
        car.lane_offset = saved.lane_offset;
        car.avoid_offset = saved.avoid_offset;
        car.stop_timer = saved.stop_timer;
    }
    for ((mut transform, mut citizen), saved) in citizens.iter_mut().zip(snapshot.citizens.iter()) {
        citizen.origin = saved.origin;
        citizen.phase = saved.phase;
        citizen.speed = saved.speed;
        citizen.range = saved.range;
        citizen.along_x = saved.along_x;
        citizen.displacement = saved.displacement;
        citizen.knock_velocity = saved.knock_velocity;
        citizen.knocked = saved.knocked;
        citizen.health = saved.health;
        citizen.dead = saved.dead;
        transform.translation = saved.origin + saved.displacement;
        transform.rotation = pedestrian_rotation(saved.along_x, 1.0);
    }
    for ((mut transform, mut patrol), saved) in patrols.iter_mut().zip(snapshot.patrols.iter()) {
        transform.translation = saved.position;
        patrol.phase = saved.phase;
    }
    for ((mut transform, mut officer, mut visibility), saved) in
        officers.iter_mut().zip(snapshot.officers.iter())
    {
        transform.translation = saved.position;
        officer.active = saved.active;
        officer.shot_timer = saved.shot_timer;
        *visibility = if saved.active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    true
}

pub struct CityLifePlugin;

impl Plugin for CityLifePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerProbe>()
            .init_resource::<TrafficBodies>()
            .init_resource::<WantedState>()
            .init_resource::<WorldState>()
            .init_resource::<WorldRestore>()
            .add_systems(Startup, spawn_city_life)
            .add_systems(Update, simulate_city_life);
    }
}

fn traffic_rotation(along_x: bool, direction: f32) -> Quat {
    let forward = (if along_x { Vec3::X } else { Vec3::Z }) * direction;
    Quat::from_rotation_y(forward.x.atan2(forward.z))
}

fn vehicle_body_from_forward(position: Vec3, forward: Vec3) -> Body {
    let forward = Vec2::new(forward.x, forward.z);
    let forward = if forward.length_squared() > 1.0e-6 {
        forward.normalize()
    } else {
        Vec2::Y
    };
    Body::vehicle(
        Vec2::new(position.x, position.z),
        forward.y.atan2(forward.x),
    )
}

fn traffic_vehicle_body(transform: &Transform, along_x: bool, direction: f32) -> Body {
    let forward = (if along_x { Vec2::X } else { Vec2::Y }) * direction;
    vehicle_body_from_forward(transform.translation, Vec3::new(forward.x, 0.0, forward.y))
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
        let patrol = commands
            .spawn((
                Mesh3d(car_mesh.clone()),
                MeshMaterial3d(police_body.clone()),
                Transform::from_translation(center),
                PolicePatrol {
                    home,
                    phase: index as f32 * 2.7,
                },
            ))
            .id();
        commands.entity(patrol).with_children(|parent| {
            parent.spawn((
                Mesh3d(bar_mesh.clone()),
                MeshMaterial3d(red_light.clone()),
                Transform::from_translation(Vec3::new(-0.28, 0.56, 0.0)),
                Visibility::Hidden,
                PoliceSirenLight { phase: 0.0 },
            ));
            parent.spawn((
                Mesh3d(bar_mesh.clone()),
                MeshMaterial3d(blue_light.clone()),
                Transform::from_translation(Vec3::new(0.28, 0.56, 0.0)),
                Visibility::Hidden,
                PoliceSirenLight {
                    phase: std::f32::consts::PI,
                },
            ));
            parent.spawn((
                Mesh3d(badge_mesh.clone()),
                MeshMaterial3d(blue_light.clone()),
                Transform::from_translation(Vec3::new(0.97, 0.05, 0.0)),
            ));
        });
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