# Neon Frontier — project state

Last checked: 2026-11-25 17:17 UTC. Source of truth: current `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; an original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current gameplay SHA: `05498998af7f7269afcf473118d9fb0359ae4aa7`. It corrects the orientation of the wall in `swept_vehicle_motion_stops_before_a_rotated_obstacle_without_tunneling`: the obstacle is now long along world Y and thin across the X movement path. The regression test has not yet passed CI.
- Run #100 started at 17:15 UTC on current main `05498998af7f7269afcf473118d9fb0359ae4aa7`. At 17:17 UTC setup and toolchain passed; Cargo cache was running. Formatting, tests, build, Windows startup smoke, and artifact upload remain pending.
- Run #99 is still in progress on the earlier gameplay SHA `63a5423131f1e3783f1f535c68ef8a2f91e7ab6b`. Formatting passed; ECS-diagnostic release tests were still running at 17:17 UTC. This SHA contains the swept OBB test before the wall-orientation correction, so its test result may expose that regression, but it cannot validate current main.
- Run #98 completed successfully at 17:16 UTC on older SHA `2b6fecf144b361447b814bef7594072140050665`. Formatting, ECS-diagnostic release tests, release build, 10-second Windows startup smoke, and artifact upload passed. Artifact `neon-frontier-windows-x64` was 29,867,711 bytes. This older SHA does not validate current gameplay changes.
- Run #97 completed successfully on older gameplay SHA `4f4da617863efe547ea52c163c3d72bb7cdae6a6`, including the same checks and a 29,867,711-byte artifact. It does not validate later OBB tests.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at the same tolerance; #92 passed that test.
- Current OBB tests exercise helper-level collision and separation only. Full player/world ECS OBB integration and broader gameplay acceptance remain open.

## Remaining work
- `TODO.md` records 24 open items: 23 gameplay requirements across P0–P3 and one current-main Windows verification. P3 stays deferred until higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police, and performance, remain unverified.
- Require passing formatting, tests, build, startup smoke, and artifact upload on the exact current `main` before claiming a current-main release result.

A successful compile or an older green run does not complete the remaining gameplay work.
