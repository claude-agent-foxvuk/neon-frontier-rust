# Neon Frontier — project state

Last checked: 2026-11-25 13:40 UTC. Source of truth: current `main`, repository files, and GitHub Actions results.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving game.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly with the GitHub API to this repository.
- Keep README, FEATURES, TODO, and this file current. A green CI run does not prove manual playability.

## Current main and startup crash
- `main` is at `8251fadc5bc9ab525f5299f619c72d4784e7ce9d`.
- Fox reported that the executable from run #66 panics while creating the window with Bevy ECS error B0001.
- Commit `145bb6d1dcd3ae92682e56463d6a6b28ac1da05a` adds `Without<PoliceSirenLight>` to the officer query to make it disjoint from the siren-light query.
- Windows workflow now includes ECS diagnostics and a short startup smoke step. Run #70 on current main is still in progress; its format step passed, and `Test with ECS diagnostics` is running. Runs #68 and #69 are also still in progress on earlier SHAs. No current-main artifact is confirmed yet.
- Code inspection found `simulate_city_life` uses a four-condition officer filter while `restore_world` accepts a three-condition filter. Verify and fix this signature mismatch before treating the current query change as buildable.

## Verified baseline
- Run #66 on `59d2e38a6aba6d8fcecddc4c93b16ace23d97464` passed format, tests, release build, and artifact upload. Fox's later startup panic shows that run did not prove the executable launches successfully.
- The current Windows workflow runs formatting, `cargo test --release --features bevy/debug`, release build, a startup smoke window that fails on B000x diagnostics, and EXE upload. The smoke step warns rather than fails if the headless runner exits early without a B000x message.

## Open work
- Fix and verify the startup B0001 and query-filter type compatibility; rerun Windows checks and inspect the smoke log.
- Finish game-play TODOs: integrate OBB collisions across player, traffic, pedestrians, and buildings; improve vehicle handling; add police response at wanted levels 3–6; add city events/missions; verify F5/F9 and menu behavior in-game.
- Test current EXE on Fox's Windows 11 machine for startup, Cyrillic text, collisions, vehicle model, and sirens. CI cannot replace this manual check.
- Update README, FEATURES, and TODO when code or verification status changes. Keep implementation claims separate from runtime verification.
