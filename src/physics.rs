use bevy::prelude::Vec2;

const MAX_SUBSTEP: f32 = 0.3;
const SEPARATION_PASSES: usize = 16;
const CONTACT_SLOP: f32 = 0.02;
const MIN_DISTANCE_SQUARED: f32 = 1.0e-8;

pub const VEHICLE_HALF_LENGTH: f32 = 2.15;
pub const VEHICLE_HALF_WIDTH: f32 = 0.95;
pub const PERSON_HALF_EXTENT: f32 = 0.35;

pub fn move_with_slide(start: Vec2, delta: Vec2, mut blocked: impl FnMut(Vec2) -> bool) -> Vec2 {
    let distance = delta.length();
    if distance <= f32::EPSILON {
        return start;
    }
    let steps = (distance / MAX_SUBSTEP).ceil().max(1.0) as usize;
    let step = delta / steps as f32;
    let mut position = start;
    for _ in 0..steps {
        let direct = position + step;
        if !blocked(direct) {
            position = direct;
            continue;
        }
        if step.x.abs() >= step.y.abs() {
            let along_x = position + Vec2::new(step.x, 0.0);
            if !blocked(along_x) {
                position = along_x;
            }
            let along_y = position + Vec2::new(0.0, step.y);
            if !blocked(along_y) {
                position = along_y;
            }
        } else {
            let along_y = position + Vec2::new(0.0, step.y);
            if !blocked(along_y) {
                position = along_y;
            }
            let along_x = position + Vec2::new(step.x, 0.0);
            if !blocked(along_x) {
                position = along_x;
            }
        }
    }
    position
}

pub fn separate_from_circles(
    mut position: Vec2,
    clearance: f32,
    obstacles: &[(Vec2, f32)],
) -> Vec2 {
    if !position.is_finite() || !clearance.is_finite() {
        return position;
    }
    let clearance = clearance.max(0.0);
    for _ in 0..SEPARATION_PASSES {
        let mut moved = false;
        for (index, (center, radius)) in obstacles.iter().enumerate() {
            if !center.is_finite() || !radius.is_finite() {
                continue;
            }
            let offset = position - *center;
            let minimum = clearance + radius.max(0.0);
            let distance_squared = offset.length_squared();
            if distance_squared >= minimum * minimum {
                continue;
            }
            let distance = distance_squared.sqrt();
            let direction = if distance_squared > MIN_DISTANCE_SQUARED {
                offset / distance
            } else if index % 2 == 0 {
                Vec2::X
            } else {
                Vec2::Y
            };
            position += direction * (minimum - distance + CONTACT_SLOP);
            moved = true;
        }
        if !moved {
            break;
        }
    }
    position
}

fn finite_half(value: f32) -> f32 {
    if value.is_finite() {
        value.abs().max(0.05)
    } else {
        0.05
    }
}

fn push_direction(value: f32) -> f32 {
    if value.abs() < 1.0e-6 {
        1.0
    } else {
        value.signum()
    }
}

/// A body footprint in its own frame: `half_x` runs along `yaw`, `half_y` across it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub center: Vec2,
    pub half_x: f32,
    pub half_y: f32,
    pub yaw: f32,
}

impl Body {
    pub fn new(center: Vec2, half_x: f32, half_y: f32, yaw: f32) -> Self {
        Self {
            center,
            half_x: finite_half(half_x),
            half_y: finite_half(half_y),
            yaw,
        }
    }

    pub fn vehicle(center: Vec2, yaw: f32) -> Self {
        Self::new(center, VEHICLE_HALF_LENGTH, VEHICLE_HALF_WIDTH, yaw)
    }

    pub fn person(center: Vec2) -> Self {
        Self::new(center, PERSON_HALF_EXTENT, PERSON_HALF_EXTENT, 0.0)
    }

    fn is_valid(&self) -> bool {
        self.center.is_finite()
            && self.half_x.is_finite()
            && self.half_y.is_finite()
            && self.yaw.is_finite()
    }

    fn axis_x(&self) -> Vec2 {
        Vec2::new(self.yaw.cos(), self.yaw.sin())
    }

    fn axis_y(&self) -> Vec2 {
        Vec2::new(-self.yaw.sin(), self.yaw.cos())
    }

    fn to_local(&self, point: Vec2) -> Vec2 {
        let offset = point - self.center;
        let (sin, cos) = self.yaw.sin_cos();
        Vec2::new(
            offset.x * cos + offset.y * sin,
            -offset.x * sin + offset.y * cos,
        )
    }

    fn to_world(&self, point: Vec2) -> Vec2 {
        let (sin, cos) = self.yaw.sin_cos();
        Vec2::new(
            self.center.x + point.x * cos - point.y * sin,
            self.center.y + point.x * sin + point.y * cos,
        )
    }

    /// Whether a point with its own clearance lies inside this footprint.
    pub fn contains_point(&self, point: Vec2, clearance: f32) -> bool {
        if !self.is_valid() || !point.is_finite() || !clearance.is_finite() {
            return false;
        }
        let clearance = clearance.max(0.0);
        let local = self.to_local(point);
        local.x.abs() <= self.half_x + clearance && local.y.abs() <= self.half_y + clearance
    }

    fn radius_on(&self, axis: Vec2) -> f32 {
        let along_x = (self.half_x * self.axis_x().dot(axis)).abs();
        let along_y = (self.half_y * self.axis_y().dot(axis)).abs();
        along_x + along_y
    }
}

/// Separating-axis test between two oriented body footprints.
pub fn bodies_overlap(first: &Body, second: &Body) -> bool {
    if !first.is_valid() || !second.is_valid() {
        return false;
    }
    let axes = [
        first.axis_x(),
        first.axis_y(),
        second.axis_x(),
        second.axis_y(),
    ];
    for axis in axes {
        if axis.length_squared() <= MIN_DISTANCE_SQUARED {
            continue;
        }
        let axis = axis.normalize();
        let distance = (second.center - first.center).dot(axis).abs();
        if distance > first.radius_on(axis) + second.radius_on(axis) {
            return false;
        }
    }
    true
}

/// Pushes a point out of every overlapping body along the nearest face.
pub fn push_out_of_bodies(mut position: Vec2, clearance: f32, bodies: &[Body]) -> Vec2 {
    if !position.is_finite() || !clearance.is_finite() {
        return position;
    }
    let clearance = clearance.max(0.0);
    for _ in 0..SEPARATION_PASSES {
        let mut moved = false;
        for body in bodies.iter().filter(|body| body.is_valid()) {
            let local = body.to_local(position);
            let overlap_x = body.half_x + clearance - local.x.abs();
            let overlap_y = body.half_y + clearance - local.y.abs();
            if overlap_x <= 0.0 || overlap_y <= 0.0 {
                continue;
            }
            let push = if overlap_x < overlap_y {
                Vec2::new(push_direction(local.x) * (overlap_x + CONTACT_SLOP), 0.0)
            } else {
                Vec2::new(0.0, push_direction(local.y) * (overlap_y + CONTACT_SLOP))
            };
            position = body.to_world(local + push);
            moved = true;
        }
        if !moved {
            break;
        }
    }
    position
}

/// Moves a lane offset toward its target at `rate` units per second.
pub fn approach_lane_offset(current: f32, target: f32, rate: f32, dt: f32) -> f32 {
    if !current.is_finite() || !target.is_finite() || !rate.is_finite() || !dt.is_finite() {
        return 0.0;
    }
    let maximum = rate.abs().max(0.0) * dt.max(0.0);
    let delta = target - current;
    if delta.abs() <= maximum {
        target
    } else {
        current + delta.signum() * maximum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movement_slides_along_a_wall_instead_of_freezing() {
        let position = move_with_slide(Vec2::ZERO, Vec2::new(1.0, 1.0), |point| point.x > 0.4);
        assert!(position.x <= 0.4);
        assert!(position.y > 0.8);
    }

    #[test]
    fn unobstructed_diagonal_motion_keeps_its_direct_path() {
        let position = move_with_slide(Vec2::ZERO, Vec2::new(1.0, 1.0), |_| false);
        assert!(position.distance(Vec2::new(1.0, 1.0)) < 1.0e-5);
    }

    #[test]
    fn substeps_stop_fast_motion_before_an_obstacle() {
        let position = move_with_slide(Vec2::ZERO, Vec2::new(10.0, 0.0), |point| point.x > 4.0);
        assert!(position.x <= 4.0);
        assert!(position.x > 3.6);
    }

    #[test]
    fn overlapping_vehicle_can_be_pushed_clear_of_a_player() {
        let position = separate_from_circles(Vec2::ZERO, 0.45, &[(Vec2::ZERO, 2.2)]);
        assert!(position.length() >= 2.65);
        assert!(position.is_finite());
    }

    #[test]
    fn crowded_vehicle_separation_clears_each_feasible_obstacle() {
        let obstacles = [
            (Vec2::new(-1.5, 0.0), 1.0),
            (Vec2::new(1.5, 0.0), 1.0),
            (Vec2::new(0.0, 2.5), 0.8),
        ];
        let position = separate_from_circles(Vec2::ZERO, 0.5, &obstacles);
        assert!(position.is_finite());
        assert!(obstacles
            .iter()
            .all(|(center, radius)| position.distance(*center) + 1.0e-4 >= radius + 0.5));
    }

    #[test]
    fn dense_obstacle_scene_starts_overlapping_and_ends_clear() {
        let obstacles = [
            (Vec2::new(-0.7, -1.0), 0.9),
            (Vec2::new(0.7, -1.0), 0.9),
            (Vec2::new(0.0, -2.1), 0.8),
        ];
        let clearance = 0.35;
        let overlapping = obstacles
            .iter()
            .filter(|(center, radius)| Vec2::ZERO.distance(*center) < radius + clearance)
            .count();
        assert_eq!(overlapping, 2);
        let position = separate_from_circles(Vec2::ZERO, clearance, &obstacles);
        assert!(position.is_finite());
        for (center, radius) in obstacles {
            assert!(position.distance(center) + 1.0e-3 >= radius + clearance);
        }
    }

    #[test]
    fn separation_is_stable_when_the_actor_starts_at_circle_centers() {
        let first = separate_from_circles(Vec2::ZERO, 0.5, &[(Vec2::ZERO, 1.0)]);
        let second = separate_from_circles(Vec2::ZERO, 0.5, &[(Vec2::ZERO, 1.0)]);
        assert_eq!(first, second);
        assert!(first.is_finite());
        assert!(first.length() >= 1.5);
    }

    #[test]
    fn invalid_obstacle_data_does_not_create_nan_positions() {
        let position = separate_from_circles(
            Vec2::ZERO,
            0.5,
            &[(Vec2::new(f32::NAN, 0.0), 2.0), (Vec2::ONE, f32::INFINITY)],
        );
        assert_eq!(position, Vec2::ZERO);
    }

    #[test]
    fn vehicle_footprint_follows_its_yaw() {
        let along_x = Body::vehicle(Vec2::ZERO, 0.0);
        let turned = Body::vehicle(Vec2::ZERO, std::f32::consts::FRAC_PI_2);
        assert!(along_x.contains_point(Vec2::new(2.0, 0.0), 0.0));
        assert!(!along_x.contains_point(Vec2::new(0.0, 2.0), 0.0));
        assert!(!turned.contains_point(Vec2::new(2.0, 0.0), 0.0));
        assert!(turned.contains_point(Vec2::new(0.0, 2.0), 0.0));
    }

    #[test]
    fn pedestrian_body_is_smaller_than_a_vehicle_body() {
        let person = Body::person(Vec2::ZERO);
        let car = Body::vehicle(Vec2::ZERO, 0.0);
        assert!(person.contains_point(Vec2::new(0.3, 0.3), 0.0));
        assert!(!person.contains_point(Vec2::new(0.6, 0.0), 0.0));
        assert!(car.contains_point(Vec2::new(0.6, 0.0), 0.0));
        assert!(car.half_x > person.half_x);
        assert!(car.half_y > person.half_y);
    }

    #[test]
    fn point_inside_a_vehicle_body_is_pushed_to_the_nearest_face() {
        let car = Body::vehicle(Vec2::ZERO, 0.0);
        let pushed = push_out_of_bodies(Vec2::new(0.5, 0.1), 0.4, &[car]);
        assert!(pushed.is_finite());
        assert!(!car.contains_point(pushed, 0.4));
        assert!(pushed.y > 1.2);
        assert_eq!(pushed.x, 0.5);
    }

    #[test]
    fn pushing_an_overlapping_center_clears_the_other_body() {
        let parked = Body::vehicle(Vec2::ZERO, 0.0);
        let intruder = Body::vehicle(Vec2::new(1.4, 0.2), 0.3);
        assert!(bodies_overlap(&parked, &intruder));
        let moved = push_out_of_bodies(intruder.center, VEHICLE_HALF_WIDTH, &[parked]);
        assert!(moved.is_finite());
        assert!(!parked.contains_point(moved, VEHICLE_HALF_WIDTH));
    }

    #[test]
    fn bodies_overlap_detects_a_rotated_contact_and_clears_a_gap() {
        let first = Body::vehicle(Vec2::ZERO, 0.0);
        let beside = Body::vehicle(Vec2::new(0.0, 2.4), 0.0);
        let crossing = Body::vehicle(Vec2::new(0.0, 2.4), std::f32::consts::FRAC_PI_2);
        assert!(!bodies_overlap(&first, &beside));
        assert!(bodies_overlap(&first, &crossing));
    }

    #[test]
    fn invalid_body_data_is_ignored() {
        let broken = Body {
            center: Vec2::new(f32::NAN, 0.0),
            half_x: 1.0,
            half_y: 1.0,
            yaw: 0.0,
        };
        let intact = Body::vehicle(Vec2::ZERO, 0.0);
        assert!(!bodies_overlap(&broken, &intact));
        assert_eq!(push_out_of_bodies(Vec2::ZERO, 0.3, &[broken]), Vec2::ZERO);
        assert!(bodies_overlap(&intact, &intact));
    }

    #[test]
    fn lane_offset_eases_back_to_zero_without_jumping() {
        let dt = 1.0 / 60.0;
        let mut offset = 4.6_f32;
        let mut steps = 0;
        while offset != 0.0 && steps < 600 {
            let next = approach_lane_offset(offset, 0.0, 6.0, dt);
            assert!(next.abs() < offset.abs());
            offset = next;
            steps += 1;
        }
        assert_eq!(offset, 0.0);
        assert!(steps < 120);
    }

    #[test]
    fn lane_offset_reaches_the_target_within_one_long_frame() {
        assert_eq!(approach_lane_offset(4.6, 0.0, 6.0, 5.0), 0.0);
        assert_eq!(approach_lane_offset(0.0, -4.6, 6.0, 0.05), -0.3);
    }

    #[test]
    fn invalid_lane_offset_input_falls_back_to_zero() {
        assert_eq!(approach_lane_offset(f32::NAN, 0.0, 6.0, 0.016), 0.0);
        assert_eq!(approach_lane_offset(0.0, f32::INFINITY, 6.0, 0.016), 0.0);
        assert_eq!(approach_lane_offset(1.0, 0.0, f32::NAN, 0.016), 0.0);
    }
}
