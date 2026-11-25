# Neon Frontier — project state

Last checked: 2026-11-25 16:15 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `cb32e5fadfaa7539984e8ceef9ffa8353284dbb5`.
- The latest main code copies traffic orientation values out before the collision closure mutates `TrafficCar`, addressing the borrow pattern that failed in run #88. The current validation is still running.
- Run #88 on `3894bf0471d20b5f480d32dec96785963d23215f` failed during `Test with ECS diagnostics`: E0502 in `src/city_life/sim.rs`. The `blocks_movement` closure immutably captured `car.along_x` and `car.direction`; later writes to `car.avoid_offset`, `car.stop_timer`, and `car.lane_offset` conflicted. Build, Windows startup smoke and artifact upload were skipped.
- Run #89 on `f02e1ad0dd2200813aa1079e57c310581c902ddf` was still in progress at 16:15 UTC, in tests. Run #90 on intermediate SHA `353b6111713a88e5ec859d580afc58390bfe9fe5` was also still in progress.
- Run #91 on current main `cb32e5fadfaa7539984e8ceef9ffa8353284dbb5`: formatting passed; release tests with ECS diagnostics were in progress at 16:15 UTC. No artifact was listed then. Build, Windows startup smoke and artifact status are not confirmed.
- The periodic-sun test compares `sun_height(1.5)` to `sun_height(0.5)` at equal tolerance. This code is present; the current #91 result is pending.

## Remaining work
- `TODO.md` lists 23 open items: P0 — 5, P1 — 4, P2 — 4, deferred P3 — 10. Do not start P3 until higher-priority items are closed.
- Continue full ECS OBB integration, stable dynamic vehicle-pair separation and regression tests; then vehicle handling, police pursuit, missions/events, HUD scaling, modularization and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police and performance, remain unverified.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. Do not count a successful compile as completion of the remaining gameplay work.
