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
