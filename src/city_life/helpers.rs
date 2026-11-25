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
    let side = Vec2::new(-forward.y, forward.x);
    let blocks_path = |position: Vec2| {
        let relative = position - baseline;
        let ahead = relative.dot(forward);
        ahead > 0.0 && ahead < CAR_BRAKE_DISTANCE && relative.dot(side).abs() < CAR_LANE_GAP
    };
    let detection_yaw = forward.y.atan2(forward.x);
    let braking_probe = crate::physics::Body::new(
        baseline + forward * (CAR_BRAKE_DISTANCE * 0.5),
        CAR_BRAKE_DISTANCE * 0.5 + crate::physics::VEHICLE_HALF_LENGTH,
        crate::physics::VEHICLE_HALF_WIDTH,
        detection_yaw,
    );
    blocks_path(player)
        || (!in_vehicle && vehicle.map_or(false, |position| blocks_path(position)))
        || police_positions
            .iter()
            .any(|position| blocks_path(*position))
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
}
