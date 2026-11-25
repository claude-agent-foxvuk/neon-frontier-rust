# Neon Frontier — project state

Last checked: 2026-11-25 17:34 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; an original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- At this check, `main` was `013793fbb2740f122f5d3fb65c1e3cec2f393087`; comparison with the passing gameplay SHA `05498998af7f7269afcf473118d9fb0359ae4aa7` shows only documentation files changed after that gameplay commit.
- Run #100 completed successfully on gameplay SHA `05498998af7f7269afcf473118d9fb0359ae4aa7`: format check, all 105 release tests with ECS diagnostics, release build, 10-second Windows startup smoke, and artifact upload passed. Artifact `neon-frontier-windows-x64` is 29,867,712 bytes. The smoke ran on a Windows Actions runner, not Fox's laptop.
- Run #101 was started by workflow dispatch on exact `main` SHA `013793fbb2740f122f5d3fb65c1e3cec2f393087` at 17:34 UTC; its result is not confirmed yet.
- Run #99 on `63a5423131f1e3783f1f535c68ef8a2f91e7ab6b` failed one swept OBB regression test (104 passed, 1 failed); the wall-orientation correction in #100 passed.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at the same tolerance; #100 passed the full suite.
- Current OBB tests exercise helper-level collision and separation only. Full player/world ECS OBB integration and broader gameplay acceptance remain open.

## Remaining work
- `TODO.md` records 24 open items: 23 gameplay requirements across P0–P3 and one Windows verification on current `main`. P3 stays deferred until higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police, and performance, remain unverified.
- Require passing formatting, tests, build, startup smoke, and artifact upload on the exact current `main` before claiming a current-main release result.

A successful compile or an older green run does not complete the remaining gameplay work.
