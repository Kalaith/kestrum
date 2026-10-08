# Formation-to-Hero discovery — 2026-10-08

This report records Stage D of
[the implementation plan](../implementation/formation-hero-progression.md).
The visible path is **Warriors → Apprentice Name + Warriors → Hero Name +
Warriors**. This checkpoint validates presentation and scripted UI intents;
the integrated campaign replay and full-project validation are still pending.

## Captures

All images came from the hidden native capture wrapper at the supported
1920 × 1080 canvas. The wrapper exited successfully for all four scenes.

| State | Capture | Evidence |
| --- | --- | --- |
| Formation roster after emergence | [Apprentice in Archers](ui_formation_emerged.png) | Layout fixture; the seeded state is presentation evidence only. |
| Apprentice after personal service | [Career, 2/3](ui_formation_hero_progress.png) | Real accepted battles created the apprentice and recorded two later qualifying personal engagements. |
| Hero in the formation | [Hero in Warriors](ui_formation_hero_earned.png) | Real accepted battles completed emergence and three later qualifying personal engagements. |
| Hero Career detail | [Career, recognized](ui_formation_hero_detail.png) | Same accepted-command path, followed by the People and Career UI intents. |

The army footer exposes **People** beside the existing Orders control. Formation
rows and the Career title both use the Apprentice/Hero stage while retaining the
person's portrait and troop assignment. The Career detail shows personal
qualifying engagement progress while the apprentice is still becoming a Hero.
The army panel leaves the existing map and camera visible behind it.

## Accepted-path capture and interaction

The `formation_hero_progress`, `formation_hero_earned` and
`formation_hero_detail` capture scenes use the production campaign setup. The
fixture declares war through an accepted command, then repeats accepted Move,
StartPendingBattle, EndTurn and NPC-pass commands. Two formation engagements
produce the apprentice; the progress scene runs two more personal engagements,
and the earned-Hero scenes run the third. Capture assertions check the
recognition state, progress and unchanged Warriors assignment.

The detail capture then dispatches the same `ArmyPeople` and
`OpenPersonProgression` intents used by the visible controls and asserts the
Career detail is selected. This verifies the app action route, not a physical
touchscreen or human playtest. The separate `formation_emerged` image remains a
seeded layout fixture and is not used as progression evidence.

Capture command:

```powershell
.\scripts\capture_ui.ps1 -Scenes formation_hero_progress,formation_hero_earned,formation_hero_detail -Frames 30 -WindowWidth 1920 -WindowHeight 1080
.\scripts\capture_ui.ps1 -Scenes formation_emerged -Frames 30 -WindowWidth 1920 -WindowHeight 1080 -SkipBuild
```

## Remaining evidence

This presentation checkpoint does not establish how often production armies
encounter enough qualifying battles to have a Hero by mid-game. The seed
260926/four-faction/round-200 replay, lifecycle replacement scenario and
full-project validation remain the pacing and integration gates in Stage E.
