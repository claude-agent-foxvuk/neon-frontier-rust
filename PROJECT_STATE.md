# Neon Frontier — project state

`FEATURES.md` records what exists in source; open requirements are grouped by priority in `TODO.md`. Source and observed Actions results are the project record.

## Scope and decisions
- Windows 11, Rust 1.100+, Bevy 0.20.1; original compact open-world action-driving prototype.
- Save format remains `NEON_FRONTIER_SAVE_V1`.
- No coding agents or paid services. Commit directly through GitHub API to `claude-agent-foxvuk/neon-frontier-rust`.
- P3 witness, traffic-stream, beach and water work stays deferred until higher-priority items are closed.
- GitHub currently reports this repository as public.

## Code and checks
- The current `main` reference is `0342b8047169cf938f67dd4fa8ac7154f8b01319`.
- Latest code workflow is #123 on `40edf8d5263adf985b8ab94419876766ba15791b`. Formatting passed; ECS release tests were in progress at the latest read. Release build, Windows startup smoke and artifact are not confirmed yet.
- #120 on `8ed4594e383ca80b906a13749b425eaadf8ad248` passed formatting and ECS release tests; release build was in progress. Smoke and artifact are not confirmed.
- #121 on `cc9bc7a1eaa754376331b91527defafeed376c7d` and #122 on `430a637e85da0ea08fe8d313261bd818fa3157da` passed formatting; ECS release tests were in progress. Build, smoke and artifact are not confirmed.
- #119–#123 were reported in progress in the latest run list. Check each result before treating its code as verified.
- The last fully verified game workflow remains #117 on `1d6212c7b1dc6546ca43cc989d9ac1a203bb7f3e`: format, 114 ECS release tests, release build, 10-second Windows startup smoke and upload succeeded. Artifact `neon-frontier-windows-x64` was 29,866,360 bytes. Smoke on a GitHub Windows runner does not confirm a manual launch on Fox's laptop.
- The smoke in #117 did not trigger B0001 or B0004 checks. The B0001 report from Fox concerned the older executable from #66; a short startup smoke does not prove the full gameplay loop.
- `sun_peaks_at_noon_and_bottoms_out_at_midnight` compares `sun_height(1.5)` to its wrapped equivalent `sun_height(0.5)` and passed in #115.
- #111–#113 exposed a wrong wanted-level expectation for traffic hitting a pedestrian without a player witness. #114 corrected the expectation and added a separate test for the player's vehicle hitting a pedestrian.
- Vehicle inertia, coasting drag, progressive braking, self-centering steering, speed-dependent turning and lateral grip are implemented and passed Windows checks in #117.
- An ECS regression for traffic stopping before the walking player's OBB was committed as `40edf8d`; its Windows run #123 is still in progress.

## Remaining work
- `TODO.md` lists 22 open gameplay requirements: P0 — 5, P1 — 3, P2 — 4, deferred P3 — 10.
- Higher-priority open work includes complete OBB integration in player movement and vehicle collisions, body/building regression coverage, police response for wanted levels 3–6, menu and F5/F9 integration checks, missions and city events, HUD scaling, performance measurements, and manual Windows 11 testing.
- Do not close a gameplay requirement solely because its commit was accepted or one CI run passed; verify the relevant source and test evidence.
- Fox's manual launch and gameplay check on Windows 11 remains unconfirmed. Only Fox can verify the build on his laptop, its input/display behavior and Cyrillic rendering there.

A green CI run confirms the steps it actually completed on its runner. It does not prove a complete gameplay session, RTX 3050 frame rate or a manual laptop test.