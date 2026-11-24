mod city_life;
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

#[derive(Component)]
struct PlayerVisual;

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
            player: Vec3::new(2.0, 0.9, 2.0),
            car: Vec3::new(0.0, 0.65, 6.0),
            yaw: 0.0,
            speed: 0.0,
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
    mut game: ResMut<Game>,
) {
    let cube = meshes.add(Cuboid::default());
    let ground = materials.add(StandardMaterial {
        base_color: Color::srgb(0.035, 0.10, 0.12),
        perceptual_roughness: 0.92,
        ..default()
    });
    let asphalt = materials.add(StandardMaterial {
        base_color: Color::srgb(0.028, 0.045, 0.075),
        perceptual_roughness: 0.96,
        ..default()
    });
    let stripe = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.65, 0.18),
        emissive: LinearRgba::rgb(0.18, 0.09, 0.02),
        perceptual_roughness: 0.72,
        ..default()
    });
    let buildings = [
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.09, 0.17, 0.28),
            metallic: 0.18,
            perceptual_roughness: 0.72,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.27, 0.15, 0.19),
            metallic: 0.16,
            perceptual_roughness: 0.76,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.06, 0.24, 0.28),
            metallic: 0.20,
            perceptual_roughness: 0.68,
            ..default()
        }),
        materials.add(StandardMaterial {
            base_color: Color::srgb(0.20, 0.15, 0.32),
            metallic: 0.18,
            perceptual_roughness: 0.72,
            ..default()
        }),
    ];
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
        PlayerVisual,
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
        Transform::from_translation(OBJECTIVES[0]).with_scale(Vec3::new(1.1, 2.6, 1.1)),
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

fn walk_direction(horizontal: f32, forward: f32, yaw: f32) -> Vec3 {
    let right = Vec3::new(yaw.cos(), 0.0, -yaw.sin());
    let ahead = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    (right * horizontal + ahead * forward).normalize_or_zero()
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
    if snapshot.stage > OBJECTIVES.len()
        || !(-8.0..=24.0).contains(&snapshot.speed)
        || collides(
            Vec2::new(snapshot.player.x, snapshot.player.z),
            0.45,
            &game.buildings,
        )
        || collides(
            Vec2::new(snapshot.car.x, snapshot.car.z),
            2.0,
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
    game.stage = snapshot.stage;
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
        return;
    }
    if keys.just_pressed(KeyCode::KeyE) {
        if game.driving {
            if game.speed.abs() < 2.0 {
                let side = Vec3::new(game.yaw.cos(), 0.0, -game.yaw.sin()) * 3.0;
                let exit = game.car + side;
                if !collides(Vec2::new(exit.x, exit.z), 0.45, &game.buildings) {
                    game.player = Vec3::new(exit.x, 0.9, exit.z);
                    game.camera_yaw = game.yaw + std::f32::consts::PI + game.camera_orbit;
                    game.driving = false;
                    game.speed = 0.0;
                }
            }
        } else if game.player.distance(game.car) < 4.5 {
            game.driving = true;
            game.camera_orbit = 0.0;
        }
    }
    if game.driving {
        let throttle =
            (keys.pressed(KeyCode::KeyW) as i32 - keys.pressed(KeyCode::KeyS) as i32) as f32;
        let steer =
            (keys.pressed(KeyCode::KeyA) as i32 - keys.pressed(KeyCode::KeyD) as i32) as f32;
        game.speed = (game.speed + throttle * 14.0 * dt).clamp(-8.0, 24.0);
        game.speed *= (1.0 - 0.7 * dt).max(0.0);
        let speed_factor = (game.speed / 8.0).clamp(-1.0, 1.0);
        game.yaw += steer * speed_factor * 1.8 * dt;
        let direction = Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos());
        let next = game.car + direction * game.speed * dt;
        if collides(Vec2::new(next.x, next.z), 2.0, &game.buildings) {
            game.speed = 0.0;
        } else {
            game.car = next;
        }
        game.player = game.car;
    } else {
        let horizontal =
            (keys.pressed(KeyCode::KeyD) as i32 - keys.pressed(KeyCode::KeyA) as i32) as f32;
        let forward =
            (keys.pressed(KeyCode::KeyW) as i32 - keys.pressed(KeyCode::KeyS) as i32) as f32;
        let direction = walk_direction(horizontal, forward, game.camera_yaw);
        let run = if keys.pressed(KeyCode::ShiftLeft) {
            8.0
        } else {
            5.0
        };
        let next = game.player + direction * run * dt;
        if !collides(Vec2::new(next.x, next.z), 0.45, &game.buildings) {
            game.player = next;
        }
    }
    if game.stage < OBJECTIVES.len() {
        let position = if game.driving { game.car } else { game.player };
        let target = OBJECTIVES[game.stage];
        if Vec2::new(position.x - target.x, position.z - target.z).length() < 4.0 {
            game.stage += 1;
            game.credits += 100;
        }
    }
}

fn update_world(
    game: Res<Game>,
    time: Res<Time>,
    mut players: Query<
        (&mut Transform, &mut Visibility),
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
        (&mut Transform, &mut Visibility),
        (
            With<MissionMarker>,
            Without<PlayerVisual>,
            Without<CarVisual>,
            Without<FollowCamera>,
        ),
    >,
    mut commands: Commands,
) {
    for (mut transform, mut visibility) in &mut players {
        transform.translation = game.player;
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
    for (mut transform, mut visibility) in &mut markers {
        if game.stage < OBJECTIVES.len() {
            transform.translation = OBJECTIVES[game.stage];
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

fn update_hud(game: Res<Game>, mut labels: Query<&mut Text, With<ui::Hud>>) {
    if !game.started {
        for mut text in &mut labels {
            text.0 = ui::MAIN_MENU_TEXT.to_string();
        }
        return;
    }
    let position = if game.driving { game.car } else { game.player };
    let objective = if game.stage < OBJECTIVES.len() {
        let target = OBJECTIVES[game.stage];
        let distance = Vec2::new(position.x - target.x, position.z - target.z).length();
        format!(
            "Маршрут «Три маяка»: точка {}/{} • {:.0} м",
            game.stage + 1,
            OBJECTIVES.len(),
            distance
        )
    } else {
        "Маршрут выполнен! Награда получена".to_string()
    };
    let state = if game.paused {
        "ПАУЗА — Esc или ЛКМ, чтобы продолжить"
    } else if game.driving {
        "ЗА РУЛЁМ"
    } else {
        "ПЕШКОМ"
    };
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
        "Камера: мышь • Esc: пауза и освободить мышь"
    } else {
        "ЛКМ: захватить мышь и управлять камерой"
    };
    let content = ui::compose_hud(
        &objective,
        game.credits,
        state,
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
}
