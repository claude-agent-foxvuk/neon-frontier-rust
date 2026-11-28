use bevy::prelude::*;

use super::{
    traffic_rotation, vehicle_city_collision, CAR_BRAKE_DISTANCE, CAR_BODY_RADIUS, CAR_PASSING_OFFSET,
    CITY_LIMIT, TrafficCar,
};

/// Select a passing lane only when the whole vehicle and its swept path fit.
pub(super) fn choose_real_width_traffic_offset(
    current: Vec2,
    baseline: Vec2,
    side: Vec2,
    road_center: f32,
    road_half_width: f32,
    mut blocked: impl FnMut(Vec2) -> bool,
) -> Option<f32> {
    let forward = Vec2::new(side.y, -side.x);
    let vehicle_yaw = forward.y.atan2(forward.x);
    let current_offset = (current - baseline).dot(side);
    let preferred_direction = if current_offset.abs() > 0.1 {
        current_offset.signum()
    } else {
        1.0
    };
    let centerline_limit = road_half_width - crate::physics::VEHICLE_HALF_WIDTH;
    if !centerline_limit.is_finite() || centerline_limit <= 0.0 {
        return None;
    }
    let lane_min = road_center - centerline_limit;
    let lane_max = road_center + centerline_limit;
    let baseline_coordinate = baseline.dot(side);

    for direction in [preferred_direction, -preferred_direction] {
        let requested_coordinate = baseline_coordinate + direction * CAR_PASSING_OFFSET;
        let target_coordinate = requested_coordinate.clamp(lane_min, lane_max);
        let offset = target_coordinate - baseline_coordinate;
        if offset * direction <= 0.1 {
            continue;
        }
        // The simulation waits for CAR_PASSING_OFFSET - 0.1 before moving forward.
        // If road-edge clamping yields a smaller shift, it can wait forever, so try
        // the opposite side instead of accepting a lane that cannot clear the obstacle.
        if offset.abs() < CAR_PASSING_OFFSET - 0.1 {
            continue;
        }
        let target = baseline + side * offset;

        // Keep a full braking-distance corridor clear after merging, not only the car's nose.
        let candidate = crate::physics::move_with_slide(current, target - current, |point| {
            vehicle_city_collision(point, vehicle_yaw) || blocked(point)
        });
        if candidate.distance(target) >= 0.05 {
            continue;
        }

        let forward_target = target + forward * CAR_BRAKE_DISTANCE;
        let forward_candidate = crate::physics::move_with_slide(
            candidate,
            forward_target - candidate,
            |point| vehicle_city_collision(point, vehicle_yaw) || blocked(point),
        );
        if forward_candidate.distance(forward_target) < 0.05 {
            return Some(offset);
        }
    }
    None
}

fn traffic_is_outbound_at_world_edge(position: Vec2, along_x: bool, direction: f32) -> bool {
    if !position.is_finite() || !direction.is_finite() || direction == 0.0 {
        return false;
    }
    let coordinate = if along_x { position.x } else { position.y };
    let edge = CITY_LIMIT - CAR_BODY_RADIUS;
    (coordinate >= edge && direction > 0.0) || (coordinate <= -edge && direction < 0.0)
}

pub(super) fn turn_traffic_at_world_edge(
    game: Res<crate::Game>,
    mut traffic: Query<(&mut Transform, &mut TrafficCar)>,
) {
    if !game.started {
        return;
    }
    for (mut transform, mut car) in &mut traffic {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        if traffic_is_outbound_at_world_edge(position, car.along_x, car.direction) {
            car.direction = -car.direction;
            transform.rotation = traffic_rotation(car.along_x, car.direction);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_turns_inward_at_all_four_world_edges() {
        let edge = CITY_LIMIT - CAR_BODY_RADIUS;
        assert!(traffic_is_outbound_at_world_edge(
            Vec2::new(edge, 0.0),
            true,
            1.0,
        ));
        assert!(traffic_is_outbound_at_world_edge(
            Vec2::new(-edge, 0.0),
            true,
            -1.0,
        ));
        assert!(traffic_is_outbound_at_world_edge(
            Vec2::new(0.0, edge),
            false,
            1.0,
        ));
        assert!(traffic_is_outbound_at_world_edge(
            Vec2::new(0.0, -edge),
            false,
            -1.0,
        ));
        assert!(!traffic_is_outbound_at_world_edge(
            Vec2::new(edge, 0.0),
            true,
            -1.0,
        ));
        assert!(!traffic_is_outbound_at_world_edge(
            Vec2::new(edge - 1.0, 0.0),
            true,
            1.0,
        ));
    }

    #[test]
    fn invalid_traffic_direction_does_not_trigger_an_edge_turn() {
        assert!(!traffic_is_outbound_at_world_edge(
            Vec2::new(CITY_LIMIT, 0.0),
            true,
            0.0,
        ));
        assert!(!traffic_is_outbound_at_world_edge(
            Vec2::new(f32::NAN, 0.0),
            true,
            1.0,
        ));
    }

    #[test]
    fn passing_uses_vehicle_width_not_longitudinal_radius() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |_| {
            false
        });
        assert!(offset.is_some());
        let target = current + Vec2::Y * offset.unwrap();
        assert!(target.y.abs() + crate::physics::VEHICLE_HALF_WIDTH <= 5.5 + 1.0e-5);
    }

    #[test]
    fn passing_uses_remaining_road_width_when_the_preferred_offset_is_too_large() {
        let current = Vec2::ZERO;
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |_| {
            false
        });
        let offset = offset.expect("a vehicle-width-safe passing lane remains");
        let target = current + Vec2::Y * offset;

        assert!(offset.abs() < CAR_PASSING_OFFSET);
        assert!(target.y.abs() + crate::physics::VEHICLE_HALF_WIDTH <= 5.5 + 1.0e-5);
    }

    #[test]
    fn passing_rejects_a_clamped_offset_that_cannot_reach_clearance() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |_| {
            false
        })
        .expect("the opposite side permits the full clearance offset");
        assert!(offset.abs() >= CAR_PASSING_OFFSET - 0.1);
        let target = current + Vec2::Y * offset;
        assert!(target.y.abs() + crate::physics::VEHICLE_HALF_WIDTH <= 5.5 + 1.0e-5);
    }

    #[test]
    fn passing_does_not_cross_an_obstacle_on_the_way_to_the_other_lane() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.y.abs() > 1.0
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_rejects_a_lane_blocked_just_after_the_merge() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.x > 1.0 && p.y.abs() > 1.0
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_rejects_an_obstacle_further_down_the_braking_corridor() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.x > 4.0 && p.y.abs() > 1.0
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_prefers_the_currently_occupied_open_lane() {
        let current = Vec2::new(0.0, -CAR_PASSING_OFFSET);
        let offset = choose_real_width_traffic_offset(
            current,
            Vec2::ZERO,
            Vec2::Y,
            0.0,
            8.0,
            |_| false,
        );

        assert_eq!(offset, Some(-CAR_PASSING_OFFSET));
    }
}
