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
browser check displayed the title at 1920×1048. The battle prototype is not yet
reachable from a live campaign in the browser; B03 will add that route and its
browser interaction review. Browser battle evidence is therefore pending B03.

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
