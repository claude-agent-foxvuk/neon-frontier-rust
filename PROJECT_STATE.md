# Neon Frontier — project state

Last checked: 2026-11-25 16:25 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `8186dcaf23c3fe6027322f2ee7b893a43d5c021e`; the latest game code is `62af7f25f0360f555c393a17d695e7d0a9112e53`. The newer commits update TODO and README only.
- Run #92 validates game SHA `62af7f25f0360f555c393a17d695e7d0a9112e53`; it started at 16:21 UTC. At 16:24 UTC, formatting had passed and release tests with ECS diagnostics were in progress. Build, Windows startup smoke and artifact have not been confirmed; the run's artifact list was empty at that check.
- Run #89 on `f02e1ad0dd2200813aa1079e57c310581c902ddf` failed during tests: 99/100 passed, with `traffic_braking_sensor_respects_vehicle_orientation_and_clear_adjacent_lanes` failing because the test excluded its sole element as the current car. The test index fix is in `src/city_life/helpers.rs`; #92 checks it.
- Run #91 on older code SHA `cb32e5fadfaa7539984e8ceef9ffa8353284dbb5` still showed `in_progress` at 16:25 UTC; it does not test the latest test fix.
- Run #88 on `3894bf0471d20b5f480d32dec96785963d23215f` failed during `Test with ECS diagnostics`: E0502 in `src/city_life/sim.rs`. The `blocks_movement` closure captured `car.along_x` and `car.direction` immutably while later code mutated `car`. Later code copies orientation values to locals; run #92 is on a subsequent code state.
- The periodic-sun test compares `sun_height(1.5)` to `sun_height(0.5)` with equal tolerance. It is present in `src/main.rs`, and run #81 passed it.
- Historical run #81 on game SHA `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a` passed formatting, release tests, release build, 10-second Windows startup smoke and artifact upload. Its artifact was `neon-frontier-windows-x64`, 29,867,198 bytes; it is not the current game code.

## Remaining work
- `TODO.md` lists 23 open items: P0 — 5, P1 — 4, P2 — 4, deferred P3 — 10. Do not start P3 until higher-priority items are closed.
- Continue full ECS OBB integration, stable dynamic vehicle-pair separation and regression tests; then vehicle handling, police pursuit, missions/events, HUD scaling, modularization and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police and performance, remain unverified.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. Do not count a successful compile as completion of the remaining gameplay work.
