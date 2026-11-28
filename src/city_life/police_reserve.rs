use bevy::prelude::*;

use crate::physics::{bodies_overlap, Body};

use super::{PlayerProbe, TrafficBodies, WantedState};

const ROAD_SPACING: f32 = 48.0;
const ROAD_HALF_WIDTH: f32 = 5.5;
const ROAD_LANE_OFFSET: f32 = 2.1;
const CITY_LIMIT: f32 = 160.0;
const BUILDING_HALF: f32 = 14.0;
const RESPONSE_LEVEL: u8 = 4;
const RESPONSE_TRIGGER_DISTANCE: f32 = 80.0;
const RESPONSE_MIN_SPAWN_DISTANCE: f32 = 26.0;
const RESPONSE_MAX_SPAWN_DISTANCE: f32 = 56.0;
const RESPONSE_STOP_DISTANCE: f32 = 12.0;
const RESPONSE_SPEED: f32 = 16.0;

#[derive(Component)]
pub(super) struct ReserveCruiser {
    pub(super) index: usize,
}

fn reserve_vehicle_count(wanted_level: u8) -> usize {
    wanted_level.saturating_sub(2).min(4) as usize
}

fn reserve_vehicle_body(position: Vec2, yaw: f32) -> Body {
    Body::vehicle(position, yaw)
}

fn reserve_vehicle_city_collision(position: Vec2, yaw: f32) -> bool {
    if !position.is_finite() || !yaw.is_finite() {
        return true;
    }
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let extent_x = crate::physics::VEHICLE_HALF_LENGTH * cos_yaw.abs()
        + crate::physics::VEHICLE_HALF_WIDTH * sin_yaw.abs();
    let extent_y = crate::physics::VEHICLE_HALF_LENGTH * sin_yaw.abs()
        + crate::physics::VEHICLE_HALF_WIDTH * cos_yaw.abs();
    if position.x.abs() + extent_x > CITY_LIMIT || position.y.abs() + extent_y > CITY_LIMIT {
        return true;
    }

    let vehicle = reserve_vehicle_body(position, yaw);
    let hits_building = (-3_i32..=2).any(|grid_x| {
        (-3_i32..=2).any(|grid_z| {
            let center = Vec2::new(
                grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
                grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
            );
            bodies_overlap(
                &vehicle,
                &Body::new(center, BUILDING_HALF, BUILDING_HALF, 0.0),
            )
        })
    });
    hits_building
        || super::city_neon::central_park_collision_bodies()
            .iter()
            .any(|obstacle| bodies_overlap(&vehicle, obstacle))
}

fn reserve_response_spawn(player: Vec2, occupied: &[Body]) -> Option<(Vec2, f32)> {
    if !player.is_finite() {
        return None;
    }
    let approach_offsets = [-44.0_f32, -36.0, -28.0, 28.0, 36.0, 44.0];
    let lane_offsets = [-ROAD_LANE_OFFSET, ROAD_LANE_OFFSET];
    let mut best: Option<(f32, Vec2, f32)> = None;

    for road_index in -3_i32..=3 {
        let road = road_index as f32 * ROAD_SPACING;
        for lane in lane_offsets {
            for offset in approach_offsets {
                let candidates = [
                    (Vec2::new(player.x + offset, road + lane), true),
                    (Vec2::new(road + lane, player.y + offset), false),
                ];
                for (position, along_x) in candidates {
                    let distance = position.distance(player);
                    if !(RESPONSE_MIN_SPAWN_DISTANCE..=RESPONSE_MAX_SPAWN_DISTANCE)
                        .contains(&distance)
                    {
                        continue;
                    }
                    let to_player = player - position;
                    let yaw = to_player.y.atan2(to_player.x);
                    let body = reserve_vehicle_body(position, yaw);
                    let on_road = (along_x && (position.y - road).abs() <= ROAD_HALF_WIDTH)
                        || (!along_x && (position.x - road).abs() <= ROAD_HALF_WIDTH);
                    if !on_road
                        || reserve_vehicle_city_collision(position, yaw)
                        || occupied.iter().any(|other| bodies_overlap(&body, other))
                    {
                        continue;
                    }
                    if best.is_none_or(|(best_distance, _, _)| distance < best_distance) {
                        best = Some((distance, position, yaw));
                    }
                }
            }
        }
    }
    best.map(|(_, position, yaw)| (position, yaw))
}

fn reserve_vehicle_center_stays_on_road(position: Vec2) -> bool {
    if !position.is_finite() {
        return false;
    }
    let center_limit = ROAD_HALF_WIDTH - crate::physics::VEHICLE_HALF_WIDTH;
    let nearest_x_road = (position.x / ROAD_SPACING).round() * ROAD_SPACING;
    let nearest_y_road = (position.y / ROAD_SPACING).round() * ROAD_SPACING;
    (position.x - nearest_x_road).abs() <= center_limit
        || (position.y - nearest_y_road).abs() <= center_limit
}

fn reserve_response_step(
    position: Vec2,
    player: Vec2,
    dt: f32,
    occupied: &[Body],
    player_body: Option<&Body>,
) -> (Vec2, f32) {
    let to_player = player - position;
    let distance = to_player.length();
    if !distance.is_finite() || distance <= RESPONSE_STOP_DISTANCE || dt <= 0.0 {
        return (position, 0.0);
    }

    let desired = to_player / distance;
    let step = (RESPONSE_SPEED * dt).min(distance - RESPONSE_STOP_DISTANCE);
    let directions = [desired, Vec2::X, Vec2::NEG_X, Vec2::Y, Vec2::NEG_Y];
    let mut best = position;
    let mut best_distance = distance;
    let mut best_yaw = desired.y.atan2(desired.x);

    for direction in directions {
        let yaw = direction.y.atan2(direction.x);
        let candidate = crate::physics::move_with_slide(position, direction * step, |point| {
            !reserve_vehicle_center_stays_on_road(point)
                || reserve_vehicle_city_collision(point, yaw)
                || occupied
                    .iter()
                    .any(|other| bodies_overlap(&reserve_vehicle_body(point, yaw), other))
                || player_body.is_some_and(|body| {
                    bodies_overlap(&reserve_vehicle_body(point, yaw), body)
                })
        });
        let candidate_distance = candidate.distance(player);
        if candidate_distance + 1.0e-4 < best_distance {
            best = candidate;
            best_distance = candidate_distance;
            best_yaw = yaw;
        }
    }

    (best, best_yaw)
}

pub(super) fn spawn_reserve_cruisers(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let body_mesh = meshes.add(Cuboid::new(1.9, 0.85, 3.6));
    let cabin_mesh = meshes.add(Cuboid::new(1.45, 0.52, 1.8));
    let lightbar_mesh = meshes.add(Cuboid::new(0.95, 0.12, 0.24));
    let lamp_mesh = meshes.add(Cuboid::new(0.30, 0.14, 0.22));
    let body_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.025, 0.07, 0.15),
        metallic: 0.62,
        perceptual_roughness: 0.28,
        ..default()
    });
    let glass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.055, 0.12, 0.18),
        metallic: 0.7,
        perceptual_roughness: 0.12,
        ..default()
    });
    let bar_material = materials.add(Color::srgb(0.84, 0.88, 0.92));
    let red_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.92, 0.08, 0.06),
        emissive: LinearRgba::rgb(4.0, 0.12, 0.05),
        ..default()
    });
    let blue_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.06, 0.22, 0.95),
        emissive: LinearRgba::rgb(0.06, 0.36, 4.2),
        ..default()
    });
    let posts = [
        (Vec3::new(-96.0, 0.48, 8.8), 0.0),
        (Vec3::new(96.0, 0.48, -8.8), 0.0),
        (Vec3::new(8.8, 0.48, 96.0), std::f32::consts::FRAC_PI_2),
        (Vec3::new(-8.8, 0.48, -96.0), std::f32::consts::FRAC_PI_2),
    ];

    for (index, (position, yaw)) in posts.into_iter().enumerate() {
        let cruiser = commands
            .spawn((
                Transform::from_translation(position).with_rotation(Quat::from_rotation_y(yaw)),
                Visibility::Hidden,
                ReserveCruiser { index },
            ))
            .id();
        commands.entity(cruiser).with_children(|parent| {
            parent.spawn((
                Mesh3d(body_mesh.clone()),
                MeshMaterial3d(body_material.clone()),
                Transform::default(),
            ));
            parent.spawn((
                Mesh3d(cabin_mesh.clone()),
                MeshMaterial3d(glass_material.clone()),
                Transform::from_xyz(0.0, 0.56, 0.05),
            ));
            parent.spawn((
                Mesh3d(lightbar_mesh.clone()),
                MeshMaterial3d(bar_material.clone()),
                Transform::from_xyz(0.0, 0.86, -0.05),
            ));
            parent.spawn((
                Mesh3d(lamp_mesh.clone()),
                MeshMaterial3d(red_material.clone()),
                Transform::from_xyz(-0.29, 0.90, -0.05),
            ));
            parent.spawn((
                Mesh3d(lamp_mesh.clone()),
                MeshMaterial3d(blue_material.clone()),
                Transform::from_xyz(0.29, 0.90, -0.05),
            ));
        });
    }
}

pub(super) fn update_reserve_cruiser_visibility(
    game: Res<crate::Game>,
    wanted: Res<WantedState>,
    probe: Option<Res<PlayerProbe>>,
    time: Option<Res<Time>>,
    mut traffic_bodies: ResMut<TrafficBodies>,
    mut cruisers: Query<(&ReserveCruiser, &mut Transform, &mut Visibility)>,
) {
    let visible_count = if game.started {
        reserve_vehicle_count(wanted.level)
    } else {
        0
    };
    let response_active = game.started && wanted.level >= RESPONSE_LEVEL;
    let player_position = probe
        .as_ref()
        .map(|probe| Vec2::new(probe.position.x, probe.position.z));
    let dt = time.map_or(0.0, |time| time.delta_secs().clamp(0.0, 0.05));

    let current_cruiser_bodies: Vec<_> = cruisers
        .iter()
        .map(|(_, transform, _)| {
            let yaw = (transform.rotation * Vec3::X).z.atan2((transform.rotation * Vec3::X).x);
            reserve_vehicle_body(Vec2::new(transform.translation.x, transform.translation.z), yaw)
        })
        .collect();
    let mut occupied = traffic_bodies.oriented.clone();
    occupied.retain(|body| !current_cruiser_bodies.contains(body));
    let mut active_bodies = Vec::with_capacity(visible_count);
    let player_body = probe.as_ref().and_then(|probe| {
        let player = Vec2::new(probe.position.x, probe.position.z);
        let forward = Vec2::new(probe.forward.x, probe.forward.z).normalize_or_zero();
        let yaw = forward.y.atan2(forward.x);
        Some(if probe.in_vehicle {
            Body::vehicle(player, yaw)
        } else {
            Body::person(player)
        })
    });

    for (cruiser, mut transform, mut visibility) in &mut cruisers {
        let active = cruiser.index < visible_count;
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if !active {
            continue;
        }

        let mut position = Vec2::new(transform.translation.x, transform.translation.z);
        let mut yaw = (transform.rotation * Vec3::X).z.atan2((transform.rotation * Vec3::X).x);
        if response_active {
            if let Some(player) = player_position {
                let distance = position.distance(player);
                if distance > RESPONSE_TRIGGER_DISTANCE {
                    if let Some((spawn, spawn_yaw)) = reserve_response_spawn(player, &occupied) {
                        position = spawn;
                        yaw = spawn_yaw;
                        transform.translation.x = spawn.x;
                        transform.translation.z = spawn.y;
                    }
                }
                let (next, next_yaw) = reserve_response_step(
                    position,
                    player,
                    dt,
                    &occupied,
                    player_body.as_ref(),
                );
                position = next;
                yaw = next_yaw;
                transform.translation.x = next.x;
                transform.translation.z = next.y;
                transform.rotation = Quat::from_rotation_y(yaw);
            }
        }

        let body = reserve_vehicle_body(position, yaw);
        occupied.push(body);
        active_bodies.push(body);
    }
    traffic_bodies.oriented.extend(active_bodies);
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn extra_cruisers_and_their_collision_bodies_scale_from_three_to_six_stars() {
        for wanted_level in 0_u8..=6 {
            let mut world = World::new();
            let mut game = crate::Game::default();
            game.started = true;
            world.insert_resource(game);
            let mut wanted = WantedState::default();
            wanted.level = wanted_level;
            world.insert_resource(wanted);
            world.insert_resource(TrafficBodies::default());
            for index in 0..4 {
                world.spawn((
                    ReserveCruiser { index },
                    Transform::from_xyz(index as f32 * 10.0, 0.48, 0.0),
                    Visibility::Hidden,
                ));
            }

            world
                .run_system_once(update_reserve_cruiser_visibility)
                .expect("reserve cruisers follow the wanted-level response");

            let mut query = world.query_filtered::<&Visibility, With<ReserveCruiser>>();
            let visible = query
                .iter(&world)
                .filter(|visibility| matches!(**visibility, Visibility::Visible))
                .count();
            assert_eq!(visible, reserve_vehicle_count(wanted_level));
            assert_eq!(
                world.resource::<TrafficBodies>().oriented.len(),
                reserve_vehicle_count(wanted_level)
            );
        }
    }

    #[test]
    fn high_wanted_response_cruisers_spawn_on_clear_roads_and_approach_the_player() {
        let mut world = World::new();
        let mut game = crate::Game::default();
        game.started = true;
        world.insert_resource(game);
        let mut wanted = WantedState::default();
        wanted.level = 4;
        world.insert_resource(wanted);
        let mut time = Time::<()>::default();
        time.advance_by(std::time::Duration::from_millis(50));
        world.insert_resource(time);
        world.insert_resource(PlayerProbe {
            position: Vec3::new(100.0, 0.9, 100.0),
            vehicle_position: Vec3::new(120.0, 0.65, 120.0),
            forward: Vec3::NEG_Z,
            in_vehicle: false,
            punch: false,
            health: 100.0,
        });
        world.insert_resource(TrafficBodies::default());
        for (index, position) in [Vec3::new(-96.0, 0.48, 8.8), Vec3::new(96.0, 0.48, -8.8)]
            .into_iter()
            .enumerate()
        {
            world.spawn((
                ReserveCruiser { index },
                Transform::from_translation(position),
                Visibility::Hidden,
            ));
        }

        world
            .run_system_once(update_reserve_cruiser_visibility)
            .expect("high wanted cruisers find nearby clear roads and pursue");

        let mut query = world.query_filtered::<(&Transform, &Visibility), With<ReserveCruiser>>();
        let active: Vec<_> = query
            .iter(&world)
            .filter(|(_, visibility)| matches!(**visibility, Visibility::Visible))
            .map(|(transform, _)| {
                let position = Vec2::new(transform.translation.x, transform.translation.z);
                let forward = transform.rotation * Vec3::X;
                let yaw = forward.z.atan2(forward.x);
                (position, reserve_vehicle_body(position, yaw))
            })
            .collect();
        let player = Vec2::new(100.0, 100.0);
        assert_eq!(active.len(), 2);
        for (position, body) in &active {
            assert!(position.distance(player) < RESPONSE_MAX_SPAWN_DISTANCE);
            assert!(position.distance(player) > RESPONSE_STOP_DISTANCE);
            assert!(!reserve_vehicle_city_collision(*position, body.yaw));
        }
        assert!(!bodies_overlap(&active[0].1, &active[1].1));
    }

    #[test]
    fn level_three_keeps_its_reserve_cruiser_at_the_city_post() {
        let mut world = World::new();
        let mut game = crate::Game::default();
        game.started = true;
        world.insert_resource(game);
        let mut wanted = WantedState::default();
        wanted.level = 3;
        world.insert_resource(wanted);
        world.insert_resource(TrafficBodies::default());
        let post = Vec3::new(-96.0, 0.48, 8.8);
        let cruiser = world
            .spawn((
                ReserveCruiser { index: 0 },
                Transform::from_translation(post),
                Visibility::Hidden,
            ))
            .id();
        world.insert_resource(PlayerProbe {
            position: Vec3::new(100.0, 0.9, 100.0),
            vehicle_position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            in_vehicle: false,
            punch: false,
            health: 100.0,
        });

        world
            .run_system_once(update_reserve_cruiser_visibility)
            .expect("the high-wanted response threshold does not affect level three");
        assert_eq!(world.get::<Transform>(cruiser).unwrap().translation, post);
    }
}
