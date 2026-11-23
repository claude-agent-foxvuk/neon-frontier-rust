mod city_life;

use bevy::{
    camera::Hdr,
    core_pipeline::tonemapping::Tonemapping,
    post_process::bloom::Bloom,
    prelude::*,
};

const WORLD_EDGE: f32 = 165.0;
const ROAD_SPACING: f32 = 48.0;
const BUILDING_HALF: f32 = 14.0;

#[derive(Component)]
struct PlayerVisual;
#[derive(Component)]
struct CarVisual;
#[derive(Component)]
struct FollowCamera;
#[derive(Component)]
struct Hud;
#[derive(Component)]
struct MissionMarker;

#[derive(Resource)]
struct Game {
    player: Vec3,
    car: Vec3,
    yaw: f32,
    speed: f32,
    driving: bool,
    paused: bool,
    bloom: bool,
    stage: usize,
    credits: u32,
    buildings: Vec<Vec2>,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            player: Vec3::new(2.0, 0.9, 2.0),
            car: Vec3::new(0.0, 0.65, 6.0),
            yaw: 0.0,
            speed: 0.0,
            driving: false,
            paused: false,
            bloom: true,
            stage: 0,
            credits: 0,
            buildings: Vec::new(),
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
    let ground = materials.add(Color::srgb(0.10, 0.14, 0.17));
    let asphalt = materials.add(Color::srgb(0.09, 0.09, 0.12));
    let stripe = materials.add(Color::srgb(0.86, 0.67, 0.24));
    let buildings = [
        materials.add(Color::srgb(0.25, 0.31, 0.38)),
        materials.add(Color::srgb(0.38, 0.31, 0.30)),
        materials.add(Color::srgb(0.20, 0.34, 0.37)),
        materials.add(Color::srgb(0.34, 0.30, 0.41)),
    ];
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
                Transform::from_xyz(px, height / 2.0, pz)
                    .with_scale(Vec3::new(BUILDING_HALF * 2.0, height, BUILDING_HALF * 2.0)),
            ));
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
        Mesh3d(cube),
        MeshMaterial3d(marker_mat),
        Transform::from_translation(OBJECTIVES[0]).with_scale(Vec3::new(1.1, 2.6, 1.1)),
        MissionMarker,
    ));
    commands.spawn((
        DirectionalLight {
            illuminance: 16_000.0,
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
        Transform::from_xyz(0.0, 10.0, 18.0).looking_at(game.player, Vec3::Y),
        FollowCamera,
    ));
    commands.spawn((
        Text::new("NEON FRONTIER"),
        Node {
            position_type: PositionType::Absolute,
            top: px(14),
            left: px(14),
            ..default()
        },
        Hud,
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

fn controls(keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut game: ResMut<Game>) {
    if keys.just_pressed(KeyCode::Escape) {
        game.paused = !game.paused;
    }
    if keys.just_pressed(KeyCode::KeyB) {
        game.bloom = !game.bloom;
    }
    if game.paused {
        return;
    }

    let dt = time.delta_secs().min(0.05);
    if keys.just_pressed(KeyCode::KeyE) {
        if game.driving {
            if game.speed.abs() < 2.0 {
                let side = Vec3::new(game.yaw.cos(), 0.0, -game.yaw.sin()) * 3.0;
                let exit = game.car + side;
                if !collides(Vec2::new(exit.x, exit.z), 0.45, &game.buildings) {
                    game.player = Vec3::new(exit.x, 0.9, exit.z);
                    game.driving = false;
                    game.speed = 0.0;
                }
            }
        } else if game.player.distance(game.car) < 4.5 {
            game.driving = true;
        }
    }

    if game.driving {
        let throttle = (keys.pressed(KeyCode::KeyW) as i32
            - keys.pressed(KeyCode::KeyS) as i32) as f32;
        let steer = (keys.pressed(KeyCode::KeyA) as i32
            - keys.pressed(KeyCode::KeyD) as i32) as f32;
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
        let x = (keys.pressed(KeyCode::KeyD) as i32
            - keys.pressed(KeyCode::KeyA) as i32) as f32;
        let z = (keys.pressed(KeyCode::KeyS) as i32
            - keys.pressed(KeyCode::KeyW) as i32) as f32;
        let direction = Vec3::new(x, 0.0, z).normalize_or_zero();
        let run = if keys.pressed(KeyCode::ShiftLeft) { 8.0 } else { 5.0 };
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
        (With<PlayerVisual>, Without<CarVisual>, Without<FollowCamera>, Without<MissionMarker>),
    >,
    mut cars: Query<
        &mut Transform,
        (With<CarVisual>, Without<PlayerVisual>, Without<FollowCamera>, Without<MissionMarker>),
    >,
    mut cameras: Query<
        (Entity, &mut Transform, Option<&Bloom>),
        (With<FollowCamera>, Without<PlayerVisual>, Without<CarVisual>, Without<MissionMarker>),
    >,
    mut markers: Query<
        (&mut Transform, &mut Visibility),
        (With<MissionMarker>, Without<PlayerVisual>, Without<CarVisual>, Without<FollowCamera>),
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
    let desired = if game.driving {
        let forward = Vec3::new(game.yaw.sin(), 0.0, game.yaw.cos());
        target - forward * 14.0 + Vec3::Y * 7.0
    } else {
        target + Vec3::new(0.0, 7.0, 12.0)
    };
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

fn update_hud(game: Res<Game>, mut labels: Query<&mut Text, With<Hud>>) {
    let objective = if game.stage < OBJECTIVES.len() {
        format!("Маршрут «Три маяка»: точка {}/{}", game.stage + 1, OBJECTIVES.len())
    } else {
        "Маршрут выполнен! Награда получена".to_string()
    };
    let state = if game.paused {
        "ПАУЗА"
    } else if game.driving {
        "ЗА РУЛЁМ"
    } else {
        "ПЕШКОМ"
    };
    for mut text in &mut labels {
        text.0 = format!(
            "NEON FRONTIER • ранняя версия\n{}\nКредиты: {}\n{}\nWASD: движение/газ • Shift: бег • E: сесть/выйти\nEsc: пауза • B: bloom",
            objective, game.credits, state
        );
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
}
