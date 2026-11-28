use super::*;
use bevy::ecs::system::RunSystemOnce;
use std::time::Duration;

#[test]
fn live_traffic_passes_a_stationary_player_car_when_lane_is_clear() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(8.0, 0.65, 0.8),
        vehicle_position: Vec3::new(8.0, 0.65, 0.8),
        forward: Vec3::X,
        in_vehicle: true,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();
    world.spawn((
        Transform::from_xyz(0.0, 0.48, 0.8),
        TrafficCar {
            speed: 6.0,
            direction: 1.0,
            along_x: true,
            stop_timer: 0.0,
            lane_offset: 0.0,
            avoid_offset: 0.0,
        },
    ));

    let player_car = Body::vehicle(Vec2::new(8.0, 0.8), 0.0);
    let mut furthest_x = 0.0_f32;
    let mut smallest_z = 0.8_f32;
    for _ in 0..160 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        world
            .run_system_once(simulate_city_life)
            .expect("traffic simulation should run");
        let mut query = world.query::<(&Transform, &TrafficCar)>();
        let (transform, car) = query.single(&world).expect("one traffic car");
        let car_body = traffic_vehicle_body(transform, car.along_x, car.direction);
        assert!(!bodies_overlap(&car_body, &player_car));
        assert!(!vehicle_city_collision(car_body.center, car_body.yaw));
        furthest_x = furthest_x.max(transform.translation.x);
        smallest_z = smallest_z.min(transform.translation.z);
    }
    assert!(smallest_z < -2.0, "car never entered the passing lane");
    assert!(furthest_x > 8.0, "car never passed the stationary vehicle");
}

#[test]
fn active_traffic_turns_inward_at_each_world_edge_but_stays_idle_in_menu() {
    let mut world = World::new();
    let mut game = crate::Game::default();
    game.started = false;
    world.insert_resource(game);

    let edge = CITY_LIMIT - CAR_BODY_RADIUS;
    let cases = [
        (Vec3::new(edge, 0.48, 0.0), true, 1.0),
        (Vec3::new(-edge, 0.48, 0.0), true, -1.0),
        (Vec3::new(0.0, 0.48, edge), false, 1.0),
        (Vec3::new(0.0, 0.48, -edge), false, -1.0),
    ];
    let cars: Vec<_> = cases
        .iter()
        .map(|(position, along_x, direction)| {
            world
                .spawn((
                    Transform::from_translation(*position)
                        .with_rotation(traffic_rotation(*along_x, *direction)),
                    TrafficCar {
                        speed: 6.0,
                        direction: *direction,
                        along_x: *along_x,
                        stop_timer: 0.0,
                        lane_offset: 0.0,
                        avoid_offset: 0.0,
                    },
                ))
                .id()
        })
        .collect();

    world
        .run_system_once(turn_traffic_at_world_edge)
        .expect("edge-turn system should stay idle in the menu");
    for (entity, (_, _, direction)) in cars.iter().zip(cases) {
        assert_eq!(world.get::<TrafficCar>(*entity).unwrap().direction, direction);
    }

    world.resource_mut::<crate::Game>().started = true;
    world
        .run_system_once(turn_traffic_at_world_edge)
        .expect("active traffic should turn inward at the world edge");
    for (entity, (position, along_x, direction)) in cars.iter().zip(cases) {
        let transform = world.get::<Transform>(*entity).unwrap();
        let car = world.get::<TrafficCar>(*entity).unwrap();
        assert_eq!(transform.translation, position);
        assert_eq!(car.direction, -direction);
        assert_eq!(transform.rotation, traffic_rotation(along_x, -direction));
    }
}

#[test]
fn live_head_on_traffic_cars_both_leave_their_approach_without_mutual_deadlock() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    let mut game = crate::Game::default();
    game.started = true;
    world.insert_resource(game);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(120.0, 0.9, 120.0),
        vehicle_position: Vec3::new(122.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    let eastbound = world
        .spawn((
            Transform::from_xyz(-14.0, 0.48, 0.0)
                .with_rotation(traffic_rotation(true, 1.0)),
            TrafficCar {
                speed: 6.0,
                direction: 1.0,
                along_x: true,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ))
        .id();
    let westbound = world
        .spawn((
            Transform::from_xyz(14.0, 0.48, 0.0)
                .with_rotation(traffic_rotation(true, -1.0)),
            TrafficCar {
                speed: 6.0,
                direction: -1.0,
                along_x: true,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ))
        .id();

    for _ in 0..180 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        world
            .run_system_once(simulate_city_life)
            .expect("opposing traffic can resolve a simultaneous lane change");
        let east_transform = world
            .get::<Transform>(eastbound)
            .expect("eastbound car remains in the simulation");
        let east_car = world
            .get::<TrafficCar>(eastbound)
            .expect("eastbound traffic state remains available");
        let west_transform = world
            .get::<Transform>(westbound)
            .expect("westbound car remains in the simulation");
        let west_car = world
            .get::<TrafficCar>(westbound)
            .expect("westbound traffic state remains available");
        let east_body = traffic_vehicle_body(east_transform, east_car.along_x, east_car.direction);
        let west_body = traffic_vehicle_body(west_transform, west_car.along_x, west_car.direction);
        assert!(east_body.center.is_finite() && west_body.center.is_finite());
        assert!(!bodies_overlap(&east_body, &west_body));
        assert!(!vehicle_city_collision(east_body.center, east_body.yaw));
        assert!(!vehicle_city_collision(west_body.center, west_body.yaw));
    }

    let east_position = world.get::<Transform>(eastbound).unwrap().translation;
    let west_position = world.get::<Transform>(westbound).unwrap().translation;
    assert!(east_position.x > 2.0, "eastbound car did not clear the opposing car");
    assert!(west_position.x < -2.0, "westbound car did not clear the opposing car");
}

#[test]
fn live_crossing_traffic_cars_both_clear_the_intersection_without_deadlock() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    let mut game = crate::Game::default();
    game.started = true;
    world.insert_resource(game);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(120.0, 0.9, 120.0),
        vehicle_position: Vec3::new(122.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    let eastbound = world
        .spawn((
            Transform::from_xyz(-14.0, 0.48, 0.0)
                .with_rotation(traffic_rotation(true, 1.0)),
            TrafficCar {
                speed: 6.0,
                direction: 1.0,
                along_x: true,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ))
        .id();
    let northbound = world
        .spawn((
            Transform::from_xyz(0.0, 0.48, -14.0)
                .with_rotation(traffic_rotation(false, 1.0)),
            TrafficCar {
                speed: 6.0,
                direction: 1.0,
                along_x: false,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ))
        .id();

    for _ in 0..220 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        world
            .run_system_once(simulate_city_life)
            .expect("crossing traffic resolves simultaneous approaches");
        let east_transform = world
            .get::<Transform>(eastbound)
            .expect("eastbound car remains in the simulation");
        let east_car = world
            .get::<TrafficCar>(eastbound)
            .expect("eastbound traffic state remains available");
        let north_transform = world
            .get::<Transform>(northbound)
            .expect("northbound car remains in the simulation");
        let north_car = world
            .get::<TrafficCar>(northbound)
            .expect("northbound traffic state remains available");
        let east_body = traffic_vehicle_body(east_transform, east_car.along_x, east_car.direction);
        let north_body = traffic_vehicle_body(north_transform, north_car.along_x, north_car.direction);
        assert!(east_body.center.is_finite() && north_body.center.is_finite());
        assert!(!bodies_overlap(&east_body, &north_body));
        assert!(!vehicle_city_collision(east_body.center, east_body.yaw));
        assert!(!vehicle_city_collision(north_body.center, north_body.yaw));
    }

    let east_position = world.get::<Transform>(eastbound).unwrap().translation;
    let north_position = world.get::<Transform>(northbound).unwrap().translation;
    assert!(east_position.x > 2.0, "eastbound car did not clear the crossing");
    assert!(north_position.z > 2.0, "northbound car did not clear the crossing");
}

#[test]
fn live_four_way_traffic_does_not_deadlock_when_all_approaches_arrive_together() {
    let mut world = World::new();
    let mut time = Time::<()>::default();
    time.advance_by(Duration::from_millis(16));
    world.insert_resource(time);
    let mut game = crate::Game::default();
    game.started = true;
    world.insert_resource(game);
    world.insert_resource(PlayerProbe {
        position: Vec3::new(120.0, 0.9, 120.0),
        vehicle_position: Vec3::new(122.0, 0.65, 120.0),
        forward: Vec3::NEG_Z,
        in_vehicle: false,
        punch: false,
        health: 100.0,
    });
    world.init_resource::<TrafficBodies>();
    world.init_resource::<WantedState>();
    world.init_resource::<WorldState>();
    world.init_resource::<WorldRestore>();

    let approaches = [
        (Vec3::new(-14.0, 0.48, 0.0), true, 1.0),
        (Vec3::new(14.0, 0.48, 0.0), true, -1.0),
        (Vec3::new(0.0, 0.48, -14.0), false, 1.0),
        (Vec3::new(0.0, 0.48, 14.0), false, -1.0),
    ];
    let cars: Vec<_> = approaches
        .iter()
        .map(|(position, along_x, direction)| {
            world
                .spawn((
                    Transform::from_translation(*position)
                        .with_rotation(traffic_rotation(*along_x, *direction)),
                    TrafficCar {
                        speed: 6.0,
                        direction: *direction,
                        along_x: *along_x,
                        stop_timer: 0.0,
                        lane_offset: 0.0,
                        avoid_offset: 0.0,
                    },
                ))
                .id()
        })
        .collect();

    for _ in 0..260 {
        world
            .resource_mut::<Time<()>>()
            .advance_by(Duration::from_millis(50));
        world
            .run_system_once(simulate_city_life)
            .expect("four simultaneous approaches resolve through live traffic simulation");

        let bodies: Vec<_> = cars
            .iter()
            .map(|entity| {
                let transform = world
                    .get::<Transform>(*entity)
                    .expect("each approaching vehicle remains in the simulation");
                let car = world
                    .get::<TrafficCar>(*entity)
                    .expect("each approaching vehicle retains its traffic state");
                traffic_vehicle_body(transform, car.along_x, car.direction)
            })
            .collect();
        for (index, body) in bodies.iter().enumerate() {
            assert!(body.center.is_finite());
            assert!(!vehicle_city_collision(body.center, body.yaw));
            for other in &bodies[(index + 1)..] {
                assert!(!bodies_overlap(body, other));
            }
        }
    }

    for (entity, (_, along_x, direction)) in cars.iter().zip(approaches) {
        let transform = world
            .get::<Transform>(*entity)
            .expect("each approaching vehicle remains in the simulation");
        let progress = if along_x {
            transform.translation.x * direction
        } else {
            transform.translation.z * direction
        };
        assert!(progress > 2.0, "a four-way approach remained blocked: {transform:?}");
    }
}
