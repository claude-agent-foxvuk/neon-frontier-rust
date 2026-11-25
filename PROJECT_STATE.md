# Neon Frontier — project state

Last checked: 2026-11-25 14:24 UTC. Source of truth: current `main`, repository files, and GitHub Actions results.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving game.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through the GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- Keep `README.md`, `FEATURES.md`, `TODO.md`, and this file current. A green CI run alone does not prove manual playability.

## Current main and verification
- Current `main` is `0867587cf3699447950d91070e1a6eee36ca6f77`; latest code snapshot is `6b5af2cdbe9801dc8a16ebc4557f5f935b175016`. The later commit only updated project state.
- `src/main.rs` compares `sun_height(1.5)` to equivalent `sun_height(0.5)` with the same tolerance. The fix is present in code; its current test result is not yet verified.
- Added `main_update_world_queries_initialize_without_conflicts` in `src/city_life/restore_tests.rs`. It invokes `update_world` on a minimal world to detect query initialization conflicts. Another in-crate test invokes `simulate_city_life` with officers, sirens, and muzzle flashes.
- Fox reported that the EXE from run #66 panicked with Bevy ECS B0001 when its window opened. No current Windows startup smoke result has confirmed the fix yet.
- Run #75 on `036213395ee54925acf5dd3fd556b5a6cc180c58` failed at 14:23 UTC in tests. Its log shows obsolete include-based integration tests tried to load `src/city_life.rs` as if `helpers` and `sim` were sibling files under `src`; rustc reported E0583 for both. The integration tests were removed in the later source history; this failure is not a result for the current code snapshot.
- Runs #77 (`b917918d0db43883956fab2f057034a2d3c0ba47`), #78 (`639e43a355efa16987a6f622c49c92f1cb2c1eb1`), and #79 (`2b5e6e75ea32a97c9a2295f6f69bc36272af24e3`) were still in the release test step at 14:24 UTC. They are older snapshots.
- Run #80 tests current code snapshot `6b5af2cdbe9801dc8a16ebc4557f5f935b175016`. At 14:24 UTC formatting had passed and `cargo test --release --features bevy/debug` was in progress; build, startup smoke, and artifact upload had not started. No `neon-frontier-windows-x64` artifact was listed yet.
- Run #79 adds a Windows startup smoke check that fails on B000 diagnostics or any exit during its 10-second window. #80 will exercise it only after tests and release build pass.

## Previously verified baseline and known limits
- Run #66 passed formatting, tests, release build, and artifact upload on `59d2e38a6aba6d8fcecddc4c93b16ace23d97464`; its EXE was 28,711,072 bytes. Fox later reported the B0001 startup panic, so it is not validated as playable.
- Run #73 on `0fed91ba357a17afbdfcd7442070c1dcb0771149` failed because include-based integration tests could not resolve modules/types after the city module split. These tests were removed and key checks moved inside the crate.
- Run #70 on obsolete `8251fadc5bc9ab525f5299f619c72d4784e7ce9d` failed with E0308 in the restore-world officer filter. Current restore caller/signature filters match.
- The workflow retains `cargo test --release --features bevy/debug`; release tests can share build artifacts with release build. Its smoke step now checks for startup B0001 and early process exit.
- OBB/SAT helpers and tests exist, but OBB is only partly connected to gameplay ECS collision paths. Manual Windows 11 testing of launch, Russian text, menu, F5/F9, HUD scaling, car handling, collisions, sirens, and performance remains open.

## Remaining work
- Read the final result and detailed test log for #80; then verify release build, Windows startup smoke, and the artifact on the current code SHA. Do not infer success from an in-progress run.
- Continue unchecked P0/P1/P2 tasks in `TODO.md`; defer the low-priority P3 witness/streaming section until higher priorities are done.
- Update `README.md`, `FEATURES.md`, and `TODO.md` from verified results only. Do not mark all TODO items complete because one run passes.
