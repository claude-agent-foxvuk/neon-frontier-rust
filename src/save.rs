use bevy::prelude::*;
use std::io;

pub const SAVE_FILE_NAME: &str = "neon_frontier_save.txt";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Snapshot {
    pub player: Vec3,
    pub car: Vec3,
    pub yaw: f32,
    pub speed: f32,
    pub stage: usize,
    pub credits: u32,
    pub driving: bool,
    pub bloom: bool,
}

pub fn encode_snapshot(snapshot: Snapshot) -> String {
    format!(
        "NEON_FRONTIER_SAVE_V1\n{:.4} {:.4} {:.4}\n{:.4} {:.4} {:.4}\n{:.4} {:.4}\n{}\n{}\n{}\n{}\n",
        snapshot.player.x,
        snapshot.player.y,
        snapshot.player.z,
        snapshot.car.x,
        snapshot.car.y,
        snapshot.car.z,
        snapshot.yaw,
        snapshot.speed,
        snapshot.stage,
        snapshot.credits,
        snapshot.driving,
        snapshot.bloom
    )
}

fn parse_floats<const N: usize>(line: &str) -> Option<[f32; N]> {
    let values = line
        .split_whitespace()
        .map(str::parse::<f32>)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    values.try_into().ok()
}

pub fn decode_snapshot(text: &str) -> Option<Snapshot> {
    let mut lines = text.lines();
    if lines.next()? != "NEON_FRONTIER_SAVE_V1" {
        return None;
    }
    let player = parse_floats::<3>(lines.next()?)?;
    let car = parse_floats::<3>(lines.next()?)?;
    let motion = parse_floats::<2>(lines.next()?)?;
    let stage = lines.next()?.parse::<usize>().ok()?;
    let credits = lines.next()?.parse::<u32>().ok()?;
    let driving = lines.next()?.parse::<bool>().ok()?;
    let bloom = lines.next()?.parse::<bool>().ok()?;
    if lines.next().is_some() {
        return None;
    }
    let values = [
        player[0], player[1], player[2], car[0], car[1], car[2], motion[0], motion[1],
    ];
    if !values.iter().all(|value| value.is_finite()) {
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
    })
}

pub fn save_snapshot(snapshot: Snapshot) -> io::Result<()> {
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

    #[test]
    fn save_format_round_trips_game_state() {
        let snapshot = Snapshot {
            player: Vec3::new(12.5, 0.9, -18.25),
            car: Vec3::new(14.0, 0.65, -20.0),
            yaw: 1.25,
            speed: 8.5,
            stage: 2,
            credits: 200,
            driving: true,
            bloom: false,
        };
        assert_eq!(decode_snapshot(&encode_snapshot(snapshot)), Some(snapshot));
    }

    #[test]
    fn save_decoder_rejects_invalid_or_non_finite_data() {
        let invalid_version = "NEON_FRONTIER_SAVE_V9\n";
        let non_finite = "NEON_FRONTIER_SAVE_V1\nNaN 0 0\n0 0 0\n0 0\n0\n0\nfalse\ntrue\n";
        assert!(decode_snapshot(invalid_version).is_none());
        assert!(decode_snapshot(non_finite).is_none());
    }
}
