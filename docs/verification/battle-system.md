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
ordered typed events. It has no graphics, campaign mutation, frame-time input,
or random draws.

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
