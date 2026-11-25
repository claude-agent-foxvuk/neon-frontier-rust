# Neon Frontier — project state

Last checked: 2026-11-25 13:54 UTC. Source of truth: current `main`, repository files, and GitHub Actions results.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving game.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through the GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- Keep README, FEATURES, TODO, and this file current. A green CI run does not prove manual playability.

## Current main and recovery
- `main` is at `0fed91ba357a17afbdfcd7442070c1dcb0771149`.
- The EXE from #66 was reported by Fox to panic at window startup with Bevy ECS B0001. The old green run did not prove that the game launches.
- The subsequent main history contained a truncated `src/city_life.rs`; commit `d8babda` added `src/city_life/sim.rs` without wiring it into the module. Commit `0fed91b` restores the preserved city core through `src/city_life.rs` and `src/city_life/core.rs`, reconnects `helpers.rs` and `sim.rs`, restores city-life tests, and keeps the officer query filter disjoint from siren lights. Compilation and runtime are not confirmed yet.
- Windows run #73 started on current SHA `0fed91ba357a17afbdfcd7442070c1dcb0771149` at 13:53:50 UTC. At the latest read, setup and checkout passed; Cargo cache was running, formatting/tests/build/smoke/upload had not started.
- Run #72 is still listed in progress on superseded SHA `d8babda9cede9505753fc17559191f7f6e9b668c`; do not treat it as verification of current main.
- Run #70 on `8251fadc5bc9ab525f5299f619c72d4784e7ce9d` failed with E0308: `restore_world` expected a three-condition officer filter while the call supplied four. The current restored source matches the four-condition filter, pending run #73.
- The workflow includes Bevy ECS diagnostics and a 10-second Windows startup smoke step. Inspect its log on #73; an artifact is not a substitute for checking the smoke result.

## Verified older baseline
- Run #66 on `59d2e38a6aba6d8fcecddc4c93b16ace23d97464` passed formatting, tests, release build, and upload. It is superseded and Fox's report of B0001 means it is not a validated playable build.
- The workflow uses `cargo test --release --features bevy/debug`; preserve `--release`. On #66 tests took 12:57 and release build took 1 second. Splitting jobs has not been measured.

## Open work
- Confirm #73 format, tests, release build, startup smoke log, and `neon-frontier-windows-x64` artifact on the current main SHA; fix any failure before relying on it.
- Continue TODO: full OBB integration across player, traffic, pedestrians, and buildings; vehicle inertia/grip/braking; police response for wanted levels 3–6; missions and city events; HUD scaling; complete module separation; manual F5/F9 and menu checks.
- Test the latest EXE on Fox's Windows 11 machine for startup, Cyrillic text, collisions, vehicle model, sirens, and graphics. CI cannot confirm those manual results.

Update this file, README, FEATURES, and TODO whenever source or verification status changes. Never mark all TODO complete based on one green run.