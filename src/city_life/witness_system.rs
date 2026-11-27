use bevy::prelude::*;

use super::*;

const INCIDENT_WITNESS_RADIUS: f32 = 32.0;
const INCIDENT_VICTIM_EXCLUSION_RADIUS: f32 = 3.5;

#[derive(Resource, Default)]
struct WantedWitnessSnapshot {
    level: u8,
    decay_timer: f32,
    captured: bool,
    restoring: bool,
}

fn capture_wanted_before_incident(
    wanted: Res<WantedState>,
    restore: Res<WorldRestore>,
    mut snapshot: ResMut<WantedWitnessSnapshot>,
) {
    snapshot.level = wanted.level;
    snapshot.decay_timer = wanted.decay_timer;
    snapshot.captured = true;
    snapshot.restoring = restore.0.is_some();
}

fn witnessed_player_incident(
    player: Vec2,
    citizens: &Query<(&Transform, &Citizen)>,
    traffic: &Query<(&Transform, &TrafficCar)>,
) -> bool {
    if !player.is_finite() {
        return false;
    }
    let radius_squared = INCIDENT_WITNESS_RADIUS * INCIDENT_WITNESS_RADIUS;
    let victim_exclusion_squared =
        INCIDENT_VICTIM_EXCLUSION_RADIUS * INCIDENT_VICTIM_EXCLUSION_RADIUS;
    citizens.iter().any(|(transform, citizen)| {
        let position = Vec2::new(transform.translation.x, transform.translation.z);
        let distance_squared = position.distance_squared(player);
        !citizen.dead
            && distance_squared > victim_exclusion_squared
            && distance_squared <= radius_squared
    }) || traffic.iter().any(|(transform, _)| {
        Vec2::new(transform.translation.x, transform.translation.z).distance_squared(player)
            <= radius_squared
    })
}

fn suppress_unwitnessed_wanted_increase(
    mut wanted: ResMut<WantedState>,
    probe: Res<PlayerProbe>,
    mut snapshot: ResMut<WantedWitnessSnapshot>,
    citizens: Query<(&Transform, &Citizen)>,
    traffic: Query<(&Transform, &TrafficCar)>,
) {
    if !snapshot.captured {
        return;
    }
    if !snapshot.restoring && wanted.level > snapshot.level {
        let player = Vec2::new(probe.position.x, probe.position.z);
        if !witnessed_player_incident(player, &citizens, &traffic) {
            wanted.level = snapshot.level;
            wanted.decay_timer = snapshot.decay_timer;
        }
    }
    snapshot.captured = false;
    snapshot.restoring = false;
}

#[cfg(test)]
mod witness_tests {
    use super::*;
    use bevy::ecs::system::RunSystemOnce;

    fn witness_world() -> World {
        let mut world = World::new();
        world.insert_resource(PlayerProbe::default());
        world.insert_resource(WantedState {
            level: 1,
            decay_timer: 9.0,
        });
        world.init_resource::<WorldRestore>();
        world.init_resource::<WantedWitnessSnapshot>();
        world
    }

    fn record_incident(world: &mut World) {
        world
            .run_system_once(capture_wanted_before_incident)
            .expect("wanted state is captured before checking an incident");
        let mut wanted = world.resource_mut::<WantedState>();
        wanted.level = 2;
        wanted.decay_timer = 18.0;
    }

    fn spawn_citizen_witness(world: &mut World, position: Vec3) {
        world.spawn((
            Transform::from_translation(position),
            Citizen {
                origin: position,
                phase: 0.0,
                speed: 0.5,
                range: 7.0,
                along_x: true,
                displacement: Vec3::ZERO,
                knock_velocity: Vec3::ZERO,
                knocked: 0.0,
                health: CITIZEN_HEALTH,
                dead: false,
            },
        ));
    }

    fn spawn_traffic_witness(world: &mut World, position: Vec3) {
        world.spawn((
            Transform::from_translation(position),
            TrafficCar {
                speed: 6.0,
                direction: 1.0,
                along_x: true,
                stop_timer: 0.0,
                lane_offset: 0.0,
                avoid_offset: 0.0,
            },
        ));
    }

    #[test]
    fn an_unwitnessed_incident_does_not_raise_wanted_or_reset_decay() {
        let mut world = witness_world();
        record_incident(&mut world);
        world
            .run_system_once(suppress_unwitnessed_wanted_increase)
            .expect("an incident without nearby witnesses is suppressed");
        let wanted = world.resource::<WantedState>();
        assert_eq!(wanted.level, 1);
        assert_eq!(wanted.decay_timer, 9.0);
    }

    #[test]
    fn a_nearby_person_witness_keeps_the_wanted_increase() {
        let mut world = witness_world();
        spawn_citizen_witness(&mut world, Vec3::new(10.0, 0.82, 0.0));
        record_incident(&mut world);
        world
            .run_system_once(suppress_unwitnessed_wanted_increase)
            .expect("a nearby person witnesses the incident");
        let wanted = world.resource::<WantedState>();
        assert_eq!(wanted.level, 2);
        assert_eq!(wanted.decay_timer, 18.0);
    }

    #[test]
    fn a_nearby_traffic_witness_keeps_the_wanted_increase() {
        let mut world = witness_world();
        spawn_traffic_witness(&mut world, Vec3::new(18.0, 0.48, 0.0));
        record_incident(&mut world);
        world
            .run_system_once(suppress_unwitnessed_wanted_increase)
            .expect("a nearby traffic driver witnesses the incident");
        let wanted = world.resource::<WantedState>();
        assert_eq!(wanted.level, 2);
        assert_eq!(wanted.decay_timer, 18.0);
    }

    #[test]
    fn restoring_a_v1_wanted_level_is_not_treated_as_a_new_incident() {
        let mut world = witness_world();
        world.resource_mut::<WorldRestore>().0 = Some(WorldSnapshot {
            wanted_level: 3,
            wanted_timer: 12.0,
            ..default()
        });
        world
            .run_system_once(capture_wanted_before_incident)
            .expect("restore state is recorded before simulation");
        world.resource_mut::<WantedState>().level = 3;
        world
            .run_system_once(suppress_unwitnessed_wanted_increase)
            .expect("a restored V1 wanted level is preserved without nearby witnesses");
        assert_eq!(world.resource::<WantedState>().level, 3);
    }
}
