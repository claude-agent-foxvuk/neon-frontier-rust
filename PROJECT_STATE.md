# Neon Frontier — project state

Last checked: 2026-11-25 13:57 UTC. Source of truth: current `main`, repository files, and GitHub Actions results.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving game.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through the GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- Keep `README.md`, `FEATURES.md`, `TODO.md`, and this file current. A green CI run alone does not prove manual playability.

## Current main and verification
- Current `main` is `99305fcfb5bd0dc98ff41687d20a16382091d1a0`; project-state docs record the restored source commit `0fed91ba357a17afbdfcd7442070c1dcb0771149` immediately before it.
- Run #73 is testing `0fed91ba357a17afbdfcd7442070c1dcb0771149`, the code tree currently on `main`. At 13:56:26 UTC setup, checkout, Rust 1.100 installation, cache restore, and format check had passed. `cargo test --release --features bevy/debug` was in progress; build, startup smoke, and artifact upload had not run. No artifact was listed yet.
- Run #70 on `8251fadc5bc9ab525f5299f619c72d4784e7ce9d` failed at compile time with E0308: `restore_world` accepted a three-filter officer query while its caller supplied four filters. The smoke step was skipped because tests failed. The city source and filters were restored in `0fed91b`; #73 is the first Windows check on that restored tree.
- Runs #71 and #72 concern superseded code SHAs and do not verify current `main`.
- `src/city_life.rs` is now split into `core.rs`, `helpers.rs`, `sim.rs`, and `tests.rs`; query filters in the restore and simulation officer queries include `Without<PoliceSirenLight>`.
- The workflow uses `cargo test --release --features bevy/debug`, a Windows startup smoke step that scans captured output for `error[B000`, and release EXE upload. The smoke step's actual result is still pending on #73.

## Previously verified baseline and known limits
- Run #66 passed on `59d2e38a6aba6d8fcecddc4c93b16ace23d97464` and uploaded `neon-frontier-windows-x64`, 28,711,072 bytes. Fox later reported that this EXE panicked with Bevy B0001 at window startup; it is not a validated playable build.
- The test `sun_peaks_at_noon_and_bottoms_out_at_midnight` already compares `sun_height(1.5)` to `sun_height(0.5)` with a tolerance; current source contains that correction. Verify its current run result in #73.
- OBB/SAT, swept movement, pair resolution helpers and unit tests exist. OBB is only partially connected to the city ECS; multiple player/vehicle/pedestrian collision paths still use circles.
- Manual Windows 11 testing is required for launch, Russian text, menu, F5/F9, collisions, vehicle model, sirens and graphics. No current EXE has passed that owner-side check.

## Remaining work
- Read #73 result, especially test output, startup smoke log and `neon-frontier-windows-x64` artifact, before treating the current source as verified.
- Continue `TODO.md`: finish gameplay OBB integration and stable pair separation, improve car inertia/grip/braking, implement higher-level police response, add city events/missions, test HUD scaling and menu/save hotkeys, and keep modules organized.
- Update `README.md`, `FEATURES.md`, and `TODO.md` with verified state, not assumptions; never mark the whole TODO complete because of one green run.
