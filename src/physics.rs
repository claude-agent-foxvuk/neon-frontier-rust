use bevy::prelude::Vec2;

const MAX_SUBSTEP: f32 = 0.3;
const SEPARATION_PASSES: usize = 16;
const CONTACT_SLOP: f32 = 0.02;
const MIN_DISTANCE_SQUARED: f32 = 1.0e-8;

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
}
