use bevy::prelude::*;
use std::fmt::Write as _;
use std::io;

use crate::city_life::{CitizenSave, OfficerSave, PatrolSave, TrafficSave, WorldSnapshot};

pub const SAVE_FILE_NAME: &str = "neon_frontier_save.txt";
const DEFAULT_WORLD_TIME: f32 = 0.32;

#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub player: Vec3,
    pub car: Vec3,
    pub yaw: f32,
    pub speed: f32,
    pub steering: f32,
    pub lateral_speed: f32,
    pub player_facing_yaw: f32,
    pub player_moving: bool,
    pub player_running: bool,
    pub player_vertical_speed: f32,
    pub player_grounded: bool,
    pub punch_cooldown: f32,
    pub camera_yaw: f32,
    pub camera_orbit: f32,
    pub camera_pitch: f32,
    pub stage: usize,
    pub credits: u32,
    pub driving: bool,
    pub bloom: bool,
    pub world_time: f32,
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

fn record_with_two_flags(values: &[f32], first: bool, second: bool) -> String {
    let mut line = record(values, Some(first));
    line.pop();
    line.push(' ');
    line.push_str(if second { "true" } else { "false" });
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
    let player_motion = [
        snapshot.steering,
        snapshot.lateral_speed,
        snapshot.player_facing_yaw,
        snapshot.player_vertical_speed,
        snapshot.punch_cooldown,
    ];
    text.push_str(&record(&player_motion, None));
    text.push_str(&format!("{}\n", snapshot.player_moving));
    text.push_str(&format!("{}\n", snapshot.player_running));
    text.push_str(&format!("{}\n", snapshot.player_grounded));
    let camera = [
        snapshot.camera_yaw,
        snapshot.camera_orbit,
        snapshot.camera_pitch,
    ];
    text.push_str(&record(&camera, None));
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
            car.avoid_offset,
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
            citizen.phase,
            citizen.speed,
            citizen.range,
            citizen.displacement.x,
            citizen.displacement.y,
            citizen.displacement.z,
            citizen.knock_velocity.x,
            citizen.knock_velocity.y,
            citizen.knock_velocity.z,
            citizen.knocked,
            citizen.health,
        ];
        text.push_str(&record_with_two_flags(&values, citizen.along_x, citizen.dead));
    }
    text.push_str(&format!("{}\n", world.patrols.len()));
    for patrol in &world.patrols {
        let values = [
            patrol.position.x,
            patrol.position.y,
            patrol.position.z,
            patrol.phase,
        ];
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
    text.push_str(&record(&[snapshot.world_time], None));
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

fn parse_numbers_and_two_flags(line: &str, count: usize) -> Option<(Vec<f32>, bool, bool)> {
    let mut parts: Vec<&str> = line.split_whitespace().collect();
    if parts.len() != count + 2 {
        return None;
    }
    let second = parse_flag(parts.pop()?)?;
    let first = parse_flag(parts.pop()?)?;
    let values = parts
        .iter()
        .map(|value| value.parse::<f32>().ok())
        .collect::<Option<Vec<_>>>()?;
    Some((values, first, second))
}

fn parse_count(line: &str, limit: usize) -> Option<usize> {
    let count = line.parse::<usize>().ok()?;
    if count <= limit {
        Some(count)
    } else {
        None
    }
}

const MAX_RECORDS: usize = 4_096;
const MAX_WORLD_COORDINATE: f32 = 200.0;
const PLAYER_GROUND_Y: f32 = 0.9;

fn valid_position(position: Vec3) -> bool {
    position.is_finite()
        && position.x.abs() <= MAX_WORLD_COORDINATE
        && position.z.abs() <= MAX_WORLD_COORDINATE
}

fn valid_snapshot(snapshot: &Snapshot) -> bool {
    valid_position(snapshot.player)
        && (PLAYER_GROUND_Y..=25.0).contains(&snapshot.player.y)
        && valid_position(snapshot.car)
        && (-2.0..=4.0).contains(&snapshot.car.y)
        && snapshot.yaw.is_finite()
        && snapshot.yaw.abs() <= 1_000_000.0
        && (-8.0..=24.0).contains(&snapshot.speed)
        && snapshot.steering.is_finite()
        && (-1.0..=1.0).contains(&snapshot.steering)
        && snapshot.lateral_speed.is_finite()
        && (-4.0..=4.0).contains(&snapshot.lateral_speed)
        && snapshot.player_facing_yaw.is_finite()
        && snapshot.player_facing_yaw.abs() <= 1_000_000.0
        && snapshot.player_vertical_speed.is_finite()
        && (-30.0..=30.0).contains(&snapshot.player_vertical_speed)
        && snapshot.punch_cooldown.is_finite()
        && (0.0..=0.45).contains(&snapshot.punch_cooldown)
        && snapshot.camera_yaw.is_finite()
        && snapshot.camera_yaw.abs() <= 1_000_000.0
        && snapshot.camera_orbit.is_finite()
        && snapshot.camera_orbit.abs() <= 1_000_000.0
        && (0.12..=1.05).contains(&snapshot.camera_pitch)
        && snapshot.world_time.is_finite()
        && (0.0..=1.0).contains(&snapshot.world_time)
        && (!snapshot.player_grounded
            || ((snapshot.player.y - PLAYER_GROUND_Y).abs() < 0.001
                && snapshot.player_vertical_speed.abs() < 0.001))
        && snapshot.world.health.is_finite()
        && (0.0..=100.0).contains(&snapshot.world.health)
        && snapshot.world.wanted_level <= 6
        && snapshot.world.wanted_timer.is_finite()
        && (0.0..=18.0).contains(&snapshot.world.wanted_timer)
        && snapshot.world.traffic.iter().all(|car| {
            valid_position(car.position)
                && car.speed.is_finite()
                && (0.0..=24.0).contains(&car.speed)
                && car.direction.is_finite()
                && (car.direction.abs() - 1.0).abs() < 0.001
                && car.lane_offset.is_finite()
                && car.lane_offset.abs() <= 12.0
                && car.avoid_offset.is_finite()
                && car.avoid_offset.abs() <= 12.0
                && car.stop_timer.is_finite()
                && (0.0..=10.0).contains(&car.stop_timer)
        })
        && snapshot.world.citizens.iter().all(|citizen| {
            valid_position(citizen.origin)
                && citizen.phase.is_finite()
                && citizen.phase.abs() <= 1_000.0
                && citizen.speed.is_finite()
                && (0.0..=5.0).contains(&citizen.speed)
                && citizen.range.is_finite()
                && (0.0..=20.0).contains(&citizen.range)
                && citizen.displacement.is_finite()
                && citizen.knock_velocity.is_finite()
                && citizen.knocked.is_finite()
                && (0.0..=2.0).contains(&citizen.knocked)
                && citizen.health.is_finite()
                && (0.0..=3.0).contains(&citizen.health)
        })
        && snapshot.world.patrols.iter().all(|patrol| {
            valid_position(patrol.position)
                && patrol.phase.is_finite()
                && patrol.phase.abs() <= 1_000.0
        })
        && snapshot.world.officers.iter().all(|officer| {
            valid_position(officer.position)
                && officer.shot_timer.is_finite()
                && (0.0..=5.0).contains(&officer.shot_timer)
        })
}

pub fn restore_counts_match(
    world: &WorldSnapshot,
    traffic: usize,
    citizens: usize,
    patrols: usize,
    officers: usize,
) -> bool {
    world.traffic.len() == traffic
        && world.citizens.len() == citizens
        && world.patrols.len() == patrols
        && world.officers.len() == officers
}

fn decode_world(lines: &mut std::str::Lines<'_>) -> Option<WorldSnapshot> {
    let status = parse_numbers(lines.next()?, 3)?;
    if !status[1].is_finite() || !(0.0..=6.0).contains(&status[1]) || status[1].fract() != 0.0 {
        return None;
    }
    let traffic_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut traffic = Vec::with_capacity(traffic_count);
    for _ in 0..traffic_count {
        let (values, along_x) = parse_numbers_and_flag(lines.next()?, 8)?;
        traffic.push(TrafficSave {
            position: Vec3::new(values[0], values[1], values[2]),
            speed: values[3],
            direction: values[4],
            lane_offset: values[5],
            avoid_offset: values[6],
            stop_timer: values[7],
            along_x,
        });
    }
    let citizen_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut citizens = Vec::with_capacity(citizen_count);
    for _ in 0..citizen_count {
        let (values, along_x, dead) = parse_numbers_and_two_flags(lines.next()?, 14)?;
        citizens.push(CitizenSave {
            origin: Vec3::new(values[0], values[1], values[2]),
            phase: values[3],
            speed: values[4],
            range: values[5],
            displacement: Vec3::new(values[6], values[7], values[8]),
            knock_velocity: Vec3::new(values[9], values[10], values[11]),
            knocked: values[12],
            health: values[13],
            along_x,
            dead,
        });
    }
    let patrol_count = parse_count(lines.next()?, MAX_RECORDS)?;
    let mut patrols = Vec::with_capacity(patrol_count);
    for _ in 0..patrol_count {
        let values = parse_numbers(lines.next()?, 4)?;
        patrols.push(PatrolSave {
            position: Vec3::new(values[0], values[1], values[2]),
            phase: values[3],
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
    let sample = Snapshot {
        player: Vec3::new(0.0, PLAYER_GROUND_Y, 0.0),
        car: Vec3::new(0.0, 0.65, 0.0),
        yaw: 0.0,
        speed: 0.0,
        steering: 0.0,
        lateral_speed: 0.0,
        player_facing_yaw: 0.0,
        player_moving: false,
        player_running: false,
        player_vertical_speed: 0.0,
        player_grounded: true,
        punch_cooldown: 0.0,
        camera_yaw: 0.0,
        camera_orbit: 0.0,
        camera_pitch: 0.38,
        stage: 0,
        credits: 0,
        driving: false,
        bloom: true,
        world_time: DEFAULT_WORLD_TIME,
        world,
    };
    if valid_snapshot(&sample) {
        Some(sample.world)
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

    let Some(player_motion_line) = lines.next() else {
        let snapshot = Snapshot {
            player: Vec3::new(player[0], player[1], player[2]),
            car: Vec3::new(car[0], car[1], car[2]),
            yaw: motion[0],
            speed: motion[1],
            steering: 0.0,
            lateral_speed: 0.0,
            player_facing_yaw: 0.0,
            player_moving: false,
            player_running: false,
            player_vertical_speed: 0.0,
            player_grounded: true,
            punch_cooldown: 0.0,
            camera_yaw: 0.0,
            camera_orbit: 0.0,
            camera_pitch: 0.38,
            stage,
            credits,
            driving,
            bloom,
            world_time: DEFAULT_WORLD_TIME,
            world: WorldSnapshot {
                health: 100.0,
                ..Default::default()
            },
        };
        return valid_snapshot(&snapshot).then_some(snapshot);
    };

    let player_motion = parse_numbers(player_motion_line, 5)?;
    let player_moving = parse_flag(lines.next()?)?;
    let player_running = parse_flag(lines.next()?)?;
    let player_grounded = parse_flag(lines.next()?)?;
    let camera = parse_numbers(lines.next()?, 3)?;
    let world = decode_world(&mut lines)?;
    let world_time = match lines.next() {
        Some(value) => parse_numbers(value, 1)?[0],
        None => DEFAULT_WORLD_TIME,
    };
    if lines.next().is_some() {
        return None;
    }
    let snapshot = Snapshot {
        player: Vec3::new(player[0], player[1], player[2]),
        car: Vec3::new(car[0], car[1], car[2]),
        yaw: motion[0],
        speed: motion[1],
        steering: player_motion[0],
        lateral_speed: player_motion[1],
        player_facing_yaw: player_motion[2],
        player_moving,
        player_running,
        player_vertical_speed: player_motion[3],
        player_grounded,
        punch_cooldown: player_motion[4],
        camera_yaw: camera[0],
        camera_orbit: camera[1],
        camera_pitch: camera[2],
        stage,
        credits,
        driving,
        bloom,
        world_time,
        world,
    };
    if valid_snapshot(&snapshot) {
        Some(snapshot)
    } else {
        None
    }
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
            player: Vec3::new(12.5, 1.1, -18.25),
            car: Vec3::new(14.0, 0.65, -20.0),
            yaw: 1.25,
            speed: 8.5,
            steering: 0.35,
            lateral_speed: -1.2,
            player_facing_yaw: 0.75,
            player_moving: true,
            player_running: true,
            player_vertical_speed: -2.4,
            player_grounded: false,
            punch_cooldown: 0.2,
            camera_yaw: 1.4,
            camera_orbit: -0.2,
            camera_pitch: 0.55,
            stage: 2,
            credits: 200,
            driving: true,
            bloom: false,
            world_time: 0.73,
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
                    avoid_offset: -4.6,
                    stop_timer: 0.4,
                }],
                citizens: vec![CitizenSave {
                    origin: Vec3::new(-8.0, 0.82, 7.0),
                    phase: 1.25,
                    speed: 0.43,
                    range: 7.0,
                    along_x: false,
                    displacement: Vec3::new(0.5, 0.0, 0.25),
                    knock_velocity: Vec3::ZERO,
                    knocked: 0.0,
                    health: 2.0,
                    dead: false,
                }],
                patrols: vec![PatrolSave {
                    position: Vec3::new(-48.0, 0.48, -2.0),
                    phase: 2.7,
                }],
                officers: vec![OfficerSave {
                    position: Vec3::new(-45.2, 0.82, -2.0),
                    active: true,
                    shot_timer: 0.35,
                }],
            },
        }
    }

    const EMPTY_WORLD_HEADER: &str = concat!(
        "NEON_FRONTIER_SAVE_V1\n",
        "0 0.9 0\n",
        "0 0.65 0\n",
        "0 0\n",
        "0\n",
        "0\n",
        "false\n",
        "true\n",
        "0 0 0 0 0\n",
        "false\n",
        "false\n",
        "true\n",
        "0 0 0.38\n",
        "100 0 0\n"
    );

    fn crowded_world() -> Snapshot {
        let mut snapshot = sample();
        snapshot.world.traffic = (0..14)
            .map(|index| {
                let step = index as f32;
                TrafficSave {
                    position: Vec3::new(step * 4.0 - 26.0, 0.48, 4.5),
                    speed: 5.0 + step * 0.4,
                    direction: if index % 2 == 0 { 1.0 } else { -1.0 },
                    along_x: index % 3 == 0,
                    lane_offset: 0.0,
                    avoid_offset: if index % 2 == 0 { 4.6 } else { -4.6 },
                    stop_timer: 0.0,
                }
            })
            .collect();
        snapshot.world.citizens = (0..14)
            .map(|index| {
                let step = index as f32;
                CitizenSave {
                    origin: Vec3::new(step * 5.0 - 30.0, 0.82, -12.0),
                    phase: step * 0.71,
                    speed: 0.35 + (index % 4) as f32 * 0.08,
                    range: 7.0,
                    along_x: index % 2 == 0,
                    displacement: Vec3::ZERO,
                    knock_velocity: Vec3::ZERO,
                    knocked: 0.0,
                    health: 2.0,
                    dead: index % 7 == 0,
                }
            })
            .collect();
        snapshot.world.patrols = vec![
            PatrolSave {
                position: Vec3::new(-48.0, 0.48, -2.0),
                phase: 0.0,
            },
            PatrolSave {
                position: Vec3::new(48.0, 0.48, 2.0),
                phase: 2.7,
            },
        ];
        snapshot.world.officers = vec![
            OfficerSave {
                position: Vec3::new(-45.2, 0.82, -2.0),
                active: true,
                shot_timer: 0.35,
            },
            OfficerSave {
                position: Vec3::new(50.2, 0.82, 2.0),
                active: false,
                shot_timer: 0.0,
            },
        ];
        snapshot
    }

    #[test]
    fn save_format_round_trips_game_state() {
        let snapshot = sample();
        assert_eq!(decode_snapshot(&encode_snapshot(&snapshot)), Some(snapshot));
    }

    #[test]
    fn save_format_round_trips_a_crowded_world() {
        let crowded = crowded_world();
        let decoded = decode_snapshot(&encode_snapshot(&crowded)).expect("save decodes");
        assert_eq!(decoded, crowded);
        assert_eq!(decoded.world.traffic.len(), 14);
        assert_eq!(decoded.world.citizens.len(), 14);
        assert_eq!(decoded.world.patrols.len(), 2);
        assert_eq!(decoded.world.officers.len(), 2);
    }

    #[test]
    fn save_v1_keeps_traffic_lanes_and_pedestrian_paths() {
        let decoded = decode_snapshot(&encode_snapshot(&crowded_world())).expect("save decodes");
        let car = decoded.world.traffic[1];
        assert_eq!(car.avoid_offset, -4.6);
        assert_eq!(car.lane_offset, 0.0);
        let walker = decoded.world.citizens[3];
        assert!((walker.phase - 2.13).abs() < 0.001);
        assert!((walker.speed - 0.59).abs() < 0.001);
        assert_eq!(walker.range, 7.0);
        assert!(!walker.along_x);
        assert_eq!(decoded.world.patrols[1].phase, 2.7);
    }

    #[test]
    fn save_v1_keeps_wanted_levels_health_and_officers() {
        let decoded = decode_snapshot(&encode_snapshot(&sample())).expect("save decodes");
        assert_eq!(decoded.world.health, 64.0);
        assert_eq!(decoded.world.wanted_level, 3);
        assert_eq!(decoded.world.wanted_timer, 12.5);
        assert!(decoded.world.officers[0].active);
        assert_eq!(decoded.world.officers[0].shot_timer, 0.35);
    }

    #[test]
    fn restore_counts_must_match_the_saved_world() {
        let world = crowded_world().world;
        assert!(restore_counts_match(&world, 14, 14, 2, 2));
        assert!(!restore_counts_match(&world, 13, 14, 2, 2));
        assert!(!restore_counts_match(&world, 14, 15, 2, 2));
        assert!(!restore_counts_match(&world, 14, 14, 1, 2));
        assert!(!restore_counts_match(&world, 14, 14, 2, 0));
    }

    #[test]
    fn save_decoder_accepts_an_empty_world() {
        let mut text = String::from(EMPTY_WORLD_HEADER);
        text.push_str("0\n0\n0\n0\n");
        let decoded = decode_snapshot(&text).expect("an empty world decodes");
        assert!(decoded.world.traffic.is_empty());
        assert!(decoded.world.citizens.is_empty());
        assert!(decoded.world.patrols.is_empty());
        assert!(decoded.world.officers.is_empty());
        assert_eq!(decoded.world.health, 100.0);
        assert_eq!(decoded.stage, 0);
        assert_eq!(decoded.world_time, DEFAULT_WORLD_TIME);
    }

    #[test]
    fn old_core_v1_save_decodes_with_defaults_for_new_fields() {
        let old_file = concat!(
            "NEON_FRONTIER_SAVE_V1\n",
            "12.5 0.9 -18.25\n",
            "14 0.65 -20\n",
            "1.25 8.5\n",
            "2\n",
            "200\n",
            "true\n",
            "false\n"
        );
        let decoded = decode_snapshot(old_file).expect("older V1 file decodes");
        assert_eq!(decoded.player, Vec3::new(12.5, 0.9, -18.25));
        assert_eq!(decoded.car, Vec3::new(14.0, 0.65, -20.0));
        assert_eq!(decoded.yaw, 1.25);
        assert_eq!(decoded.speed, 8.5);
        assert!(decoded.driving);
        assert!(!decoded.bloom);
        assert_eq!(decoded.world_time, DEFAULT_WORLD_TIME);
        assert_eq!(decoded.world.health, 100.0);
        assert!(decoded.world.traffic.is_empty());
    }

    #[test]
    fn save_decoder_rejects_truncated_padded_and_countless_files() {
        let encoded = encode_snapshot(&crowded_world());
        let half = encoded.len() / 2;
        assert!(decode_snapshot(&encoded[..half]).is_none());

        let mut padded = encoded.clone();
        padded.push_str("0\n");
        assert!(decode_snapshot(&padded).is_none());

        let mut missing_records = String::from(EMPTY_WORLD_HEADER);
        missing_records.push_str("2\n");
        assert!(decode_snapshot(&missing_records).is_none());

        let mut absurd_count = String::from(EMPTY_WORLD_HEADER);
        absurd_count.push_str("5000\n");
        assert!(decode_snapshot(&absurd_count).is_none());
    }

    #[test]
    fn save_format_round_trips_player_motion_and_camera() {
        let decoded = decode_snapshot(&encode_snapshot(&sample())).expect("save decodes");
        assert_eq!(decoded.player.y, 1.1);
        assert_eq!(decoded.player_vertical_speed, -2.4);
        assert!(!decoded.player_grounded);
        assert!(decoded.player_moving && decoded.player_running);
        assert_eq!(decoded.steering, 0.35);
        assert_eq!(decoded.lateral_speed, -1.2);
        assert_eq!(decoded.camera_yaw, 1.4);
        assert_eq!(decoded.camera_pitch, 0.55);
        assert_eq!(decoded.punch_cooldown, 0.2);
        assert_eq!(decoded.world_time, 0.73);
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
        snapshot.world_time = 0.123_456_78;

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

    #[test]
    fn save_decoder_rejects_broken_world_detail_columns() {
        let encoded = encode_snapshot(&crowded_world());
        let without_flag = encoded.replacen("true true", "true", 1);
        assert!(decode_snapshot(&without_flag).is_none());

        let mut snapshot = sample();
        snapshot.world.citizens[0].range = 40.0;
        assert!(decode_snapshot(&encode_snapshot(&snapshot)).is_none());

        let mut snapshot = sample();
        snapshot.world.traffic[0].avoid_offset = 40.0;
        assert!(decode_snapshot(&encode_snapshot(&snapshot)).is_none());
    }
}
