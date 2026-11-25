# Neon Frontier — project state

Last checked: 2026-11-25 14:21 UTC. Source of truth: current `main`, repository files, and GitHub Actions results.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving game.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through the GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- Keep `README.md`, `FEATURES.md`, `TODO.md`, and this file current. A green CI run alone does not prove manual playability.

## Current main and verification
- Current `main`: `6b5af2cdbe9801dc8a16ebc4557f5f935b175016`.
- `src/main.rs` already compares `sun_height(1.5)` to `sun_height(0.5)` at the same tolerance. The time-phase correction is present; green test results still need confirmation from Actions.
- Added `main_update_world_queries_initialize_without_conflicts` in `src/city_life/restore_tests.rs`. It runs the `update_world` ECS system with minimal sun and camera entities to catch query conflicts. The existing round-trip tests also initialize `simulate_city_life` with siren and muzzle-flash entities.
- The owner reported that EXE from #66 panicked with Bevy ECS B0001 when its window opened. This remains an unresolved launch defect until a current Windows smoke run completes without B0001.
- Run #77 on `b917918d0db43883956fab2f057034a2d3c0ba47` and #78 on `639e43a355efa16987a6f622c49c92f1cb2c1eb1` were still in `cargo test --release --features bevy/debug` at 14:20 UTC. Their build, startup smoke, and artifact steps had not run.
- Run #79 on `2b5e6e75ea32a97c9a2295f6f69bc36272af24e3` includes a stricter Windows smoke step: it fails on a B000 diagnostic or any process exit during the 10-second window. At 14:20 UTC its test step was running.
- Run #80 on current `main` (`6b5af2cdbe9801dc8a16ebc4557f5f935b175016`) includes the `update_world` query regression test. At 14:20 UTC the runner was still restoring Cargo cache; tests, build, smoke, and artifact upload had not run.
- No current run has yet verified a fresh executable with the new smoke behavior. Read each run's final job result and artifact before claiming success.

## Previously verified baseline and known limits
- Run #66 passed on `59d2e38a6aba6d8fcecddc4c93b16ace23d97464` and uploaded `neon-frontier-windows-x64`, 28,711,072 bytes. Fox later reported the B0001 startup panic, so this EXE is not validated as playable.
- Run #73 on `0fed91ba357a17afbdfcd7442070c1dcb0771149` failed because include-based integration tests could not resolve modules and types after the city module split. Those tests were removed and key checks moved inside the crate; #77/#78 are testing that path.
- #70 on obsolete SHA `8251fadc5bc9ab525f5299f619c72d4784e7ce9d` failed with E0308 in the restore-world officer filter. The current restore caller and signature both use the siren-disjoint filter.
- The Windows workflow retains `cargo test --release --features bevy/debug`; the release build reuses its artifacts. The workflow smoke step now fails on early exit instead of only warning.
- OBB/SAT helpers and tests exist, but OBB is only partly connected to gameplay ECS collision paths. Manual Windows 11 testing of launch, Russian text, menu, F5/F9, HUD scaling, car handling, collisions, sirens, and performance remains open.

## Remaining work
- Read and fix any failures in #77, #78, #79, and #80; prioritize the current `main` run and confirm the smoke test and `neon-frontier-windows-x64` artifact.
- Continue the unchecked P0/P1/P2 items in `TODO.md`; defer its low-priority P3 witness/streaming section until higher priorities are done.
- Update `README.md`, `FEATURES.md`, and `TODO.md` from verified results only. Do not mark all TODO items complete because one run passes.
