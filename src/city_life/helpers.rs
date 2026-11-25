use super::*;

pub(super) fn resolve_knock(displacement: &mut Vec3, velocity: &mut Vec3, knocked: &mut f32, dt: f32) {
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

pub(super) fn damage_citizen(citizen: &mut Citizen, damage: f32) -> bool {
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

pub(super) fn city_collision(position: Vec2, radius: f32) -> bool {
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

pub(super) fn vehicle_city_collision(position: Vec2, yaw: f32) -> bool {
    if !position.is_finite() || !yaw.is_finite() {
        return true;
    }
    let (sin_yaw, cos_yaw) = yaw.sin_cos();
    let extent_x = crate::physics::VEHICLE_HALF_LENGTH * cos_yaw.abs()
        + crate::physics::VEHICLE_HALF_WIDTH * sin_yaw.abs();
    let extent_z = crate::physics::VEHICLE_HALF_LENGTH * sin_yaw.abs()
        + crate::physics::VEHICLE_HALF_WIDTH * cos_yaw.abs();
    if position.x.abs() + extent_x > CITY_LIMIT || position.y.abs() + extent_z > CITY_LIMIT {
        return true;
    }

    let vehicle = Body::vehicle(position, yaw);
    (-3_i32..=2).any(|grid_x| {
        (-3_i32..=2).any(|grid_z| {
            let center = Vec2::new(
                grid_x as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
                grid_z as f32 * ROAD_SPACING + ROAD_SPACING * 0.5,
            );
            let building = Body::new(center, BUILDING_HALF, BUILDING_HALF, 0.0);
            crate::physics::bodies_overlap(&vehicle, &building)
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

pub(super) fn police_line_of_sight_clear(start: Vec2, end: Vec2) -> bool {
    let distance = start.distance(end);
    let sample_count = (distance / LINE_OF_SIGHT_SAMPLE_SPACING).ceil().max(1.0) as usize;
    (0..=sample_count).all(|sample| {
        let point = start.lerp(end, sample as f32 / sample_count as f32);
        !building_contains(point)
    })
}

pub(super) fn can_police_shoot(distance: f32, line_of_sight_clear: bool) -> bool {
    line_of_sight_clear && (POLICE_SHOT_MIN_DISTANCE..=POLICE_SHOT_MAX_DISTANCE).contains(&distance)
}

pub(super) fn damage_player(probe: &mut PlayerProbe, damage: f32) {
    if damage > 0.0 {
        probe.health = (probe.health - damage).max(0.0);
    }
}

pub(super) fn traffic_obstacle_ahead(
    baseline: Vec2,
    forward: Vec2,
    player: Vec2,
    vehicle: Option<Vec2>,
    in_vehicle: bool,
    car_positions: &[Vec3],
    police_positions: &[Vec2],
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
        || police_positions
            .iter()
            .any(|position| blocks_path(*position))
        || car_positions.iter().enumerate().any(|(other, position)| {
            other != car_index && blocks_path(Vec2::new(position.x, position.z))
        })
}

pub(super) fn traffic_obstacle_ahead_with_bodies(
    baseline: Vec2,
    forward: Vec2,
    player: Vec2,
    vehicle: Option<Vec2>,
    in_vehicle: bool,
    traffic_bodies: &[crate::physics::Body],
    police_positions: &[Vec2],
    car_index: usize,
) -> bool {
    let detection_yaw = forward.y.atan2(forward.x);
    let braking_probe = crate::physics::Body::new(
        baseline + forward * (CAR_BRAKE_DISTANCE * 0.5),
        CAR_BRAKE_DISTANCE * 0.5 + crate::physics::VEHICLE_HALF_LENGTH,
        crate::physics::VEHICLE_HALF_WIDTH,
        detection_yaw,
    );
    let player_body = if in_vehicle {
        crate::physics::Body::vehicle(player, detection_yaw)
    } else {
        crate::physics::Body::person(player)
    };
    crate::physics::bodies_overlap(&braking_probe, &player_body)
        || (!in_vehicle
            && vehicle.map_or(false, |position| {
                let parked_vehicle = crate::physics::Body::vehicle(position, detection_yaw);
                crate::physics::bodies_overlap(&braking_probe, &parked_vehicle)
            }))
        || police_positions.iter().any(|position| {
            let patrol = crate::physics::Body::vehicle(*position, detection_yaw);
            crate::physics::bodies_overlap(&braking_probe, &patrol)
        })
        || traffic_bodies.iter().enumerate().any(|(other, body)| {
            other != car_index && crate::physics::bodies_overlap(&braking_probe, body)
        })
}

pub(super) fn choose_safe_traffic_offset(
    current: Vec2,
    baseline: Vec2,
    side: Vec2,
    road_center: f32,
    road_half_width: f32,
    vehicle_yaw: f32,
    mut blocked: impl FnMut(Vec2) -> bool,
) -> Option<f32> {
    for direction in [1.0, -1.0] {
        let offset = direction * CAR_PASSING_OFFSET;
        let target = baseline + side * offset;
        if (target.dot(side) - road_center).abs() + CAR_BODY_RADIUS > road_half_width {
            continue;
        }
        let candidate = crate::physics::move_with_slide(current, target - current, |point| {
            vehicle_city_collision(point, vehicle_yaw) || blocked(point)
        });
        if candidate.distance(target) < 0.05 {
            return Some(offset);
        }
    }
    None
}

#[cfg(test)]
mod oriented_traffic_tests {
    use super::*;

    fn has_car_ahead(body: crate::physics::Body) -> bool {
        traffic_obstacle_ahead_with_bodies(
            Vec2::ZERO,
            Vec2::X,
            Vec2::new(200.0, 200.0),
            None,
            false,
            &[body],
            &[],
            0,
        )
    }

    fn has_obstacle_at(
        position: Vec2,
        in_vehicle: bool,
        parked_vehicle: Option<Vec2>,
        police: &[Vec2],
    ) -> bool {
        traffic_obstacle_ahead_with_bodies(
            Vec2::ZERO,
            Vec2::X,
            position,
            parked_vehicle,
            in_vehicle,
            &[],
            police,
            0,
        )
    }

    #[test]
    fn traffic_braking_sensor_respects_vehicle_orientation_and_clear_adjacent_lanes() {
        let same_lane = crate::physics::Body::vehicle(Vec2::new(4.0, 0.0), 0.0);
        assert!(has_car_ahead(same_lane));

        let clear_adjacent_lane = crate::physics::Body::vehicle(Vec2::new(4.0, 2.0), 0.0);
        assert!(!has_car_ahead(clear_adjacent_lane));

        let crossing_lane = crate::physics::Body::vehicle(
            Vec2::new(4.0, 1.8),
            std::f32::consts::FRAC_PI_2,
        );
        assert!(has_car_ahead(crossing_lane));
    }

    #[test]
    fn traffic_braking_sensor_uses_body_shapes_for_player_parked_car_and_patrol() {
        let adjacent_lane = Vec2::new(4.0, 2.0);
        let same_lane = Vec2::new(4.0, 0.0);
        let far_away = Vec2::splat(200.0);

        assert!(!has_obstacle_at(adjacent_lane, false, None, &[]));
        assert!(has_obstacle_at(same_lane, false, None, &[]));
        assert!(!has_obstacle_at(far_away, false, Some(adjacent_lane), &[]));
        assert!(has_obstacle_at(far_away, false, Some(same_lane), &[]));
        assert!(!has_obstacle_at(far_away, false, None, &[adjacent_lane]));
        assert!(has_obstacle_at(far_away, false, None, &[same_lane]));
        assert!(!has_obstacle_at(adjacent_lane, true, None, &[]));
        assert!(has_obstacle_at(same_lane, true, None, &[]));
    }

    #[test]
    fn vehicle_building_collision_uses_the_oriented_footprint() {
        let near_building_wall = Vec2::new(7.9, 24.0);
        assert!(city_collision(near_building_wall, CAR_BODY_RADIUS));
        assert!(vehicle_city_collision(near_building_wall, 0.0));
        assert!(!vehicle_city_collision(
            near_building_wall,
            std::f32::consts::FRAC_PI_2,
        ));
    }

    #[test]
    fn vehicle_city_collision_rejects_invalid_pose_and_world_edge_overlap() {
        assert!(vehicle_city_collision(Vec2::new(f32::NAN, 0.0), 0.0));
        assert!(vehicle_city_collision(Vec2::new(CITY_LIMIT - 1.0, 0.0), 0.0));
        assert!(!vehicle_city_collision(Vec2::ZERO, 0.0));
    }

    #[test]
    fn offset_planning_rejects_an_oriented_car_body_inside_a_building() {
        let current = Vec2::new(8.5, 5.0);
        let side = Vec2::Y;
        let yaw = 0.0;
        let selected = choose_safe_traffic_offset(
            current,
            current,
            side,
            0.0,
            10.0,
            yaw,
            |_| false,
        );
        assert!(selected.is_some());

        let near_corner = Vec2::new(8.5, 10.5);
        assert!(vehicle_city_collision(near_corner, yaw));
    }
}
