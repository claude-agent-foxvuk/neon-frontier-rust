use super::*;

#[test]
fn police_tactics_escalate_across_wanted_levels_three_to_six() {
    let levels: Vec<_> = (3..=6)
        .map(|level| police_tactics(level, 0, 1.0))
        .collect();

    for pair in levels.windows(2) {
        assert!(pair[1].speed > pair[0].speed);
        assert!(pair[1].stop_distance > pair[0].stop_distance);
        assert!(pair[1].flank_distance > pair[0].flank_distance);
        assert!(pair[1].search_radius > pair[0].search_radius);
    }
}

#[test]
fn high_wanted_levels_use_more_aggressive_tactics_than_low_levels() {
    let low = police_tactics(2, 0, 1.0);
    let high = police_tactics(6, 0, 1.0);

    assert!(high.speed > low.speed);
    assert!(high.flank_distance > low.flank_distance);
    assert!(high.search_radius > low.search_radius);
}
