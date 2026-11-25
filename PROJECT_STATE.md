# Neon Frontier — project state

Last checked: 2026-11-25 19:08 UTC. Source of truth: `main`, repository files and GitHub Actions. `FEATURES.md` lists code already present; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water stay deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Code and checks
- Last code SHA read before this document update: `2c1aabacd412db90c154dcaeafd1ea029925017c`. The prior commit `6da5ce5ad63d3f6ad1900995a34b77bf874fc276` updates pedestrian knockback: analytic horizontal drag, ballistic vertical integration and continuation until landing. Two new helper tests compare frame rates and check landing; they are not yet verified by CI.
- The traffic-pedestrian test was corrected to expect no wanted level for a distant player; a separate test checks wanted for the player's own vehicle impact. This correction is in `2c1aabacd412db90c154dcaeafd1ea029925017c`. The new tests are not yet verified by CI.
- Run #109 succeeded on older SHA `a1d544601ac483a16027ac36f6330d89549b0555`: formatting, ECS release tests, release build, 10-second Windows startup smoke and upload passed. Artifact `neon-frontier-windows-x64`: 29,867,517 bytes. It does not cover the later code.
- Run #108 succeeded on older SHA `b3432d33788d1336b966dfdbc625c2cc5447af28`; artifact: 29,867,513 bytes.
- Runs #110, #111 and #112 were still running ECS tests when checked at 19:07 UTC. #111 and #112 include the old traffic-pedestrian test expectation; inspect their actual results rather than assuming failure.
- Run #113 started for `6da5ce5ad63d3f6ad1900995a34b77bf874fc276`. Run #114 started for `2c1aabacd412db90c154dcaeafd1ea029925017c`; at 19:08 UTC it was restoring cache, with formatting, tests, build, smoke and artifact still pending. This document commit may change `main` after that code SHA.
- The last TODO summary listed 23 gameplay requirements plus a Windows check on exact `main`. No gameplay item has been closed solely based on these changes. P3 is deferred.

## Remaining work
- Priorities above P3: complete ECS OBB integration, stable vehicle-pair behavior, game-level collision regressions, vehicle handling, police behavior, menu/F5/F9 integration checks, missions and events, HUD scaling, graphics/performance validation.
- Manually launch and test the current Windows EXE on Fox's Windows 11 laptop, including Russian text, menu, F5/F9, collisions, police and performance.
- Require formatting, tests, release build, Windows startup smoke and artifact upload on the exact current `main` before claiming the current-HEAD workflow is green.

A successful compile or an older green run does not complete the remaining gameplay work.
