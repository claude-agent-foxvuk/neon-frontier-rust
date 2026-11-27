use std::collections::HashMap;

use bevy::ecs::system::RunSystemOnce;
use bevy::prelude::*;
use std::time::Duration;

use super::*;

const OFFICER_REINFORCEMENT_DELAY: f32 = 4.5;
const OFFICER_REINFORCEMENT_SAVE_MARKER: f32 = 4.5;

#[derive(Clone, Copy)]
struct PendingOfficerReinforcement {
    patrol_index: usize,
    remaining: f32,
}

#[derive(Resource, Default)]
struct OfficerReinforcementState {
    pending: HashMap<Entity, PendingOfficerReinforcement>,
}

fn is_saved_reinforcement(active: bool, shot_timer: f32) -> bool {
    !active
        && shot_timer.is_finite()
        && (shot_timer - OFFICER_REINFORCEMENT_SAVE_MARKER).abs() <= 0.001
}

fn advance_officer_reinforcements(
    time: Res<Time>,
    restore: Res<WorldRestore>,
    mut state: ResMut<OfficerReinforcementState>,
    traffic: Query<(), With<TrafficCar>>,
    citizens: Query<(), With<Citizen>>,
    patrols: Query<(), With<PolicePatrol>>,
    mut officers: Query<
        (Entity, &mut Transform, &mut FootOfficer, &mut Visibility),
        (
            Without<TrafficCar>,
            Without<Citizen>,
            Without<PolicePatrol>,
            Without<PoliceSirenLight>,
        ),
    >,
) {
    if let Some(snapshot) = restore.0.as_ref() {
        if crate::save::restore_counts_match(
            snapshot,
            traffic.iter().count(),
            citizens.iter().count(),
            patrols.iter().count(),
            officers.iter().count(),
        ) {
            for ((entity, _, mut officer, mut visibility), saved) in
                officers.iter_mut().zip(snapshot.officers.iter())
            {
                if !is_saved_reinforcement(saved.active, saved.shot_timer) {
                    continue;
                }
                let patrol_index = state
                    .pending
                    .get(&entity)
                    .map_or(officer.patrol_index, |pending| pending.patrol_index);
                state.pending.insert(
                    entity,
                    PendingOfficerReinforcement {
                        patrol_index,
                        remaining: OFFICER_REINFORCEMENT_DELAY,
                    },
                );
                officer.patrol_index = usize::MAX;
                officer.active = false;
                officer.shot_timer = OFFICER_REINFORCEMENT_SAVE_MARKER;
                *visibility = Visibility::Hidden;
            }
        }
    }

    let dt = time.delta_secs().clamp(0.0, 0.05);
    let mut completed = Vec::new();
    for (entity, pending) in &mut state.pending {
        let Ok((_, _, mut officer, mut visibility)) = officers.get_mut(*entity) else {
            completed.push(*entity);
            continue;
        };
        pending.remaining = (pending.remaining - dt).max(0.0);
        if pending.remaining <= 0.0 {
            officer.patrol_index = pending.patrol_index;
            officer.active = false;
            officer.shot_timer = 0.0;
            *visibility = Visibility::Hidden;
            completed.push(*entity);
        } else {
            officer.patrol_index = usize::MAX;
            officer.active = false;
            officer.shot_timer = OFFICER_REINFORCEMENT_SAVE_MARKER;
            *visibility = Visibility::Hidden;
        }
    }
    for entity in completed {
        state.pending.remove(&entity);
    }
}

fn register_officer_casualties(
    game: Option<Res<crate::Game>>,
    probe: Res<PlayerProbe>,
    mut wanted: ResMut<WantedState>,
    mut state: ResMut<OfficerReinforcementState>,
    mut officers: Query<
        (Entity, &Transform, &mut FootOfficer, &mut Visibility),
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
    let player_position = Vec2::new(probe.position.x, probe.position.z);
    let player_forward = if probe.forward.length_squared() > 1.0e-6 {
        probe.forward.normalize()
    } else {
        Vec3::NEG_Z
    };
    let player_vehicle = probe
        .in_vehicle
        .then(|| vehicle_body_from_forward(probe.position, player_forward));

    for (entity, transform, mut officer, mut visibility) in &mut officers {
        if !officer.active || state.pending.contains_key(&entity) {
            continue;
        }
        let officer_position = Vec2::new(transform.translation.x, transform.translation.z);
        let officer_body = Body::person(officer_position);
        let vehicle_hit = player_vehicle
            .as_ref()
            .is_some_and(|vehicle| bodies_overlap(vehicle, &officer_body));
        let to_officer = officer_position - player_position;
        let distance = to_officer.length();
        let punch_hit = !probe.in_vehicle
            && probe.punch
            && distance > 1.0e-4
            && distance < PUNCH_RANGE
            && (Vec3::new(to_officer.x, 0.0, to_officer.y)
                .normalize()
                .dot(player_forward)
                > PUNCH_CONE
                || distance < BODY_PUSH);
        if !vehicle_hit && !punch_hit {
            continue;
        }

        let patrol_index = officer.patrol_index;
        officer.patrol_index = usize::MAX;
        officer.active = false;
        officer.shot_timer = OFFICER_REINFORCEMENT_SAVE_MARKER;
        *visibility = Visibility::Hidden;
        state.pending.insert(
            entity,
            PendingOfficerReinforcement {
                patrol_index,
                remaining: OFFICER_REINFORCEMENT_DELAY,
            },
        );
        wanted.report_incident();
    }
}

#[cfg(test)]
mod officer_reinforcement_tests {
    use super::*;
    use super::sim::PoliceSearchMemory;

    #[test]
    fn v1_officer_timer_marker_is_only_a_hidden_casualty() {
        assert!(is_saved_reinforcement(false, OFFICER_REINFORCEMENT_SAVE_MARKER));
        assert!(!is_saved_reinforcement(true, OFFICER_REINFORCEMENT_SAVE_MARKER));
        assert!(!is_saved_reinforcement(false, 1.25));
        assert!(!is_saved_reinforcement(false, f32::NAN));
    }

    #[test]
    fn vehicle_impact_reports_an_incident_once_and_starts_reinforcement() {
        let mut world = World::new();
        let mut game = crate::Game::default();
        game.started = true;
        world.insert_resource(game);
        world.insert_resource(PlayerProbe {
            position: Vec3::ZERO,
            vehicle_position: Vec3::ZERO,
            forward: Vec3::NEG_Z,
            in_vehicle: true,
            punch: false,
            health: 100.0,
        });
        let mut wanted = WantedState::default();
        wanted.level = 1;
        world.insert_resource(wanted);
        world.init_resource::<OfficerReinforcementState>();
        let officer = world
            .spawn((
                Transform::from_xyz(0.9, 0.82, 0.0),
                FootOfficer {
                    patrol_index: 0,
                    active: true,
                    shot_timer: 0.0,
                },
                Visibility::Visible,
            ))
            .id();

        world
            .run_system_once(register_officer_casualties)
            .expect("vehicle impact reports a police casualty");
        assert_eq!(world.resource::<WantedState>().level, 2);
        assert!(!world.get::<FootOfficer>(officer).unwrap().active);
        assert_eq!(
            world.get::<FootOfficer>(officer).unwrap().shot_timer,
            OFFICER_REINFORCEMENT_SAVE_MARKER
        );
        assert_eq!(*world.get::<Visibility>(officer).unwrap(), Visibility::Hidden);
        assert_eq!(world.resource::<OfficerReinforcementState>().pending.len(), 1);

        world
            .run_system_once(register_officer_casualties)
            .expect("the casualty is not reported twice");
        assert_eq!(world.resource::<WantedState>().level, 2);
    }

    #[test]
    fn saved_v1_casualty_resumes_its_reinforcement_delay() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_millis(16));
        world.insert_resource(time);
        let mut snapshot = WorldSnapshot::default();
        snapshot.officers.push(OfficerSave {
            position: Vec3::new(12.0, 0.82, -4.0),
            active: false,
            shot_timer: OFFICER_REINFORCEMENT_SAVE_MARKER,
        });
        world.insert_resource(WorldRestore(Some(snapshot)));
        world.init_resource::<OfficerReinforcementState>();
        let officer = world
            .spawn((
                Transform::from_xyz(0.0, 0.82, 0.0),
                FootOfficer {
                    patrol_index: 1,
                    active: true,
                    shot_timer: 0.0,
                },
                Visibility::Visible,
            ))
            .id();

        world
            .run_system_once(advance_officer_reinforcements)
            .expect("a V1 casualty marker restores a pending reinforcement");

        let restored = world.get::<FootOfficer>(officer).unwrap();
        assert_eq!(restored.patrol_index, usize::MAX);
        assert!(!restored.active);
        assert_eq!(restored.shot_timer, OFFICER_REINFORCEMENT_SAVE_MARKER);
        assert!(world
            .resource::<OfficerReinforcementState>()
            .pending
            .contains_key(&officer));
    }

    #[test]
    fn reinforcement_remains_hidden_until_its_delay_expires() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_millis(50));
        world.insert_resource(time);
        world.insert_resource(WorldRestore::default());
        world.init_resource::<OfficerReinforcementState>();
        let officer = world
            .spawn((
                Transform::from_xyz(8.0, 0.82, 0.0),
                FootOfficer {
                    patrol_index: usize::MAX,
                    active: false,
                    shot_timer: OFFICER_REINFORCEMENT_SAVE_MARKER,
                },
                Visibility::Hidden,
            ))
            .id();
        world
            .resource_mut::<OfficerReinforcementState>()
            .pending
            .insert(
                officer,
                PendingOfficerReinforcement {
                    patrol_index: 0,
                    remaining: OFFICER_REINFORCEMENT_DELAY,
                },
            );

        for _ in 0..89 {
            world
                .run_system_once(advance_officer_reinforcements)
                .expect("reinforcement timer advances without exposing the officer");
            let state = world.resource::<OfficerReinforcementState>();
            assert!(state.pending.contains_key(&officer));
            assert_eq!(*world.get::<Visibility>(officer).unwrap(), Visibility::Hidden);
            world
                .resource_mut::<Time<()>>()
                .advance_by(Duration::from_millis(50));
        }
        for _ in 0..2 {
            world
                .run_system_once(advance_officer_reinforcements)
                .expect("reinforcement arrives after its delay");
            world
                .resource_mut::<Time<()>>()
                .advance_by(Duration::from_millis(50));
        }

        let officer_state = world.get::<FootOfficer>(officer).unwrap();
        assert_eq!(officer_state.patrol_index, 0);
        assert!(!officer_state.active);
        assert_eq!(officer_state.shot_timer, 0.0);
        assert!(!world
            .resource::<OfficerReinforcementState>()
            .pending
            .contains_key(&officer));
    }

    #[test]
    fn a_casualty_gets_a_visible_replacement_in_the_live_simulation() {
        let mut world = World::new();
        let mut time = Time::<()>::default();
        time.advance_by(Duration::from_millis(50));
        world.insert_resource(time);
        let mut game = crate::Game::default();
        game.started = true;
        world.insert_resource(game);
        world.insert_resource(PlayerProbe {
            position: Vec3::new(0.0, 0.9, 0.0),
            vehicle_position: Vec3::new(0.0, 0.65, 0.0),
            forward: Vec3::NEG_Z,
            in_vehicle: true,
            punch: false,
            health: 100.0,
        });
        let mut wanted = WantedState::default();
        wanted.level = 1;
        world.insert_resource(wanted);
        world.init_resource::<TrafficBodies>();
        world.init_resource::<WorldState>();
        world.init_resource::<WorldRestore>();
        world.init_resource::<OfficerReinforcementState>();
        world.init_resource::<PoliceSearchMemory>();
        world.spawn((
            Transform::from_xyz(-20.0, 0.48, 0.0),
            PolicePatrol {
                home: Vec2::new(-20.0, 0.0),
                phase: 0.0,
            },
        ));
        let officer = world
            .spawn((
                Transform::from_xyz(0.9, 0.82, 0.0),
                FootOfficer {
                    patrol_index: 0,
                    active: true,
                    shot_timer: 0.0,
                },
                Visibility::Visible,
            ))
            .id();

        world
            .run_system_once(advance_officer_reinforcements)
            .expect("no casualty is pending before the player hits the officer");
        world
            .run_system_once(register_officer_casualties)
            .expect("the player impact registers the officer casualty");
        world
            .run_system_once(simulate_city_life)
            .expect("the live city sim hides the fallen officer");
        assert_eq!(world.resource::<WantedState>().level, 2);
        assert!(!world.get::<FootOfficer>(officer).unwrap().active);
        assert_eq!(*world.get::<Visibility>(officer).unwrap(), Visibility::Hidden);

        let mut replacement_arrived = false;
        for _ in 0..100 {
            world
                .resource_mut::<Time<()>>()
                .advance_by(Duration::from_millis(50));
            world
                .run_system_once(advance_officer_reinforcements)
                .expect("the delayed response advances each live tick");
            world
                .run_system_once(register_officer_casualties)
                .expect("a pending casualty is not counted twice");
            world
                .run_system_once(simulate_city_life)
                .expect("the live simulation deploys the replacement");
            if world.get::<FootOfficer>(officer).unwrap().active
                && *world.get::<Visibility>(officer).unwrap() == Visibility::Visible
            {
                replacement_arrived = true;
                break;
            }
        }

        assert!(
            replacement_arrived,
            "the replacement remains hidden after the 4.5-second response delay"
        );
        assert_eq!(world.get::<FootOfficer>(officer).unwrap().patrol_index, 0);
        assert_eq!(world.resource::<WantedState>().level, 2);
    }
}
