use bevy::prelude::*;

use super::{PlayerProbe, TrafficBodies, WantedState};

const TRAFFIC_TARGET_SECONDS: f32 = 20.0;
const EVENT_LIMIT_SECONDS: f32 = 90.0;
const EVENT_COOLDOWN_SECONDS: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventKind {
    EscapePolice,
    TrafficDrive,
}

struct ActiveEvent {
    kind: EventKind,
    remaining: f32,
    progress: f32,
    initial_health: f32,
}

#[derive(Resource, Default)]
pub(super) struct CityEvents {
    active: Option<ActiveEvent>,
    cooldown: f32,
}

#[derive(Clone, Copy)]
struct EventInput {
    wanted: u8,
    driving: bool,
    speed: f32,
    near_traffic: bool,
    health: f32,
}

#[derive(Debug, PartialEq, Eq)]
enum EventSignal {
    Started(EventKind),
    Completed(EventKind),
    Failed(EventKind),
}

fn advance_event(state: &mut CityEvents, input: EventInput, dt: f32) -> Option<EventSignal> {
    if dt <= 0.0 {
        return None;
    }
    if let Some(event) = state.active.as_mut() {
        event.remaining = (event.remaining - dt).max(0.0);
        let kind = event.kind;
        let failed = event.remaining <= 0.0
            || input.health <= 0.0
            || (kind == EventKind::TrafficDrive
                && (!input.driving || input.wanted > 0 || input.health < event.initial_health));
        let completed = match kind {
            EventKind::EscapePolice => input.wanted == 0,
            EventKind::TrafficDrive => {
                if input.speed.abs() >= 4.0 {
                    event.progress += dt;
                }
                event.progress >= TRAFFIC_TARGET_SECONDS
            }
        };
        if failed || completed {
            state.active = None;
            state.cooldown = EVENT_COOLDOWN_SECONDS;
            return Some(if failed {
                EventSignal::Failed(kind)
            } else {
                EventSignal::Completed(kind)
            });
        }
        return None;
    }
    state.cooldown = (state.cooldown - dt).max(0.0);
    if state.cooldown > 0.0 || input.health <= 0.0 {
        return None;
    }
    let kind = if input.wanted >= 2 {
        Some(EventKind::EscapePolice)
    } else if input.wanted == 0 && input.driving && input.speed.abs() >= 4.0 && input.near_traffic {
        Some(EventKind::TrafficDrive)
    } else {
        None
    }?;
    state.active = Some(ActiveEvent {
        kind,
        remaining: EVENT_LIMIT_SECONDS,
        progress: 0.0,
        initial_health: input.health,
    });
    Some(EventSignal::Started(kind))
}

pub(super) fn city_event_system(
    time: Res<Time>,
    wanted: Res<WantedState>,
    traffic: Res<TrafficBodies>,
    mut probe: ResMut<PlayerProbe>,
    mut game: ResMut<crate::Game>,
    mut events: ResMut<CityEvents>,
) {
    if !game.started {
        *events = CityEvents::default();
        return;
    }
    if game.paused {
        return;
    }
    let position = Vec2::new(game.car.x, game.car.z);
    let input = EventInput {
        wanted: wanted.level,
        driving: game.driving,
        speed: game.speed,
        near_traffic: traffic.cars.iter().any(|(car, _)| car.distance(position) < 18.0),
        health: probe.health,
    };
    if let Some(signal) = advance_event(&mut events, input, time.delta_secs().min(0.05)) {
        game.notice = match signal {
            EventSignal::Started(EventKind::EscapePolice) => {
                "Событие: скройся от полиции за 90 секунд".to_string()
            }
            EventSignal::Started(EventKind::TrafficDrive) => {
                "Событие: проедь 20 секунд без ранений и розыска".to_string()
            }
            EventSignal::Completed(_) => {
                probe.health = (probe.health + 20.0).min(100.0);
                "Событие выполнено: здоровье +20".to_string()
            }
            EventSignal::Failed(_) => "Событие не выполнено".to_string(),
        };
        game.notice_timer = 5.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn input() -> EventInput {
        EventInput {
            wanted: 0,
            driving: true,
            speed: 8.0,
            near_traffic: true,
            health: 80.0,
        }
    }

    #[test]
    fn traffic_event_rewards_clean_driving_and_honors_cooldown() {
        let mut state = CityEvents::default();
        assert_eq!(advance_event(&mut state, input(), 0.1), Some(EventSignal::Started(EventKind::TrafficDrive)));
        assert_eq!(advance_event(&mut state, input(), 19.0), None);
        assert_eq!(advance_event(&mut state, input(), 1.0), Some(EventSignal::Completed(EventKind::TrafficDrive)));
        assert_eq!(advance_event(&mut state, input(), 1.0), None);
    }

    #[test]
    fn traffic_event_fails_on_damage_or_wanted_level() {
        for change in 0..2 {
            let mut state = CityEvents::default();
            advance_event(&mut state, input(), 0.1);
            let mut next = input();
            if change == 0 { next.health = 79.0; } else { next.wanted = 1; }
            assert_eq!(advance_event(&mut state, next, 0.1), Some(EventSignal::Failed(EventKind::TrafficDrive)));
        }
    }

    #[test]
    fn police_event_starts_at_two_stars_and_ends_when_wanted_clears() {
        let mut state = CityEvents::default();
        let mut next = input();
        next.wanted = 2;
        assert_eq!(advance_event(&mut state, next, 0.1), Some(EventSignal::Started(EventKind::EscapePolice)));
        next.wanted = 1;
        assert_eq!(advance_event(&mut state, next, 1.0), None);
        next.wanted = 0;
        assert_eq!(advance_event(&mut state, next, 1.0), Some(EventSignal::Completed(EventKind::EscapePolice)));
    }

    #[test]
    fn expired_event_fails_even_if_wanted_clears_at_the_deadline() {
        let mut state = CityEvents::default();
        let mut next = input();
        next.wanted = 2;
        advance_event(&mut state, next, 0.1);
        next.wanted = 0;
        assert_eq!(advance_event(&mut state, next, EVENT_LIMIT_SECONDS), Some(EventSignal::Failed(EventKind::EscapePolice)));
    }
}
