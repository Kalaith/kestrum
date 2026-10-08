# Formation and hero progression baseline

Fresh pre-rule-change baseline for the formation-hero progression handoff. This records the current checkout's deterministic run; it does not reconstruct a prior build.

## Run

Command:

```powershell
..\rust_management\cargo.ps1 run -p kestrum --locked --release --example observe_campaign '--' 260926 200 4
```

The pooled launcher completed successfully. Seed 260926 ran with four factions through round 200 (3,603 steps, 90.9 seconds); the campaign had not naturally finished at the requested horizon. Three factions were eliminated by round 200. Checkpoints are the replay's reported faction state and old-emergence diagnostics.

## Checkpoints

| Round | Faction | Status / armies | People, emerged, invites, births | Attached apprentices / Heroes | Hero armies | Other notes |
|---:|---|---|---|---|---|---|
| 40 | Rose | Active, 2 | 8 / 0 / 4 / 3 | 0 / 0 | 0/2 | |
| 40 | Ashen Lark | Active, 2 | 9 / 1 / 5 / 2 | 1 / 0 | 0/2 | |
| 40 | Bera Lark | Active, 0 | 5 / 0 / 4 / 0 | 0 / 0 | 0/0 | |
| 40 | Ashen Wren | Active, 2 | 7 / 0 / 4 / 2 | 0 / 0 | 0/2 | |
| 80 | Rose | Active, 4 | 10 / 1 / 4 / 4 | 0 / 0 | 0/4 | 1 death |
| 80 | Ashen Lark | Active, 3 | 12 / 3 / 5 / 3 | 3 / 0 | 0/3 | Apprentice in 1 army |
| 80 | Bera Lark | Eliminated, 0 | 5 / 0 / 4 / 0 | 0 / 0 | 0/0 | Eliminated at round 56 |
| 80 | Ashen Wren | Active, 4 | 8 / 0 / 4 / 3 | 0 / 0 | 0/4 | 1 death |
| 120 | Rose | Active, 5 | 13 / 3 / 4 / 5 | 0 / 0 | 0/5 | 1 death; 30 unstaffed formations; 5 leaderless armies |
| 120 | Ashen Lark | Active, 4 | 12 / 3 / 5 / 3 | 0 / 1 | 1/4 | 2 leaderless armies |
| 120 | Bera Lark | Eliminated, 0 | 5 / 0 / 4 / 0 | 0 / 0 | 0/0 | |
| 120 | Ashen Wren | Active, 5 | 9 / 0 / 4 / 4 | 0 / 0 | 0/5 | 1 death; 30 unstaffed formations; 5 leaderless armies |
| 160 | Rose | Active, 7 | 12 / 2 / 4 / 5 | 0 / 0 | 0/7 | 42 unstaffed formations; 7 leaderless armies |
| 160 | Ashen Lark | Active, 2 | 15 / 3 / 8 / 3 | 0 / 0 | 0/2 | 1 death; 8 unstaffed formations; 2 leaderless armies |
| 160 | Bera Lark | Eliminated, 0 | 5 / 0 / 4 / 0 | 0 / 0 | 0/0 | |
| 160 | Ashen Wren | Active, 6 | 10 / 1 / 5 / 4 | 1 / 0 | 0/6 | 1 apprentice army; 35 unstaffed formations; 5 leaderless armies |
| 200 | Rose | Active, 8 | 18 / 2 / 10 / 5 | 0 / 0 | 0/8 | 44 unstaffed formations; 8 leaderless armies |
| 200 | Ashen Lark | Eliminated, 0 | 15 / 3 / 8 / 3 | 0 / 0 | 0/0 | Eliminated at round 180; 2 deaths |
| 200 | Bera Lark | Eliminated, 0 | 5 / 0 / 4 / 0 | 0 / 0 | 0/0 | Eliminated at round 56; 2 deaths |
| 200 | Ashen Wren | Active, 6 | 16 / 2 / 9 / 5 | 0 / 2 | 2/6 | 1 death; 28 unstaffed formations; 4 leaderless armies |

People/emergence/invitation/birth values are the replay's faction report at that checkpoint; invitation and birth counters are lifetime counts. Army coverage is named Hero members divided by current armies.

## Old emergence opportunity totals at round 200

These diagnostics describe the existing path before simulation-rule changes. “Eligible” is the number of formation service seasons where the audit found qualifying service; “selected” is the old rule's highest-service source selection. The old rule can select an occupied formation, and selection is not itself an emergence.

| Faction | Service seasons | Eligible | Selected | Occupied candidates | Emerged from selected source | Chance misses | Zero-XP selections | No-candidate seasons |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| Rose | 2,538 | 220 | 137 | 46 | 3 | 32 | 102 | 63 |
| Ashen Lark | 1,311 | 200 | 127 | 99 | 3 | 29 | 95 | 73 |
| Bera Lark | 111 | 4 | 16 | 14 | 0 | 1 | 15 | 184 |
| Ashen Wren | 1,811 | 195 | 142 | 29 | 2 | 32 | 108 | 58 |

A `chance miss` is a selected source with a computed chance and no emergence; a zero computed chance is also reported separately. Existing observer logs did not reliably distinguish created people from incomplete action receipts; this baseline motivated typed life-event and before/after-state audit changes.

## Stage E — pacing and lifecycle result (2026-10-08)

This is a new replay after the deterministic vacancy-service and personal-Hero
rules; the baseline above remains unchanged. The final data thresholds are one
qualifying engagement on a vacant formation to produce an Apprentice and one
later qualifying engagement personally fought while fit to grant Hero status.
The one-plus-one pair is the minimum meaningful path: at emergence the person
has zero Hero service, and the later battle awards exactly once. A one-plus-two
recognition comparison produced the same Hero-bearing army coverage at rounds
120 and 200, so the final personal threshold stayed at one.

Command:

```powershell
..\rust_management\cargo.ps1 run -p kestrum --locked --release --example observe_campaign '--' 260926 200 4
```

The run completed 3,263 observer steps in 193.4 seconds and reached the round
cap without ending the campaign. Hero-bearing army coverage was:

| Round | Rose | Ashen Lark | Ashen Wren | Total |
|---:|---:|---:|---:|---:|
| 80 | 3/5 | 1/3 | 1/3 | 5/11 |
| 120 | 4/8 | 2/4 | 0/1 | 6/13 |
| 160 | 5/8 | 1/8 | 0/1 | 6/17 |
| 200 | 5/8 | 1/8 | 0/1 | 6/17 |

The midgame target is **not met**: at round 120 only 6 of 13 living armies have
a Hero, below a majority. At round 120 there were 13 attached Apprentices in
Rose, five in Ashen Lark and none in Ashen Wren; these remain distinct from
recognized Heroes. At round 200 the remaining attached Apprentices had 0/1
personal service. They had not fought a later qualifying battle, so lowering the
recognition threshold further would not help. The replay points to post-emergence
battle exposure and army distribution as the remaining pacing problem; it does
not justify granting Hero credit for service the named person did not witness.

Lifetime emergence diagnostics recorded threshold reached/emerged counts of
Rose 39/39, Ashen Lark 17/17, Bera 2/2 and Ashen Wren 19/19. This confirms that
the final vacancy threshold creates a person whenever its qualifying condition
is reached; it does not show that most armies subsequently earn a Hero.

## Validation

Passed:

- `cargo fmt -p kestrum -- --check`
- `..\rust_management\cargo.ps1 test -p kestrum --locked --test code_standards`
- Focused `hero_progression` tests, including transfer, retirement, replacement
  and save/reload continuation
- Focused AI paused-step replay, evidence-retention and formation-recovery
  save/replay regressions
- `..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings`
- 400-round four/eight-faction continuity test

The no-fail-fast full suite completed with five failing targets:

- `ai_growth`: three fixtures require 24 developable sites but only nine
  non-ruined, non-headquarters sites are available.
- `ai_recovery`: two fixtures reject their authored truce date or retained
  economy receipt as invalid.
- `k18_integrated_people`: its wounded-commander scenario instead destroys the
  defending formation and kills Mira Dusk.
- `k18_production_battle`: seed 88 reaches Defeat instead of the expected
  Victory at the 240-round cap; this is the documented production-victory
  baseline exception.
- `midgame`: the active-rival fixture has fewer than ten supplied settlements.

These five failures were not retested against pristine HEAD in this run, so this
report does not classify them as proven pre-existing failures. They are outside
the files changed by Stage E except for the documented production-victory
baseline; keep them visible as unresolved integration results. The source-size
gate passed. The hidden native wrapper recaptured and exited successfully for
`ui_formation_hero_progress.png`, `ui_formation_hero_earned.png` and
`ui_formation_hero_detail.png` at 1920×1080; all three were visually inspected.
No physical-touch playtest or publication was performed.

The deterministic discovery and Hero award path is working. The user's
majority-of-armies-by-midgame goal remains open until campaign rules create
enough later qualifying exposure for Apprentices spread across the active armies.
