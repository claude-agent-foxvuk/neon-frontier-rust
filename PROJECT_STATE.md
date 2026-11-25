# Neon Frontier — project state

Last checked: 2026-11-25 17:05 UTC. Source of truth: current `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; an original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `59d5f56172d0090dcd3521930f80e8426e58e7b9` (`Record current Windows checks on main`).
- Run #97 completed successfully on gameplay SHA `4f4da617863efe547ea52c163c3d72bb7cdae6a6`: formatting, release tests with ECS diagnostics, release build, 10-second Windows startup smoke, and upload all passed. Artifact `neon-frontier-windows-x64` is 29,867,711 bytes. The smoke stayed alive and reported no `B000` error.
- Run #98 started at 16:56 UTC on SHA `2b6fecf144b361447b814bef7594072140050665`. At 17:04:57 UTC, formatting had passed and ECS-diagnostic release tests were still in progress; build, startup smoke and artifact were pending. The current `main` is newer than the run SHA, so #98 does not certify the exact current ref.
- Run #96 failed at compile time in `src/city_life/sim.rs:262`: `traffic_vehicle_body` expected `&Transform` but received `Mut<Transform>`. The borrow fix is in `4f4da617863efe547ea52c163c3d72bb7cdae6a6`, checked by successful #97.
- Run #95 failed formatting on an intermediate SHA because `traffic_regression_tests` referenced a file not yet present at that commit; the file exists on current `main`.
- Run #92 succeeded on the earlier SHA `62af7f25f0360f555c393a17d695e7d0a9112e53`, including the day-cycle test and artifact upload. Its artifact does not verify later OBB changes.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at the same tolerance; #92 passed the test.
- The in-crate OBB traffic-pair stability regression is included in #97, which passed. Full player/world OBB integration and the broader gameplay acceptance tests remain open.

## Remaining work
- `TODO.md` records 24 open items: 23 gameplay requirements across P0–P3 and one current-main Windows verification. P3 stays deferred until higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police and performance, remain unverified.
- Require passing formatting, tests, build, startup smoke and artifact upload on the exact current `main` before claiming a current-main release result.

A successful compile or an older green run does not complete the remaining gameplay work.