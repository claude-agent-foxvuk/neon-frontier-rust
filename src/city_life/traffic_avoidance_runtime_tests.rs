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
        assert_eq!(
            transform.rotation,
            traffic_rotation(along_x, -direction)
        );
    }
}
