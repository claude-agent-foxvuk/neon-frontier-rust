use bevy::prelude::Vec2;

use super::{vehicle_city_collision, CAR_BRAKE_DISTANCE, CAR_PASSING_OFFSET};

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
    for direction in [1.0, -1.0] {
        let offset = direction * CAR_PASSING_OFFSET;
        let target = baseline + side * offset;
        if (target.dot(side) - road_center).abs() + crate::physics::VEHICLE_HALF_WIDTH
            > road_half_width
        {
            continue;
        }
        let candidate = crate::physics::move_with_slide(current, target - current, |point| {
            vehicle_city_collision(point, vehicle_yaw) || blocked(point)
        });
        if candidate.distance(target) >= 0.05 {
            continue;
        }

        // Keep a full braking-distance corridor clear after merging, not only the car's nose.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn passing_uses_vehicle_width_not_longitudinal_radius() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |_| {
            false
        });
        assert_eq!(offset, Some(-CAR_PASSING_OFFSET));
        let target = current + Vec2::Y * offset.unwrap();
        assert!(target.y.abs() + crate::physics::VEHICLE_HALF_WIDTH <= 5.5);
    }

    #[test]
    fn passing_does_not_cross_an_obstacle_on_the_way_to_the_other_lane() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.y < -1.0
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_rejects_the_lane_if_vehicle_overhangs_road() {
        let current = Vec2::ZERO;
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |_| {
            false
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_rejects_a_lane_blocked_just_after_the_merge() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.x > 1.0 && p.y < -1.0
        });
        assert_eq!(offset, None);
    }

    #[test]
    fn passing_rejects_an_obstacle_further_down_the_braking_corridor() {
        let current = Vec2::new(0.0, 0.8);
        let offset = choose_real_width_traffic_offset(current, current, Vec2::Y, 0.0, 5.5, |p| {
            p.x > 4.0 && p.y < -1.0
        });
        assert_eq!(offset, None);
    }
}
