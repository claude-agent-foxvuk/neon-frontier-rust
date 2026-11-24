use bevy::prelude::Vec2;

const MAX_SUBSTEP: f32 = 0.3;
const SEPARATION_PASSES: usize = 6;

pub fn move_with_slide(
    start: Vec2,
    delta: Vec2,
    mut blocked: impl FnMut(Vec2) -> bool,
) -> Vec2 {
    let distance = delta.length();
    if distance <= f32::EPSILON {
        return start;
    }
    let steps = (distance / MAX_SUBSTEP).ceil().max(1.0) as usize;
    let step = delta / steps as f32;
    let mut position = start;
    for _ in 0..steps {
        let along_x = position + Vec2::new(step.x, 0.0);
        if !blocked(along_x) {
            position = along_x;
        }
        let along_y = position + Vec2::new(0.0, step.y);
        if !blocked(along_y) {
            position = along_y;
        }
    }
    position
}

pub fn separate_from_circles(
    mut position: Vec2,
    clearance: f32,
    obstacles: &[(Vec2, f32)],
) -> Vec2 {
    for _ in 0..SEPARATION_PASSES {
        let mut moved = false;
        for (index, (center, radius)) in obstacles.iter().enumerate() {
            let offset = position - *center;
            let minimum = clearance + radius;
            let distance_squared = offset.length_squared();
            if distance_squared >= minimum * minimum {
                continue;
            }
            let direction = if distance_squared > 1.0e-8 {
                offset / distance_squared.sqrt()
            } else if index % 2 == 0 {
                Vec2::X
            } else {
                Vec2::Y
            };
            position += direction * (minimum - distance_squared.sqrt() + 0.02);
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
}
