use bevy::prelude::*;
use std::fmt::Write as _;
use std::io;

use crate::city_life::{CitizenSave, OfficerSave, PatrolSave, TrafficSave, WorldSnapshot};

pub const SAVE_FILE_NAME: &str = "neon_frontier_save.txt";

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub player: Vec3,
    pub car: Vec3,
    pub yaw: f32,
    pub speed: f32,
    pub stage: usize,
    pub credits: u32,
    pub driving: bool,
    pub bloom: bool,
    pub world: WorldSnapshot,
}

fn record(values: &[f32], flag: Option<bool>) -> String {
    let mut line = String::new();
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            line.push(' ');
        }
        let _ = write!(line, "{:.9}", value);
    }
    if let Some(flag) = flag {
        line.push(' ');
        line.push_str(if flag { "true" } else { "false" });
    }
    line.push('\n');
    line
}

pub fn encode_snapshot(snapshot: &Snapshot) -> String {
    let mut text = String::from("NEON_FRONTIER_SAVE_V1\n");
    let player = [snapshot.player.x, snapshot.player.y, snapshot.player.z];
    text.push_str(&record(&player, None));
    let car = [snapshot.car.x, snapshot.car.y, snapshot.car.z];
    text.push_str(&record(&car, None));
    let motion = [snapshot.yaw, snapshot.speed];
    text.push_str(&record(&motion, None));
    text.push_str(&format!("{}\n", snapshot.stage));
    text.push_str(&format!("{}\n", snapshot.credits));
    text.push_str(&format!("{}\n", snapshot.driving));
    text.push_str(&format!("{}\n", snapshot.bloom));
    let world = &snapshot.world;
    let status = [world.health, world.wanted_level as f32, world.wanted_timer];
    text.push_str(&record(&status, None));
    text.push_str(&format!("{}\n", world.traffic.len()));
    for car in &world.traffic {
        let values = [
            car.position.x,
            car.position.y,
            car.position.z,
            car.speed,
            car.direction,
            car.lane_offset,
            car.stop_timer,
        ];
        text.push_str(&record(&values, Some(car.along_x)));
    }
    text.push_str(&format!("{}\n", world.citizens.len()));
    for citizen in &world.citizens {
        let values = [
            citizen.origin.x,
            citizen.origin.y,
            citizen.origin.z,
            citizen.displacement.x,
            citizen.displacement.y,
            citizen.displacement.z,
            citizen.knock_velocity.x,
            citizen.knock_velocity.y,
            citizen.knock_velocity.z,
            citizen.knocked,
            citizen.health,
        ];
        text.push_str(&record(&values, Some(citizen.dead)));
    }
    text.push_str(&format!("{}\n", world.patrols.len()));
    for patrol in &world.patrols {
        let values = [patrol.position.x, patrol.position.y, patrol.position.z];
        text.push_str(&record(&values, None));
    }
    text.push_str(&format!("{}\n", world.officers.len()));
    for officer in &world.officers {
        let values = [
            officer.position.x,
            officer.position.y,
            officer.position.z,
            officer.shot_timer,
        ];
        text.push_str(&record(&values, Some(officer.active)));
    }
    text
}

fn parse_numbers(line: &str, count: usize) -> Option<Vec<f32>> {
    let values = line
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    if values.len() == count {
        Some(values)
    } else {
        None
    }
}

fn parse_flag(value: &str) -> Option<bool> {
    match value {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_numbers_and_flag(line: &str, count: usize) -> Option<(Vec<f32>, bool)> {
    let mut parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != count + 1 {
        return None;
    }
    let flag = parse_flag(parts.pop()?)?;
    let values = parts
        .iter()
        .map(|value| value.parse::<f32>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some((values, flag))
}

fn parse_count(line: &str, limit: usize) -> Option<usize> {
    let count = line.parse::<usize>().ok()?;
    if count <= limit { Some(count) } else { None }
}

const MAX_RECORDS: usize = 4_096;
const MAX_WORLD_COORDINATE: f32 = 200.0;

fn valid_position(position: Vec3) -> bool {
    position.is_finite()
        && position.x.abs() <= MAX_WORLD_COORDINATE
        && position.z.abs() <= MAX_WORLD_COORDINATE
}

fn valid_world(world: &WorldSnapshot) -> bool {
    world.health.is_finite()
        && (0.0..=100.0).contains(&world.health)
        && world.wanted_level <= 6
        && world.wanted_timer.is_finite()
        && (0.0..=18.0).contains(&world.wanted_timer)
        && world.traffic.iter().all(|car| {
            valid_position(car.position)
                && car.speed.is_finite()
                && (0.0..=24.0).contains(&car.speed)
                && car.direction.is_finite()
                && (car.direction.abs() - 1.0).abs() < 0.001
                && car.lane_offset.is_finite()
                && car.lane_offset.abs() <= 12.0
                && car.stop_timer.is_finite()
                && (0.0..=10.0).contains(&car.stop_timer)
        })
        && world.citizens.iter().all(|citizen| {
            valid_position(citizen.origin)
                && citizen.displacement.is_finite()
                && citizen.knock_velocity.is_finite()
                && citizen.knocked.is_finite()
                && (0.0..=2.0).contains(&citizen.knocked)
                && citizen.health.is_finite()
                && (0.0..=3.0).contains(&citizen.health)
        })
        && world
            .patrols
            .iter()
            .all(|patrol| valid_position(patrol.position))
        && world.officers.iter().all(|officer| {
            valid_position(officer.position)
                && officer.shot_timer.is_finite()
                && (0.0..=5.0).contains(&officer.shot_timer)
        })
}

fn decode_world(lines: &mut std::str::Lines<'_>) -> Option<WorldSnapshot> {
    let status = parse_numbers(lines.next()?, 3)?;
    if !status[1].is_finite() || !(0.0..=6.0).contains(&status[1]) || status[1].fract() != 0.0 {
        return None;
    }
    let traffic_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut traffic = Vec::with_capacity(traffic_count);
    for _ in 0..traffic_count {
        let (values, along_x) = parse_numbers_and_flag(lines.next()?, 7)?;
        traffic.push(TrafficSave {
            position: Vec3::new(values[0], values[1], values[2]),
            speed: values[3],
            direction: values[4],
            lane_offset: values[5],
            stop_timer: values[6],
            along_x,
        });
    }
    let citizen_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut citizens = Vec::with_capacity(citizen_count);
    for _ in 0..citizen_count {
        let (values, dead) = parse_numbers_and_flag(lines.next()?, 11)?;
        citizens.push(CitizenSave {
            origin: Vec3::new(values[0], values[1], values[2]),
            displacement: Vec3::new(values[3], values[4], values[5]),
            knock_velocity: Vec3::new(values[6], values[7], values[8]),
            knocked: values[9],
            health: values[10],
            dead,
        });
    }
    let patrol_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut patrols = Vec::with_capacity(patrol_count);
    for _ in 0..patrol_count {
        let values = parse_numbers(lines.next()?, 3)?;
        patrols.push(PatrolSave {
            position: Vec3::new(values[0], values[1], values[2]),
        });
    }
    let officer_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut officers = Vec::with_capacity(officer_count);
    for _ in 0..officer_count {
        let (values, active) = parse_numbers_and_flag(lines.next()?, 4)?;
        officers.push(OfficerSave {
            position: Vec3::new(values[0], values[1], values[2]),
            shot_timer: values[3],
            active,
        });
    }
    let world = WorldSnapshot {
        health: status[0],
        wanted_level: status[1] as u8,
        wanted_timer: status[2],
        traffic,
        citizens,
        patrols,
        officers,
    };
    if valid_world(&world) {
        Some(world)
    } else {
        None
    }
}

pub fn decode_snapshot(text: &str) -> Option<Snapshot> {
    let mut lines = text.lines();
    if lines.next()? != "NEON_FRONTIER_SAVE_V1" {
        return None;
    }
    let player = parse_numbers(lines.next()?, 3)?;
    let car = parse_numbers(lines.next()?, 3)?;
    let motion = parse_numbers(lines.next()?, 2)?;
    let stage = lines.next()?.parse::<usize>().ok()?;
    let credits = lines.next()?.parse::<u32>().ok()?;
    let driving = parse_flag(lines.next()?)?;
    let bloom = parse_flag(lines.next()?)?;
    let world = decode_world(&mut lines)?;
    if lines.next().is_some() {
        return None;
    }
    let finites = [
        player[0], player[1], player[2], car[0], car[1], car[2], motion[0], motion[1],
    ];
    if !finites.iter().all(|value| value.is_finite())
        || !valid_position(Vec3::new(player[0], player[1], player[2]))
        || !valid_position(Vec3::new(car[0], car[1], car[2]))
        || motion[1].abs() > 24.0
    {
        return None;
    }
    Some(Snapshot {
        player: Vec3::new(player[0], player[1], player[2]),
        car: Vec3::new(car[0], car[1], car[2]),
        yaw: motion[0],
        speed: motion[1],
        stage,
        credits,
        driving,
        bloom,
        world,
    })
}

pub fn save_snapshot(snapshot: &Snapshot) -> io::Result<()> {
    std::fs::write(SAVE_FILE_NAME, encode_snapshot(snapshot))
}

pub fn load_snapshot() -> io::Result<Snapshot> {
    let text = std::fs::read_to_string(SAVE_FILE_NAME)?;
    decode_snapshot(&text).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid or unsupported save file",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Snapshot {
        Snapshot {
            player: Vec3::new(12.5, 0.9, -18.25),
            car: Vec3::new(14.0, 0.65, -20.0),
            yaw: 1.25,
            speed: 8.5,
            stage: 2,
            credits: 200,
            driving: true,
            bloom: false,
            world: WorldSnapshot {
                health: 64.0,
                wanted_level: 3,
                wanted_timer: 12.5,
                traffic: vec![TrafficSave {
                    position: Vec3::new(4.0, 0.48, 2.1),
                    speed: 7.2,
                    direction: -1.0,
                    along_x: true,
                    lane_offset: 0.6,
                    stop_timer: 0.4,
                }],
                citizens: vec![CitizenSave {
                    origin: Vec3::new(-8.0, 0.82, 7.0),
                    displacement: Vec3::new(0.5, 0.0, 0.25),
                    knock_velocity: Vec3::ZERO,
                    knocked: 0.0,
                    health: 2.0,
                    dead: false,
                }],
                patrols: vec![PatrolSave {
                    position: Vec3::new(-48.0, 0.48, -2.0),
                }],
                officers: vec![OfficerSave {
                    position: Vec3::new(-45.2, 0.82, -2.0),
                    active: true,
                    shot_timer: 0.35,
                }],
            },
        }
    }

    #[test]
    fn save_format_round_trips_game_state() {
        let snapshot = sample();
        assert_eq!(decode_snapshot(&encode_snapshot(&snapshot)), Some(snapshot));
    }

    #[test]
    fn save_format_round_trips_the_whole_world() {
        let snapshot = sample();
        let decoded = decode_snapshot(&encode_snapshot(&snapshot)).expect("save decodes");
        assert_eq!(decoded.world.health, 64.0);
        assert_eq!(decoded.world.wanted_level, 3);
        assert_eq!(decoded.world.traffic.len(), 1);
        assert_eq!(decoded.world.citizens.len(), 1);
        assert_eq!(decoded.world.patrols.len(), 1);
        assert_eq!(decoded.world.officers.len(), 1);
        assert!(decoded.world.officers[0].active);
    }

    #[test]
    fn save_v1_round_trips_f32_precision() {
        let mut snapshot = sample();
        snapshot.player.x = 12.123_456;
        snapshot.yaw = 0.123_456_78;
        snapshot.world.traffic[0].lane_offset = 0.123_456_78;

        let encoded = encode_snapshot(&snapshot);

        assert!(encoded.starts_with("NEON_FRONTIER_SAVE_V1\n"));
        assert_eq!(decode_snapshot(&encoded), Some(snapshot));
    }

    #[test]
    fn save_decoder_rejects_invalid_or_non_finite_data() {
        let invalid_version = "NEON_FRONTIER_SAVE_V9\n";
        let non_finite = "NEON_FRONTIER_SAVE_V1\nNaN 0 0\n0 0 0\n0 0\n0\n0\nfalse\ntrue\n";
        assert!(decode_snapshot(invalid_version).is_none());
        assert!(decode_snapshot(non_finite).is_none());
    }

    #[test]
    fn save_decoder_rejects_non_finite_world_state() {
        let encoded = encode_snapshot(&sample()).replace("64.000000000", "NaN");
        assert!(decode_snapshot(&encoded).is_none());
    }

    #[test]
    fn save_decoder_rejects_impossible_wanted_and_vehicle_state() {
        let mut snapshot = sample();
        snapshot.world.wanted_level = 7;
        assert!(decode_snapshot(&encode_snapshot(&snapshot)).is_none());
        let mut snapshot = sample();
        snapshot.world.traffic[0].direction = 0.0;
        assert!(decode_snapshot(&encode_snapshot(&snapshot)).is_none());
    }
}
