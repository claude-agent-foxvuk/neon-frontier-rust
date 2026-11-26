mod city_life;
mod physics;
mod procedural;
mod save;
mod ui;

use bevy::{
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    input::mouse::MouseMotion,
    post_process::bloom::Bloom,
    prelude::*,
    window::{CursorGrabMode, CursorOptions, Window},
};
use std::io;

const WORLD_EDGE: f32 = 165.0;
const ROAD_SPACING: f32 = 48.0;
const BUILDING_HALF: f32 = 14.0;
const MOUSE_SENSITIVITY: f32 = 0.0035;
const PLAYER_GROUND_Y: f32 = 0.9;
const PLAYER_JUMP_SPEED: f32 = 9.9;
const PLAYER_GRAVITY: f32 = 25.0;
const PLAYER_WALK_SPEED: f32 = 5.4;
const PLAYER_RUN_SPEED: f32 = 9.2;
const PUNCH_COOLDOWN: f32 = 0.45;
const MISSION_MARKER_PULSE: f32 = 0.12;
const PLAYER_BODY_RADIUS: f32 = 0.45;
const WALK_LIMB_SWING: f32 = 0.55;
const RUN_LIMB_SWING: f32 = 0.95;
const DAY_LENGTH_SECONDS: f32 = 240.0;
const STAR_COUNT: usize = 150;
const SPAWN_PLAYER: Vec3 = Vec3::new(2.0, PLAYER_GROUND_Y, 2.0);
const SPAWN_CAR: Vec3 = Vec3::new(0.0, 0.65, 6.0);

#[derive(Component)]
struct PlayerVisual {
    facing_yaw: f32,
}

#[derive(Component)]
struct CarVisual;

#[derive(Component)]
struct FollowCamera;

#[derive(Component)]
struct MissionMarker;

#[derive(Component)]
struct SunLight;

#[derive(Component)]
struct Star;

#[derive(Component)]
struct Limb {
    phase_offset: f32,
}

#[derive(Resource)]
struct Game {
    player: Vec3,
    car: Vec3,
    yaw: f32,
    speed: f32,
    steering: f32,
    lateral_speed: f32,
    player_facing_yaw: f32,
    player_moving: bool,
    player_running: bool,
    player_vertical_speed: f32,
    player_grounded: bool,
    punch_cooldown: f32,
    started: bool,
    driving: bool,
    paused: bool,
    bloom: bool,
    menu_index: usize,
    credits_screen: bool,
    stage: usize,
    credits: u32,
    world_time: f32,
    buildings: Vec<Vec2>,
    notice: String,
    notice_timer: f32,
    camera_yaw: f32,
    camera_orbit: f32,
    camera_pitch: f32,
    mouse_captured: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            player: SPAWN_PLAYER,
            car: SPAWN_CAR,
            yaw: 0.0,
            speed: 0.0,
            steering: 0.0,
            lateral_speed: 0.0,
            player_facing_yaw: 0.0,
            player_moving: false,
            player_running: false,
            player_vertical_speed: 0.0,
            player_grounded: true,
            punch_cooldown: 0.0,
            started: false,
            driving: false,
            paused: false,
            bloom: true,
            menu_index: 0,
            credits_screen: false,
            stage: 0,
            credits: 0,
            world_time: 0.32,
            buildings: Vec::new(),
            notice: String::new(),
            notice_timer: 0.0,
            camera_yaw: 0.0,
            camera_orbit: 0.0,
            camera_pitch: 0.38,
            mouse_captured: false,
        }
    }
}

fn reset_run(game: &mut Game) {
    let buildings = std::mem::take(&mut game.buildings);
    *game = Game::default();
    game.buildings = buildings;
    game.started = true;
}

const OBJECTIVES: [Vec3; 3] = [
    Vec3::new(48.0, 1.3, 0.0),
    Vec3::new(48.0, 1.3, -96.0),
    Vec3::new(-96.0, 1.3, -96.0),
];

const ALTERNATE_OBJECTIVES: [Vec3; 3] = [
    Vec3::new(-48.0, 1.3, 0.0),
    Vec3::new(-48.0, 1.3, 96.0),
    Vec3::new(96.0, 1.3, 96.0),
];

fn mission_target(stage: usize, credits: u32) -> Vec3 {
    let objectives = if (credits / 300) % 2 == 0 {
        &OBJECTIVES
    } else {
        &ALTERNATE_OBJECTIVES
    };
    objectives[stage]
}

fn mission_name(credits: u32) -> &'static str {
    if (credits / 300) % 2 == 0 {
        "Три маяка"
    } else {
        "Ночной маршрут"
    }
}

fn day_phase(world_time: f32) -> f32 {
    world_time.rem_euclid(1.0)
}

fn sun_height(world_time: f32) -> f32 {
    -(day_phase(world_time) * std::f32::consts::TAU).cos()
}

fn night_factor(world_time: f32) -> f32 {
    ((0.05 - sun_height(world_time)) / 0.45).clamp(0.0, 1.0)
}

fn daylight_factor(world_time: f32) -> f32 {
    (1.0 - night_factor(world_time)).clamp(0.0, 1.0)
}

fn advance_clock(world_time: f32, delta_seconds: f32) -> f32 {
    (world_time + delta_seconds / DAY_LENGTH_SECONDS).rem_euclid(1.0)
}

fn clock_label(world_time: f32) -> String {
    let hours = day_phase(world_time) * 24.0;
    let whole = hours.floor() as u32;
    let minutes = ((hours - whole as f32) * 60.0).floor() as u32;
    format!("{:02}:{:02}", whole.min(23), minutes.min(59))
}

fn main() {
    App::new()
        .insert_resource(Game::default())
        .add_plugins(DefaultPlugins)
        .add_plugins(city_life::CityLifePlugin)
        .add_systems(Startup, setup)
        .add_systems(Update, (controls, update_world, update_hud).chain())
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut game: ResMut<Game>,
) {
    let cube = meshes.add(Cuboid::default());
    let wheel_mesh = meshes.add(Cylinder::new(0.34, 0.24));
    let surfaces = procedural::add_surface_textures(&mut images);
    let ground = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.86, 0.82),
        base_color_texture: Some(surfaces.ground.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(240.0)),
        perceptual_roughness: 0.98,
        ..default()
    });
    let asphalt = materials.add(StandardMaterial {
        base_color: Color::srgb(0.82, 0.88, 0.96),
        base_color_texture: Some(surfaces.asphalt.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(15.0, 300.0)),
        perceptual_roughness: 0.96,
        ..default()
    });
    let stripe = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.65, 0.18),
        emissive: LinearRgba::rgb(0.18, 0.09, 0.02),
        perceptual_roughness: 0.72,
        ..default()
    });
    let sidewalk = materials.add(StandardMaterial {
        base_color: Color::srgb(0.88, 0.92, 0.94),
        base_color_texture: Some(surfaces.pavement.clone()),
        uv_transform: bevy::math::Affine2::from_scale(Vec2::new(32.0, 5.0)),
        metallic: 0.08,
        perceptual_roughness: 0.88,
        ..default()
    });
    let curb = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.54, 0.58),
        perceptual_roughness: 0.78,
        ..default()
    });
    let crosswalk = materials.add(StandardMaterial {
        base_color: Color::srgb(0.78, 0.84, 0.85),
        emissive: LinearRgba::rgb(0.04, 0.06, 0.07),
        perceptual_roughness: 0.82,
        ..default()
    });
    let building_colors = [
        Color::srgb(0.86, 0.90, 0.94),
        Color::srgb(0.94, 0.78, 0.82),
        Color::srgb(0.76, 0.92, 0.94),
        Color::srgb(0.88, 0.82, 0.96),
    ];
    let facade_scales = [
        Vec2::new(3.0, 4.6),
        Vec2::new(4.8, 5.6),
        Vec2::new(2.6, 6.4),
        Vec2::new(5.2, 3.4),
    ];
    let buildings: Vec<_> = building_colors
        .into_iter()
        .enumerate()
        .map(|(index, color)| {
            materials.add(StandardMaterial {
                base_color: color,
                base_color_texture: Some(surfaces.facade.clone()),
                uv_transform: bevy::math::Affine2::from_scale(facade_scales[index]),
                metallic: 0.08,
                perceptual_roughness: 0.82,
                ..default()
            })
        })
        .collect();
    let building_trim_mesh = meshes.add(Cuboid::new(0.28, 1.0, 0.28));
    let building_trim_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.12, 0.30, 0.38),
        metallic: 0.72,
        perceptual_roughness: 0.28,
        ..default()
    });
    let body = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.48, 0.67),
        metallic: 0.65,
        perceptual_roughness: 0.28,
        ..default()
    });
    let glass = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.12, 0.18),
        metallic: 0.85,
        perceptual_roughness: 0.08,
        ..default()
    });
    let rubber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.07, 0.08),
        perceptual_roughness: 0.95,
        ..default()
    });
    let headlight = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.82),
        emissive: LinearRgba::rgb(6.0, 5.2, 3.6),
        ..default()
    });
    let player_mat = materials.add(Color::srgb(0.94, 0.61, 0.22));
    let skin_mat = materials.add(Color::srgb(0.96, 0.78, 0.62));
    let marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.7, 0.95),
        emissive: LinearRgba::rgb(0.0, 3.0, 5.0),
        ..default()
    });
    let star_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        emissive: LinearRgba::rgb(7.0, 7.4, 8.6),
        ..default()
    });

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(ground),
        Transform::from_xyz(0.0, -0.25, 0.0).with_scale(Vec3::new(340.0, 0.5, 340.0)),
    ));
    for n in -3..=3 {
        let c = n as f32 * ROAD_SPACING;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(asphalt.clone()),
            Transform::from_xyz(c, 0.01, 0.0).with_scale(Vec3::new(11.0, 0.04, 340.0)),
        ));
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(asphalt.clone()),
            Transform::from_xyz(0.0, 0.02, c).with_scale(Vec3::new(340.0, 0.04, 11.0)),
        ));
        for dash in -16..=16 {
            let d = dash as f32 * 10.0;
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(stripe.clone()),
                Transform::from_xyz(c, 0.05, d).with_scale(Vec3::new(0.18, 0.03, 4.0)),
            ));
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(stripe.clone()),
                Transform::from_xyz(d, 0.06, c).with_scale(Vec3::new(4.0, 0.03, 0.18)),
            ));
        }
    }

    let sidewalk_mesh = meshes.add(Cuboid::new(340.0, 0.12, 1.8));
    let side_curb_mesh = meshes.add(Cuboid::new(340.0, 0.18, 0.18));
    let crosswalk_x_mesh = meshes.add(Cuboid::new(0.42, 0.035, 6.2));
    let crosswalk_z_mesh = meshes.add(Cuboid::new(6.2, 0.035, 0.42));
    for road in -3..=3 {
        let c = road as f32 * ROAD_SPACING;
        for side in [-1.0, 1.0] {
            commands.spawn((
                Mesh3d(sidewalk_mesh.clone()),
                MeshMaterial3d(sidewalk.clone()),
                Transform::from_xyz(c + side * 7.2, 0.045, 0.0),
            ));
            commands.spawn((
                Mesh3d(sidewalk_mesh.clone()),
                MeshMaterial3d(sidewalk.clone()),
                Transform::from_xyz(0.0, 0.045, c + side * 7.2)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            ));
            commands.spawn((
                Mesh3d(side_curb_mesh.clone()),
                MeshMaterial3d(curb.clone()),
                Transform::from_xyz(c + side * 5.9, 0.08, 0.0),
            ));
            commands.spawn((
                Mesh3d(side_curb_mesh.clone()),
                MeshMaterial3d(curb.clone()),
                Transform::from_xyz(0.0, 0.08, c + side * 5.9)
                    .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            ));
        }
    }
    for intersection_x in [-96.0, 0.0, 96.0] {
        for intersection_z in [-96.0, 0.0, 96.0] {
            for offset in [-3.3, -1.1, 1.1, 3.3] {
                for side in [-1.0, 1.0] {
                    commands.spawn((
                        Mesh3d(crosswalk_x_mesh.clone()),
                        MeshMaterial3d(crosswalk.clone()),
                        Transform::from_xyz(
                            intersection_x + offset,
                            0.075,
                            intersection_z + side * 8.0,
                        ),
                    ));
                    commands.spawn((
                        Mesh3d(crosswalk_z_mesh.clone()),
                        MeshMaterial3d(crosswalk.clone()),
                        Transform::from_xyz(
                            intersection_x + side * 8.0,
                            0.075,
                            intersection_z + offset,
                        ),
                    ));
                }
            }
        }
    }

    for x in -3_i32..=2 {
        for z in -3_i32..=2 {
            let px = x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let pz = z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5;
            let height = 8.0 + (((x + 3) * 7 + (z + 3) * 11) % 6) as f32 * 4.0;
            let color_index = (((x + 3) + 2 * (z + 3)) % 4) as usize;
            game.buildings.push(Vec2::new(px, pz));
            commands.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(buildings[color_index].clone()),
                Transform::from_xyz(px, height / 2.0, pz).with_scale(Vec3::new(
                    BUILDING_HALF * 2.0,
                    height,
                    BUILDING_HALF * 2.0,
                )),
            ));
            for trim_x in [-1.0, 1.0] {
                for trim_z in [-1.0, 1.0] {
                    commands.spawn((
                        Mesh3d(building_trim_mesh.clone()),
                        MeshMaterial3d(building_trim_material.clone()),
                        Transform::from_xyz(
                            px + trim_x * (BUILDING_HALF - 0.05),
                            height / 2.0,
                            pz + trim_z * (BUILDING_HALF - 0.05),
                        )
                        .with_scale(Vec3::new(1.0, height, 1.0)),
                    ));
                }
            }
        }
    }

    commands
        .spawn((
            Transform::from_translation(game.player),
            Visibility::default(),
            PlayerVisual {
                facing_yaw: game.player_facing_yaw,
            },
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(player_mat.clone()),
                Transform::from_xyz(0.0, 0.10, 0.0).with_scale(Vec3::new(0.60, 0.80, 0.36)),
            ));
            parent.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(skin_mat.clone()),
                Transform::from_xyz(0.0, 0.74, 0.0).with_scale(Vec3::splat(0.34)),
            ));
            for side in [-1.0_f32, 1.0] {
                parent
                    .spawn((
                        Transform::from_xyz(side * 0.40, 0.42, 0.0),
                        Visibility::default(),
                        Limb {
                            phase_offset: if side < 0.0 {
                                0.0
                            } else {
                                std::f32::consts::PI
                            },
                        },
                    ))
                    .with_children(|arm| {
                        arm.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(player_mat.clone()),
                            Transform::from_xyz(0.0, -0.28, 0.0)
                                .with_scale(Vec3::new(0.16, 0.62, 0.20)),
                        ));
                    });
            }
            for side in [-1.0_f32, 1.0] {
                parent
                    .spawn((
                        Transform::from_xyz(side * 0.17, -0.15, 0.0),
                        Visibility::default(),
                        Limb {
                            phase_offset: if side < 0.0 {
                                std::f32::consts::PI
                            } else {
                                0.0
                            },
                        },
                    ))
                    .with_children(|leg| {
                        leg.spawn((
                            Mesh3d(cube.clone()),
                            MeshMaterial3d(glass.clone()),
                            Transform::from_xyz(0.0, -0.37, 0.0)
                                .with_scale(Vec3::new(0.22, 0.74, 0.24)),
                        ));
                    });
            }
        });

    commands
        .spawn((
            Transform::from_translation(game.car),
            Visibility::default(),
            CarVisual,
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(body.clone()),
                Transform::from_scale(Vec3::new(1.9, 0.9, 4.0)),
            ));
            parent.spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(glass.clone()),
                Transform::from_xyz(0.0, 0.62, -0.15).with_scale(Vec3::new(1.62, 0.62, 1.9)),
            ));
            for side in [-1.0_f32, 1.0] {
                for front in [-1.35_f32, 1.35] {
                    parent.spawn((
                        Mesh3d(wheel_mesh.clone()),
                        MeshMaterial3d(rubber.clone()),
                        Transform::from_xyz(side * 0.95, -0.30, front)
                            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2)),
                    ));
                }
                parent.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(headlight.clone()),
                    Transform::from_xyz(side * 0.62, 0.10, -2.02)
                        .with_scale(Vec3::new(0.34, 0.20, 0.10)),
                ));
                parent.spawn((
                    Mesh3d(cube.clone()),
                    MeshMaterial3d(headlight.clone()),
                    Transform::from_xyz(side * 0.62, 0.10, 2.02)
                        .with_scale(Vec3::new(0.34, 0.16, 0.10)),
                ));
            }
        });

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(marker_mat),
        PointLight {
            color: Color::srgb(0.15, 0.72, 1.0),
            intensity: 850.0,
            range: 20.0,
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_translation(OBJECTIVES[0]).with_scale(mission_marker_scale(0.0)),
        MissionMarker,
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(0.82, 0.89, 1.0),
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(15.0, 30.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        SunLight,
    ));
    for index in 0..STAR_COUNT {
        let step = index as f32;
        let azimuth = step * 2.399_963;
        let elevation = 0.05 + (step * 0.618_034).fract() * 0.95;
        let horizontal = 320.0 * (1.0 - elevation).sqrt();
        let size = 1.4 + (step * 0.372_1).fract() * 2.0;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(star_mat.clone()),
            Transform::from_xyz(
                azimuth.cos() * horizontal,
                40.0 + elevation * 220.0,
                azimuth.sin() * horizontal,
            )
            .with_scale(Vec3::splat(size)),
            Visibility::Hidden,
            Star,
        ));
    }
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Tonemapping::TonyMcMapface,
        Bloom::NATURAL,
        DistanceFog {
            color: Color::srgb(0.035, 0.06, 0.10),
            directional_light_color: Color::srgba(0.35, 0.55, 0.80, 0.18),
            directional_light_exponent: 12.0,
            falloff: FogFalloff::from_visibility_colors(
                190.0,
                Color::srgb(0.04, 0.06, 0.10),
                Color::srgb(0.10, 0.15, 0.22),
            ),
        },
        Transform::from_xyz(0.0, 6.0, 14.0).looking_at(game.player, Vec3::Y),
        FollowCamera,
    ));
    commands.spawn((
        Text::new("NEON FRONTIER"),
        ui::hud_font(),
        TextColor(Color::srgb(0.82, 0.94, 1.0)),
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(14),
            width: px(540),
            padding: UiRect::all(px(16)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.015, 0.028, 0.055, 0.88)),
        ui::Hud,
    ));
}

fn building_body(center: Vec2) -> physics::Body {
    physics::Body::new(center, BUILDING_HALF, BUILDING_HALF, 0.0)
}

fn walking_player_body(position: Vec2, clearance: f32) -> physics::Body {
    physics::Body::new(position, clearance, clearance, 0.0)
}

fn collides(pos: Vec2, radius: f32, buildings: &[Vec2]) -> bool {
    if !pos.is_finite()
        || !radius.is_finite()
        || pos.x.abs() + radius > WORLD_EDGE
        || pos.y.abs() + radius > WORLD_EDGE
    {
        return true;
    }
    let player = walking_player_body(pos, radius);
    buildings
        .iter()
        .any(|center| physics::bodies_overlap(&player, &building_body(*center)))
}

fn player_vehicle_body(position: Vec2, yaw: f32) -> physics::Body {
    physics::Body::vehicle(position, yaw + std::f32::consts::FRAC_PI_2)
}

fn collides_vehicle(pos: Vec2, yaw: f32, buildings: &[Vec2]) -> bool {
    if !pos.is_finite() || !yaw.is_finite() {
        return true;
    }
    let body = player_vehicle_body(pos, yaw);
    let (sin_yaw, cos_yaw) = body.yaw.sin_cos();
    let extent_x = physics::VEHICLE_HALF_LENGTH * cos_yaw.abs()
        + physics::VEHICLE_HALF_WIDTH * sin_yaw.abs();
    let extent_z = physics::VEHICLE_HALF_LENGTH * sin_yaw.abs()
        + physics::VEHICLE_HALF_WIDTH * cos_yaw.abs();
    if pos.x.abs() + extent_x > WORLD_EDGE || pos.y.abs() + extent_z > WORLD_EDGE {
        return true;
    }
    buildings
        .iter()
        .any(|center| physics::bodies_overlap(&body, &building_body(*center)))
}

fn blocked_by_traffic(bodies: &[physics::Body], next: Vec3, clearance: f32) -> bool {
    let player = walking_player_body(Vec2::new(next.x, next.z), clearance);
    bodies
        .iter()
        .any(|body| physics::bodies_overlap(&player, body))
}

fn blocked_by_parked_car(next: Vec3, car: Vec3, yaw: f32, clearance: f32) -> bool {
    let player = walking_player_body(Vec2::new(next.x, next.z), clearance);
    let parked = player_vehicle_body(Vec2::new(car.x, car.z), yaw);
    physics::bodies_overlap(&player, &parked)
}

fn vehicle_overlaps_any(position: Vec2, yaw: f32, obstacles: &[physics::Body]) -> bool {
    let vehicle = player_vehicle_body(position, yaw);
    obstacles
        .iter()
        .any(|obstacle| physics::bodies_overlap(&vehicle, obstacle))
}

fn collides_walking_with_bodies(
    position: Vec2,
    buildings: &[Vec2],
    obstacles: &[physics::Body],
    clearance: f32,
) -> bool {
    collides(position, clearance, buildings)
        || obstacles.iter().any(|obstacle| {
            physics::bodies_overlap(&walking_player_body(position, clearance), obstacle)
        })
}

fn collides_vehicle_with_bodies(
    position: Vec2,
    yaw: f32,
    buildings: &[Vec2],
    obstacles: &[physics::Body],
) -> bool {
    collides_vehicle(position, yaw, buildings) || vehicle_overlaps_any(position, yaw, obstacles)
}

fn walk_direction(horizontal: f32, forward: f32, yaw: f32) -> Vec3 {
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let ahead = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    (right * horizontal + ahead * forward).normalize_or_zero()
}

fn facing_yaw(direction: Vec3) -> Option<f32> {
    if direction.length_squared() > 1.0e-6 {
        Some(direction.x.atan2(direction.z))
    } else {
        None
    }
}

fn limb_swing(phase: f32, offset: f32, amplitude: f32, moving: bool) -> f32 {
    if moving {
        (phase + offset).sin() * amplitude
    } else {
        0.0
    }
}

fn vehicle_speed_step(speed: f32, accelerating: bool, braking: bool, dt: f32) -> f32 {
    let dt = dt.max(0.0);
    if dt == 0.0 {
        return speed;
    }

    if braking {
        if speed > 0.0 {
            (speed - 15.0 * dt).max(0.0)
        } else {
            (speed - 5.5 * dt).max(-8.0)
        }
    } else if accelerating {
        if speed < 0.0 {
            (speed + 15.0 * dt).min(0.0)
        } else {
            (speed + 9.0 * dt).min(24.0)
        }
    } else {
        let rolling_and_aero_drag = (0.35 + speed.abs() * 0.025) * dt;
        if speed.abs() <= rolling_and_aero_drag {
            0.0
        } else {
            speed - speed.signum() * rolling_and_aero_drag
        }
    }
}

fn vehicle_steering_step(steering: f32, input: f32, dt: f32) -> f32 {
    let input = input.clamp(-1.0, 1.0);
    let dt = dt.max(0.0);
    let steering_rate = if input.abs() > 0.01 { 3.2 } else { 2.0 };
    let max_change = steering_rate * dt;
    let change = (input - steering).clamp(-max_change, max_change);
    (steering + change).clamp(-1.0, 1.0)
}

fn vehicle_yaw_step(yaw: f32, steering: f32, speed: f32, dt: f32) -> f32 {
    let yaw_rate = (speed / 3.2 * 0.50).clamp(-1.8, 1.8);
    yaw - steering * yaw_rate * dt.max(0.0)
}

fn integrate_jump(
    y: f32,
    vertical_speed: f32,
    grounded: bool,
    jump_pressed: bool,
    dt: f32,
) -> (f32, f32, bool) {
    let mut speed = vertical_speed;
    let mut is_grounded = grounded;
    if is_grounded && jump_pressed {
        speed = PLAYER_JUMP_SPEED;
        is_grounded = false;
    }
    if !is_grounded {
        speed -= PLAYER_GRAVITY * dt;
        let next_y = y + speed * dt;
        if next_y <= PLAYER_GROUND_Y {
            return (PLAYER_GROUND_Y, 0.0, true);
        }
        return (next_y, speed, false);
    }
    (PLAYER_GROUND_Y, 0.0, true)
}

fn mission_marker_scale(elapsed_secs: f32) -> Vec3 {
    let pulse = 1.0 + (elapsed_secs * 2.2).sin() * MISSION_MARKER_PULSE;
    Vec3::new(1.1, 2.6, 1.1) * pulse
}

fn mission_marker_light_intensity(elapsed_secs: f32) -> f32 {
    850.0 + (elapsed_secs * 2.2).sin() * 250.0
}

fn snapshot_from_game(game: &Game) -> save::Snapshot {
    save::Snapshot {
        player: game.player,
        car: game.car,
        yaw: game.yaw,
        speed: game.speed,
        steering: game.steering,
        lateral_speed: game.lateral_speed,
        player_facing_yaw: game.player_facing_yaw,
        player_moving: game.player_moving,
        player_running: game.player_running,
        player_vertical_speed: game.player_vertical_speed,
        player_grounded: game.player_grounded,
        punch_cooldown: game.punch_cooldown,
        camera_yaw: game.camera_yaw,
        camera_orbit: game.camera_orbit,
        camera_pitch: game.camera_pitch,
        stage: game.stage,
        credits: game.credits,
        driving: game.driving,
        bloom: game.bloom,
        world_time: game.world_time,
        world: city_life::WorldSnapshot::default(),
    }
}

fn save_game(game: &Game, world: &city_life::WorldSnapshot) -> io::Result<()> {
    let mut snapshot = snapshot_from_game(game);
    snapshot.world = world.clone();
    save::save_snapshot(&snapshot)
}

fn load_game(
    game: &mut Game,
    probe: &mut city_life::PlayerProbe,
    restore: &mut city_life::WorldRestore,
) -> io::Result<()> {
    let snapshot = save::load_snapshot()?;
    let stage = if snapshot.stage == OBJECTIVES.len() {
        0
    } else {
        snapshot.stage
    };
    if stage > OBJECTIVES.len()
        || collides(
            Vec2::new(snapshot.player.x, snapshot.player.z),
            PLAYER_BODY_RADIUS,
            &game.buildings,
        )
        || collides_vehicle(
            Vec2::new(snapshot.car.x, snapshot.car.z),
            snapshot.yaw,
            &game.buildings,
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Save position or progress is outside the supported game state",
        ));
    }
    game.player = snapshot.player;
    game.car = snapshot.car;
    game.yaw = snapshot.yaw;
    game.speed = snapshot.speed;
    game.steering = snapshot.steering;
    game.lateral_speed = snapshot.lateral_speed;
    game.player_facing_yaw = snapshot.player_facing_yaw;
    game.player_moving = snapshot.player_moving;
    game.player_running = snapshot.player_running;
    game.player_vertical_speed = snapshot.player_vertical_speed;
    game.player_grounded = snapshot.player_grounded;
    game.punch_cooldown = snapshot.punch_cooldown;
    game.stage = stage;
    game.credits = snapshot.credits;
    game.driving = snapshot.driving;
    game.paused = false;
    game.bloom = snapshot.bloom;
    game.world_time = snapshot.world_time;
    game.camera_yaw = snapshot.camera_yaw;
    game.camera_orbit = snapshot.camera_orbit;
    game.camera_pitch = snapshot.camera_pitch;
    game.mouse_captured = false;
    game.notice = "Сохранение загружено".to_string();
    game.notice_timer = 3.0;
    probe.health = snapshot.world.health.clamp(0.0, 100.0);
    probe.position = if game.driving { game.car } else { game.player };
    probe.in_vehicle = game.driving;
    probe.vehicle_position = game.car;
    probe.forward = if game.driving {
        Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos())
    } else {
        walk_direction(0.0, 1.0, game.camera_yaw)
    };
    restore.0 = Some(snapshot.world.clone());
    Ok(())
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    window: Single<&Window>,
    mut cursor_options: Single<&mut CursorOptions>,
    time: Res<Time>,
    traffic: Res<city_life::TrafficBodies>,
    world: Res<city_life::WorldState>,
    mut probe: ResMut<city_life::PlayerProbe>,
    mut restore: ResMut<city_life::WorldRestore>,
    mut game: ResMut<Game>,
) {
    if !game.started {
        let options = ui::MENU_OPTIONS.len();
        if game.credits_screen {
            if keys.just_pressed(KeyCode::Escape)
                || keys.just_pressed(KeyCode::Enter)
                || keys.just_pressed(KeyCode::Space)
            {
                game.credits_screen = false;
            }
        } else {
            if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
                game.menu_index = (game.menu_index + 1) % options;
            }
            if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
                game.menu_index = (game.menu_index + options - 1) % options;
            }
            let mouse_selected = if mouse_buttons.just_pressed(MouseButton::Left) {
                window
                    .cursor_position()
                    .and_then(ui::menu_index_at_cursor)
            } else {
                None
            };
            if let Some(index) = mouse_selected {
                game.menu_index = index;
            }
            let activate = keys.just_pressed(KeyCode::Enter)
                || keys.just_pressed(KeyCode::Space)
                || mouse_selected.is_some();
            if activate {
                match ui::MENU_OPTIONS[game.menu_index.min(options - 1)] {
                    ui::MenuOption::NewGame => reset_run(&mut game),
                    ui::MenuOption::LoadGame => {
                        if load_game(&mut game, &mut probe, &mut restore).is_ok() {
                            game.started = true;
                            game.mouse_captured = false;
                        } else {
                            game.notice = format!("Не удалось загрузить {}", save::SAVE_FILE_NAME);
                            game.notice_timer = 3.0;
                        }
                    }
                    ui::MenuOption::Credits => game.credits_screen = true,
                }
            }
        }
        game.paused = false;
        cursor_options.visible = true;
        cursor_options.grab_mode = CursorGrabMode::None;
        return;
    }

    if keys.just_pressed(KeyCode::Escape) {
        game.paused = !game.paused;
        game.mouse_captured = !game.paused;
    }
    if mouse_buttons.just_pressed(MouseButton::Left) {
        game.paused = false;
        game.mouse_captured = true;
    }
    cursor_options.visible = !game.mouse_captured;
    cursor_options.grab_mode = if game.mouse_captured {
        CursorGrabMode::Locked
    } else {
        CursorGrabMode::None
    };

    let mouse_delta = mouse_motion
        .read()
        .fold(Vec2::ZERO, |delta, event| delta + event.delta);
    if game.mouse_captured && !game.paused {
        if game.driving {
            game.camera_orbit -= mouse_delta.x * MOUSE_SENSITIVITY;
        } else {
            game.camera_yaw -= mouse_delta.x * MOUSE_SENSITIVITY;
        }
        game.camera_pitch =
            (game.camera_pitch - mouse_delta.y * MOUSE_SENSITIVITY).clamp(0.12, 1.05);
    }

    if keys.just_pressed(KeyCode::KeyB) {
        game.bloom = !game.bloom;
    }
    let dt = time.delta_secs().min(0.05);
    if game.notice_timer > 0.0 {
        game.notice_timer -= dt;
        if game.notice_timer <= 0.0 {
            game.notice.clear();
        }
    }
    if keys.just_pressed(KeyCode::F5) {
        let saved = save_game(&game, &world.snapshot).is_ok();
        game.notice = if saved {
            format!("Сохранение записано: {}", save::SAVE_FILE_NAME)
        } else {
            "Не удалось записать сохранение".to_string()
        };
        game.notice_timer = 3.0;
    }
    if keys.just_pressed(KeyCode::F9) {
        let loaded = load_game(&mut game, &mut probe, &mut restore).is_ok();
        if !loaded {
            game.notice = format!("Не удалось загрузить {}", save::SAVE_FILE_NAME);
            game.notice_timer = 3.0;
        }
    }
    if game.paused {
        game.player_moving = false;
        game.player_running = false;
        return;
    }

    game.punch_cooldown = (game.punch_cooldown - dt).max(0.0);
    if !game.driving && game.punch_cooldown <= 0.0 && keys.just_pressed(KeyCode::KeyF) {
        game.punch_cooldown = PUNCH_COOLDOWN;
        probe.punch = true;
    }

    if keys.just_pressed(KeyCode::KeyE) {
        if game.driving {
            if game.speed.abs() < 2.0 {
                let side = Vec3::new(game.yaw.cos(), 0.0, -game.yaw.sin()) * 3.0;
                let candidates = [game.car + side, game.car - side];
                if let Some(exit) = candidates.into_iter().find(|exit| {
                    !collides(
                        Vec2::new(exit.x, exit.z),
                        PLAYER_BODY_RADIUS,
                        &game.buildings,
                    ) && !blocked_by_traffic(&traffic.oriented, *exit, PLAYER_BODY_RADIUS)
                        && !blocked_by_parked_car(
                            *exit,
                            game.car,
                            game.yaw,
                            PLAYER_BODY_RADIUS,
                        )
                }) {
                    game.player = Vec3::new(exit.x, PLAYER_GROUND_Y, exit.z);
                    game.camera_yaw = game.yaw + std::f32::consts::PI + game.camera_orbit;
                    game.driving = false;
                    game.speed = 0.0;
                    game.lateral_speed = 0.0;
                    game.player_moving = false;
                    game.player_vertical_speed = 0.0;
                    game.player_grounded = true;
                }
            }
        } else if game.player_grounded && game.player.distance(game.car) < 4.5 {
            game.driving = true;
            game.lateral_speed = 0.0;
            game.player_moving = false;
            game.camera_orbit = 0.0;
            game.player_vertical_speed = 0.0;
            game.player_grounded = true;
        }
    }

    game.player_running = false;
    if game.driving {
        game.player_moving = false;
        let accelerating = keys.pressed(KeyCode::KeyW);
        let braking = keys.pressed(KeyCode::KeyS);
        let steer_input =
            (keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32) as f32;
        game.speed = vehicle_speed_step(game.speed, accelerating, braking, dt);
        game.steering = vehicle_steering_step(game.steering, steer_input, dt);
        let speed_ratio = (game.speed.abs() / 24.0).clamp(0.0, 1.0);
        game.yaw = vehicle_yaw_step(game.yaw, game.steering, game.speed, dt);
        let direction = Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos());
        let right = Vec3::new(game.yaw.cos(), 0.0, -game.yaw.sin());
        let grip = 2.5 + 4.0 * (1.0 - speed_ratio);
        let slide_gain = 0.28 * speed_ratio;
        game.lateral_speed = (game.lateral_speed
            + game.steering * game.speed.abs() * slide_gain * dt)
            .clamp(-3.5, 3.5);
        game.lateral_speed *= (1.0 - grip * dt).max(0.0);
        let intended = (Vec2::new(direction.x, direction.z) * game.speed
            + Vec2::new(right.x, right.z) * game.lateral_speed)
            * dt;
        let start = Vec2::new(game.car.x, game.car.z);
        let blockers = traffic.oriented.clone();
        let separated = physics::resolve_body_overlaps(
            player_vehicle_body(start, game.yaw),
            &blockers,
        )
        .center;
        let moved = physics::move_with_slide(separated, intended, |point| {
            collides_vehicle_with_bodies(point, game.yaw, &game.buildings, &blockers)
        });
        if moved.distance(start) + 0.02 < intended.length() {
            game.speed *= 0.28;
            game.lateral_speed *= 0.25;
        }
        game.car.x = moved.x;
        game.car.z = moved.y;
        game.player = game.car;
    } else {
        let horizontal =
            (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32;
        let forward =
            (keys.pressed(KeyCode::KeyW) as i32 - keys.pressed(KeyCode::KeyS) as i32) as f32;
        let direction = walk_direction(horizontal, forward, game.camera_yaw);
        let run = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) {
            PLAYER_RUN_SPEED
        } else {
            PLAYER_WALK_SPEED
        };
        let mut blockers = traffic.oriented.clone();
        blockers.push(player_vehicle_body(
            Vec2::new(game.car.x, game.car.z),
            game.yaw,
        ));
        let start = Vec2::new(game.player.x, game.player.z);
        let separated = physics::resolve_body_overlaps(
            walking_player_body(start, PLAYER_BODY_RADIUS),
            &blockers,
        )
        .center;
        let delta = Vec2::new(direction.x, direction.z) * run * dt;
        let moved = physics::move_with_slide(separated, delta, |point| {
            collides_walking_with_bodies(
                point,
                &game.buildings,
                &blockers,
                PLAYER_BODY_RADIUS,
            )
        });
        game.player.x = moved.x;
        game.player.z = moved.y;
        let movement = Vec3::new(moved.x - start.x, 0.0, moved.y - start.y);
        game.player_moving = movement.length_squared() > 1.0e-6;
        game.player_running = game.player_moving && run > PLAYER_WALK_SPEED;
        if let Some(yaw) = facing_yaw(movement) {
            game.player_facing_yaw = yaw;
        }
        let (y, vertical_speed, grounded) = integrate_jump(
            game.player.y,
            game.player_vertical_speed,
            game.player_grounded,
            keys.just_pressed(KeyCode::Space),
            dt,
        );
        game.player.y = y;
        game.player_vertical_speed = vertical_speed;
        game.player_grounded = grounded;
    }
    if game.stage < OBJECTIVES.len() {
        let position = if game.driving { game.car } else { game.player };
        let target = mission_target(game.stage, game.credits);
        if Vec2::new(position.x - target.x, position.z - target.z).length() < 4.0 {
            game.credits += 100;
            if game.stage + 1 == OBJECTIVES.len() {
                game.stage = 0;
            } else {
                game.stage += 1;
            }
        }
    }

    probe.position = if game.driving { game.car } else { game.player };
    probe.forward = if game.driving {
        Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos())
    } else {
        walk_direction(0.0, 1.0, game.camera_yaw)
    };
    probe.in_vehicle = game.driving;
    probe.vehicle_position = game.car;

    if probe.health <= 0.0 {
        probe.health = 100.0;
        game.driving = false;
        game.speed = 0.0;
        game.steering = 0.0;
        game.lateral_speed = 0.0;
        game.player = SPAWN_PLAYER;
        game.car = SPAWN_CAR;
        game.player_vertical_speed = 0.0;
        game.player_grounded = true;
        game.player_moving = false;
        game.camera_yaw = 0.0;
        game.camera_orbit = 0.0;
        game.mouse_captured = false;
        game.notice = "Здоровье на нуле: игрок вернулся на старт".to_string();
        game.notice_timer = 3.0;
        probe.position = game.player;
        probe.forward = walk_direction(0.0, 1.0, game.camera_yaw);
        probe.in_vehicle = false;
        probe.vehicle_position = game.car;
    }
}

fn update_world(
    mut game: ResMut<Game>,
    time: Res<Time>,
    mut sun: Single<&mut DirectionalLight, With<SunLight>>,
    mut fog: Single<&mut DistanceFog, With<FollowCamera>>,
    mut players: Query<
        (&mut Transform, &mut Visibility, &mut PlayerVisual),
        (
            With<PlayerVisual>,
            Without<CarVisual>,
            Without<FollowCamera>,
            Without<MissionMarker>,
            Without<Limb>,
            Without<Star>,
        ),
    >,
    mut cars: Query<
        &mut Transform,
        (
            With<CarVisual>,
            Without<PlayerVisual>,
            Without<FollowCamera>,
            Without<MissionMarker>,
            Without<Limb>,
        ),
    >,
    mut cameras: Query<
        (Entity, &mut Transform, Option<&Bloom>),
        (
            With<FollowCamera>,
            Without<PlayerVisual>,
            Without<CarVisual>,
            Without<MissionMarker>,
        ),
    >,
    mut markers: Query<
        (&mut Transform, &mut Visibility, &mut PointLight),
        (
            With<MissionMarker>,
            Without<PlayerVisual>,
            Without<CarVisual>,
            Without<FollowCamera>,
        ),
    >,
    mut limbs: Query<
        (&mut Transform, &Limb),
        (
            With<Limb>,
            Without<PlayerVisual>,
            Without<CarVisual>,
            Without<FollowCamera>,
            Without<MissionMarker>,
        ),
    >,
    mut stars: Query<
        &mut Visibility,
        (
            With<Star>,
            Without<PlayerVisual>,
            Without<MissionMarker>,
        ),
    >,
    mut commands: Commands,
) {
    let elapsed_secs = time.elapsed_secs();
    let dt = time.delta_secs().min(0.05);
    let daylight = daylight_factor(game.world_time);
    let dusk = (1.0 - (2.0 * daylight - 1.0).abs()).clamp(0.0, 1.0);
    sun.illuminance = 240.0 + daylight * daylight * 14_000.0;
    sun.color = Color::srgb(
        (0.55 + daylight * 0.30 + dusk * 0.34).min(1.0),
        (0.58 + daylight * 0.33 + dusk * 0.12).min(1.0),
        (0.74 + daylight * 0.22 - dusk * 0.28).clamp(0.28, 1.0),
    );
    fog.color = Color::srgb(
        0.018 + daylight * 0.022,
        0.030 + daylight * 0.036,
        0.060 + daylight * 0.048,
    );
    fog.directional_light_color = Color::srgba(
        0.28 + daylight * 0.12,
        0.42 + daylight * 0.16,
        0.72 + daylight * 0.10,
        0.18,
    );
    for mut visibility in &mut stars {
        *visibility = if daylight < 0.45 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    game.world_time = advance_clock(game.world_time, dt);
    let walking = game.player_moving && !game.driving;
    let step_rate = if game.player_running { 12.0 } else { 7.0 };
    let step_phase = elapsed_secs * step_rate;
    let limb_amplitude = if game.player_running {
        RUN_LIMB_SWING
    } else {
        WALK_LIMB_SWING
    };
    for (mut transform, mut visibility, mut visual) in &mut players {
        transform.translation = game.player;
        let running = walking && game.player_running;
        if walking {
            let bob_amount = if running { 0.06 } else { 0.035 };
            transform.translation.y += (step_phase * 2.0).sin() * bob_amount;
        }

        let turn_delta = (game.player_facing_yaw - visual.facing_yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        visual.facing_yaw += turn_delta * (1.0 - (-14.0 * dt).exp());

        let stride_sway = if walking {
            step_phase.sin() * if running { 0.10 } else { 0.055 }
        } else {
            0.0
        };
        let run_lean = if running { -0.18 } else { 0.0 };
        transform.rotation = Quat::from_rotation_y(visual.facing_yaw)
            * Quat::from_rotation_x(run_lean)
            * Quat::from_rotation_z(stride_sway);
        *visibility = if game.driving {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }
    for (mut transform, limb) in &mut limbs {
        let angle = limb_swing(step_phase, limb.phase_offset, limb_amplitude, walking);
        transform.rotation = Quat::from_rotation_x(angle);
    }
    for mut transform in &mut cars {
        transform.translation = game.car;
        transform.rotation = Quat::from_rotation_y(game.yaw);
    }
    for (mut transform, mut visibility, mut light) in &mut markers {
        if game.stage < OBJECTIVES.len() {
            transform.translation = mission_target(game.stage, game.credits);
            transform.scale = mission_marker_scale(elapsed_secs);
            light.intensity = mission_marker_light_intensity(elapsed_secs);
            *visibility = Visibility::Visible;
        } else {
            *visibility = Visibility::Hidden;
        }
    }
    let target = if game.driving { game.car } else { game.player };
    let camera_yaw = if game.driving {
        game.yaw + std::f32::consts::PI + game.camera_orbit
    } else {
        game.camera_yaw
    };
    let radius = if game.driving { 14.0 } else { 12.0 };
    let pitch = game.camera_pitch;
    let offset = Vec3::new(
        camera_yaw.sin() * pitch.cos(),
        pitch.sin(),
        camera_yaw.cos() * pitch.cos(),
    ) * radius;
    let desired = target + offset;
    for (entity, mut camera, bloom) in &mut cameras {
        let blend = (time.delta_secs() * 4.0).clamp(0.0, 1.0);
        camera.translation = camera.translation.lerp(desired, blend);
        camera.look_at(target, Vec3::Y);
        if game.bloom && bloom.is_none() {
            commands.entity(entity).insert(Bloom::NATURAL);
        } else if !game.bloom && bloom.is_some() {
            commands.entity(entity).remove::<Bloom>();
        }
    }
}

fn update_hud(
    game: Res<Game>,
    wanted: Res<city_life::WantedState>,
    probe: Res<city_life::PlayerProbe>,
    mut labels: Query<&mut Text, With<ui::Hud>>,
) {
    if !game.started {
        let menu = ui::main_menu_text(game.menu_index, game.credits_screen, &game.notice);
        for mut text in &mut labels {
            text.0 = menu.clone();
        }
        return;
    }
    let position = if game.driving { game.car } else { game.player };
    let target = mission_target(game.stage, game.credits);
    let distance = Vec2::new(position.x - target.x, position.z - target.z).length();
    let objective = format!(
        "Маршрут «{}»: точка {}/{} • {:.0} м",
        mission_name(game.credits),
        game.stage + 1,
        OBJECTIVES.len(),
        distance
    );
    let mode = if game.paused {
        "ПАУЗА — Esc или ЛКМ, чтобы продолжить"
    } else if game.driving {
        "ЗА РУЛЁМ"
    } else {
        "ПЕШКОМ"
    };
    let state = format!(
        "{} • розыск: {} • {}",
        mode,
        "★".repeat(wanted.level as usize),
        clock_label(game.world_time)
    );
    let interaction = if game.driving {
        if game.speed.abs() < 2.0 {
            "E: выйти из машины"
        } else {
            "Сбавь скорость и нажми E, чтобы выйти"
        }
    } else if game.player.distance(game.car) < 4.5 {
        "E: сесть в машину"
    } else {
        "Подойди к бирюзовой машине, чтобы сесть"
    };
    let speed = if game.driving {
        format!("Скорость: {:.1} м/с", game.speed.abs())
    } else {
        String::new()
    };
    let mouse_hint = if game.mouse_captured {
        "Камера: мышь • Esc: пауза • F: удар • Shift: бег"
    } else {
        "ЛКМ: захватить мышь и управлять камерой"
    };
    let content = ui::compose_hud(
        &objective,
        game.credits,
        probe.health,
        &state,
        interaction,
        &speed,
        mouse_hint,
        &game.notice,
    );
    for mut text in &mut labels {
        text.0 = content.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn building_collision_and_clear_road() {
        let buildings = vec![Vec2::new(24.0, 24.0)];
        assert!(collides(Vec2::new(24.0, 24.0), 0.5, &buildings));
        assert!(!collides(Vec2::new(0.0, 0.0), 0.5, &buildings));
    }

    #[test]
    fn world_boundary_blocks_escape() {
        assert!(collides(Vec2::new(WORLD_EDGE, 0.0), 1.0, &[]));
        assert!(!collides(Vec2::new(0.0, 0.0), 1.0, &[]));
    }

    #[test]
    fn player_vehicle_building_contact_uses_its_oriented_obb() {
        let buildings = [Vec2::new(24.0, 24.0)];
        let near_wall = Vec2::new(39.5, 24.0);

        assert!(collides(near_wall, 2.2, &buildings));
        assert!(!collides_vehicle(near_wall, 0.0, &buildings));
        assert!(collides_vehicle(
            near_wall,
            std::f32::consts::FRAC_PI_2,
            &buildings,
        ));

        let start = Vec2::new(42.0, 24.0);
        let stopped = physics::move_with_slide(start, Vec2::new(-5.0, 0.0), |position| {
            collides_vehicle(position, std::f32::consts::FRAC_PI_2, &buildings)
        });
        assert!(stopped.x < start.x);
        assert!(stopped.x > 40.0);
        assert!(!collides_vehicle(
            stopped,
            std::f32::consts::FRAC_PI_2,
            &buildings,
        ));
    }

    #[test]
    fn walking_uses_camera_relative_directions() {
        let forward = walk_direction(0.0, 1.0, 0.0);
        let right = walk_direction(1.0, 0.0, 0.0);
        assert!(forward.distance(Vec3::NEG_Z) < 1.0e-5);
        assert!(right.distance(Vec3::X) < 1.0e-5);
        let turned_forward = walk_direction(0.0, 1.0, std::f32::consts::FRAC_PI_2);
        assert!(turned_forward.distance(Vec3::NEG_X) < 1.0e-5);
    }

    #[test]
    fn parked_own_car_blocks_the_walking_player() {
        let car = SPAWN_CAR;
        assert!(blocked_by_parked_car(
            Vec3::new(0.0, 0.9, 8.0),
            car,
            0.0,
            PLAYER_BODY_RADIUS,
        ));
        assert!(!blocked_by_parked_car(
            Vec3::new(2.0, 0.9, 6.0),
            car,
            0.0,
            PLAYER_BODY_RADIUS,
        ));
    }

    #[test]
    fn spawn_points_start_on_the_ground_and_on_the_road() {
        assert_eq!(SPAWN_PLAYER.y, PLAYER_GROUND_Y);
        assert!(!collides(
            Vec2::new(SPAWN_PLAYER.x, SPAWN_PLAYER.z),
            PLAYER_BODY_RADIUS,
            &[]
        ));
        assert!(!collides_vehicle(
            Vec2::new(SPAWN_CAR.x, SPAWN_CAR.z),
            0.0,
            &[],
        ));
    }

    #[test]
    fn jump_rises_and_returns_to_ground() {
        let (mut y, mut speed, mut grounded) =
            integrate_jump(PLAYER_GROUND_Y, 0.0, true, true, 0.1);
        assert!(y > PLAYER_GROUND_Y && speed > 0.0 && !grounded);
        for _ in 0..40 {
            (y, speed, grounded) = integrate_jump(y, speed, grounded, false, 0.05);
        }
        assert_eq!(y, PLAYER_GROUND_Y);
        assert_eq!(speed, 0.0);
        assert!(grounded);
    }

    #[test]
    fn jump_clears_a_tall_step() {
        let mut y = PLAYER_GROUND_Y;
        let mut speed = PLAYER_JUMP_SPEED;
        let mut grounded = false;
        let mut peak = y;
        for _ in 0..200 {
            let (next_y, next_speed, next_grounded) =
                integrate_jump(y, speed, grounded, false, 0.016);
            y = next_y;
            speed = next_speed;
            grounded = next_grounded;
            peak = peak.max(y);
            if grounded {
                break;
            }
        }
        assert!(peak - PLAYER_GROUND_Y > 1.5);
        assert_eq!(y, PLAYER_GROUND_Y);
    }

    #[test]
    fn limbs_swing_in_opposite_phase_and_rest_when_standing() {
        let phase = 0.8;
        let left = limb_swing(phase, 0.0, WALK_LIMB_SWING, true);
        let right = limb_swing(phase, std::f32::consts::PI, WALK_LIMB_SWING, true);
        assert!((left + right).abs() < 1.0e-5);
        assert!(left.abs() > 0.0);
        assert_eq!(limb_swing(phase, 0.0, RUN_LIMB_SWING, false), 0.0);
        assert!(limb_swing(phase, 0.0, RUN_LIMB_SWING, true).abs() <= RUN_LIMB_SWING);
    }

    #[test]
    fn oriented_traffic_and_pedestrians_block_a_walking_player() {
        let car = physics::Body::vehicle(Vec2::new(4.0, 0.0), std::f32::consts::FRAC_PI_2);
        let pedestrian = physics::Body::person(Vec2::new(12.0, 0.0));
        let bodies = [car, pedestrian];
        assert!(blocked_by_traffic(&bodies, Vec3::new(5.0, 0.0, 0.0), 0.45));
        assert!(blocked_by_traffic(&bodies, Vec3::new(12.0, 0.0, 0.0), 0.45));
        assert!(!blocked_by_traffic(&bodies, Vec3::new(12.0, 0.0, 2.0), 0.45));
    }

    #[test]
    fn player_vehicle_and_walking_contacts_use_obb_not_circle_radius() {
        let parallel_car = physics::Body::vehicle(
            Vec2::new(2.0, 0.0),
            std::f32::consts::FRAC_PI_2,
        );
        assert!(!vehicle_overlaps_any(Vec2::ZERO, 0.0, &[parallel_car]));
        assert!(vehicle_overlaps_any(
            Vec2::ZERO,
            0.0,
            &[physics::Body::vehicle(Vec2::new(0.0, 1.8), 0.0)],
        ));
        assert!(collides_vehicle_with_bodies(
            Vec2::ZERO,
            0.0,
            &[],
            &[physics::Body::person(Vec2::new(0.0, 1.5))],
        ));
        assert!(!collides_vehicle_with_bodies(
            Vec2::ZERO,
            0.0,
            &[],
            &[physics::Body::person(Vec2::new(1.5, 0.0))],
        ));

        let nearby_person = physics::Body::person(Vec2::new(0.75, 0.0));
        assert!(collides_walking_with_bodies(
            Vec2::ZERO,
            &[],
            &[nearby_person],
            PLAYER_BODY_RADIUS,
        ));
        assert!(!collides_walking_with_bodies(
            Vec2::new(0.9, 0.0),
            &[],
            &[nearby_person],
            PLAYER_BODY_RADIUS,
        ));
    }

    #[test]
    fn driving_player_slides_before_vehicle_and_pedestrian_obbs() {
        let traffic_car = physics::Body::vehicle(Vec2::ZERO, 0.0);
        let start = Vec2::new(-6.0, 0.0);
        let stopped = physics::move_with_slide(start, Vec2::new(12.0, 0.0), |point| {
            collides_vehicle_with_bodies(point, 0.0, &[], &[traffic_car])
        });
        assert!(stopped.x < -3.0);
        assert!(!collides_vehicle_with_bodies(
            stopped,
            0.0,
            &[],
            &[traffic_car],
        ));

        let pedestrian = physics::Body::person(Vec2::ZERO);
        let start = Vec2::new(-4.0, 0.0);
        let stopped = physics::move_with_slide(start, Vec2::new(8.0, 0.0), |point| {
            collides_vehicle_with_bodies(point, 0.0, &[], &[pedestrian])
        });
        assert!(stopped.x < -1.2);
        assert!(!collides_vehicle_with_bodies(
            stopped,
            0.0,
            &[],
            &[pedestrian],
        ));
    }

    #[test]
    fn walking_player_slides_along_a_rotated_vehicle_obb() {
        let vehicle = physics::Body::vehicle(Vec2::ZERO, std::f32::consts::FRAC_PI_2);
        let start = Vec2::new(-3.0, 0.0);
        let moved = physics::move_with_slide(start, Vec2::new(6.0, 0.0), |point| {
            collides_walking_with_bodies(point, &[], &[vehicle], PLAYER_BODY_RADIUS)
        });
        assert!(moved.x < -1.3);
        assert!(moved.y.abs() < 1.0e-5);
        assert!(!collides_walking_with_bodies(
            moved,
            &[],
            &[vehicle],
            PLAYER_BODY_RADIUS,
        ));
    }

    #[test]
    fn mission_beacon_pulse_stays_within_visual_limits() {
        for time in [0.0, 0.7, 1.4, 2.1, 2.8] {
            let scale = mission_marker_scale(time);
            let intensity = mission_marker_light_intensity(time);
            assert!(scale.x >= 1.1 * (1.0 - MISSION_MARKER_PULSE));
            assert!(scale.y <= 2.6 * (1.0 + MISSION_MARKER_PULSE));
            assert!((340.0..=1_100.0).contains(&intensity));
        }
    }

    #[test]
    fn missions_alternate_routes_every_300_credits() {
        assert_eq!(mission_target(0, 0), OBJECTIVES[0]);
        assert_eq!(mission_target(1, 299), OBJECTIVES[1]);
        assert_eq!(mission_target(0, 300), ALTERNATE_OBJECTIVES[0]);
        assert_eq!(mission_target(0, 600), OBJECTIVES[0]);
        assert_ne!(mission_name(0), mission_name(300));
    }

    #[test]
    fn braking_reaches_zero_before_reverse() {
        let moving = vehicle_speed_step(12.0, false, true, 0.25);
        assert!(moving < 12.0 && moving > 0.0);
        let stopped = vehicle_speed_step(moving, false, true, 1.0);
        assert_eq!(stopped, 0.0);
        let reversing = vehicle_speed_step(stopped, false, true, 0.1);
        assert!(reversing < 0.0);
    }

    #[test]
    fn throttle_change_from_reverse_brakes_before_accelerating_forward() {
        let first = vehicle_speed_step(-8.0, true, false, 0.25);
        assert!(first < 0.0);
        let at_rest = vehicle_speed_step(first, true, false, 1.0);
        assert_eq!(at_rest, 0.0);
        let forward = vehicle_speed_step(at_rest, true, false, 0.1);
        assert!(forward > 0.0);
    }

    #[test]
    fn coasting_keeps_momentum_then_settles_without_reversing() {
        let first = vehicle_speed_step(20.0, false, false, 0.1);
        assert!(first < 20.0 && first > 0.0);
        let mut speed = 1.0;
        for _ in 0..300 {
            speed = vehicle_speed_step(speed, false, false, 0.05);
            assert!(speed >= 0.0);
        }
        assert_eq!(speed, 0.0);
    }

    #[test]
    fn steering_changes_smoothly_toward_input_and_self_centers() {
        let first = vehicle_steering_step(0.0, 1.0, 0.05);
        let second = vehicle_steering_step(first, 1.0, 0.05);
        assert!(first > 0.0 && first < 1.0);
        assert!(second > first && second < 1.0);
        let centered = vehicle_steering_step(second, 0.0, 0.05);
        assert!(centered < second && centered > 0.0);
    }

    #[test]
    fn steering_turns_left_in_forward_motion_and_reverses_in_reverse() {
        let forward = vehicle_yaw_step(0.0, 1.0, 12.0, 0.1);
        let reverse = vehicle_yaw_step(0.0, 1.0, -12.0, 0.1);
        assert!(forward < 0.0);
        assert!(reverse > 0.0);
        assert_eq!(vehicle_yaw_step(0.7, 1.0, 12.0, 0.0), 0.7);
    }

    #[test]
    fn player_facing_yaw_matches_forward_backward_and_rightward_motion() {
        let forward = facing_yaw(Vec3::NEG_Z).unwrap();
        let backward = facing_yaw(Vec3::Z).unwrap();
        let right = facing_yaw(Vec3::X).unwrap();
        assert!((forward - std::f32::consts::PI).abs() < 1.0e-5);
        assert!(backward.abs() < 1.0e-5);
        assert!((right - std::f32::consts::FRAC_PI_2).abs() < 1.0e-5);
        assert_eq!(facing_yaw(Vec3::ZERO), None);
    }

    #[test]
    fn new_game_keeps_the_building_list_and_starts_playing() {
        let mut game = Game::default();
        game.buildings.push(Vec2::new(24.0, 24.0));
        reset_run(&mut game);
        assert!(game.started);
        assert_eq!(game.buildings.len(), 1);
        assert_eq!(game.credits, 0);
        assert_eq!(game.stage, 0);
    }

    #[test]
    fn sun_peaks_at_noon_and_bottoms_out_at_midnight() {
        assert!((sun_height(0.5) - 1.0).abs() < 1.0e-5);
        assert!((sun_height(0.0) + 1.0).abs() < 1.0e-5);
        assert!(sun_height(0.25).abs() < 1.0e-4);
        assert!((sun_height(1.5) - sun_height(0.5)).abs() < 1.0e-5);
    }

    #[test]
    fn night_factor_runs_from_full_dark_to_full_day() {
        assert!((night_factor(0.0) - 1.0).abs() < 1.0e-5);
        assert_eq!(night_factor(0.5), 0.0);
        assert!(night_factor(0.25) > 0.0 && night_factor(0.25) < 0.2);
        assert!((daylight_factor(0.5) - 1.0).abs() < 1.0e-5);
    }

    #[test]
    fn clock_wraps_after_a_full_day_and_labels_hours() {
        let start = 0.92;
        let next = advance_clock(start, DAY_LENGTH_SECONDS * 0.1);
        assert!((next - 0.02).abs() < 1.0e-4);
        assert_eq!(clock_label(0.0), "00:00");
        assert_eq!(clock_label(0.5), "12:00");
        assert_eq!(clock_label(0.75), "18:00");
    }
}
