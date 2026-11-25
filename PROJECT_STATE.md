# Neon Frontier — project state

Last checked: 2026-11-25 16:30 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main` and latest game code: `8fed02d6250982aaf241c0be2c4dd012785b8f10` (`Resolve overlapping traffic vehicles with OBB`).
- Windows runs #93 (push) and #94 (manual dispatch) both target the current SHA `8fed02d`. At 16:30 UTC both were still in progress. #94 had completed setup, checkout, toolchain and cache setup; formatting, release tests, build, Windows startup smoke and artifact upload were still pending. No artifact was listed yet.
- Run #92 on older SHA `62af7f25f0360f555c393a17d695e7d0a9112e53` was still in progress at 16:30 UTC; formatting passed and release tests with ECS diagnostics were running. It does not test current `main`.
- Run #89 on `f02e1ad0dd2200813aa1079e57c310581c902ddf` failed during tests: 99/100 passed, with `traffic_braking_sensor_respects_vehicle_orientation_and_clear_adjacent_lanes` failing because the test excluded its sole element as the current car. The index fix is in `src/city_life/helpers.rs`; #92 checks that fix, but remains pending.
- Run #88 on `3894bf0471d20b5f480d32dec96785963d23215f` failed during `Test with ECS diagnostics`: E0502 in `src/city_life/sim.rs`. Later code copies orientation values to locals before mutation; current SHA still requires a completed Windows check.
- The periodic-sun test compares `sun_height(1.5)` to `sun_height(0.5)` with equal tolerance. It is present in `src/main.rs` and passed in historical run #81.
- Historical run #81 on game SHA `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a` passed formatting, release tests, release build, 10-second Windows startup smoke and artifact upload. Its `neon-frontier-windows-x64` artifact was 29,867,198 bytes; it is not the current game code.

## Recent code change
- `src/city_life/sim.rs` now separates overlapping traffic vehicles symmetrically with OBB contacts, rolls back a correction that would put a vehicle into a building, updates lane offset and briefly stops moved vehicles. A regression test covers a crossing pair. Commit `8fed02d` has not yet completed Windows tests or startup smoke.

## Remaining work
- `TODO.md` currently lists 23 open items: P0 — 5, P1 — 4, P2 — 4, deferred P3 — 10. Do not start P3 until higher-priority items are closed.
- Continue full ECS OBB integration, stable dynamic vehicle-pair separation and regression tests; then vehicle handling, police pursuit, missions/events, HUD scaling, modularization and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police and performance, remain unverified.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. Do not count a successful compile as completion of the remaining gameplay work.
