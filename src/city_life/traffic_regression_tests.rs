use bevy::prelude::Vec2;
use crate::physics::{bodies_overlap, resolve_body_pairs, Body};

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
