# Neon Frontier — project state

Last checked: 2026-11-25 18:35 UTC. Source of truth: `main`, repository files, and GitHub Actions. `FEATURES.md` lists existing code; `TODO.md` tracks open work by priority.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; an original compact open-world action-driving prototype.
- Save format stays `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness/streaming behavior, beach and water remain deferred until higher-priority TODO items are done.
- GitHub reports this repository as public.

## Current code and checks
- Last observed `main` before this state-file commit was `6b0c23791ba9cd7c788b904edd014bb7b403f009`. Latest gameplay SHA is `c201365c09c8f65417c8391ec28121683a085702`, including 512×512 procedural surfaces.
- The comparison from gameplay SHA `c201365c09c8f65417c8391ec28121683a085702` to the earlier docs HEAD `4ae93bf815b27d5aad57ff70f7363422abaf00ff` showed documentation changes only. Read-back after the current docs refresh confirms README, FEATURES and TODO were updated; no gameplay source was changed in these documentation commits.
- Run #107 passed formatting, ECS release tests, release build, 10-second Windows startup smoke and artifact upload on gameplay SHA `c201365c09c8f65417c8391ec28121683a085702`. Artifact `neon-frontier-windows-x64`: 29,867,514 bytes. The smoke ran on a Windows Actions runner, not Fox's laptop.
- Runs #105 and #106 also passed all stages. Their artifacts were 29,867,227 and 29,867,226 bytes.
- A Windows workflow on the documentation HEAD after the latest read-back has not yet appeared in the runs list. The exact current-HEAD Windows result therefore remains pending; the game code itself matches #107's tested tree.
- The `sun_peaks_at_noon_and_bottoms_out_at_midnight` comparison is corrected; #107 passed the test suite with ECS diagnostics.
- A successful runner smoke does not replace Fox's manual Windows 11 launch and gameplay check.

## Remaining work
- `TODO.md` has 24 open items: 23 gameplay requirements across P0–P3 plus one Windows check on the exact current `main`.
- Highest priorities remain complete ECS OBB integration, stable vehicle pair separation, in-game collision regression tests, vehicle handling, police behavior, menu/F5/F9 integration checks, and missions/events. P3 remains deferred.
- Manual launch and gameplay by Fox on Windows 11, including Russian text, menu, F5/F9, collisions, police and performance, remain unverified.
- Require passing formatting, tests, build, startup smoke and artifact upload on the exact current `main` before claiming that the current-HEAD workflow is green.

A successful compile or an older green run does not complete the remaining gameplay work.
