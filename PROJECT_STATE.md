# Neon Frontier — project state

Last checked: 2026-11-25 14:30 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks outstanding work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior is deferred until higher-priority TODO items are done.

## Current code and checks
- `main` was `72b26750cc06e2783fe6e0eb5abbdb39f3e58c81` at 14:30 UTC. Latest game-code SHA is `6b5af2cdbe9801dc8a16ebc4557f5f935b175016`; subsequent commits observed here update documentation only.
- `src/main.rs` now compares `sun_height(1.5)` with equivalent `sun_height(0.5)` at the same tolerance. The passing result for that test on the latest code is still pending run #80.
- Run #77 on older code SHA `b917918d0db43883956fab2f057034a2d3c0ba47` completed successfully at 14:27 UTC: formatting, release tests, release build, Windows startup smoke, and artifact upload all passed. Artifact `neon-frontier-windows-x64` was listed, 29,867,510 bytes. This is not the latest code and predates the new `update_world` query-initialization regression test and the stricter smoke check; it does not verify the current executable.
- Run #80 on code SHA `6b5af2cdbe9801dc8a16ebc4557f5f935b175016` was still in release tests at 14:30 UTC. Formatting passed; build, startup smoke, and artifact upload had not started. No artifact was listed for #80.
- Run #79 on older SHA `2b5e6e75ea32a97c9a2295f6f69bc36272af24e3` was also still in release tests at 14:30 UTC.
- The current workflow enables Bevy ECS diagnostics for release tests and build, then runs a Windows startup smoke test that rejects B000 diagnostics or early process exit. This newer smoke has not yet passed on the latest code.
- Fox reported that the EXE from #66 panicked with B0001 on window creation. The added in-crate tests exercise `simulate_city_life` restoration and `update_world` query initialization; run #80 is the first observed check that includes the latter.

## Remaining work
- Read run #80's final test result. If it passes, verify release build, the stricter Windows startup smoke, and `neon-frontier-windows-x64` on the current code SHA.
- Continue P0 collision integration and gameplay regressions, P1 vehicle handling and police pursuit, and P2 mission variety, HUD scaling, modularization, and RTX 3050 performance. Do not count code-only work as manual Windows verification.
- Update `README.md`, `FEATURES.md`, and `TODO.md` only from verified code and run results; one successful CI run does not close the whole TODO.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collision, police, and performance, remain unverified here.
