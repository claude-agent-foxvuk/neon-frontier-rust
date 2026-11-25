# Neon Frontier — project state

Last checked: 2026-11-25 18:55 UTC. Source of truth: `main`, repository files and GitHub Actions. `FEATURES.md` lists code already present; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water stay deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Current `main`: `848c5a1b1bc44cb486c0a21b872216ef67db90c9`. This adds an in-crate ECS regression for traffic hitting a pedestrian; it is committed and read back, but its test result is pending.
- Run #108 completed successfully on `b3432d33788d1336b966dfdbc625c2cc5447af28`; prior read reported format, ECS release tests, release build, 10-second Windows startup smoke, and artifact upload passed. Artifact `neon-frontier-windows-x64`: 29,867,513 bytes. This is an older SHA, not current `main`.
- Run #109 on `a1d544601ac483a16027ac36f6330d89549b0555` remains in progress. Formatting passed; `Test with ECS diagnostics` was running at 18:55 UTC. Build, smoke and artifact are not yet confirmed.
- Run #110 on `1b1381b3981d5e9d11c6adb3c7e4ce689ed112a1` remains in progress. Formatting passed; ECS release tests were running at 18:55 UTC. Build, smoke and artifact are not yet confirmed.
- Run #111 started on current SHA `848c5a1b1bc44cb486c0a21b872216ef67db90c9`; its test, build, smoke and artifact results are pending.
- The newest test commit adds regression coverage only; it does not complete the remaining gameplay requirements or verify the game on Fox's Windows 11 laptop.
- At the last TODO read, 24 items remained open: 23 gameplay requirements plus a Windows check on the exact current `main`. P3 is deferred. No new gameplay item is marked complete by an unpassed test.

## Remaining work
- Priorities above P3: complete ECS OBB integration, stable vehicle-pair behavior, game-level collision regressions, vehicle handling, police behavior, menu/F5/F9 integration checks, missions and events, HUD scaling, graphics/performance validation.
- Manually launch and test the current Windows EXE on Fox's Windows 11 laptop, including Russian text, menu, F5/F9, collisions, police and performance.
- Require formatting, tests, release build, Windows startup smoke and artifact upload on the exact current `main` before claiming the current-HEAD workflow is green.

A successful compile or an older green run does not complete the remaining gameplay work.
