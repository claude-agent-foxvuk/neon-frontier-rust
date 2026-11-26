# Neon Frontier — project state

`FEATURES.md` records implemented behavior; `TODO.md` contains open requirements grouped by priority. Source code and observed Windows Actions runs are the project record.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness, traffic-stream, beach and water work stays deferred until higher-priority items are closed.
- GitHub reports this repository as public.

## Current code and verification
- Windows run #129 passed on game commit `7ab41523dbf27a02e016e66e657303da4b0c77d8`: formatting, release tests with ECS diagnostics, release build, 10-second startup smoke and artifact upload all succeeded.
- Run #129 produced `neon-frontier-windows-x64` (29,878,829 bytes). Its live ECS tests cover player-vehicle/pedestrian contact and reject an old circle-only false positive.
- After #129, only documentation changed; project code remains exactly the tested code on `7ab41523dbf27a02e016e66e657303da4b0c77d8`.
- The artifact was built on a GitHub Windows runner, not Fox's laptop. Fox's Windows 11 launch and full gameplay session remain unverified.
- `TODO.md` has 22 open gameplay requirements: P0 — 5, P1 — 3, P2 — 4, deferred P3 — 10. Run #129 does not close manual testing or the broader OBB integration item.
- Higher-priority work remains: full OBB integration for walking-player and vehicle contacts with traffic/people, live vehicle/building regressions, police response for wanted levels 3–6, menu and F5/F9 integration checks, missions, HUD scaling, performance measurements, and manual Windows 11 testing.
- Do not close a gameplay requirement solely because a commit was accepted or one CI run passed; verify relevant source and test evidence.

A green CI run confirms only the steps completed on its runner. It does not prove a full gameplay session, RTX 3050 frame rate, Cyrillic rendering on Fox's laptop, or manual Windows 11 testing.