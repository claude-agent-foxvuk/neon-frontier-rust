# Neon Frontier — project state

`FEATURES.md` records implemented behavior; `TODO.md` contains open requirements grouped by priority. Source code and observed Windows Actions runs are the project record.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness, traffic-stream, beach and water work stays deferred until higher-priority items are closed.
- GitHub reports this repository as public.

## Current code and verification
- `main` is `adac32dd2bdf7e41ad50ce0806f756fc64312e8e`.
- The last successful Windows build check is #124 on game commit `17302cc6c54792c175252b0e322b256a680cf4db`. Format, ECS diagnostic release tests, release build, 10-second Windows startup smoke, and executable upload passed.
- Artifact `neon-frontier-windows-x64` for #124 was 29,878,427 bytes. The smoke ran on GitHub's Windows runner; Fox's laptop launch and full gameplay session remain unverified.
- Commit `Add ECS pedestrian collision regression` adds a live system test that separates two overlapping pedestrians by OBB without raising wanted level. Windows run #125 on this commit was in progress at last read.
- `TODO.md` still has 22 open gameplay requirements: P0 — 5, P1 — 3, P2 — 4, deferred P3 — 10. The new regression provides coverage for one slice of P0 collision work, not completion of the broader requirement.
- Higher-priority work includes full OBB integration in player movement and vehicle collisions, body/building regression coverage, police response for wanted levels 3–6, menu and F5/F9 integration checks, missions, HUD scaling, performance measurements, and manual Windows 11 testing.
- Do not close a gameplay requirement solely because a commit was accepted or one CI run passed; verify relevant source and test evidence.

A green CI run confirms only the steps it completed on its runner. It does not prove a full gameplay session, RTX 3050 frame rate, Cyrillic rendering on Fox's laptop, or manual Windows 11 testing.