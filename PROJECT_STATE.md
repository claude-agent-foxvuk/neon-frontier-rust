# Neon Frontier — project state

`FEATURES.md` records implemented behavior; `TODO.md` contains open requirements grouped by priority. Source code and observed Windows Actions runs are the project record.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness, traffic-stream, beach and water work stays deferred until higher-priority items are closed.
- GitHub reports this repository as public.

## Current code and verification
- Windows run #127 passed on game commit `d7da475ebc3e049eacbf97d9c04bccd447f1b638`.
- Commit `0547cb38881024b5f37d1cbe634142edabdd0015` adds OBB/SAT collision between the player vehicle and buildings during driving, applies the oriented check when loading a saved vehicle position, and adds a swept-motion regression.
- Windows run #128 for that commit is in progress; its checks and artifact are not confirmed yet. The latest known artifact is `neon-frontier-windows-x64` from #124, 29,878,427 bytes.
- Fox's laptop launch and full gameplay session remain unverified.
- `TODO.md` still has 22 open gameplay requirements: P0 — 5, P1 — 3, P2 — 4, deferred P3 — 10. The new vehicle/building OBB slice does not close the broader player and traffic integration items.
- Higher-priority work includes full OBB integration for the walking player and vehicle contacts with traffic/people, live vehicle/building regressions, police response for wanted levels 3–6, menu and F5/F9 integration checks, missions, HUD scaling, performance measurements, and manual Windows 11 testing.
- Do not close a gameplay requirement solely because a commit was accepted or one CI run passed; verify relevant source and test evidence.

A green CI run confirms only the steps completed on its runner. It does not prove a full gameplay session, RTX 3050 frame rate, Cyrillic rendering on Fox's laptop, or manual Windows 11 testing.