fn citizen() -> Citizen {
    Citizen {
        origin: Vec3::ZERO,
        phase: 0.0,
        speed: 0.0,
        range: 0.0,
        along_x: true,
        displacement: Vec3::ZERO,
        knock_velocity: Vec3::ZERO,
        knocked: 0.0,
        health: CITIZEN_HEALTH,
        dead: false,
    }
}

#[test]
fn traffic_models_face_the_direction_they_drive() {
    for (along_x, direction, expected) in [
        (true, 1.0, Vec3::X),
        (true, -1.0, Vec3::new(-1.0, 0.0, 0.0)),
        (false, 1.0, Vec3::Z),
        (false, -1.0, Vec3::new(0.0, 0.0, -1.0)),
    ] {
        assert!((traffic_rotation(along_x, direction) * Vec3::Z).distance(expected) < 1.0e-5);
    }
}

#[test]
fn oriented_vehicle_overlaps_pedestrian_on_its_footprint() {
    let vehicle = traffic_vehicle_body(&Transform::from_xyz(0.0, 0.0, 0.0), true, 1.0);
    let pedestrian = Body::person(Vec2::new(1.8, 0.0));
    assert!(bodies_overlap(&vehicle, &pedestrian));
}

#[test]
fn oriented_vehicle_does_not_hit_beyond_its_side_or_length() {
    let vehicle = traffic_vehicle_body(&Transform::from_xyz(0.0, 0.0, 0.0), true, 1.0);
    assert!(!bodies_overlap(&vehicle, &Body::person(Vec2::new(0.0, 1.5))));
    assert!(!bodies_overlap(&vehicle, &Body::person(Vec2::new(3.0, 0.0))));
}

#[test]
fn oriented_vehicle_footprint_respects_a_ninety_degree_rotation() {
    let vehicle = traffic_vehicle_body(&Transform::from_xyz(0.0, 0.0, 0.0), false, 1.0);
    assert!(bodies_overlap(&vehicle, &Body::person(Vec2::new(0.0, 1.8))));
    assert!(!bodies_overlap(&vehicle, &Body::person(Vec2::new(1.5, 0.0))));
}

#[test]
fn pedestrian_paths_fit_between_roads_and_buildings() {
    assert!(SIDEWALK_OFFSET > ROAD_HALF_WIDTH);
    assert!(SIDEWALK_OFFSET < BUILDING_EDGE_FROM_ROAD);
    assert!((6.0 * ROAD_SPACING - 144.0 + SIDEWALK_OFFSET).abs() < CITY_LIMIT);
}

#[test]
fn streets_keep_lights_and_windows_inside_the_city() {
    assert_eq!(streetlight_positions().len(), 36);
    assert_eq!(intersection_light_positions().len(), 9);
    let windows = facade_window_transforms();
    assert_eq!(windows.len(), 1_008);
    assert!(windows.iter().all(|(p, _)| {
        p.x.abs() < CITY_LIMIT && p.z.abs() < CITY_LIMIT && p.y > 0.0
    }));
}

#[test]
fn traffic_and_police_cannot_drive_through_buildings() {
    assert!(city_collision(Vec2::new(24.0, 24.0), CAR_BODY_RADIUS));
    assert!(!city_collision(Vec2::new(0.0, 0.0), CAR_BODY_RADIUS));
}

#[test]
fn traffic_detects_parked_player_vehicle_after_exit() {
    let baseline = Vec2::ZERO;
    let forward = Vec2::Y;
    let player = Vec2::new(100.0, 0.0);
    let vehicle = Some(Vec2::new(0.0, 3.0));
    assert!(traffic_obstacle_ahead(baseline, forward, player, vehicle, false, &[], &[], 0));
    assert!(!traffic_obstacle_ahead(baseline, forward, player, vehicle, true, &[], &[], 0));
}

#[test]
fn traffic_detects_a_police_car_ahead() {
    let blocker = [Vec2::new(0.0, 4.0)];
    assert!(traffic_obstacle_ahead(
        Vec2::ZERO, Vec2::Y, Vec2::new(200.0, 0.0), None, false, &[], &blocker, 0,
    ));
    assert!(!traffic_obstacle_ahead(
        Vec2::ZERO, Vec2::Y, Vec2::new(200.0, 0.0), None, false, &[], &[Vec2::new(0.0, -4.0)], 0,
    ));
}

#[test]
fn traffic_chooses_the_clear_adjacent_side() {
    let current = Vec2::new(0.0, CAR_LANE_OFFSET);
    let side = Vec2::Y;
    let offset = choose_safe_traffic_offset(current, current, side, 0.0, 10.0, |point| {
        point.y > current.y + 0.05
    });
    assert_eq!(offset, Some(-CAR_PASSING_OFFSET));
}

#[test]
fn traffic_brakes_when_both_sides_are_blocked() {
    let current = Vec2::ZERO;
    let offset = choose_safe_traffic_offset(current, current, Vec2::Y, 0.0, 10.0, |point| {
        point.y.abs() > 0.05
    });
    assert_eq!(offset, None);
}

#[test]
fn traffic_avoidance_stays_inside_the_road_width() {
    let current = Vec2::new(0.0, CAR_LANE_OFFSET);
    let side = Vec2::Y;
    let offset = choose_safe_traffic_offset(current, current, side, 0.0, ROAD_HALF_WIDTH, |_| false)
        .expect("the inner passing lane should fit");
    let target = current + side * offset;
    assert!((target.dot(side)).abs() + CAR_BODY_RADIUS <= ROAD_HALF_WIDTH);
}

#[test]
fn knock_moves_the_body_and_settles_on_the_ground() {
    let mut displacement = Vec3::ZERO;
    let mut velocity = Vec3::new(6.0, KNOCK_LIFT, 0.0);
    let mut knocked = KNOCK_SECONDS;
    for _ in 0..500 {
        if knocked <= 0.0 {
            break;
        }
        resolve_knock(&mut displacement, &mut velocity, &mut knocked, 0.02);
    }
    assert_eq!(knocked, 0.0);
    assert!(displacement.x > 0.5 && displacement.y >= 0.0);
    assert!(displacement.length() <= MAX_KNOCK_OFFSET + 1.0e-3);
}

#[test]
fn repeated_hits_kill_and_dead_npc_stays_dead() {
    let mut person = citizen();
    assert!(!damage_citizen(&mut person, 1.0));
    assert!(!damage_citizen(&mut person, 1.0));
    assert!(damage_citizen(&mut person, 1.0));
    assert!(person.dead);
    assert!(!damage_citizen(&mut person, 1.0));
    assert_eq!(person.health, 0.0);
}

#[test]
fn player_health_starts_full_and_never_drops_below_zero() {
    let mut probe = PlayerProbe::default();
    assert_eq!(probe.health, 100.0);
    damage_player(&mut probe, 120.0);
    assert_eq!(probe.health, 0.0);
}

#[test]
fn building_blocks_police_line_of_sight() {
    assert!(!police_line_of_sight_clear(Vec2::new(0.0, 24.0), Vec2::new(48.0, 24.0)));
    assert!(police_line_of_sight_clear(Vec2::new(0.0, 0.0), Vec2::new(0.0, 20.0)));
}

#[test]
fn police_shooting_requires_clear_sight_and_valid_distance() {
    assert!(!can_police_shoot(7.9, true));
    assert!(can_police_shoot(POLICE_SHOT_MIN_DISTANCE, true));
    assert!(can_police_shoot(POLICE_SHOT_MAX_DISTANCE, true));
    assert!(!can_police_shoot(18.1, true));
    assert!(!can_police_shoot(12.0, false));
}

#[test]
fn police_siren_lights_stay_off_without_wanted_level() {
    assert!(!police_siren_visible(0, 0.0, 0.0));
    assert!(!police_siren_visible(0, std::f32::consts::PI, 0.25));
}

#[test]
fn police_siren_lights_alternate_with_active_wanted_level() {
    let red_phase = 0.0;
    let blue_phase = std::f32::consts::PI;
    assert!(police_siren_visible(1, red_phase, 0.0));
    assert!(!police_siren_visible(1, blue_phase, 0.0));
    let half_cycle = std::f32::consts::PI / 12.0;
    assert!(!police_siren_visible(1, red_phase, half_cycle));
    assert!(police_siren_visible(1, blue_phase, half_cycle));
}

#[test]
fn police_tactics_distinguish_all_wanted_levels() {
    let tactics: Vec<_> = (1..=WANTED_LEVEL_MAX)
        .map(|level| police_tactics(level, 0, 0.0))
        .collect();
    for index in 0..tactics.len() {
        assert!(tactics[..index].iter().all(|other| other.speed != tactics[index].speed));
        assert!(tactics[..index]
            .iter()
            .all(|other| other.stop_distance != tactics[index].stop_distance));
        assert!(tactics[..index]
            .iter()
            .all(|other| other.flank_distance != tactics[index].flank_distance));
    }
    assert!(tactics[5].speed > tactics[0].speed);
}

#[test]
fn wanted_caps_at_six_and_decays_in_stages() {
    let mut wanted = WantedState::default();
    for _ in 0..8 {
        wanted.report_incident();
    }
    assert_eq!(wanted.level, WANTED_LEVEL_MAX);
    for _ in 0..7 {
        wanted.tick(WANTED_DECAY_START + 0.1, false);
    }
    assert_eq!(wanted.level, 0);
}
