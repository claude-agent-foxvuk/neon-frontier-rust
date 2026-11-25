# Neon Frontier — project state

Last checked: 2026-11-25 16:55 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `88d68a132a5f37cf4cd8e6ae6b030cab755bd51`. The gameplay code at `4f4da617863efe547ea52c163c3d72bb7cdae6a6` is unchanged; later commits refreshed TODO, README, FEATURES, and project state.
- The game-code commit fixes two calls in `src/city_life/sim.rs` that passed mutable ECS `Transform` query items by value to `traffic_vehicle_body(&Transform, ...)`.
- Run #97 checks gameplay SHA `4f4da617863efe547ea52c163c3d72bb7cdae6a6`. At 16:55 UTC, formatting had passed and release tests with ECS diagnostics were still running; release build, 10-second Windows startup smoke, and artifact upload were pending. No artifact was listed. It does not check the exact current `main` commit.
- Runs #93 and #94 on `8fed02d6250982aaf241c0be2c4dd012785b8f10` completed with E0308 in `src/city_life/sim.rs`: expected `&Transform`, found `Mut<Transform>`. The borrow fix is included in #97's source SHA.
- Run #95 on intermediate SHA `5109a1c0119e8a05532f81e943ce6ba75f4c7a4f` completed with a formatting failure because `traffic_regression_tests` referenced a file absent at that SHA. The file exists in the subsequent source.
- Run #96 checks older SHA `085dc508e55553b947312758c6dfc34358451951`. It was still in release tests at the last status read; it predates the E0308 fix.
- Run #92 on prior gameplay SHA `62af7f25f0360f555c393a17d695e7d0a9112e53` passed formatting, release tests with ECS diagnostics, release build, 10-second Windows startup smoke and upload. Artifact `neon-frontier-windows-x64` was 29,867,560 bytes; it does not verify the later OBB changes.
- The OBB stability test in `src/city_life/traffic_regression_tests.rs` is included in #96 and #97; its result is not yet confirmed.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at equal tolerance. Run #92 passed that test.
- No completed Windows run for exact current `main` `88d68a132a5f37cf4cd8e6ae6b030cab755bd51` was present in the latest Actions listing at 16:55 UTC.

## Remaining work
- `TODO.md` records 24 open items: 23 gameplay requirements across P0–P3 and one current Windows verification. P3 stays deferred until higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police and performance, remain unverified.
- No Windows EXE or artifact is confirmed for the current `main`; require passing tests, build, startup smoke and upload on an up-to-date commit before claiming a release result.
- The README, FEATURES, TODO, and this file were refreshed and read back against current `main`; these document updates do not count as gameplay completion.

A successful compile alone does not complete the remaining gameplay work.
