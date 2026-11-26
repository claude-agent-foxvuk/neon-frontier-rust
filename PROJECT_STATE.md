# Neon Frontier — project state

`FEATURES.md` records implemented behavior; `TODO.md` contains open requirements grouped by priority. Source code and observed Windows Actions runs are the project record.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness, traffic-stream, beach and water work stays deferred until higher-priority items are closed.
- GitHub reports this repository as public.

## Current code and verification
- Windows run #128 passed on game commit `0547cb38881024b5f37d1cbe634142edabdd0015`: formatting, release tests, release build, 10-second startup smoke and artifact upload all succeeded.
- Run #128 produced `neon-frontier-windows-x64` (29,878,829 bytes). Its code commit adds OBB/SAT collision between the player vehicle and buildings while driving and when loading a saved vehicle pose.
- Current main `7ab41523dbf27a02e016e66e657303da4b0c77d8` adds live ECS regressions for player-vehicle/pedestrian impact and a circle-only false positive; Windows run #129 is checking that commit.
- The latest green artifact was built on a GitHub Windows runner, not Fox's laptop. Fox's Windows 11 launch and full gameplay session remain unverified.
- `TODO.md` still has 22 open gameplay requirements: P0 — 5, P1 — 3, P2 — 4, deferred P3 — 10. The new tests cover only the live player-vehicle/pedestrian contact slice.
- Higher-priority work remains: full OBB integration for the walking player and vehicle contacts with traffic/people, live vehicle/building regressions, police response for wanted levels 3–6, menu and F5/F9 integration checks, missions, HUD scaling, performance measurements, and manual Windows 11 testing.
- Do not close a gameplay requirement solely because a commit was accepted or one CI run passed; verify relevant source and test evidence.

A green CI run confirms only the steps completed on its runner. It does not prove a full gameplay session, RTX 3050 frame rate, Cyrillic rendering on Fox's laptop, or manual Windows 11 testing.
