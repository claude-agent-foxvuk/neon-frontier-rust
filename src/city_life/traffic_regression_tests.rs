use bevy::prelude::Vec2;
use crate::physics::{bodies_overlap, move_with_slide, resolve_body_pairs, Body};

#[test]
fn separated_crossing_traffic_pair_stays_stable_on_the_next_tick() {
    let mut bodies = [
        Body::vehicle(Vec2::ZERO, 0.0),
        Body::vehicle(
            Vec2::new(0.0, 1.4),
            std::f32::consts::FRAC_PI_2,
        ),
    ];
    assert!(bodies_overlap(&bodies[0], &bodies[1]));

    resolve_body_pairs(&mut bodies);
    assert!(!bodies_overlap(&bodies[0], &bodies[1]));
    let settled = bodies;

    resolve_body_pairs(&mut bodies);
    assert_eq!(bodies, settled);
    assert!(bodies.iter().all(|body| body.center.is_finite()));
}

#[test]
fn swept_vehicle_motion_stops_before_a_rotated_obstacle_without_tunneling() {
    let wall = Body::new(
        Vec2::new(4.0, 0.0),
        5.0,
        0.25,
        std::f32::consts::FRAC_PI_2,
    );
    let start = Vec2::ZERO;
    let end = move_with_slide(start, Vec2::new(10.0, 0.0), |position| {
        bodies_overlap(&Body::vehicle(position, 0.0), &wall)
    });
    let stopped = Body::vehicle(end, 0.0);

    assert!(end.x > 0.0 && end.x < 4.0);
    assert!(!bodies_overlap(&stopped, &wall));
}

#[test]
fn dynamic_vehicle_person_pair_separates_and_remains_stable() {
    let mut bodies = [
        Body::vehicle(Vec2::ZERO, 0.4),
        Body::person(Vec2::new(1.0, 0.2)),
    ];
    assert!(bodies_overlap(&bodies[0], &bodies[1]));

    resolve_body_pairs(&mut bodies);
    assert!(!bodies_overlap(&bodies[0], &bodies[1]));
    let settled = bodies;

    resolve_body_pairs(&mut bodies);
    assert_eq!(bodies, settled);
}
