use bevy::prelude::*;

const POPULATION_STREAM_RADIUS: f32 = 96.0;
const TRAFFIC_STREAM_OFFSETS: [f32; 8] = [48.0, -48.0, 64.0, -64.0, 80.0, -80.0, 36.0, -36.0];
const PEDESTRIAN_STREAM_OFFSETS: [f32; 8] = [28.0, -28.0, 40.0, -40.0, 52.0, -52.0, 64.0, -64.0];

fn is_inside_population_stream(position: Vec2, player: Vec2) -> bool {
    position.is_finite()
        && player.is_finite()
        && position.distance_squared(player) <= POPULATION_STREAM_RADIUS * POPULATION_STREAM_RADIUS
}

fn recycle_city_population(
    game: Option<Res<crate::Game>>,
    probe: Res<PlayerProbe>,
    mut traffic: Query<
        (&mut Transform, &mut TrafficCar),
        (Without<Citizen>, Without<PolicePatrol>, Without<FootOfficer>),
    >,
    mut citizens: Query<
        (&mut Transform, &mut Citizen),
        (Without<TrafficCar>, Without<PolicePatrol>, Without<FootOfficer>),
    >,
    patrols: Query<
        &Transform,
        (With<PolicePatrol>, Without<TrafficCar>, Without<Citizen>, Without<FootOfficer>),
    >,
    officers: Query<
        (&Transform, &FootOfficer),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<PoliceSirenLight>,
        ),
    >,
) {
    if game
        .as_ref()
        .is_some_and(|game| !game.started || game.paused)
    {
        return;
    }
    let player = Vec2::new(probe.position.x, probe.position.z);
    if !player.is_finite() {
        return;
    }
    let player_forward = if probe.forward.is_finite() && probe.forward.length_squared() > 1.0e-6 {
        probe.forward.normalize()
    } else {
        Vec3::NEG_Z
    };
    let player_body = if probe.in_vehicle {
        vehicle_body_from_forward(probe.position, player_forward)
    } else {
        Body::person(player)
    };

    let patrol_bodies: Vec<_> = patrols
        .iter()
        .map(|transform| {
            vehicle_body_from_forward(transform.translation, transform.rotation * Vec3::Z)
        })
        .collect();
    let officer_bodies: Vec<_> = officers
        .iter()
        .filter_map(|(transform, officer)| {
            officer.active.then(|| {
                Body::person(Vec2::new(
                    transform.translation.x,
                    transform.translation.z,
                ))
            })
        })
        .collect();
    let mut traffic_bodies: Vec<_> = traffic
        .iter()
        .map(|(transform, car)| traffic_vehicle_body(transform, car.along_x, car.direction))
        .collect();
    let mut citizen_bodies: Vec<Option<Body>> = citizens
        .iter()
        .map(|(transform, citizen)| {
            (!citizen.dead).then(|| {
                Body::person(Vec2::new(
                    transform.translation.x,
                    transform.translation.z,
                ))
            })
        })
        .collect();

    // Recycle fixed entity slots so V1 save/restore keeps matching population counts.
    for (index, (mut transform, mut car)) in (&mut traffic).into_iter().enumerate() {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        if is_inside_population_stream(position, player) {
            continue;
        }
        let yaw = traffic_bodies[index].yaw;
        let cross_road = if car.along_x {
            (player.y / ROAD_SPACING).round() * ROAD_SPACING + CAR_LANE_OFFSET
        } else {
            (player.x / ROAD_SPACING).round() * ROAD_SPACING - CAR_LANE_OFFSET
        };
        for offset in TRAFFIC_STREAM_OFFSETS {
            let candidate = if car.along_x {
                Vec2::new(player.x + offset, cross_road)
            } else {
                Vec2::new(cross_road, player.y + offset)
            };
            if vehicle_city_collision(candidate, yaw) {
                continue;
            }
            let candidate_transform = Transform::from_xyz(
                candidate.x,
                transform.translation.y,
                candidate.y,
            )
            .with_rotation(traffic_rotation(car.along_x, car.direction));
            let candidate_body = traffic_vehicle_body(&candidate_transform, car.along_x, car.direction);
            let blocked = crate::physics::bodies_overlap(&candidate_body, &player_body)
                || traffic_bodies.iter().enumerate().any(|(other, body)| {
                    other != index && crate::physics::bodies_overlap(&candidate_body, body)
                })
                || citizen_bodies.iter().flatten().any(|body| {
                    crate::physics::bodies_overlap(&candidate_body, body)
                })
                || patrol_bodies
                    .iter()
                    .any(|body| crate::physics::bodies_overlap(&candidate_body, body))
                || officer_bodies
                    .iter()
                    .any(|body| crate::physics::bodies_overlap(&candidate_body, body));
            if blocked {
                continue;
            }
            transform.translation = candidate_transform.translation;
            transform.rotation = candidate_transform.rotation;
            car.lane_offset = 0.0;
            car.avoid_offset = 0.0;
            car.stop_timer = 0.0;
            traffic_bodies[index] = candidate_body;
            break;
        }
    }

    let traffic_obstacles = traffic_bodies;
    for (index, (mut transform, mut citizen)) in (&mut citizens).into_iter().enumerate() {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        if citizen.knocked > 0.0 && !citizen.dead {
            continue;
        }
        if is_inside_population_stream(position, player) {
            continue;
        }
        let cross_road = if citizen.along_x {
            (player.y / ROAD_SPACING).round() * ROAD_SPACING
        } else {
            (player.x / ROAD_SPACING).round() * ROAD_SPACING
        };
        let mut recycled = false;
        for offset in PEDESTRIAN_STREAM_OFFSETS {
            for side in [-1.0, 1.0] {
                let sidewalk = side * SIDEWALK_OFFSET;
                let candidate = if citizen.along_x {
                    Vec2::new(player.x + offset, cross_road + sidewalk)
                } else {
                    Vec2::new(cross_road + sidewalk, player.y + offset)
                };
                if city_collision(candidate, CITIZEN_BODY_RADIUS) {
                    continue;
                }
                let candidate_body = Body::person(candidate);
                let blocked = crate::physics::bodies_overlap(&candidate_body, &player_body)
                    || traffic_obstacles.iter().any(|body| {
                        crate::physics::bodies_overlap(&candidate_body, body)
                    })
                    || citizen_bodies.iter().enumerate().any(|(other, body)| {
                        other != index
                            && body.is_some_and(|body| {
                                crate::physics::bodies_overlap(&candidate_body, &body)
                            })
                    })
                    || patrol_bodies
                        .iter()
                        .any(|body| crate::physics::bodies_overlap(&candidate_body, body))
                    || officer_bodies
                        .iter()
                        .any(|body| crate::physics::bodies_overlap(&candidate_body, body));
                if blocked {
                    continue;
                }
                let origin = Vec3::new(candidate.x, citizen.origin.y, candidate.y);
                citizen.origin = origin;
                citizen.phase = 0.0;
                citizen.displacement = Vec3::ZERO;
                citizen.knock_velocity = Vec3::ZERO;
                citizen.knocked = 0.0;
                if citizen.dead {
                    citizen.health = CITIZEN_HEALTH;
                }
                citizen.dead = false;
                transform.translation = origin;
                citizen_bodies[index] = Some(candidate_body);
                recycled = true;
                break;
            }
            if recycled {
                break;
            }
        }
    }
}

#[cfg(test)]
mod population_stream_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    #[test]
    fn distant_cars_and_people_recycle_into_safe_nearby_slots() {
        let mut world = World::new();
        let mut game = crate::Game::default();
        game.started = true;
        world.insert_resource(game);
        world.insert_resource(PlayerProbe {
            position: Vec3::new(0.0, 0.9, 0.0),
            vehicle_position: Vec3::new(0.0, 0.65, 0.0),
            forward: Vec3::NEG_Z,
            in_vehicle: false,
            punch: false,
            health: 100.0,
        });
        let car = world
            .spawn((
                Transform::from_xyz(130.0, 0.48, 2.1),
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
        let citizen = world
            .spawn((
                Transform::from_xyz(130.0, 0.82, 7.0),
                Citizen {
                    origin: Vec3::new(130.0, 0.82, 7.0),
                    phase: 1.0,
                    speed: 0.5,
                    range: 7.0,
                    along_x: true,
                    displacement: Vec3::ZERO,
                    knock_velocity: Vec3::ZERO,
                    knocked: 0.0,
                    health: 2.0,
                    dead: false,
                },
            ))
            .id();
        world
            .run_system_once(recycle_city_population)
            .expect("the city streams distant entities without changing their count");

        let car_position = world.get::<Transform>(car).unwrap().translation;
        let citizen_position = world.get::<Transform>(citizen).unwrap().translation;
        assert!(is_inside_population_stream(
            Vec2::new(car_position.x, car_position.z),
            Vec2::ZERO,
        ));
        assert!(is_inside_population_stream(
            Vec2::new(citizen_position.x, citizen_position.z),
            Vec2::ZERO,
        ));
        assert!(!vehicle_city_collision(
            Vec2::new(car_position.x, car_position.z),
            traffic_vehicle_body(
                world.get::<Transform>(car).unwrap(),
                true,
                1.0,
            )
            .yaw,
        ));
        assert!(!city_collision(
            Vec2::new(citizen_position.x, citizen_position.z),
            CITIZEN_BODY_RADIUS,
        ));
        assert!(!crate::physics::bodies_overlap(
            &traffic_vehicle_body(
                world.get::<Transform>(car).unwrap(),
                true,
                1.0,
            ),
            &Body::person(Vec2::new(citizen_position.x, citizen_position.z)),
        ));
        assert_eq!(
            world.get::<Citizen>(citizen).unwrap().health,
            2.0,
            "recycling a living pedestrian preserves their health",
        );
        assert_eq!(
            world.query_filtered::<Entity, With<TrafficCar>>().iter(&world).count(),
            1,
        );
        assert_eq!(
            world.query_filtered::<Entity, With<Citizen>>().iter(&world).count(),
            1,
        );
    }

    #[test]
    fn nearby_population_and_menu_state_are_left_unchanged() {
        let mut world = World::new();
        world.insert_resource(crate::Game::default());
        world.insert_resource(PlayerProbe::default());
        let car = world
            .spawn((
                Transform::from_xyz(24.0, 0.48, 2.1),
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
        let start = world.get::<Transform>(car).unwrap().translation;

        world
            .run_system_once(recycle_city_population)
            .expect("population stays frozen in the main menu");

        assert_eq!(world.get::<Transform>(car).unwrap().translation, start);
    }
}
