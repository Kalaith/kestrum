# Fresh battle review — 2026-09-30

Reviewed the B01–B07 implementation from `ca8c87f` on master: pure combat,
campaign preparation/commit, application command guards, immutable replay,
doctrines, leaders, save loading and battlefield composition. This review uses
the actual checkout and shared Cargo/capture/publish tools.

## Fixed findings

| Priority | Finding and player consequence | Correction and regression |
| --- | --- | --- |
| P1 | Start Battle calls `GameState::command` while the Battlefield overlay is open, but its guard rejects that command as obstructed. Live encounters cannot start even though direct engine tests pass. | Authorize StartPendingBattle only at the pending battlefield boundary. `visible_start_battle_commits_through_the_application_guard_once` exercises the real screen guard, duplicate rejection and unrelated order blocking. The aftermath capture now opens preparation before starting. |
| P2 | Assigning, replacing or clearing a leader in pending preparation updates simulation input but leaves the witnessed report leader unchanged. Validation rejects the legal edit. | Synchronize report leader IDs with live formations before recomputing preparation. Regression checks four assignment transitions, save compatibility, invalid rival assignment rollback and the immutable committed receipt. The native review-save generator exposed the original error through a normal command. |
| P2 | Brace can kill a charger, but the subsequent attack still inflicts the minimum one casualty. | Revalidate the attacker after retaliation. `lethal_brace_retaliation_cancels_the_chargers_damage` preserves all defenders. |
| P2 | Incoming Guard does not react to Attack/Volley and applies its reduction twice against Charge/Breakthrough. An earlier Brace also obscures a later eligible Guard. | Evaluate applicable reactions for every attack; Brace remains charge-only and raised Guard reduces damage once. Table-driven coverage checks all four attacks. |
| P2 | A later unit's rout shock can reduce an already visited ally to zero morale, leaving it active into the next round. | Process eligible routs to completion in stable ID order. Each unit clears its slot once. Regression covers a shock that routes an earlier ID. |
| P2 | Round-limit comparison includes routed survivors as remaining combat power, potentially turning a win into a stalemate. | Count only active positions while preserving living routed troops in the receipt. Regression pins the changed outcome without inventing casualties. |

The four resolver regressions failed against the reviewed implementation before
their fixes. Resolver version 3 identifies corrected results. Existing receipts
remain immutable and readable; already saved pending resolutions commit their
saved outcome unless the player edits preparation and refreshes it.

Six new cases target these distinct faults. Existing battle suites have more
than five tests because positioning, reactions, support abilities and campaign
transactions retain independent regression coverage.

## Remaining presentation findings

These are outside the completed correctness fixes and remain actionable:

- **P2 — multiple army presentation:** `src/ui/battlefield/projection.rs`
  shifts each successive army 116 logical pixels outward. With four armies on
  a side, rear centers reach x=-20 or x=1300 on a 1280-wide canvas. Legal boards
  can be clipped and adjacent army groups overlap. The current dense scene is
  twelve units from two armies total; it does not exercise maximum army counts.
- **P2 — target priority authoring:** rules persist `target_priority`, but the
  preparation editor offers only action, condition and target-filter cycling.
  Players cannot author the supported strength/resistance priorities or see
  which priority a selected rule uses.
- **P2 — minimum-size touch controls:** tactic rows and leader arrows are only
  18 logical pixels high. They are visible and mouse-selectable at 1280×720,
  but the dense editor needs larger targets/reflow for comfortable touch use.
- **P3 — aggregate morale:** the header averages groups equally, including
  destroyed groups, although the plan specifies a troop-weighted summary.
  Per-unit values are preserved correctly.

## Verification

The original all-features suite passed before fixes; earlier README failure
notes were stale. The subsequent complete run passed all but three targets:
threats, diplomacy and the seed-88 production victory. Threat expectations now
pin the legitimate Guard reduction (four casualties rather than five).
Diplomacy now pins DefenderVictory and one surviving spear after lethal Brace,
instead of relying on the dead charger's hit to manufacture mutual destruction.

The **production victory remains a balance blocker**: the unchanged seed-88
script does not reach victory within its 240-round cap after corrected
reactions/routs. At the cap the player is still independent, two opponents are
eliminated, and one independent opponent remains. The failed assertion is
preserved. A legal deployment/extra-archer experiment also failed to win, so its
exploratory test changes were removed. This review does not claim restored
production balance or a completely passing suite.

Focused combat, doctrines, playback, application guard, diplomacy, threats and
source-size results are recorded below. Formatting and Clippy pass. The required
no-parameter publish passes Windows/WebGL builds, packaging, Preview deployment,
tracker recording and catalogue synchronization.

Final focused command: `cargo.ps1 test -p kestrum --test battle_formation --test
battle_playback --test battle_doctrines --test combat --test diplomacy --test
threats --test code_standards` — **59 passed**. Clippy with all targets/features
and warnings denied, package formatting, source-size gate and diff checks pass.

Native evidence replaces stable captures for preparation, the five-row editor,
twelve-unit playback and aftermath at 1920×1080 and 1280×720. All ten captures
were inspected. The shared wrapper completed and its launched games exited.
The actual campaign aftermath capture also passes through the Battlefield
application guard. Screenshots establish these supported scenes, not
maximum-board usability, live browser input or physical touch acceptance.

## Native midgame save

The original #21 fixture below is superseded by the
[developed midgame campaign](midgame-campaign.md), installed as #25 on 2026-10-01
with broad discovery, settlements, rival borders and ten named adults.

`examples/prepare_midgame.rs` creates a separate native catalogue save through
the toolkit's normal writer and game SaveLibrary APIs. Its production campaign
uses seed 88, recruits three Warrior formations, clears an opening threat,
returns to supplied headquarters and advances NPC turns through 60 completed
rounds. It accepts incoming peace and resolves player victories through normal
commands; no time, army health or treasury is fabricated. Preparation stores the
Ranged Support doctrine and Homeward Line template.

The installed entry is **#21, Briarhold - Midgame Battle Review**, Year 16,
60 completed rounds, eight retained battle receipts, no pending encounter and
a surviving player army. The payload reloads identically. Earlier native entries
remain in place. Native debug startup now loads Continue once storage is ready;
`--title` selects the title screen explicitly. Captures, benchmarks, native
release and WebGL startup keep their existing behavior.

The second focused command covers combat, doctrines, persistence, the new
midgame fixture and source-size checks: **33 passed**. It verifies save equality,
immutable battle history and subsequent ordinary End Turn behavior.

The actual pooled native debug run restored Spring, Year 16 / Round 61 without
a title-screen action. Live mouse navigation opened Rose Host at Westmere
Green: six formations, 580 troops, six remaining movement and headquarters
supply. Treasury showed 101 Gold, 1436 Wood and 927 Stone; the last upkeep was
fully paid. Closing the verification game returned exit code 0 and released its
native writer. The catalogue still selects #21 and retains the five earlier
entries. These observations used the computer-use skill on the actual window.

The explicit `--title` option also opens the title screen. The shared
`rust_management/cargo.ps1` has a separate forwarding issue: a single game
argument becomes a scalar and splats into characters (`- - t i t l e` in the
observed native command line). Passing `--title --title` through the same
launcher preserves both strings and verified the option. This tool issue is
recorded without modifying another project's files; it does not affect normal
argument-free run or `cargo run -- --title` directly. All verification windows
exited successfully and no game process remains.

Final no-parameter `publish.ps1` also passes after the leader snapshot and debug
startup changes: Windows/WebGL release builds, packaging, Preview deployment,
Project Roost recording and catalogue synchronization complete successfully.
