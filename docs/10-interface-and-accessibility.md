# 10 Interface and accessibility

[Documentation index](README.md) · [Shared UI authority](../UI_STYLE.md) ·
[Active map plan](map-playability-plan.md)

## Status and goals

This chapter owns the target interface for the map improvements. The
[project README](../README.md#map-controls) describes the implemented controls.
The redesign is planned. Earlier K/B captures establish only their recorded
scenes and interactions; the current [evidence index](verification/README.md)
records remaining limitations.

The player needs to recognize a strategic situation, act on a place or force,
and understand the consequence. Keep the world dominant, with one contextual
inspector and quiet supporting information. The current sparse HUD hides useful
state; reducing visible information further is not the objective.

## Viewport targets

Normal target is a usable 1920×1080 canvas. Minimum supported landscape canvas
is 1280×720. Check native and embedded/full-screen WebGL separately; a virtual
resolution does not establish browser usability. Smaller portrait support is
outside current scope.

Use visible touch controls, with ordinary primary controls at least 48 logical
pixels high at the minimum canvas. Reflow or collapse secondary information
before shrinking text or targets. Dense battlefield tactic controls currently
fall short of this target and remain a recorded issue. Physical-device touch
testing is waived under the existing scope; mouse-equivalent checks must be
reported as such.

## Screen briefs

### World and region map

| Question | Planned behavior |
| --- | --- |
| Current decision | Choose where to expand, defend, invest or redirect a force. |
| Dominant focus | Known territory, meaningful places and active armies. |
| Primary action | Issue the selected object's relevant order with its cost and constraints visible. End Turn remains separate. |
| Supporting information | Calendar, actual resources/income/upkeep, owned force status, known threats, relevant supply and a compact attention list. |
| Deferred information | Detailed accounts, composition, tactics, biographies, household administration, historical filters and utilities. |
| Layout and camera | Keep most of the canvas for the map. Collapse the attention list at 720p when an inspector opens. Fit known land at a useful scale and retain orientation across views. |
| Input and feedback | Tap objects and visible controls; drag/pinch or use zoom/recenter. Selection, saved orders and important consequences survive the end of transient feedback. |

Territory display distinguishes political claim, physical occupation, contested
control and unknown geography. Kingdom labels and capital symbols establish
orientation. Settlement silhouettes reflect their current tier; relevant
resource/facility symbols and growth, construction, siege or damage states
explain purpose. Labels depend on zoom, importance and available space.

A regional map depicts its own terrain and approaches, with roads and bridge/pass
symbols placed meaningfully. Its breadcrumb and return control preserve the
world context. The chosen first delivery retains separate world and region
scopes; a seamless camera is deferred.

### Map orders and selection

Direct movement is implemented: tap Army, then a destination to execute the
affordable travel immediately and retain any remaining route. End Turn continues
saved orders after refresh. Cancel Route stops the plan; Review Route inspects
it. Peaceful borders and encounter rules remain authoritative.

The planned inspector keeps the object, cost, allowance, supply and expected
action together. Common recruitment, construction, focus and diplomacy entry
points stay in that context. Existing spending, battle and diplomatic review
rules remain where relevant. Movement does not acquire a second confirmation.

Army banners show identity, owned strength and status; dense stacks offer a
readable selection list. Relevant saved routes remain visible after the card
closes. Idle, moving, blocked, besieged and unsupplied states use understandable
symbols plus text or shape. Unknown enemy strength and plans stay hidden.

### Seasonal outcomes and attention

Planned seasonal feedback prioritizes problems requiring a decision, then
completed orders and useful changes. Selecting an entry focuses the place,
army or person. Unresolved conditions remain discoverable; informational entries
can be dismissed. Preserve receipt/acknowledgement state across reload without
repeating actions or misleading the player about which season changed a value.

The attention list is a route into the map, not a second management dashboard.
Collapse quiet categories. A history link provides supporting detail when
needed; full biographies remain deliberate inspection.

### Introduction and contextual help

The implemented guide teaches movement, region navigation, careers, households,
End Turn and Records. M04 replaces it with a useful opening action and visible
payoff: secure a nearby opportunity, resolve its consequence, invest the benefit
and respond to a frontier. Prompts use actual legal choices and exact visible
control labels; alternative actions can satisfy the lesson.

Teach careers after relevant service and households when their context matters.
Dismiss/resume stays visible and save-compatible. Existing players are not
forced into a new guide. Ordinary play retains concise labels and accessible
help after prompts end.

### Named characters within the army roster

Six formation slots remain the composition focus. Each occupied row shows its
one named member with troop type, or troops alone; headcount is separate.
Starting Warriors read `Lord Name + Warriors`, recognized people can read
`Hero Name + Archers`. Commander appointment is a separate role.

Transfers into an already staffed formation explain the restriction. Preserve
troop type and headcount when shortening names; People exposes full identity,
Career, History and Transfer. Dedicated composition views remain appropriate
when comparison is the decision. See [army membership](04-armies-and-logistics.md#characters-belong-inside-formation-slots).

### Settlement and kingdom decisions

Current Manage contains Overview, Build, Roads, Focus and Local Actions.
Population, growth causes, damage, facilities, last income and work orders
already exist. The redesign makes key consequences visible on the map and
brings common orders into the selected-place inspector.

War, peace and defeat decisions remain explicit, with their consequences and
disabled reasons. Foreign-place inspection links to known controller information.
Urgent diplomatic decisions stay discoverable after dismissal. Save/settings
utilities must be visually separate from gameplay decisions.

### Battle and siege

Battle preparation, playback and aftermath share a landscape. Preparation
selects a formation for legal slot, tactic and leader edits. Start Battle resolves
once; play/pause, step, speed and skip present the immutable result. Continue
returns to the campaign and applies continuation once. Reports retain losses,
wounds, locations and observed enemy facts.

Siege choices depend on side and available forces. Keep the place, legal exits,
participants, supply, elapsed seasons, damage and known relief risk together.
Dedicated preparation/comparison views are appropriate; preserve the route back
to the affected place. Do not reveal hidden enemy rosters through map warnings.

### Saved campaigns

The catalogue supports named saves and seasonal checkpoints, with pagination
and no game-imposed slot cap. Overwrite and Delete confirm the selected entry.
Naming provides a visible touch keyboard. Retry and Continue Unsaved preserve a
failed snapshot without replaying the season. Storage/writer conflicts explain
recovery. Utility screens stay on demand.

## Visual and interaction review

For each changed milestone, review normal/minimum sizes, early fog, developed
territory, long labels, dense stacks, expanded inspectors, multiple orders and
urgent conditions. Exercise selection, dismissal, movement, cancellation,
regional boundaries, seasonal feedback and save/reload through visible controls.
Check picking after zoom/resize and ensure drag release never issues an order.

Judge whether the player can locate their realm, choose a useful action and
explain its result. Readability and unclipped controls are necessary but do not
prove that loop. Store stable captures directly in `docs/verification/`, follow
the shared capture/publish workflow, and record remaining limitations in the
[evidence index](verification/README.md).
