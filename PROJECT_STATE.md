# Neon Frontier — project state

Last checked: 2026-11-25 16:42 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Latest gameplay change: `8fed02d6250982aaf241c0be2c4dd012785b8f10` (`Resolve overlapping traffic vehicles with OBB`). New test-only commits `5109a1c` and `085dc50` wire an in-crate OBB stability regression test; current `main` is `085dc508e55553b947312758c6dfc34358451951`.
- Run #92 on prior gameplay SHA `62af7f25f0360f555c393a17d695e7d0a9112e53` completed successfully at 16:37 UTC: formatting, release tests with ECS diagnostics, release build, 10-second Windows startup smoke, and upload passed. Artifact `neon-frontier-windows-x64` was 29,867,560 bytes. This does not test the later OBB gameplay change.
- Runs #93 (push) and #94 (manual dispatch) both target `8fed02d`. At the 16:39 UTC job read they were still in progress at `Test with ECS diagnostics`; formatting passed, and release build, startup smoke, and artifact were not yet confirmed.
- Run #95 checks intermediate SHA `5109a1c0119e8a05532f81e943ce6ba75f4c7a4f`; at 16:42 UTC it was in progress at Cargo cache setup. It predates the test file commit.
- Run #96 checks current `main` `085dc508e55553b947312758c6dfc34358451951`, including the new test. At 16:42 UTC it was in progress at Cargo cache setup; test, build, startup smoke and artifact are not yet confirmed.
- Run #89 on `f02e1ad0dd2200813aa1079e57c310581c902ddf` failed 99/100 tests: `traffic_braking_sensor_respects_vehicle_orientation_and_clear_adjacent_lanes` used index 0 and excluded its only car. The index fix is in `62af7f2`; #92 passed on that older SHA.
- Run #88 on `3894bf0471d20b5f480d32dec96785963d23215f` failed with E0502 in `src/city_life/sim.rs`; the subsequent code copies orientation into locals before mutating the car.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at equal tolerance. Run #92 passed that test.
- Historical #81 on `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a` passed the workflow and produced a 29,867,198-byte artifact; it does not validate current code.

## Recent code and regression coverage
- `src/city_life/sim.rs` symmetrically separates overlapping traffic cars with OBB contacts, rolls back a correction that would put a car inside a building, updates lane offset and briefly stops moved cars. Gameplay change is at `8fed02d`; #93/#94 are checking it.
- `src/city_life/traffic_regression_tests.rs` now verifies that an already separated crossing OBB car pair does not move again on the next resolution pass. The test is wired through `src/city_life.rs`; run #96 checks the current tree, but its result is pending.

## Remaining work
- `TODO.md` lists 24 open checkboxes: 23 gameplay requirements across P0–P3 (P0 — 5, P1 — 4, P2 — 4, deferred P3 — 10), plus current Windows validation for #93/#94. Run #96 also checks the latest test-only change. Do not start P3 until higher-priority work is done.
- Continue full ECS OBB integration, stable dynamic vehicle-pair separation and gameplay regression tests; then vehicle handling, police pursuit, missions/events, HUD scaling, modularization and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police and performance, remain unverified.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and `PROJECT_STATE.md` only from verified code and run results. Do not count a successful compile as completion of remaining gameplay work.
