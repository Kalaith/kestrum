# K05 — Six-slot armies and seasonal economy

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `871bdd1` (K04), with a clean project checkout.

## Delivered behavior

New campaigns instantiate four authored armies, twelve full formations and four
Officer founders. Stable identities, six slots, current/capacity headcounts,
assignments, leadership contribution and per-member spent movement are persisted.
The public projection includes only the observer's military records.

Recruitment and disbanding use atomic commands shared by player and NPC actors.
Recruitment checks the acting faction, local ownership/security/supply, capacity,
resources, deficits, facilities and site tags. All six troop types have exact
costs and explanatory errors. Recruits start exhausted; a new army can be created
at the same site. Disbanding refunds nothing, retains named people and removes
empty armies without reusing identities. Removing a zero-headcount formation
uses the same identity cleanup for subsequent combat integration.

At the common seasonal boundary, secure local income and one eligible HQ bonus
precede full-formation upkeep. Gold never becomes debt. Shortfalls block recruits
and clear only after a later boundary pays all upkeep. The last statement exposes
actual income, paid/due upkeep and shortfall. Movement resets once. Headcount
recovery, movement orders and transfers remain the scheduled K06 implementation.

I04 documents earlier-v2 migration: no invented army, founder, service or income;
Recruit can create a first army at the saved date. Current and earlier schema-2
catalogue entries retain their metadata and original bytes.

## Automated validation

The actual checkout passed `cargo fmt -p kestrum -- --check`, locked strict
all-target/all-feature Clippy and all **38 tests**, including the source-size gate.
The five K05 tests cover:

1. Authored grants, IDs, slots, people, projection, current round-trip, earlier
   toolkit envelopes and indexed catalogue loads. Migration permits fresh legal
   recruitment without changing the old payload.
2. Every troop type, exact costs, full capacities, new exhausted formations,
   existing/new armies and equivalent NPC prerequisites.
3. Rejected ownership, phase, cost, supply, slot, facility, tag, damage and deficit
   orders preserve the complete campaign, counters and RNG. Army overlays allow
   military commands while blocking End Turn.
4. Income-before-upkeep arithmetic, no duplicate HQ income, full upkeep despite
   losses, damage and contested effects, exact deficit clearing, movement reset
   and malformed statement/deficit save rejection.
5. No-refund disbanding, founder reassignment, last-formation army removal,
   zero-headcount cleanup and no resurrected identities.

The final economy suite was rerun after the last migration regressions; all five
passed. Existing strategic-phase expectations now include real income/upkeep.

## Native visual review

The established hidden capture wrapper produced and replaced stable evidence at
1920 × 1080 fullscreen and 1280 × 720. Processes 1332 and 3792 exited. Reviewed
normal rosters, all six slots, empty armies, unavailable facilities, a seventh-slot
rejection, normal/shortfall statements, disband confirmation, last-formation
removal, long names, combined long-name/deficit state, headquarters entry and Help.

Evidence is `ui_army*`, `ui_recruit*`, `ui_disband*`, `ui_headquarters*` and
`ui_help*`; each new scene has a `_minimum` counterpart. The dense capture uses
64-character army and commander names. Review moved Disband to the footer and
removed a redundant commander-name repetition so long summaries remain readable.
Recruitment cards retain cost, capacity, upkeep and missing prerequisite text.
All six slots and 48-pixel action targets fit at the minimum size. Help names
Armies, Recruit, New Army, Confirm Recruit, Disband and Confirm Disband.

## Published browser interaction

The actual Preview WebGL build ran in the Codex in-app browser at a measured
1936 × 1048 fullscreen canvas. Pointer-only interaction:

- Loaded the earlier K03 Round 5 save and opened an empty army roster.
- Recruited Warriors: 500/200/150 became 440/190/150, with 100/100 headcount,
  no invented commander and a visible exhausted-until-next-round explanation.
- Completed the round: income 50/19/12, upkeep 10/10, zero shortfall and final
  resources 480/209/162. Reload/Continue preserved the same formation and statement.
- Started a fresh campaign: Rose Ward had three founding formations and Aveline
  Rose, with the expected 93.3% leadership contribution.
- Selected Riders and saw its missing Stable with Confirm Recruit disabled.
- Created a separate army with Warriors, opened/cancelled its disband prompt,
  then confirmed removal. The roster returned to the founding army, with no refund.

No browser error or warning log entries were reported. The inherited forced-720p
browser screenshot/input scaling mismatch remains unresolved; native minimum-size
captures do not close that platform criterion. Physical touch/pinch remains
unverified. Both limitations remain explicit K18 work.

## Publication and continuation

The no-argument publisher passed Windows and WebGL release builds, packages,
Preview deployment, catalogue update and Project Roost tracking. It passed again
after updating the host page's army controls and milestone description.

Commit subject: `Kestrum musters its founding armies (K05 armies and economy)`.
Next: K06 physical movement, composition, supply-connected headcount recovery.
K06–K18 remain required; this package is not full-release completion.
