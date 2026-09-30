# Formation membership verification — 2026-09-30

The army roster previously showed only troop types in its six formation slots,
while naming the lord in the separate army Commander summary. The design already
called for `Squire Elian + Warriors`. The state already attached the founding
lord to Warriors; this follow-up makes that membership visible for every named
formation member, including non-commanders and characters emerging from Archers.

Rows now show `Lord Catrin Cairn + Warriors`, an untitled `Elian Dale + Archers`,
or `Hero Elian Dale + Archers` after formal recognition. A formation with several
people uses one representative and an explicit `1 other` / `N others` count.
The representative is the attached commander, otherwise a lord, recognized hero,
then the lowest stable person ID. Long names shorten while preserving troop type,
member count and current/capacity headcount. Tier and movement share the second
line. Wounds do not erase continuing membership; dead, displaced and site-assigned
people are excluded. Formation and person transfer rules, headcounts, capacity,
slot usage, upkeep, combat and persistence are unchanged.

The complete [formation-slot contract](../04-armies-and-logistics.md#characters-belong-inside-formation-slots)
defines the examples, membership, multi-person ordering, transfers, wounds,
headcount conservation, overflow and save behavior. Character development,
interface design, implementation rules and the README now repeat or link the
contract at the relevant decision.

## Checks

All validation used the actual checkout and its registered shared workspace.

| Check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Passed. |
| Shared Cargo strict Clippy, all targets with `-D warnings` | Passed after the final display correction. |
| Shared Cargo tests: founding_lord, economy, movement, progression, knowledge, code_standards | All 31 cases passed. Existing emergence and transfer regressions preserve membership and headcount; no rendering-mirroring test suite was added. |
| Expanded founding_lord regression plus source-size gate | All six cases passed after explicitly checking the founding lord's Warriors assignment, full headcount/capacity and three occupied slots; the final data-label recheck also passed. |
| Shared hidden-window captures | Five supported states passed at each native size. The first dense capture exposed missing evidence counters in its fixture; the fixture was repaired, validated and recaptured. |
| Final `.\publish.ps1` with no arguments | Passed Windows/WebGL release builds, packaging, Preview deployment, Project Roost recording and catalogue update. |

The previously recorded full-suite long-campaign failures are not addressed by
this presentation change. See [founding-lord verification](founding-lord.md) for
the two existing simulation blockers and diagnostics. That full suite was not
repeated for this roster correction.

## Native visual review

The capture wrapper saved directly to the stable paths below. Normal windows
requested 1920 × 1080 and produced 1920 × 1061 client images; minimum captures are
1280 × 720. The transferred-member scene uses the real TransferPerson command.
Archer and dense scenes are isolated, semantically validated capture fixtures;
they do not write production saves. Transfer images were refreshed after fixing
the singular others label.

The wrapper's game processes exited after both successful captures and the
repaired fixture failure; no Kestrum process remained at the final check.

| State | Normal | Minimum |
| --- | --- | --- |
| New kingdom, lord within Warriors | [Army](ui_founder_army.png) | [Army](ui_founder_army_minimum.png) |
| Six Warriors members, one wounded, and a recognized archer | [Members](ui_formation_members.png) | [Members](ui_formation_members_minimum.png) |
| Named archer before recognition | [Emergence](ui_formation_emerged.png) | [Emergence](ui_formation_emerged_minimum.png) |
| Lord transferred from Warriors to Archers | [Transfer](ui_formation_transfer.png) | [Transfer](ui_formation_transfer_minimum.png) |
| Maximum-length person names and several members | [Long names](ui_formation_long_name.png) | [Long names](ui_formation_long_name_minimum.png) |

Review followed `UI_STYLE.md`. Composition remains the dominant focus and the
right-hand summary retains the army's leadership, command appointment and supply.
Troop identities, membership counts, headcount and movement remain readable at
both sizes, without overlap or clipped actions. The 64-character wide-letter
names shorten in the row and wrap in the commander summary. No extra panel or
slot is created. Selection rectangles remain 52 logical pixels high; visible
Orders, Back, training and recruitment controls remain reachable. Individual
full names, condition, Career, History and Transfer remain in Orders > People.

## Browser review and limitations

At the published Preview's 1920 × 1080 fullscreen viewport, Continue restored
the previous campaign. Army > Army Details visibly showed `Lord Catrin Cairn +
Warriors`, 100/100 Warriors and 98.3% leadership. Selecting the Archers row moved
the highlight; Train Formation opened the Archers specialization view with its
correct 80 headcount, and Back returned to the roster.

At 1280 × 720, the previously documented WebGL scaling issue remained: the game
occupied a smaller upper-left area with black space around it. This review cannot
sign off minimum browser usability or physical-device touch. Native minimum
layout passes, and the visible pointer controls were exercised at normal browser
size. Temporary browser viewport overrides were reset and the review tab closed.
