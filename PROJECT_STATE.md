# Neon Frontier — project state

Last checked: 2026-11-25 14:52 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water are deferred until higher-priority TODO items are done.

## Current code and checks
- `main` is `412c7ff2e4b97fc797f99da6360e4e434b32f625`; latest game-code SHA is `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a`. The newer main commit updates README only.
- `src/main.rs` compares `sun_height(1.5)` with equivalent `sun_height(0.5)` at the same tolerance, as required by the periodic `day_phase` calculation. Its result on the latest game-code SHA is pending run #81.
- Run #81 was still `in_progress` at 14:52 UTC for game SHA `e1b13b2027cd2c4d574ebbaad48dd12bd41fd63a`. The format step passed; `cargo test --release --features bevy/debug` was in progress. Release build, Windows startup smoke and artifact upload had not begun; no artifact was listed at 14:50 UTC.
- The #81 job log download returned `BlobNotFound`; the run and check-run endpoints still report `in_progress`. No test output or failure cause is confirmed yet.
- Runs #79 and #80 completed successfully on earlier SHAs. Run #80 checked `6b5af2cdbe9801dc8a16ebc4557f5f935b175016`, before the latest OBB pedestrian-pair change.
- Run #77 on older code SHA `b917918d0db43883956fab2f057034a2d3c0ba47` completed successfully at 14:27 UTC: format, release tests, release build, Windows startup smoke and artifact upload passed. Artifact `neon-frontier-windows-x64` was listed, 29,867,510 bytes. It does not verify the latest code.
- The current workflow enables Bevy ECS diagnostics for release tests and build, then runs a 10-second Windows startup smoke that rejects B000 diagnostics or early process exit. That smoke has not yet passed on the latest game-code SHA.
- Fox reported that the EXE from #66 panicked with B0001 on window creation. The in-crate tests exercise city restore and `update_world` query initialization, but only the current Windows smoke can verify the latest executable starts.

## Remaining work
- TODO has 25 open items: P0 — 5; P1 — 4; P2 — 4; deferred P3 — 10, including beach and water; release check and status update — 2.
- Continue P0 OBB integration and gameplay regressions, P1 vehicle handling and police pursuit, and P2 mission variety, HUD scaling, modularization and RTX 3050 performance. Do not count code-only work as manual Windows verification or mark the full TODO complete after one green CI run.
- Refresh `README.md`, `FEATURES.md`, `TODO.md` and this file only from verified code and run results. The README now reflects 25 open items and the 14:50 UTC run #81 observation.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police and performance, remain unverified here.
