use bevy::prelude::*;

use super::{PlayerProbe, TrafficBodies, WantedState};

const TRAFFIC_TARGET_SECONDS: f32 = 20.0;
const SPEED_TARGET_SECONDS: f32 = 12.0;
const SPEED_MIN_METERS_PER_SECOND: f32 = 12.0;
const EVENT_LIMIT_SECONDS: f32 = 90.0;
const EVENT_COOLDOWN_SECONDS: f32 = 60.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EventKind {
    EscapePolice,
    TrafficDrive,
    SpeedDrive,
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
                && (!input.driving || input.wanted > 0 || input.health < event.initial_health))
            || (kind == EventKind::SpeedDrive
                && (!input.driving || input.wanted > 0 || input.health < event.initial_health));
        let completed = match kind {
            EventKind::EscapePolice => input.wanted == 0,
            EventKind::TrafficDrive => {
                if input.speed.abs() >= 4.0 {
                    event.progress += dt;
                }
                event.progress >= TRAFFIC_TARGET_SECONDS
            }
            EventKind::SpeedDrive => {
                if input.speed.abs() >= SPEED_MIN_METERS_PER_SECOND && input.near_traffic {
                    event.progress += dt;
                }
                event.progress >= SPEED_TARGET_SECONDS
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
    } else if input.wanted == 0
        && input.driving
        && input.speed.abs() >= SPEED_MIN_METERS_PER_SECOND
        && input.near_traffic
    {
        Some(EventKind::SpeedDrive)
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

fn display_seconds_ceil(remaining: f32) -> u32 {
    let remaining = remaining.max(0.0);
    let nearest_second = remaining.round();
    let tolerance = remaining.max(1.0) * f32::EPSILON * 4.0;
    if nearest_second >= 1.0 && (remaining - nearest_second).abs() <= tolerance {
        nearest_second as u32
    } else {
        remaining.ceil() as u32
    }
}

fn active_event_notice(state: &CityEvents) -> Option<String> {
    let event = state.active.as_ref()?;
    let remaining = display_seconds_ceil(event.remaining);
    match event.kind {
        EventKind::EscapePolice => Some(format!(
            "Событие: скройся от полиции • осталось {remaining} с"
        )),
        EventKind::TrafficDrive => Some(format!(
            "Событие: чистая езда {}/20 с • осталось {remaining} с",
            event.progress.floor() as u32,
        )),
        EventKind::SpeedDrive => Some(format!(
            "Событие: скоростная езда у трафика {}/12 с • осталось {remaining} с",
            event.progress.floor() as u32,
        )),
    }
}

fn apply_completion_reward(health: &mut f32) -> String {
    let current = health.clamp(0.0, 100.0);
    let restored = (100.0 - current).min(20.0);
    *health = current + restored;
    if restored > f32::EPSILON {
        format!("Событие выполнено: здоровье +{restored:.1}")
    } else {
        "Событие выполнено: здоровье уже полное".to_string()
    }
}

fn apply_speed_reward(credits: &mut u32) -> String {
    *credits = credits.saturating_add(100);
    "Событие выполнено: +100 кредитов".to_string()
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
            EventSignal::Started(EventKind::SpeedDrive) => {
                "Событие: продержись 12 секунд на скорости у трафика".to_string()
            }
            EventSignal::Completed(EventKind::SpeedDrive) => {
                apply_speed_reward(&mut game.credits)
            }
            EventSignal::Completed(_) => apply_completion_reward(&mut probe.health),
            EventSignal::Failed(_) => "Событие не выполнено".to_string(),
        };
        game.notice_timer = 5.0;
    } else if let Some(status) = active_event_notice(&events) {
        game.notice = status;
        game.notice_timer = 0.1;
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
        assert_eq!(
            advance_event(&mut state, input(), 0.1),
            Some(EventSignal::Started(EventKind::TrafficDrive))
        );
        assert_eq!(advance_event(&mut state, input(), 19.0), None);
        assert_eq!(
            advance_event(&mut state, input(), 1.0),
            Some(EventSignal::Completed(EventKind::TrafficDrive))
        );
        assert_eq!(advance_event(&mut state, input(), 1.0), None);
    }

    #[test]
    fn speed_event_requires_clean_fast_driving_near_traffic_and_rewards_credits() {
        let mut state = CityEvents::default();
        let mut fast = input();
        fast.speed = 14.0;
        assert_eq!(
            advance_event(&mut state, fast, 0.1),
            Some(EventSignal::Started(EventKind::SpeedDrive))
        );
        assert_eq!(
            active_event_notice(&state).as_deref(),
            Some("Событие: скоростная езда у трафика 0/12 с • осталось 90 с")
        );

        let mut outside_traffic = fast;
        outside_traffic.near_traffic = false;
        assert_eq!(advance_event(&mut state, outside_traffic, 5.0), None);
        assert!(active_event_notice(&state)
            .unwrap()
            .contains("0/12 с • осталось 85 с"));
        assert_eq!(advance_event(&mut state, fast, 11.9), None);
        assert_eq!(
            advance_event(&mut state, fast, 0.1),
            Some(EventSignal::Completed(EventKind::SpeedDrive))
        );

        let mut credits = 200;
        assert_eq!(apply_speed_reward(&mut credits), "Событие выполнено: +100 кредитов");
        assert_eq!(credits, 300);
    }

    #[test]
    fn speed_event_fails_on_damage_wanted_level_or_leaving_the_car() {
        for change in 0..3 {
            let mut state = CityEvents::default();
            let mut fast = input();
            fast.speed = 14.0;
            advance_event(&mut state, fast, 0.1);
            let mut next = fast;
            match change {
                0 => next.health = 79.0,
                1 => next.wanted = 1,
                _ => next.driving = false,
            }
            assert_eq!(
                advance_event(&mut state, next, 0.1),
                Some(EventSignal::Failed(EventKind::SpeedDrive))
            );
        }
    }

    #[test]
    fn remaining_event_seconds_ignore_only_float_precision_noise() {
        assert_eq!(display_seconds_ceil(85.000_01), 85);
        assert_eq!(display_seconds_ceil(85.01), 86);
        assert_eq!(display_seconds_ceil(0.000_01), 1);
        assert_eq!(display_seconds_ceil(0.0), 0);
    }

    #[test]
    fn active_event_notice_shows_progress_and_remaining_time() {
        let mut state = CityEvents::default();
        advance_event(&mut state, input(), 0.1);
        assert_eq!(
            active_event_notice(&state).as_deref(),
            Some("Событие: чистая езда 0/20 с • осталось 90 с")
        );

        advance_event(&mut state, input(), 4.9);
        assert_eq!(
            active_event_notice(&state).as_deref(),
            Some("Событие: чистая езда 4/20 с • осталось 86 с")
        );

        let mut police = input();
        police.wanted = 2;
        let mut state = CityEvents::default();
        advance_event(&mut state, police, 0.1);
        assert_eq!(
            active_event_notice(&state).as_deref(),
            Some("Событие: скройся от полиции • осталось 90 с")
        );
    }

    #[test]
    fn event_reward_reports_only_health_actually_restored() {
        let mut health = 80.0;
        assert_eq!(
            apply_completion_reward(&mut health),
            "Событие выполнено: здоровье +20.0"
        );
        assert_eq!(health, 100.0);

        let mut health = 95.0;
        assert_eq!(
            apply_completion_reward(&mut health),
            "Событие выполнено: здоровье +5.0"
        );
        assert_eq!(health, 100.0);

        let mut health = 100.0;
        assert_eq!(
            apply_completion_reward(&mut health),
            "Событие выполнено: здоровье уже полное"
        );
        assert_eq!(health, 100.0);
    }

    #[test]
    fn traffic_event_fails_on_damage_or_wanted_level() {
        for change in 0..2 {
            let mut state = CityEvents::default();
            advance_event(&mut state, input(), 0.1);
            let mut next = input();
            if change == 0 {
                next.health = 79.0;
            } else {
                next.wanted = 1;
            }
            assert_eq!(
                advance_event(&mut state, next, 0.1),
                Some(EventSignal::Failed(EventKind::TrafficDrive))
            );
        }
    }

    #[test]
    fn police_event_starts_at_two_stars_and_ends_when_wanted_clears() {
        let mut state = CityEvents::default();
        let mut next = input();
        next.wanted = 2;
        assert_eq!(
            advance_event(&mut state, next, 0.1),
            Some(EventSignal::Started(EventKind::EscapePolice))
        );
        next.wanted = 1;
        assert_eq!(advance_event(&mut state, next, 1.0), None);
        next.wanted = 0;
        assert_eq!(
            advance_event(&mut state, next, 1.0),
            Some(EventSignal::Completed(EventKind::EscapePolice))
        );
    }

    #[test]
    fn expired_event_fails_even_if_wanted_clears_at_the_deadline() {
        let mut state = CityEvents::default();
        let mut next = input();
        next.wanted = 2;
        advance_event(&mut state, next, 0.1);
        next.wanted = 0;
        assert_eq!(
            advance_event(&mut state, next, EVENT_LIMIT_SECONDS),
            Some(EventSignal::Failed(EventKind::EscapePolice))
        );
    }
}
