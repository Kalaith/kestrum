# Battle system verification

Implementation record for B01–B07, maintained as milestones are completed.

## B01 — deterministic formation combat

**Status:** complete on 2026-09-30. The resolver remains headless and is not
connected to live campaign outcomes; B03 owns that transaction boundary.

`assets/data/battle_tactics.json` now authors bounded battle limits, damage and
morale factors, and legal default rules for all six troop kinds. Kestrum loads
and validates this content through `GameData`. The public
`engine::resolve_battle` accepts a plain opening snapshot and returns a versioned
resolution with its opening data, final group positions and counts, outcome, and
ordered typed events. The receipt preserves opening morale with the revision,
so a future balance edit cannot change the displayed initial condition. It has
no graphics, campaign mutation, frame-time input, or random draws.

Every army retains six slots; up to four boards per side share one initiative
order. Ordinary melee cannot pass an occupied front, while an empty front opens
its rear to ordinary melee and a cleared partner slot permits Breakthrough.
Activations inspect rules in order and record why earlier rules did not run.
Brace executes before cavalry charge damage, has one reaction budget per unit
per round, and cannot start another reaction. Guard expires at the end of its
round. Advance consumes its activation and moves a rear group into its paired
empty front slot. Casualties reduce morale; routed survivors leave their slot,
remain in the result, and lower allied morale in stable order. Equal initiative
alternates side priority each round, then resolves by army, slot and unit ID.

### Behavioral evidence

The ten `tests/battle_formation.rs` cases cover protected and reachable rear
targets, cavalry exploiting a cleared lane, tactic fallthrough, reaction order
and budget, morale rout and ally shock, Advance, alternating initiative, repeat
determinism, the bounded all-Wait stalemate, and multiple boards with a tagged
threat aggregate.

### Validation

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed. |
| `cargo.ps1 test -p kestrum --test battle_formation --all-features` | Passed: 10 cases. |
| `cargo.ps1 test -p kestrum --test code_standards` | Passed: source-size gate. |
| `cargo.ps1 test -p kestrum --all-features` | Stopped at `campaign_scenarios::four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds`; campaign ended at round 188. This is the long-campaign continuity limitation called out in the implementation plan. B01 does not route campaign battles through this resolver. |
| `publish.ps1` with no parameters | Passed Windows and WebGL release builds, packaging, Preview deployment, tracker recording and catalogue synchronization. |
| Battlefield visual review | Not applicable to B01; no UI changed. B02 requires the normal and minimum size visual review and durable captures. |

The B01 implementation commit is recorded in the project history. Detailed
playback storage, campaign receipts, combat balance and scene presentation remain
in their later milestones.

## B02 — Battlefield presentation

**Status:** complete on 2026-09-30. The screen is a playback prototype staged
from the same deterministic fixture used by the resolver. Campaign battles
continue to use the existing system until B03 adds the committed receipt and
encounter entry point.

The battlefield now places both six-slot forces across a shared hillside. Side
identity, army strength, morale, front and rear rows, individual troop kinds,
headcounts and morale bars stay next to the scene. Troop silhouettes distinguish
spears, bows, riders, medics and siege engines. Playback shows cavalry movement,
Brace posture, volleys, damage pulses, opened slots, routed survivors and the
recorded result. Group selection outlines a battlefield slot and opens a compact
condition strip. Pause, step, 1×/2×/3× and skip-to-result controls only advance
the display cursor over an immutable receipt.

The six `tests/battle_playback.rs` cases cover opening/cursor bounds, damage and
morale projection, Advance positions, guard expiry, routed survivors and receipt
immutability. The existing ten B01 resolver cases pass unchanged.

### Visual evidence

The capture harness produced these states at 1920×1080 and the minimum 1280×720.
Every image is stored directly in this directory and can be regenerated with
`scripts/capture_ui.ps1` using its `battle_scene_*` IDs.

| State | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Twelve-group opening scene | [image](ui_battle_scene_dense.png) | [image](ui_battle_scene_dense_minimum.png) |
| Centre lane cleared | [image](ui_battle_scene_gap.png) | [image](ui_battle_scene_gap_minimum.png) |
| Cavalry breakthrough | [image](ui_battle_scene_charge.png) | [image](ui_battle_scene_charge_minimum.png) |
| Spear Brace reaction | [image](ui_battle_scene_brace.png) | [image](ui_battle_scene_brace_minimum.png) |
| Archer volley | [image](ui_battle_scene_volley.png) | [image](ui_battle_scene_volley_minimum.png) |
| Impact | [image](ui_battle_scene_impact.png) | [image](ui_battle_scene_impact_minimum.png) |
| Rout and withdrawal | [image](ui_battle_scene_rout.png) | [image](ui_battle_scene_rout_minimum.png) |
| Aftermath | [image](ui_battle_scene_aftermath.png) | [image](ui_battle_scene_aftermath_minimum.png) |
| Selected group | [image](ui_battle_scene_selected.png) | [image](ui_battle_scene_selected_minimum.png) |

The 1920×1080 captures show the landscape filling the scene above a shallow
control strip. The 1280×720 captures keep all twelve groups, lane labels, group
labels and playback actions visible. The selection strip, charge trail, center
gap, reaction posture, volley and rout are distinguishable at both sizes.

The deployed Preview page loads its embedded game and full-screen canvas. Its
embedded canvas measures 734×414 in the current browser surface, below the
declared minimum, and offers a visible full-screen control. The full-screen
browser check displayed the title at 1920×1048. B03 integrated battle into the
live campaign flow; browser battle interaction remains unverified because the
open campaign contains unsaved orders and its replacement prompt was cancelled.

### Validation

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed after final formatting. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed. |
| `cargo.ps1 test -p kestrum --test battle_playback --all-features` | Passed: 6 playback contracts. |
| `cargo.ps1 test -p kestrum --test battle_formation --all-features` | Passed: 10 resolver cases. |
| `cargo.ps1 test -p kestrum --test code_standards --all-features` | Passed: every Rust file remains under 800 total lines. |
| `cargo.ps1 test -p kestrum --all-features` | Stopped at `campaign_scenarios::four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds`; the campaign ended at round 188. All reported targets before that test, including both battle suites, passed. |
| `publish.ps1` with no parameters | Passed Windows and WebGL release builds, packaging, Preview deployment, tracker recording and catalogue synchronization. |
| Browser review | Preview and full-screen title loaded. The battle scene is pending live campaign integration in B03. |

## B03 — Campaign encounter integration

**Status:** complete on 2026-09-30. Field, threat and siege contacts prepare a saved
pending battle using the deterministic resolver. The current player accepts it
from the battlefield scene; the committed receipt applies formation and threat
losses, retreats, person events, site consequences and one battle fact. A pending
movement receipt preserves the route spent before field contact. Player-participant
NPC attacks pause on the pending encounter until acceptance, and report playback
projects the stored receipt without mutating campaign state.

Threat rewards that would overflow are rejected before a pending battle can
strand the campaign. Siege-engine wall reduction is applied to defender
resistance and stored as the opening battle's effective wall factor for every
playback exchange.

### Checkpoint evidence

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed. |
| `cargo.ps1 test -p kestrum --test combat '--' --nocapture` | Passed: 7 focused campaign-combat cases, including saved pending acceptance, no duplicate confirmation, replay immutability and NPC phase pause. |
| `cargo.ps1 test -p kestrum --test movement --test ai --test development_integrity --test k18_integrated_scenarios` | Passed: 16 campaign movement, NPC decision, compatibility and retreat/rematch cases. |
| `cargo.ps1 test -p kestrum --test siege` | Passed: 5 cases, including relief acceptance, assault wall factors, siege reconciliation and persistent damage. |
| `cargo.ps1 test -p kestrum --test threats` | Passed: 5 cases, including threat contact, exact losses, reward overflow, replay and reclamation. |
| `cargo.ps1 test -p kestrum --test k18_siege_continuity` | Passed: saved relief battle commits identically after reload. |
| `cargo.ps1 test -p kestrum --test code_standards` | Passed: the Rust source-size gate. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed. |
| `cargo.ps1 test -p kestrum --test k18_integrated_people` | Known balance failure reserved for B07: Hawthorn's second counterattack destroys the Rose Medics formation and kills its attached apprentice at completed round 7, so this older two-treatment progression route cannot continue. The casualty and succession assertion remains intact. |

### Integration visuals

The stable capture harness now stages a real campaign movement contact, saved
pending encounter and accepted aftermath. It uses the same command and resolver
path as the campaign. Capture processes exited after completion.

| State | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Pending battle with the current-plan action | [image](ui_battle_campaign_pending.png) | [image](ui_battle_campaign_pending_minimum.png) |
| Committed campaign aftermath | [image](ui_battle_campaign_aftermath.png) | [image](ui_battle_campaign_aftermath_minimum.png) |

### Remaining B03 verification

The focused campaign, movement, relief/assault and threat suites pass. The
Rosemarch medic progression outcome and the 120-round production victory fixture
remain for B07's casualty pacing and campaign balance work. The full test run
reaches round 284 of the four/eight-faction continuity case before an NPC
`EndTurn` is rejected because its pending battle remains; B07 owns this case.
`publish.ps1` built
Windows and WebGL releases and deployed the Preview target. The browser preview
loaded the updated title screen and fullscreen presentation. Its existing
Continue entry indicated a local campaign; the New Game confirmation warned that
unsaved orders would be left behind, so it was cancelled to preserve that state.
The pending and aftermath campaign visuals therefore rely on the real campaign
capture harness at both supported review sizes; browser battle interaction has
not yet been exercised.

## B04 — Deployment and tactics authoring

**Status:** complete on 2026-09-30. Each formation can save validated activation
and reaction rules. The pending battlefield exposes a compact editor for the
selected friendly group, including row order, legal action/condition/target
cycles, explicit resolver fallback, and swaps with adjacent slots. The editor
rebuilds the pending receipt deterministically without consuming random state or
replaying the accepted movement edge. Existing saves without a tactics field use
the authored troop defaults.

The Rider default waits while enemy cavalry remains, then uses Breakthrough when
an enemy rear becomes exposed. The resolver test proves both rounds and confirms
the ordered authored rules affect its action.

### Behavioral evidence

| Check | Result |
| --- | --- |
| `cargo.ps1 test -p kestrum --all-features --test battle_formation --test battle_playback --test combat --test movement --test recovery --test ai --test siege --test threats --test k18_siege_continuity --test code_standards` | Passed: 55 cases covering combat, immutable playback, preparation authorization, save/reload, slot identity, strategic integration and source size. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo.ps1 test -p kestrum --all-features` | Stopped at the four/eight-faction 400-round continuity case: an NPC `EndTurn` met an unresolved pending battle at round 284. Tracked for B07. |
| `publish.ps1` with no parameters | Passed Windows and WebGL release builds, packaging, Preview deployment, tracker recording and catalogue synchronization. |

### Visual evidence

The campaign capture harness stages movement contact, pending preparation and
committed aftermath. The selected-group editor includes a dense five-row state
with long army names. Capture processes exited after completion.

| State | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Pending battle and selected-group editor | [image](ui_battle_campaign_pending.png) | [image](ui_battle_campaign_pending_minimum.png) |
| Five-row editor and long army names | [image](ui_battle_campaign_editor_dense.png) | [image](ui_battle_campaign_editor_dense_minimum.png) |
| Committed campaign aftermath and long names | [image](ui_battle_campaign_aftermath.png) | [image](ui_battle_campaign_aftermath_minimum.png) |

The minimum-size dense capture keeps all five rows, the fallback, group labels
and playback or confirmation controls visible. Both sizes preserve the
battlefield as the dominant area. The open browser contains a local campaign;
starting a new one warned that unsaved orders would be left behind, so that
action was cancelled to preserve the existing save. Live browser battle input
remains unverified.

## B05 — Leaders and support roles

**Status:** complete on 2026-09-30. Formations can save a selected battle leader
without moving other attached people or replacing the army commander. Eligibility
requires the selected person to be fit, attached to the formation, unretired,
old enough for field service and compatible with the troop role. Older campaign
saves that omit the new field inherit an eligible attached army commander for
that formation; an explicit `null` remains unassigned. Pending edits rebuild the
receipt deterministically, and the immutable opening snapshot records leader
identity, class, active status and granted capabilities.

An Officer can use Rally once per battle to restore morale; the deterministic
regression case triggers it below half morale. An Infantry leader can Hold the
Line once by moving into the paired open front slot and guarding through that
round. A Cavalry leader can charge an exposed enemy rear before ordinary
activations. Medics stabilize the most shaken living ally's morale without
restoring headcount. Siege Engines bombard once at the opening and take increased
close-combat damage; the existing siege-wall interaction still contributes to
the battle's recorded resistance.

If a selected leader becomes wounded, retires or transfers to another formation,
the stored selection remains visible as unavailable. Rally and Hold the Line
rows identify the missing leader prerequisite, resolve as unavailable and fall
through to the visible basic attack-or-wait behavior. Older receipts without a
leader snapshot remain readable.

### Behavioral evidence

| Check | Result |
| --- | --- |
| `cargo.ps1 test -p kestrum --all-features --test battle_formation --test battle_playback --test combat --test movement --test recovery --test ai --test siege --test threats --test k18_siege_continuity --test code_standards` | Passed: 64 cases covering Rally bounds, infantry movement/guard, exposed-rear cavalry opening, Medic morale support, siege bombardment and weakness, leader selection, old-save migration, invalid prerequisites, immutable playback, campaign consequences and source size. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed. |
| `cargo fmt --all -- --check` | Passed. |
| Source-size gate (`cargo.ps1 test -p kestrum --all-features --test code_standards`) | Passed; every Rust file remains within the 800-total-line limit. |
| `publish.ps1` with no parameters | Passed Windows and WebGL release builds and packaging, Preview deployment, tracker recording and catalogue synchronization. |

The full all-features campaign suite remains scheduled for B07. The B05 run
stopped at the four/eight-faction continuity scenario at round 284: NPC faction
4 could not end its phase while a pending battle remained. B07 also owns the
Rosemarch medic progression casualty route and the 120-round production victory
fixture.

### Visual evidence

The campaign capture harness stages selected, unavailable, dense and aftermath
states. A separate deterministic battlefield capture shows the Cavalry leader's
opening charge into an exposed rear. Each capture below was inspected at both
supported sizes, and each capture process exited after completion.

| State | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Cavalry Exploit Opening in the shared battlefield | [image](ui_battle_scene_leader.png) | [image](ui_battle_scene_leader_minimum.png) |
| Selected active battle leader | [image](ui_battle_campaign_leader.png) | [image](ui_battle_campaign_leader_minimum.png) |
| Wounded leader and unavailable Rally row | [image](ui_battle_campaign_unavailable.png) | [image](ui_battle_campaign_unavailable_minimum.png) |
| Pending preparation | [image](ui_battle_campaign_pending.png) | [image](ui_battle_campaign_pending_minimum.png) |
| Five-row tactics editor with long army names | [image](ui_battle_campaign_editor_dense.png) | [image](ui_battle_campaign_editor_dense_minimum.png) |
| Committed campaign aftermath | [image](ui_battle_campaign_aftermath.png) | [image](ui_battle_campaign_aftermath_minimum.png) |

The unavailable state now distinguishes a missing leader from no selection and
places the row reason inside the inspector. The default fallback label fits at
both sizes. The existing browser tab retains a local campaign and was left
unchanged; live browser interaction remains unverified.

## B06 — Doctrines and rival preparation

The six troop roles each receive legal Defensive Line, Ranged Support and
Breakthrough rule blocks from the authored tactics catalogue. Choosing a
doctrine snapshots its defaults onto non-customized formations and preserves
explicit tactics overrides, including legacy saves whose override marker did
not yet exist. Campaign-local personal plans save role, slot and tactic blocks
without storing character or formation identities; plans can be selected,
updated and applied from the pending battle's selected-group inspector.

Faction rivals choose a doctrine and deployment deterministically from their
visible troop types and observed slot reports. The planner cannot read enemy
tactics or leader assignments. Rivals can move ranged and support groups behind
the line, exploit an observed rear gap with Breakthrough and meet a charge with
the same Spear Brace reaction available to the player. No special combat bonus
is granted for AI preparation.

### Behavioral evidence

| Check | Result |
| --- | --- |
| `cargo.ps1 test -p kestrum --all-features --test battle_doctrines --test battle_formation --test battle_playback --test combat --test movement --test recovery --test siege --test threats --test k18_siege_continuity --test code_standards` | Passed: 60 cases, including six B06 doctrine/template/rival cases, battle action and playback behavior, campaign consequences, sieges, threats and the source-size gate. |
| `cargo.ps1 test -p kestrum --all-features --test ai` | Four of five cases pass. `private_enemy_changes_do_not_grant_strength_and_empty_hostile_land_is_captured` now chooses `OfferPeace` after its newly doctrine-prepared battle records a recent faction loss; the test expects an attack. This encounter balance and peace pacing regression is carried into B07 without suppressing the assertion. |
| `cargo fmt --all -- --check` | Passed. |
| `cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings` | Passed without warnings. |
| `publish.ps1` with no parameters | Passed Windows and WebGL release builds, packaging, Preview deployment, tracker recording and catalogue synchronization. |

The expanded B06 controls were inspected in captures at both supported sizes;
the selected-group inspector remains compact above the shared battlefield.
Live browser interaction was not re-tested in the existing campaign tab.

| State | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Pending battle with Ranged Support and the saved “Northern Screen” plan selected | [image](ui_battle_campaign_doctrine.png) | [image](ui_battle_campaign_doctrine_minimum.png) |
