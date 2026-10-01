# Formation membership verification — 2026-10-01

Each army slot holds one named person plus ordinary troops, or troops alone.
The previous implementation allowed eight people in one formation and rendered
`Ashen Cairn + 7 others + Warriors`. The roster now shows a separate formation
for each named person. Transfers and service entry reject a second person;
emergence, AI deployment, disbanding and combat recovery choose an unstaffed
formation or preserve the person at the local site when every formation is staffed.

Older saves redistribute stacked members among available same-faction formations
at the same physical site, preferring the least staffed army. The commander stays
with their formation. Overflow remains at the site without losing people, moving
them across the map, or inventing troops. Historical battle receipts, troop
headcounts, RNG state and identifier counters survive the migration. Saves carrying
the new roster schema reject duplicate assignments rather than silently repairing them.

The midgame generator recruits additional formations through normal paid orders
and patrols learner armies together. Its ten living named adults occupy ten
different formations across three armies: two in Rose Host, six in Army 5 and
two in Army 12. The two other armies contain troops alone. Native Continue now
loads review save #29, written and reloaded identically through the catalogue API.
Older entries remain available.

The [formation-slot contract](../04-armies-and-logistics.md#characters-belong-inside-formation-slots)
defines assignments, transfers, wounds, overflow and save compatibility.

## Validation

All checks use the actual checkout and registered shared workspace.

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed. |
| Shared Cargo strict Clippy, all targets with `-D warnings` | Passed after the final transfer-label and capture changes. |
| Shared Cargo full integration suite, `--tests --no-fail-fast` | 315 passed, one failed, one profiling case ignored. |
| Six new formation-person regressions | Passed: atomic transfer rejection, service entry, eight-person legacy distribution across three armies, local overflow conservation, modern invalid-save rejection and disband fallback. |
| Midgame regressions | All five passed; every living named person has a distinct formation, with at least three staffed armies. |
| Source-size gate | Passed; every Rust file remains within the 800-line limit. |
| Shared hidden-window captures | Eight supported states reviewed at normal and minimum native sizes. |
| `.\publish.ps1` with no arguments | Passed Windows and WebGL release builds, packaging, Preview deployment, Project Roost recording and catalogue update. |

The sole full-suite failure is the previously documented
`four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
deadline: no production victory by round 240. A baseline run before this rule
change reproduced it. The four- and eight-faction 400-round continuity cases pass.
This change does not tune the production victory script or campaign balance.

## Native visual review

The wrapper captures directly to the stable paths below. Normal windows request
1920 × 1080 and produce 1920 × 1061 client images; minimum captures are 1280 × 720.
The midgame scene uses the real generator. Founder transfer uses the actual
TransferPerson command; the staffed-target scene uses the real transfer UI and
command preview. Dense and long-name fixtures are semantically validated.

| State | Normal | Minimum |
| --- | --- | --- |
| Founding lord with Warriors; troops-only Spearmen and Archers | [Army](ui_founder_army.png) | [Army](ui_founder_army_minimum.png) |
| Seven named people in separate formations across two armies; one wounded | [Members](ui_formation_members.png) | [Members](ui_formation_members_minimum.png) |
| Named archer before recognition | [Emergence](ui_formation_emerged.png) | [Emergence](ui_formation_emerged_minimum.png) |
| Lord transferred from Warriors to unstaffed Archers | [Transfer](ui_formation_transfer.png) | [Transfer](ui_formation_transfer_minimum.png) |
| Maximum-length names in separate formation slots | [Long names](ui_formation_long_name.png) | [Long names](ui_formation_long_name_minimum.png) |
| Real midgame Army 5, six named people in six formations | [Midgame](ui_midgame_army.png) | [Midgame](ui_midgame_army_minimum.png) |
| Transfer into a troops-only formation | [Recipient](ui_transfer_person.png) | [Recipient](ui_transfer_person_minimum.png) |
| Transfer into a staffed formation, long recipient name, disabled confirmation | [Blocked](ui_formation_transfer_blocked.png) | [Blocked](ui_formation_transfer_blocked_minimum.png) |

Review follows `UI_STYLE.md`. The six-slot comparison stays dominant, with
leadership, upkeep and supply in the existing supporting summary. Troop identities,
headcounts and movement remain readable without overlap. Long names shorten while
preserving troop type and headcount; the commander's full name wraps. Transfer
recipients show their named occupant and explain the blocked choice beside the
disabled Confirm Transfer control. Formation rows are 52 logical pixels high,
transfer rows 50, and action buttons 48. Full names and Career, History and Transfer
remain available through Orders > People. Capture processes exit after completion.

## Browser review and limitations

The published [Preview](http://127.0.0.1/games/kestrum/) was reloaded after deployment.
At a 1920 × 1080 fullscreen viewport, Continue restored the previous browser
campaign. Army > Army Details showed `Lord Catrin Cairn + Warriors`, troops-only
Spearmen and Archers, and preserved headcounts. Visible Orders > People > Transfer
controls opened the local army, showed the named occupant in Warriors, and enabled
Confirm Transfer when troops-only Spearmen were selected. Cancelling returned
without changing the assignment.

At 1280 × 720, the existing WebGL scaling problem remains: the game occupies a
smaller upper-left area with black space around it. Picking after resize also
mismatches the displayed rows: selecting visible Archers selected Warriors.
Minimum browser usability therefore remains blocked, despite passing native
minimum layouts. Physical-device touch is untested. The normal browser review
used visible pointer controls; the temporary viewport override was reset and the
review tab closed. No native capture process remained at the final process check.
