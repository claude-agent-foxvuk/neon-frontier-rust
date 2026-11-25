# Neon Frontier — project state

Last checked: 2026-11-25 15:10 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.
- GitHub currently reports this repository as public.

## Current code and checks
- `main` is `7fb5c102043b9661dcb22bd50ff2c936e5383566`. Latest verified gameplay SHA is `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a`; the commits after it update documentation only.
- Windows run #81 completed successfully on `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a` at 14:56 UTC. Formatting, release tests with ECS diagnostics, release build, Windows startup smoke and artifact upload all passed.
- Artifact `neon-frontier-windows-x64` is present for #81, 29,867,198 bytes; it has not expired.
- The run took about 20 minutes. Release tests took 17 minutes 27 seconds; build, smoke and upload took 31 seconds. `--release` remains on tests so the release build can reuse their artifacts. Parallel jobs have not been measured.
- The 10-second Windows smoke kept the process alive and found no B000 diagnostics. It does not replace Fox's manual game test on Windows 11.
- `src/main.rs` compares `sun_height(1.5)` with the periodic equivalent `sun_height(0.5)` at the same tolerance. Run #81 passed.
- Fox reported that the older #66 EXE panicked with Bevy ECS B0001 during window startup. #81's current code and startup smoke passed; the #66 failure remains relevant history, not the current run result.

## Remaining work
- `TODO.md` currently lists 23 open items: P0 — 5, P1 — 4, P2 — 4, deferred P3 — 10. Do not start P3 until higher-priority items are closed.
- Continue with complete ECS OBB collision integration, stable dynamic vehicle-pair separation and collision regression tests; then vehicle handling, police pursuit, missions/events, HUD scaling, modularization and measured RTX 3050 performance.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police and performance, remain unverified.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. The files in this commit update the main SHA and run #81 status; they do not close any gameplay TODO.
