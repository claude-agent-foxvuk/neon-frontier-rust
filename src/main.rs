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
    window::{CursorGrabMode, CursorOptions},
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
const PLAYER_CAR_RADIUS: f32 = 2.2;

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
    stage: usize,
    credits: u32,
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
            player: Vec3::new(2.0, PLAYER_GROUND_Y, 2.0),
            car: Vec3::new(0.0, 0.65, 6.0),
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
            stage: 0,
            credits: 0,
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
    let buildings: Vec<_> = building_colors
        .into_iter()
        .map(|color| {
            materials.add(StandardMaterial {
                base_color: color,
                base_color_texture: Some(surfaces.facade.clone()),
                uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(4.0)),
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
    let player_mat = materials.add(Color::srgb(0.94, 0.61, 0.22));
    let marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.03, 0.7, 0.95),
        emissive: LinearRgba::rgb(0.0, 3.0, 5.0),
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

    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(player_mat),
        Transform::from_translation(game.player).with_scale(Vec3::new(0.7, 1.8, 0.5)),
        PlayerVisual {
            facing_yaw: game.player_facing_yaw,
        },
    ));
    commands.spawn((
        Mesh3d(cube.clone()),
        MeshMaterial3d(body),
        Transform::from_translation(game.car).with_scale(Vec3::new(2.0, 1.3, 4.0)),
        CarVisual,
    ));
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
    ));
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

fn collides(pos: Vec2, radius: f32, buildings: &[Vec2]) -> bool {
    if pos.x.abs() + radius > WORLD_EDGE || pos.y.abs() + radius > WORLD_EDGE {
        return true;
    }
    buildings.iter().any(|center| {
        let dx = ((pos.x - center.x).abs() - BUILDING_HALF).max(0.0);
        let dz = ((pos.y - center.y).abs() - BUILDING_HALF).max(0.0);
        dx * dx + dz * dz < radius * radius
    })
}

fn blocked_by_traffic(cars: &[(Vec2, f32)], next: Vec3, clearance: f32) -> bool {
    let flat = Vec2::new(next.x, next.z);
    cars.iter()
        .any(|(center, radius)| center.distance(flat) < *radius + clearance)
}

fn blocked_by_parked_car(next: Vec3, car: Vec3, clearance: f32) -> bool {
    Vec2::new(next.x - car.x, next.z - car.z).length() < PLAYER_CAR_RADIUS + clearance
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

fn vehicle_speed_step(speed: f32, accelerating: bool, braking: bool, dt: f32) -> f32 {
    let dt = dt.max(0.0);
    if braking {
        if speed > 0.0 {
            (speed - 22.0 * dt).max(0.0)
        } else {
            (speed - 8.0 * dt).max(-8.0)
        }
    } else if accelerating {
        (speed + 14.0 * dt).clamp(-8.0, 24.0)
    } else {
        speed * (1.0 - 0.7 * dt).max(0.0)
    }
}

fn vehicle_steering_step(steering: f32, input: f32, dt: f32) -> f32 {
    let blend = (dt.max(0.0) * 8.0).clamp(0.0, 1.0);
    steering + (input.clamp(-1.0, 1.0) - steering) * blend
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
        stage: game.stage,
        credits: game.credits,
        driving: game.driving,
        bloom: game.bloom,
    }
}

fn save_game(game: &Game) -> io::Result<()> {
    save::save_snapshot(snapshot_from_game(game))
}

fn load_game(game: &mut Game) -> io::Result<()> {
    let snapshot = save::load_snapshot()?;
    let stage = if snapshot.stage == OBJECTIVES.len() {
        0
    } else {
        snapshot.stage
    };
    if stage > OBJECTIVES.len()
        || !(-8.0..=24.0).contains(&snapshot.speed)
        || collides(
            Vec2::new(snapshot.player.x, snapshot.player.z),
            PLAYER_BODY_RADIUS,
            &game.buildings,
        )
        || collides(
            Vec2::new(snapshot.car.x, snapshot.car.z),
            PLAYER_CAR_RADIUS,
            &game.buildings,
        )
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Save position or progress is outside the supported game state",
        ));
    }
    game.player = Vec3::new(snapshot.player.x, PLAYER_GROUND_Y, snapshot.player.z);
    game.car = snapshot.car;
    game.yaw = snapshot.yaw;
    game.speed = snapshot.speed;
    game.steering = 0.0;
    game.lateral_speed = 0.0;
    game.player_moving = false;
    game.player_vertical_speed = 0.0;
    game.player_grounded = true;
    game.stage = stage;
    game.credits = snapshot.credits;
    game.driving = snapshot.driving;
    game.bloom = snapshot.bloom;
    Ok(())
}

fn controls(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut mouse_motion: MessageReader<MouseMotion>,
    mut cursor_options: Single<&mut CursorOptions>,
    time: Res<Time>,
    traffic: Res<city_life::TrafficBodies>,
    mut probe: ResMut<city_life::PlayerProbe>,
    mut game: ResMut<Game>,
) {
    if !game.started {
        if keys.just_pressed(KeyCode::Enter) || mouse_buttons.just_pressed(MouseButton::Left) {
            game.started = true;
            game.mouse_captured = mouse_buttons.just_pressed(MouseButton::Left);
        }
        cursor_options.visible = !game.mouse_captured;
        cursor_options.grab_mode = if game.mouse_captured {
            CursorGrabMode::Locked
        } else {
            CursorGrabMode::None
        };
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
        let saved = save_game(&game).is_ok();
        game.notice = if saved {
            format!("Сохранение записано: {}", save::SAVE_FILE_NAME)
        } else {
            "Не удалось записать сохранение".to_string()
        };
        game.notice_timer = 3.0;
    }
    if keys.just_pressed(KeyCode::F9) {
        let loaded = load_game(&mut game).is_ok();
        game.notice = if loaded {
            "Сохранение загружено".to_string()
        } else {
            format!("Не удалось загрузить {}", save::SAVE_FILE_NAME)
        };
        game.notice_timer = 3.0;
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
                    ) && !blocked_by_traffic(&traffic.cars, *exit, PLAYER_BODY_RADIUS)
                        && !blocked_by_parked_car(*exit, game.car, PLAYER_BODY_RADIUS)
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
        let speed_factor = (game.speed / 8.0).clamp(-1.0, 1.0);
        game.yaw += game.steering * speed_factor * 1.8 * dt;
        let direction = Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos());
        let right = Vec3::new(game.yaw.cos(), 0.0, -game.yaw.sin());
        game.lateral_speed =
            (game.lateral_speed + game.steering * game.speed.abs() * 0.06 * dt).clamp(-2.0, 2.0);
        game.lateral_speed *= (1.0 - 1.8 * dt).max(0.0);
        let intended = (Vec2::new(direction.x, direction.z) * game.speed
            + Vec2::new(right.x, right.z) * game.lateral_speed)
            * dt;
        let start = Vec2::new(game.car.x, game.car.z);
        let mut blockers = traffic.cars.clone();
        let separated = physics::separate_from_circles(start, PLAYER_CAR_RADIUS, &blockers);
        let moved = physics::move_with_slide(separated, intended, |point| {
            collides(point, PLAYER_CAR_RADIUS, &game.buildings)
                || blockers
                    .iter()
                    .any(|(center, radius)| center.distance(point) < radius + PLAYER_CAR_RADIUS)
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
        let mut blockers = traffic.cars.clone();
        blockers.push((Vec2::new(game.car.x, game.car.z), PLAYER_CAR_RADIUS));
        let start = Vec2::new(game.player.x, game.player.z);
        let separated = physics::separate_from_circles(start, PLAYER_BODY_RADIUS, &blockers);
        let delta = Vec2::new(direction.x, direction.z) * run * dt;
        let moved = physics::move_with_slide(separated, delta, |point| {
            collides(point, PLAYER_BODY_RADIUS, &game.buildings)
                || blockers
                    .iter()
                    .any(|(center, radius)| center.distance(point) < radius + PLAYER_BODY_RADIUS)
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
}

fn update_world(
    game: Res<Game>,
    time: Res<Time>,
    mut players: Query<
        (&mut Transform, &mut Visibility, &mut PlayerVisual),
        (
            With<PlayerVisual>,
            Without<CarVisual>,
            Without<FollowCamera>,
            Without<MissionMarker>,
        ),
    >,
    mut cars: Query<
        &mut Transform,
        (
            With<CarVisual>,
            Without<PlayerVisual>,
            Without<FollowCamera>,
            Without<MissionMarker>,
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
    mut commands: Commands,
) {
    let elapsed_secs = time.elapsed_secs();
    let dt = time.delta_secs().min(0.05);
    for (mut transform, mut visibility, mut visual) in &mut players {
        transform.translation = game.player;
        let moving = game.player_moving && !game.driving;
        let running = moving && game.player_running;
        let step_rate = if running { 12.0 } else { 7.0 };
        let step_phase = elapsed_secs * step_rate;
        if moving {
            let bob_amount = if running { 0.06 } else { 0.035 };
            transform.translation.y += (step_phase * 2.0).sin() * bob_amount;
        }

        let turn_delta = (game.player_facing_yaw - visual.facing_yaw + std::f32::consts::PI)
            .rem_euclid(std::f32::consts::TAU)
            - std::f32::consts::PI;
        visual.facing_yaw += turn_delta * (1.0 - (-14.0 * dt).exp());

        let stride_sway = if moving {
            step_phase.sin() * if running { 0.12 } else { 0.065 }
        } else {
            0.0
        };
        let run_lean = if running { -0.2 } else { 0.0 };
        transform.rotation = Quat::from_rotation_y(visual.facing_yaw)
            * Quat::from_rotation_x(run_lean)
            * Quat::from_rotation_z(stride_sway);
        *visibility = if game.driving {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
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
    mut labels: Query<&mut Text, With<ui::Hud>>,
) {
    if !game.started {
        for mut text in &mut labels {
            text.0 = ui::MAIN_MENU_TEXT.to_string();
        }
        return;
    }
    let position = if game.driving { game.car } else { game.player };
    let objective = if game.stage < OBJECTIVES.len() {
        let target = mission_target(game.stage, game.credits);
        let distance = Vec2::new(position.x - target.x, position.z - target.z).length();
        format!(
            "Маршрут «{}»: точка {}/{} • {:.0} м",
            mission_name(game.credits),
            game.stage + 1,
            OBJECTIVES.len(),
            distance
        )
    } else {
        "Маршрут выполнен! Награда получена".to_string()
    };
    let mode = if game.paused {
        "ПАУЗА — Esc или ЛКМ, чтобы продолжить"
    } else if game.driving {
        "ЗА РУЛЁМ"
    } else {
        "ПЕШКОМ"
    };
    let state = format!("{} • розыск: {}", mode, "★".repeat(wanted.level as usize));
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
        let car = Vec3::new(0.0, 0.65, 6.0);
        assert!(blocked_by_parked_car(
            Vec3::new(0.0, 0.9, 8.0),
            car,
            PLAYER_BODY_RADIUS
        ));
        assert!(!blocked_by_parked_car(
            Vec3::new(0.0, 0.9, 10.0),
            car,
            PLAYER_BODY_RADIUS
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
    fn traffic_cars_block_the_player() {
        let cars = vec![(Vec2::new(4.0, 0.0), 2.2)];
        assert!(blocked_by_traffic(&cars, Vec3::new(5.0, 0.0, 0.0), 0.45));
        assert!(!blocked_by_traffic(&cars, Vec3::new(12.0, 0.0, 0.0), 0.45));
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
    fn steering_changes_smoothly_toward_input() {
        let first = vehicle_steering_step(0.0, 1.0, 0.05);
        let second = vehicle_steering_step(first, 1.0, 0.05);
        assert!(first > 0.0 && first < 1.0);
        assert!(second > first && second < 1.0);
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
}
