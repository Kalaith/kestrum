# K12 — Rival orders, diplomacy and kingdom endings

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `1d9ddcd` (K11). Work stops after this package at the user's request.

## Gameplay

Rivals use ordinary validated commands for recruitment, movement, construction,
local threats, sieges, retreat and HQ relocation. Planning preserves objectives,
uses owned forces and observer knowledge, and has a persisted command budget.
Unknown enemy presence does not reveal private strength. Automatic progression and
Pause/Step use the same deterministic orders.

War declarations, peace offers and four-round truces use explicit commands. Peace
withdraws foreign occupants through legal adjacent exits; one missing exit rejects
the agreement without changing the campaign. Incoming offers and defeated rivals
stop simulation for a player decision and can be saved at that boundary.

Losing a capital alone is insufficient for defeat. A faction falls after losing
all functioning Outpost-or-larger bases and armies. Annexation eliminates it;
player submission makes it inactive, with no army, income or revival. Surviving
people become displaced, open works close, and existing vassals follow the
surviving conqueror. Victory requires every rival to be eliminated or the player's
vassal. The player's defeat takes precedence in mutual destruction.

Terminal actions consume their real participation facts once without inventing a
seasonal boundary. Ending checkpoints and manual saves retain the final battle,
history and outcome. Reloaded campaigns allow inspection and saving but reject
simulation commands. Old saves receive diplomacy/AI defaults only when the whole
new group is absent; partially missing or forged state is rejected atomically.

Delegated choices and bounds are recorded in
[I11](../13-decisions-and-open-questions.md#i11--bounded-sovereign-decisions-and-lasting-outcomes).

## Behavioral checks

All **127 tests** pass through the shared launcher:
`..\rust_management\cargo.ps1 test --all-features --no-fail-fast --quiet`.
This includes the 800-line source gate with no exceptions. Formatting, strict
Clippy across all targets/features with `-D warnings`, and `git diff --check` pass.

- Five AI cases cover affordable legal orders/reserves, local prerequisites and
  private knowledge, retained objectives/emergencies, bounded rejection/budget,
  and identical automatic/stepped seeded phases despite observation timing.
- Five diplomacy cases cover War/Peace/truce, atomic withdrawal and siege lifting,
  capital loss versus actual defeat, both defeat choices and vassal inheritance,
  and reachable victory/defeat including mutual last-army destruction.
- Four integrity cases cover old-save migration, malformed timers/budgets/endings,
  pending-decision phase resumption, and public versus private history.
- Two additional persistence regressions cover a terminal partial-round checkpoint
  and manual saving while an NPC peace offer awaits the player.

The conquest regression recruits three formations at their actual price, moves
through the small graph, fights full-strength founding forces and resolves all
three rivals through real commands before checking the saved ending. Its scenario
removes initial threats and forts to isolate the field-conquest path; rival phases
pass explicitly. Separate tests cover real AI expansion/attacks, forts and sieges.
This is not a full production-world balance playthrough.

Earlier seasonal subsystem tests now pass NPC phases explicitly so they test
their own boundary rules independently of the new strategic policy. Replay and AI
tests continue to exercise actual NPC planning. Inactive-faction fixtures release
their forces and displace survivors rather than retaining impossible armies.

## Interface review

Kingdom relations and sovereign choices are dismissible views reached from Menu
or a rival place. Normal play keeps the atlas dominant. The ending exposes Records,
Save, Menu and New Game and calls itself the kingdom milestone.

Eight native captures cover relations, incoming peace, defeated-rival choice and
the reloaded ending at 1920×1080 fullscreen and 1280×720. Review found the current
decision, consequences, 48-pixel minimum actions and Back/Save routes legible,
without clipping or overlapping controls. The fixture exercises actual commands
and serialized reloads. Capture processes exited; the final minimum and refreshed
normal runs were PIDs 3828 and 15020. Stable evidence:

| Decision | Normal | Minimum |
| --- | --- | --- |
| Relations | [Capture](ui_diplomacy_relations.png) | [Capture](ui_diplomacy_relations_minimum.png) |
| Peace response | [Capture](ui_diplomacy_peace_response.png) | [Capture](ui_diplomacy_peace_response_minimum.png) |
| Defeat choice | [Capture](ui_diplomacy_defeat_choice.png) | [Capture](ui_diplomacy_defeat_choice_minimum.png) |
| Reloaded ending | [Capture](ui_diplomacy_ending_reload.png) | [Capture](ui_diplomacy_ending_reload_minimum.png) |

## Publication and browser verification

No-argument `publish.ps1` passed: Windows release 58.08 s and WebGL release
47.37 s, both packages, Preview deployment, catalogue refresh and Project Roost
tracking. No alternate checkout or dependency configuration was used.

The published game at `http://127.0.0.1:8765/kestrum/` restored the earlier K11
Summer Year 1 save. Menu > Kingdom > Declare War > Confirm Action changed Oak
from Peace to War. End Turn ran the real NPC policy and returned to the player's
Autumn Year 1 / Round 3. A separate manual save, `Campaign 3`, was created through
visible controls. Reload and Continue restored that season and the Oak War
relation. Warning/error logs were empty. The browser verification tab was closed.
[Reloaded diplomacy evidence](ui_diplomacy_browser_reload.png).

Normal browser play used a 1936×1048 canvas. A 1280×720 viewport override produced
a correct title screen but a campaign rendered into a smaller upper-left area
with black space, despite DOM canvas/bounds reporting 1280×720. Reload reproduced
that inherited campaign scaling problem; resetting the override restored normal
rendering. Minimum WebGL acceptance therefore remains open for K18. Native
minimum captures do not establish browser or physical-touch acceptance. The
verified browser flow required only visible pointer controls and no keyboard.

## Remaining scope

K13–K18 remain planned. Emergence, ordinary careers, aging, mentorship, succession,
items/eras and the 80-node production world are not completed by K12. Physical
touch hardware and integrated platform acceptance remain K18 checks.
