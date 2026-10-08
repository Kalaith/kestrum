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
| Apprentice at emergence | [Career, 0/1](ui_formation_hero_progress.png) | A real accepted formation engagement created the apprentice; no personal service has yet been credited. |
| Hero in the formation | [Hero in Warriors](ui_formation_hero_earned.png) | Real accepted battles completed emergence and one later qualifying personal engagement. |
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
StartPendingBattle, EndTurn and NPC-pass commands. One formation engagement
produces the apprentice; the progress scene keeps the apprentice at zero
personal service, and the earned-Hero scenes run one later personal engagement.
Capture assertions check the
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

At the time of this Stage D checkpoint, production pacing, the lifecycle
replacement scenario and full-project validation remained open. Stage E ran
those checks and recorded the current majority-coverage shortfall and full-suite
results in [the progression verification report](formation-hero-progression.md).
