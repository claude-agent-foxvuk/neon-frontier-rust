# Neon Frontier — project state

Last checked: 2026-11-25 17:45 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; an original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Latest gameplay SHA: `3da51616882fe2b7a0e4761ee83a792b22de475f` (`Assert OBB braking for traffic approaching player car`). Documentation commits follow it; they do not change gameplay code.
- Run #100 passed on gameplay SHA `05498998af7f7269afcf473118d9fb0359ae4aa7`: format check, 105 release tests with ECS diagnostics, release build, 10-second Windows startup smoke, and artifact upload. Artifact `neon-frontier-windows-x64` is 29,867,712 bytes. The smoke ran on a Windows Actions runner, not Fox's laptop.
- Run #101 is in progress on documentation SHA `013793fbb2740f122f5d3fb65c1e3cec2f393087`; its format check passed and its release tests were still running at 17:45 UTC. It predates the latest game commits.
- Run #102 is in progress on `d7bb24821be4ace8b5e688af07cd825efc5931a9`; its format check passed and its release tests were still running at 17:45 UTC.
- Run #103 is in progress on latest gameplay SHA `3da51616882fe2b7a0e4761ee83a792b22de475f`; its format check passed and release tests were still running at 17:45 UTC. Build, startup smoke, and artifact are not confirmed.
- `src/main.rs` compares `sun_height(1.5)` with `sun_height(0.5)` at the same tolerance; the corrected test passed in #100.
- #103 tests OBB-based traffic braking before the player vehicle. Result remains pending.
- Full ECS OBB integration remains open. Player movement still has circular collision paths; gameplay-level checks beyond helper tests remain necessary.

## Remaining work
- `TODO.md` currently lists 24 open items: 23 gameplay requirements across P0–P3 and one Windows verification on current `main`; P3 stays deferred until higher-priority work is complete.
- Continue full ECS OBB integration, vehicle-pair stability and gameplay collision tests, vehicle handling, police pursuit, missions/events, HUD scaling, modularization, and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police, and performance, remain unverified.
- Require passing formatting, tests, build, startup smoke, and artifact upload on exact current `main` before claiming a current-main release result.

A successful compile or an older green run does not complete the remaining gameplay work.
