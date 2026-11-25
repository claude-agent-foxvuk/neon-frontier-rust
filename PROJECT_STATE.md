# Neon Frontier — project state

Last checked: 2026-11-25 16:48 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `4f4da617863efe547ea52c163c3d72bb7cdae6a6` (`Fix mutable transform borrowing in OBB pass`). The commit corrects two calls that passed mutable ECS `Transform` query items by value to `traffic_vehicle_body`, whose parameter is `&Transform`.
- Run #97 checks `4f4da617863efe547ea52c163c3d72bb7cdae6a6`. At 16:48 UTC, Windows setup and checkout had passed; Cargo cache restore was in progress. Formatting, tests, release build, startup smoke and artifact are not yet verified for this SHA.
- Run #93 on `8fed02d6250982aaf241c0be2c4dd012785b8f10` failed during release tests with E0308 at `src/city_life/sim.rs:262`: expected `&Transform`, found `Mut<Transform>`. The exact error was fixed in the current commit; #97 is the first Windows run of that fix.
- Run #95 on intermediate SHA `5109a1c0119e8a05532f81e943ce6ba75f4c7a4f` failed during formatting because `src/city_life.rs` referenced `traffic_regression_tests` before that file existed in that SHA. The test file appears on the subsequent current-main history; #96 checks it.
- Run #96 on `085dc508e55553b947312758c6dfc34358451951` was still in the release-test step at 16:48 UTC. It predates the borrow fix and does not validate current `main`.
- Run #94 on `8fed02d6250982aaf241c0be2c4dd012785b8f10` was still in the release-test step at the last job read, 16:48 UTC. It predates the borrow fix.
- Run #92 on prior gameplay SHA `62af7f25f0360f555c393a17d695e7d0a9112e53` passed formatting, release tests with ECS diagnostics, release build, 10-second Windows startup smoke and upload. Its artifact `neon-frontier-windows-x64` was 29,867,560 bytes; it does not validate the later OBB code.
- The OBB stability regression test in `src/city_life/traffic_regression_tests.rs` checks that a vehicle pair does not move again on the next resolution pass. #96 includes it; #97 checks the same current source plus the compile fix.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at equal tolerance. Run #92 passed that test.

## Remaining work
- `TODO.md` currently records 24 open items: 23 gameplay requirements across P0–P3 and one Windows check for #93/#94/#96. Update counts and run references after another TODO review. Do not start P3 before higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police and performance, remain unverified.
- No Windows EXE or artifact is confirmed for current `main`; #97 must pass tests, build, startup smoke and upload before claiming that result.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. A successful compile alone does not complete the remaining gameplay work.
