use bevy::prelude::*;

use crate::physics::bodies_overlap;

use super::*;

fn traffic_vehicle_body_at(position: Vec2, along_x: bool, direction: f32) -> Body {
    traffic_vehicle_body(
        &Transform::from_xyz(position.x, 0.48, position.y),
        along_x,
        direction,
    )
}

fn traffic_car_overlaps_body(position: Vec2, along_x: bool, direction: f32, obstacle: &Body) -> bool {
    bodies_overlap(
        &traffic_vehicle_body_at(position, along_x, direction),
        obstacle,
    )
}

fn police_vehicle_blocked(position: Vec2, yaw: f32, blockers: &[Body], player: &Body) -> bool {
    let patrol = Body::vehicle(position, yaw);
    bodies_overlap(&patrol, player) || blockers.iter().any(|body| bodies_overlap(&patrol, body))
}

fn officer_blocked(position: Vec2, blockers: &[Body], player: &Body) -> bool {
    let officer = Body::person(position);
    bodies_overlap(&officer, player) || blockers.iter().any(|body| bodies_overlap(&officer, body))
}

fn resolve_traffic_body_pairs(bodies: &mut [Body]) -> Vec<bool> {
    let original_centers: Vec<_> = bodies.iter().map(|body| body.center).collect();
    for _ in 0..16 {
        let mut moved = false;
        for first_index in 0..bodies.len() {
            for second_index in (first_index + 1)..bodies.len() {
                let first = bodies[first_index];
                let second = bodies[second_index];
                let Some(translation) = crate::physics::body_separation_vector(&first, &second)
                else {
                    continue;
                };

                let mut first_split = first;
                first_split.center += translation * 0.5;
                let mut second_split = second;
                second_split.center -= translation * 0.5;
                if !vehicle_city_collision(first_split.center, first_split.yaw)
                    && !vehicle_city_collision(second_split.center, second_split.yaw)
                {
                    bodies[first_index] = first_split;
                    bodies[second_index] = second_split;
                    moved = true;
                    continue;
                }

                let mut first_only = first;
                first_only.center += translation;
                if !vehicle_city_collision(first_only.center, first_only.yaw) {
                    bodies[first_index] = first_only;
                    moved = true;
                    continue;
                }

                let mut second_only = second;
                second_only.center -= translation;
                if !vehicle_city_collision(second_only.center, second_only.yaw) {
                    bodies[second_index] = second_only;
                    moved = true;
                }
            }
        }
        if !moved {
            break;
        }
    }

    bodies
        .iter()
        .zip(original_centers)
        .map(|(body, original)| body.center.distance_squared(original) > 1.0e-6)
        .collect()
}

pub(super) fn simulate_city_life(
    time: Res<Time>,
    mut probe: ResMut<PlayerProbe>,
    game: Option<Res<crate::Game>>,
    mut bodies: ResMut<TrafficBodies>,
    mut wanted: ResMut<WantedState>,
    mut world: ResMut<WorldState>,
    mut restore: ResMut<WorldRestore>,
    mut traffic: Query<
        (&mut Transform, &mut TrafficCar),
        (
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    mut citizens: Query<
        (&mut Transform, &mut Citizen),
        (
            Without<TrafficCar>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
        ),
    >,
    mut patrols: Query<
        (&mut Transform, &mut PolicePatrol),
        (Without<TrafficCar>, Without<Citizen>, Without<FootOfficer>),
    >,
    mut officers: Query<
        (&mut Transform, &mut FootOfficer, &mut Visibility),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<PoliceSirenLight>,
        ),
    >,
    mut siren_lights: Query<
        (&mut Visibility, &PoliceSirenLight),
        (Without<FootOfficer>, Without<OfficerMuzzleFlash>),
    >,
    mut muzzle_flashes: Query<
        (&mut Visibility, &mut OfficerMuzzleFlash),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<FootOfficer>,
            Without<PoliceSirenLight>,
        ),
    >,
) {
    let dt = time.delta_secs().min(0.05);
    if dt <= 0.0 {
        return;
    }
    if let Some(snapshot) = restore.0.take() {
        restore_world(
            &snapshot,
            &mut probe,
            &mut wanted,
            &mut traffic,
            &mut citizens,
            &mut patrols,
            &mut officers,
        );
    }
    let now = time.elapsed_secs();
    let player = probe.position;
    let player_flat = Vec2::new(player.x, player.z);
    let parked_vehicle = if probe.in_vehicle {
        None
    } else {
        let yaw = game.as_ref().map_or(0.0, |game| game.yaw);
        let forward = Vec3::new(yaw.sin(), 0.0, yaw.cos());
        Some(vehicle_body_from_forward(probe.vehicle_position, forward))
    };
    let punching = probe.punch;
    let player_forward = if probe.forward.length_squared() > 1.0e-6 {
        probe.forward.normalize()
    } else {
        Vec3::NEG_Z
    };
    let player_vehicle_body = probe
        .in_vehicle
        .then(|| vehicle_body_from_forward(player, player_forward));
    let player_contact_body = player_vehicle_body.unwrap_or_else(|| Body::person(player_flat));
    let mut player_incident = false;
    let mut traffic_vehicle_bodies: Vec<Body> = traffic
        .iter()
        .map(|(transform, car)| traffic_vehicle_body(&transform, car.along_x, car.direction))
        .collect();
    let citizen_positions: Vec<Vec3> = citizens
        .iter()
        .map(|(transform, _)| transform.translation)
        .collect();
    let mut patrol_vehicle_bodies: Vec<Body> = patrols
        .iter()
        .map(|(transform, _)| {
            vehicle_body_from_forward(transform.translation, transform.rotation * Vec3::Z)
        })
        .collect();
    let mut knock_events: Vec<(usize, Vec3)> = Vec::with_capacity(4);
    let mut car_bodies: Vec<(Vec2, f32)> =
        Vec::with_capacity(traffic_vehicle_bodies.len() + patrol_vehicle_bodies.len());

    for (index, (mut transform, mut car)) in traffic.iter_mut().enumerate() {
        car.stop_timer = (car.stop_timer - dt).max(0.0);
        let along_x = car.along_x;
        let direction = car.direction;
        let forward = (if along_x { Vec3::X } else { Vec3::Z }) * direction;
        let forward = Vec2::new(forward.x, forward.z);
        let side = Vec2::new(-forward.y, forward.x);
        let current = Vec2::new(transform.translation.x, transform.translation.z);
        let baseline = current - side * car.lane_offset;
        let road_center = (baseline.dot(side) / ROAD_SPACING).round() * ROAD_SPACING;
        let blocked_ahead = traffic_obstacle_ahead_with_player_forward(
            baseline,
            forward,
            player_flat,
            Vec2::new(player_forward.x, player_forward.z),
            parked_vehicle,
            probe.in_vehicle,
            &traffic_vehicle_bodies,
            &patrol_vehicle_bodies,
            index,
        );
        let blocks_movement = |point: Vec2| {
            let moving_body = traffic_vehicle_body_at(point, along_x, direction);
            bodies_overlap(&moving_body, &player_contact_body)
                || parked_vehicle
                    .as_ref()
                    .map_or(false, |vehicle| bodies_overlap(&moving_body, vehicle))
                || patrol_vehicle_bodies
                    .iter()
                    .any(|body| bodies_overlap(&moving_body, body))
                || traffic_vehicle_bodies.iter().enumerate().any(|(other, body)| {
                    other != index && bodies_overlap(&moving_body, body)
                })
        };

        if blocked_ahead {
            match choose_safe_traffic_offset(
                current,
                baseline,
                side,
                road_center,
                ROAD_HALF_WIDTH,
                |point| blocks_movement(point),
            ) {
                Some(offset) => car.avoid_offset = offset,
                None => {
                    car.avoid_offset = car.lane_offset;
                    car.stop_timer = car.stop_timer.max(0.2);
                }
            }
        } else {
            car.avoid_offset = 0.0;
        }

        if car.stop_timer <= 0.0 {
            let lateral_step = (car.avoid_offset - car.lane_offset)
                .clamp(-car.speed.abs() * dt, car.speed.abs() * dt);
            let lateral_target = current + side * lateral_step;
            let vehicle_yaw = traffic_vehicle_body_at(current, along_x, direction).yaw;
            let shifted = crate::physics::move_with_slide(current, side * lateral_step, |point| {
                vehicle_city_collision(point, vehicle_yaw) || blocks_movement(point)
            });
            let lateral_clear = shifted.distance(lateral_target) < 0.05;
            car.lane_offset += (shifted - current).dot(side);
            let mut candidate = shifted;

            // Do not move forward into the obstacle while still crossing lanes.
            let has_passing_clearance = car.lane_offset.abs() >= CAR_PASSING_OFFSET - 0.1;
            if lateral_clear && (!blocked_ahead || has_passing_clearance) {
                let delta = forward * car.speed * dt;
                candidate = crate::physics::move_with_slide(candidate, delta, |point| {
                    vehicle_city_collision(point, vehicle_yaw) || blocks_movement(point)
                });
                if candidate.distance(shifted) < delta.length() * 0.5 && blocked_ahead {
                    car.stop_timer = 0.2;
                }
            }
            transform.translation.x = candidate.x;
            transform.translation.z = candidate.y;
        }
        traffic_vehicle_bodies[index] =
            traffic_vehicle_body(&transform, car.along_x, car.direction);
        for (citizen_index, position) in citizen_positions.iter().enumerate() {
            let person_body = Body::person(Vec2::new(position.x, position.z));
            if bodies_overlap(&traffic_vehicle_bodies[index], &person_body) {
                let flat = Vec2::new(
                    transform.translation.x - position.x,
                    transform.translation.z - position.z,
                );
                let push = if flat.length() > 1.0e-4 {
                    flat.normalize()
                } else {
                    Vec2::Y
                };
                let velocity = Vec3::new(push.x, 0.0, push.y) * (KNOCK_SPEED + car.speed * 0.35)
                    + Vec3::Y * KNOCK_LIFT;
                knock_events.push((citizen_index, velocity));
                car.stop_timer = CAR_STOP_SECONDS;
            }
        }
        car_bodies.push((
            Vec2::new(transform.translation.x, transform.translation.z),
            CAR_BODY_RADIUS,
        ));
    }

    let mut pair_bodies: Vec<Body> = traffic
        .iter_mut()
        .map(|(transform, car)| traffic_vehicle_body(&transform, car.along_x, car.direction))
        .collect();
    let pair_moved = resolve_traffic_body_pairs(&mut pair_bodies);
    for (index, ((mut transform, mut car), body)) in
        traffic.iter_mut().zip(pair_bodies.into_iter()).enumerate()
    {
        if pair_moved[index] {
            let current = Vec2::new(transform.translation.x, transform.translation.z);
            let forward = (if car.along_x { Vec2::X } else { Vec2::Y }) * car.direction;
            let side = Vec2::new(-forward.y, forward.x);
            car.lane_offset += (body.center - current).dot(side);
            car.stop_timer = car.stop_timer.max(0.15);
            transform.translation.x = body.center.x;
            transform.translation.z = body.center.y;
        }
        traffic_vehicle_bodies[index] =
            traffic_vehicle_body(&transform, car.along_x, car.direction);
        car_bodies[index] = (
            Vec2::new(transform.translation.x, transform.translation.z),
            CAR_BODY_RADIUS,
        );
    }

    let mut citizen_index = 0;
    for (mut transform, mut citizen) in &mut citizens {
        let index = citizen_index;
        citizen_index += 1;
        if citizen.dead {
            continue;
        }
        if citizen.knocked <= 0.0 {
            citizen.displacement = citizen.displacement.lerp(Vec3::ZERO, (dt * 3.0).min(1.0));
            citizen.knock_velocity = Vec3::ZERO;
        }
        let phase = now * citizen.speed + citizen.phase;
        let offset = phase.sin() * citizen.range;
        let direction = if phase.cos() >= 0.0 { 1.0 } else { -1.0 };
        let along = if citizen.along_x { Vec3::X } else { Vec3::Z };
        let base = citizen.origin + along * offset;
        let current = base + citizen.displacement;
        let mut impact_velocity = Vec3::ZERO;
        let mut impact_damage = 0.0;
        for (target, velocity) in &knock_events {
            if *target == index {
                impact_velocity = *velocity;
                impact_damage = CITIZEN_HEALTH;
            }
        }
        let person_body = Body::person(Vec2::new(current.x, current.z));
        for (other, position) in citizen_positions.iter().enumerate() {
            if other == index {
                continue;
            }
            let other_body = Body::person(Vec2::new(position.x, position.z));
            if let Some(separation) =
                crate::physics::body_separation_vector(&person_body, &other_body)
            {
                citizen.displacement += Vec3::new(separation.x, 0.0, separation.y) * 0.5;
            }
        }
        if !probe.in_vehicle {
            if let Some(separation) =
                crate::physics::body_separation_vector(&person_body, &player_contact_body)
            {
                citizen.displacement += Vec3::new(separation.x, 0.0, separation.y);
            }
        }
        let to_player = Vec2::new(current.x - player.x, current.z - player.z);
        let player_distance = to_player.length();
        if player_vehicle_body
            .as_ref()
            .map_or(false, |vehicle| bodies_overlap(vehicle, &person_body))
        {
            let push = if player_distance > 1.0e-4 {
                to_player.normalize()
            } else {
                Vec2::Y
            };
            impact_velocity = Vec3::new(push.x, 0.0, push.y) * KNOCK_SPEED + Vec3::Y * KNOCK_LIFT;
            impact_damage = CITIZEN_HEALTH;
            player_incident = true;
        } else if punching && player_distance < PUNCH_RANGE && player_distance > 1.0e-4 {
            let to_citizen = Vec3::new(current.x - player.x, 0.0, current.z - player.z);
            let aim = to_citizen.normalize().dot(player_forward);
            if aim > PUNCH_CONE || player_distance < BODY_PUSH {
                impact_velocity = to_citizen.normalize() * PUNCH_KNOCK + Vec3::Y * KNOCK_LIFT;
                impact_damage = 1.0;
                player_incident = true;
            }
        }
        if impact_damage > 0.0 {
            damage_citizen(&mut citizen, impact_damage);
            citizen.knocked = KNOCK_SECONDS;
            citizen.knock_velocity = impact_velocity;
        }
        if citizen.knocked > 0.0 {
            let mut displacement = citizen.displacement;
            let mut velocity = citizen.knock_velocity;
            let mut knocked = citizen.knocked;
            resolve_knock(&mut displacement, &mut velocity, &mut knocked, dt);
            citizen.displacement = displacement;
            citizen.knock_velocity = velocity;
            citizen.knocked = knocked;
        }
        let fallen = if citizen.dead {
            1.0
        } else if citizen.knocked > 0.0 {
            (citizen.knocked / KNOCK_SECONDS).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let path_rotation = pedestrian_rotation(citizen.along_x, direction);
        transform.translation = base + citizen.displacement - Vec3::Y * (FALL_TILT * fallen);
        transform.rotation =
            path_rotation * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 * fallen);
    }

    wanted.tick(dt, player_incident);
    for (mut visibility, siren) in &mut siren_lights {
        *visibility = if police_siren_visible(wanted.level, siren.phase, now) {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    for (index, (mut transform, patrol)) in (&mut patrols).into_iter().enumerate() {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let tactics = if wanted.level > 0 {
            Some(police_tactics(wanted.level, index, now))
        } else {
            None
        };
        let target = if let Some(tactics) = tactics {
            if wanted.level >= 2 {
                player_flat
            } else {
                let to_player = player_flat - position;
                let side = Vec2::new(-to_player.y, to_player.x).normalize_or_zero();
                let flank_side = if index % 2 == 0 { 1.0 } else { -1.0 };
                let search_phase = now * (0.45 + wanted.level as f32 * 0.08) + patrol.phase;
                let search_offset =
                    Vec2::new(search_phase.cos(), search_phase.sin()) * tactics.search_radius;
                player_flat + side * (tactics.flank_distance * flank_side) + search_offset
            }
        } else {
            patrol.home
                + Vec2::new(
                    (now * 0.35 + patrol.phase).sin() * 8.0,
                    (now * 0.28 + patrol.phase).cos() * 5.0,
                )
        };
        let delta = target - position;
        let distance = delta.length();
        let stop_distance = if wanted.level >= 2 {
            9.0
        } else {
            tactics.map_or(5.5, |tactics| tactics.stop_distance)
        };
        if distance > stop_distance {
            let speed = tactics.map_or(2.4, |tactics| tactics.speed);
            let movement = delta.normalize() * (speed * dt).min(distance - stop_distance);
            let vehicle_yaw = movement.y.atan2(movement.x);
            let mut blockers = traffic_vehicle_bodies.clone();
            blockers.extend(
                patrol_vehicle_bodies
                    .iter()
                    .enumerate()
                    .filter_map(|(other, body)| (other != index).then_some(*body)),
            );
            let next = crate::physics::move_with_slide(position, movement, |point| {
                vehicle_city_collision(point, vehicle_yaw)
                    || police_vehicle_blocked(point, vehicle_yaw, &blockers, &player_contact_body)
            });
            let step = next - position;
            transform.translation.x = next.x;
            transform.translation.z = next.y;
            if step.length_squared() > 1.0e-5 {
                transform.rotation = Quat::from_rotation_y(step.x.atan2(step.y));
            }
            patrol_vehicle_bodies[index] =
                vehicle_body_from_forward(transform.translation, transform.rotation * Vec3::Z);
        }
        car_bodies.push((
            Vec2::new(transform.translation.x, transform.translation.z),
            CAR_BODY_RADIUS + 0.25,
        ));
    }

    let updated_patrol_positions: Vec<Vec2> = patrols
        .iter()
        .map(|(transform, _)| Vec2::new(transform.translation.x, transform.translation.z))
        .collect();
    let mut officer_bodies: Vec<Option<Body>> = officers
        .iter()
        .map(|(transform, officer, _)| {
            officer.active.then(|| {
                Body::person(Vec2::new(
                    transform.translation.x,
                    transform.translation.z,
                ))
            })
        })
        .collect();
    let mut shots_fired = Vec::new();
    for (officer_index, (mut transform, mut officer, mut visibility)) in
        (&mut officers).into_iter().enumerate()
    {
        if wanted.level < 2 {
            officer.active = false;
            officer_bodies[officer_index] = None;
            *visibility = Visibility::Hidden;
            continue;
        }
        if !officer.active {
            let patrol_position = updated_patrol_positions
                .get(officer.patrol_index)
                .copied()
                .unwrap_or(player_flat);
            let side = if officer.patrol_index % 2 == 0 {
                Vec2::X
            } else {
                -Vec2::X
            };
            transform.translation = Vec3::new(
                patrol_position.x + side.x * (CAR_BODY_RADIUS + OFFICER_BODY_RADIUS + 0.35),
                0.82,
                patrol_position.y + side.y * (CAR_BODY_RADIUS + OFFICER_BODY_RADIUS + 0.35),
            );
            officer.active = true;
        }
        *visibility = Visibility::Visible;

        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let to_player = player_flat - position;
        let distance = to_player.length();
        let movement = if distance > OFFICER_STOP_DISTANCE {
            to_player.normalize() * (OFFICER_SPEED * dt).min(distance - OFFICER_STOP_DISTANCE)
        } else if distance < OFFICER_RETREAT_DISTANCE {
            -to_player.normalize() * (OFFICER_SPEED * dt).min(OFFICER_RETREAT_DISTANCE - distance)
        } else {
            Vec2::ZERO
        };
        let mut next = position;
        if movement.length_squared() > 0.0 {
            let mut blockers = traffic_vehicle_bodies.clone();
            blockers.extend(patrol_vehicle_bodies.iter().copied());
            blockers.extend(officer_bodies.iter().enumerate().filter_map(|(other, body)| {
                (other != officer_index).then_some(*body).flatten()
            }));
            next = crate::physics::move_with_slide(position, movement, |point| {
                city_collision(point, OFFICER_BODY_RADIUS)
                    || officer_blocked(point, &blockers, &player_contact_body)
            });
        }
        transform.translation.x = next.x;
        transform.translation.z = next.y;
        officer_bodies[officer_index] = Some(Body::person(next));
        let facing = player_flat - next;
        if facing.length_squared() > 1.0e-5 {
            transform.rotation = Quat::from_rotation_y(facing.x.atan2(facing.y));
        }

        officer.shot_timer = (officer.shot_timer - dt).max(0.0);
        let officer_distance = next.distance(player_flat);
        let clear_line = police_line_of_sight_clear(next, player_flat);
        if officer.shot_timer <= 0.0 && can_police_shoot(officer_distance, clear_line) {
            damage_player(&mut probe, POLICE_SHOT_DAMAGE);
            officer.shot_timer = POLICE_SHOT_COOLDOWN;
            shots_fired.push(officer.patrol_index);
        }
    }
    for (mut visibility, mut flash) in &mut muzzle_flashes {
        if wanted.level < 2 {
            flash.remaining = 0.0;
        } else if shots_fired.contains(&flash.patrol_index) {
            flash.remaining = POLICE_MUZZLE_FLASH_SECONDS;
        } else {
            flash.remaining = (flash.remaining - dt).max(0.0);
        }
        *visibility = if flash.remaining > 0.0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
    bodies.cars = car_bodies;
    bodies.oriented = traffic_vehicle_bodies;
    bodies.oriented.extend(patrols.iter().map(|(transform, _)| {
        vehicle_body_from_forward(transform.translation, transform.rotation * Vec3::Z)
    }));
    bodies.oriented.extend(citizens.iter().filter_map(|(transform, citizen)| {
        (!citizen.dead).then(|| {
            Body::person(Vec2::new(
                transform.translation.x,
                transform.translation.z,
            ))
        })
    }));
    bodies.oriented.extend(officers.iter().filter_map(|(transform, officer, _)| {
        officer.active.then(|| {
            Body::person(Vec2::new(
                transform.translation.x,
                transform.translation.z,
            ))
        })
    }));
    probe.punch = false;

    let mut snapshot = WorldSnapshot {
        health: probe.health,
        wanted_level: wanted.level,
        wanted_timer: wanted.decay_timer,
        ..Default::default()
    };
    for (transform, car) in &mut traffic {
        snapshot.traffic.push(TrafficSave {
            position: transform.translation,
            speed: car.speed,
            direction: car.direction,
            along_x: car.along_x,
            lane_offset: car.lane_offset,
            avoid_offset: car.avoid_offset,
            stop_timer: car.stop_timer,
        });
    }
    for (_, citizen) in &mut citizens {
        snapshot.citizens.push(CitizenSave {
            origin: citizen.origin,
            phase: citizen.phase,
            speed: citizen.speed,
            range: citizen.range,
            along_x: citizen.along_x,
            displacement: citizen.displacement,
            knock_velocity: citizen.knock_velocity,
            knocked: citizen.knocked,
            health: citizen.health,
            dead: citizen.dead,
        });
    }
    for (transform, patrol) in &mut patrols {
        snapshot.patrols.push(PatrolSave {
            position: transform.translation,
            phase: patrol.phase,
        });
    }
    for (transform, officer, _) in &mut officers {
        snapshot.officers.push(OfficerSave {
            position: transform.translation,
            active: officer.active,
            shot_timer: officer.shot_timer,
        });
    }
    world.snapshot = snapshot;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn traffic_avoidance_uses_oriented_vehicle_contacts() {
        let parallel_lane = Body::vehicle(Vec2::new(0.0, 2.0), 0.0);
        assert!(!traffic_car_overlaps_body(
            Vec2::ZERO,
            true,
            1.0,
            &parallel_lane,
        ));

        let crossing_vehicle = Body::vehicle(
            Vec2::new(0.0, 1.8),
            std::f32::consts::FRAC_PI_2,
        );
        assert!(traffic_car_overlaps_body(
            Vec2::ZERO,
            true,
            1.0,
            &crossing_vehicle,
        ));

        let nearby_pedestrian = Body::person(Vec2::new(0.0, 1.2));
        assert!(traffic_car_overlaps_body(
            Vec2::ZERO,
            true,
            1.0,
            &nearby_pedestrian,
        ));
    }

    #[test]
    fn police_vehicle_and_officer_contacts_use_oriented_body_shapes() {
        let player = Body::person(Vec2::new(20.0, 20.0));
        let traffic_car = Body::vehicle(Vec2::new(0.0, 1.8), 0.0);
        assert!(police_vehicle_blocked(
            Vec2::ZERO,
            0.0,
            &[traffic_car],
            &player,
        ));
        assert!(!police_vehicle_blocked(
            Vec2::new(0.0, 10.0),
            0.0,
            &[traffic_car],
            &player,
        ));

        let nearby_car = Body::vehicle(Vec2::new(8.0, 0.0), 0.0);
        assert!(officer_blocked(
            Vec2::new(8.2, 0.0),
            &[nearby_car],
            &player,
        ));
        assert!(!officer_blocked(Vec2::new(12.0, 0.0), &[nearby_car], &player));
    }

    #[test]
    fn overlapping_traffic_vehicles_are_separated_symmetrically_by_obb() {
        let mut bodies = [
            Body::vehicle(Vec2::new(0.0, 0.0), 0.0),
            Body::vehicle(Vec2::new(0.0, 1.4), std::f32::consts::FRAC_PI_2),
        ];
        assert!(bodies_overlap(&bodies[0], &bodies[1]));

        let moved = resolve_traffic_body_pairs(&mut bodies);

        assert!(moved.iter().all(|did_move| *did_move));
        assert!(!bodies_overlap(&bodies[0], &bodies[1]));
        assert!(bodies.iter().all(|body| body.center.is_finite()));
    }

    #[test]
    fn traffic_pair_separation_uses_the_free_side_when_building_blocks_a_split() {
        let yaw = std::f32::consts::FRAC_PI_2;
        let mut bodies = [
            Body::vehicle(Vec2::new(7.5, 8.0), yaw),
            Body::vehicle(Vec2::new(9.0, 8.0), yaw),
        ];
        assert!(bodies_overlap(&bodies[0], &bodies[1]));
        assert!(bodies
            .iter()
            .all(|body| !vehicle_city_collision(body.center, body.yaw)));

        let moved = resolve_traffic_body_pairs(&mut bodies);
        assert!(moved[0]);
        assert!(!moved[1]);
        assert!(!bodies_overlap(&bodies[0], &bodies[1]));
        assert!(bodies
            .iter()
            .all(|body| !vehicle_city_collision(body.center, body.yaw)));

        let settled = bodies;
        for _ in 0..5 {
            assert!(resolve_traffic_body_pairs(&mut bodies).iter().all(|did_move| !did_move));
            assert_eq!(bodies, settled);
        }
    }
}
