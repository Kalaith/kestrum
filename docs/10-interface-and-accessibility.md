# 10 Interface and accessibility

[Documentation index](README.md) · [Shared UI authority](../UI_STYLE.md) ·
[Active map plan](map-playability-plan.md)

## Status and goals

This chapter owns the implemented M01 overview, M01A spatial navigation and
the target follow-on map interface. M01 has
[scoped verification](verification/kingdom-overview.md). M01A is complete under
its [recorded scope](verification/spatial-scale.md), including engineering checks,
14 native captures, automated native actions, background browser gameplay,
publishing and reload. M02–M05 remain unstarted. M01A establishes a more spacious
map at the sole 1920×1080 spec.
The [project README](../README.md#map-controls)
describes controls. Earlier K/B captures establish only their recorded scenes
and interactions; the [evidence index](verification/README.md) records remaining
limitations.

The player needs to recognize a strategic situation, act on a place or force,
and understand the consequence. Keep the world dominant, with one contextual
inspector and quiet supporting information. M01 brings useful known state into
that view while deferring detailed management until the player selects it.

## Viewport specification

The user's 2026-10-01 correction establishes a usable **1920×1080 canvas as the
sole design and acceptance spec**, including logical UI coordinates. It replaces
the earlier dual-resolution target. The map HUD and title now compose directly
in 1920×1080 logical coordinates. Controls retain their intended pixel sizes;
the map uses the added space for geography.

Check native and embedded/full-screen WebGL at an actual 1920×1080 canvas;
window decorations and host chrome do not count toward its usable area.
Other host sizes may scale or letterbox this same composition with the toolkit's
viewport mapping. They do not require separate layouts or visual acceptance.
Historical smaller-size captures retain their original results.

Use visible touch controls, with ordinary primary controls at least 48 logical
pixels high at the 1920×1080 design canvas. Reflow or collapse secondary information
before shrinking text or targets. Dense battlefield tactic controls currently
fall short of this target and remain a recorded issue. Physical-device touch
testing is waived under the existing scope; mouse-equivalent checks must be
reported as such.

## Screen briefs

### World and region map

| Question | Screen contract |
| --- | --- |
| Current decision | Choose where to expand, defend, invest or redirect a force. |
| Dominant focus | Known territory, meaningful places and active armies. |
| Primary action | Issue the selected object's relevant order with its cost and constraints visible. End Turn remains separate. |
| Supporting information | Calendar, actual resources/income/upkeep, owned force status, known threats, relevant supply and a compact attention list. |
| Deferred information | Detailed accounts, composition, tactics, biographies, household administration, historical filters and utilities. |
| Layout and camera | At 1920×1080, keep most of the canvas for a working neighborhood with connected geography beyond the screen. Overview explicitly fits known land. Keep one inspector, compact accounts and collapsed Attention; retain the camera across selections and views. |
| Input and feedback | Tap objects and visible controls; drag/pinch or use zoom/recenter. Selection, saved orders and important consequences survive the end of transient feedback. |

M01 territory display distinguishes political claim, physical occupation,
contested control and unknown geography. Claim fill and boundaries are clipped
to discovered geography; the world view additionally uses authored land/water
masks for the existing atlas artwork. Regional fill retains the existing local
coordinates. These masks have no role in movement or topology. Local rings show
physical control; occupation and contested badges preserve exceptions. A region
with undiscovered internal sites withholds its claim, rather than appearing
neutral or implying its hidden sites are secure. Kingdom labels and the owned
capital's crown establish orientation; foreign capital roles stay private.

Settlement silhouettes distinguish unsettled places, camps, outposts, hamlets,
villages, towns, cities and major cities; fort marks remain a separate layer.
Work, damage and Wood/Stone cues appear at useful local or zoomed scales. Label
placement prioritizes selection, capitals, danger and regions, then owned and
developed places as zoom increases. Collision checks reserve space for controls,
symbols, force marks and the inspector; marker count no longer hides all ordinary
names at once. Full facility and development detail stays on selection.

M03 will give each regional map its own terrain and approaches, with roads and
bridge/pass symbols placed meaningfully. The current regional background and
repeated topology remain unchanged. World Map and Enter Region preserve their
camera contexts; a seamless transition between scopes is deferred. M01A changes
their presentation extent and scale before M03 changes local geography.

### Implemented spatial scale and navigation

M01A separates the 1920×1080 UI from independent 5280×2970 world bounds and
3360×1890 regional bounds. Normal zoom 1 displays 2.75 world screen widths or
1.75 regional screen widths across their full extent. The values live in
validated `map_presentation.json`; normalized authored geometry, route costs,
fog and campaign saves keep their existing meaning.

New and loaded campaigns focus the physical home at normal campaign scale, opening its region when needed. Overview fits
known land and becomes Return View, which restores the previous working camera.
Recenter focuses the selected force or home at that normal scale; +/− stay
visible. Selection, dismissal, End Turn and management return preserve context.
Explicit attention navigation may focus the relevant observed place. World and
each region retain independent camera contexts for the current session; reload
uses the home default rather than introducing camera fields into saved campaigns.

At overview scale, emphasize territory, capitals, major names and known urgent
conditions. Campaign scale adds compact places, useful routes and concise army
markers. Local detail adds names and relevant place detail. Full army counts,
orders and costs appear with selection. Overview enters at zoom 0.68 and exits
at 0.78; detail enters at 1.45 and exits at 1.30. These separate thresholds
prevent repeated information changes near a boundary. Maximum zoom is 2.4.
Symbols remain compact while interactive targets retain a 48-pixel floor.
Crowded place and nearby-force groups focus without issuing movement. A regional
force group identifies armies across sites separately from a stack at one site.

Map Key opens the scale and symbol explanations in Help. The long permanent
legend is removed. First-use guidance, selection costs, known danger and End
Turn remain discoverable. Terrain separates places; map expansion preserves
travel rules, fog and observer secrecy. The
[scale contract](map-playability-plan.md#proposed-scale-and-information-contract)
owns M01A's intended outcome; [verification](verification/spatial-scale.md)
records measured results and remaining limits.

### Management and setup sheets

Army, place, records, save, setup and other management content is centered in a
1280×720 content region of the 1920×1080 canvas. Its existing readable controls
keep their pixel size. A full-canvas scrim makes the current sheet the focus.
The toolkit viewport camera and pointer/press origin receive the same translation,
so drawing and picking use identical coordinates. Sheet navigation preserves
the map camera. Costs, eligibility, confirmation, Back and text-entry controls
remain beside the current decision.

### M01 accounts and known attention

The lower strip shows current owned Gold/Wood/Stone balances and one dated actual
receipt: last completed season's income, upkeep paid/due and recovery Gold
spending. Any unpaid upkeep appears beside current balances, so a truncated
receipt cannot conceal the shortfall. A new campaign identifies the absence of
a completed receipt. No forecast or hidden rival treasury is displayed.

Attention starts collapsed beside the accounts strip and expands upward with at
most three rows per page and visible previous/next controls. It automatically
collapses during place or army selection. M01's original captures covered both
then-supported sizes; new work follows the sole 1920×1080 spec above. The lower
placement preserves capitals and other targets near the top of a dense overview;
inspectors and order cards end above the accounts. Tapping Attention while an
object is selected dismisses that selection and reopens the list. Site entries
use their most urgent known condition in
this order: participant siege, hostile contact, local threat, owned contested
control, owned occupation. Unsupplied owned forces follow. Selecting an entry
focuses its exact observed object and clears any previous movement selection;
it neither travels nor advances the simulation. Army entries then use the
existing movement card. Hidden sites and foreign armies are not focus targets.

World-region danger badges sum only already-known internal threats, participant
sieges and hostile-contact sites. A number on a danger badge counts conditions,
never enemy armies or strength. Current warnings are derived from current
observer facts, so attention has no separate saved acknowledgement or event
history. M02 owns seasonal outcome receipts and their acknowledgement behavior.

### Map orders and selection

Direct movement is implemented: tap Army, then a destination to execute the
affordable travel immediately and retain any remaining route. End Turn continues
saved orders after refresh. Cancel Route stops the plan; Review Route inspects
it. Peaceful borders and encounter rules remain authoritative.

The planned inspector keeps the object, cost, allowance, supply and expected
action together. Common recruitment, construction, focus and diplomacy entry
points stay in that context. Existing spending, battle and diplomatic review
rules remain where relevant. Movement does not acquire a second confirmation.

M01A replaces the large unselected army banners with compact force marks.
Status still prioritizes Siege, then Cut off, Route and Idle. The existing
movement selector preserves each force's physical location, including forces
represented by a regional group. Selection exposes exact counts, allowance and
composition. A queued order remains
recognizable after the card closes; persistent route paths/destinations and
interruption detail are M02 work. Unknown enemy strength and plans stay hidden.

### Seasonal outcomes and attention

Planned seasonal feedback prioritizes problems requiring a decision, then
completed orders and useful changes. Selecting an entry focuses the place,
army or person. Unresolved conditions remain discoverable; informational entries
can be dismissed. Preserve receipt/acknowledgement state across reload without
repeating actions or misleading the player about which season changed a value.

The existing M01 attention list provides current conditions. M02 adds the
seasonal outcome path and supporting history where needed; full biographies
remain deliberate inspection. Keep this compact route into the affected map
objects separate from a full management dashboard.

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
already exist. M01 exposes current habitation, known danger and selected local
cues; M02 brings common orders and seasonal consequences into the selected-place
inspector.

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

M01A centers the existing battlefield presentation at its original pixel size.
Inherited small tactic/playback controls and dense placement remain recorded
limitations; this migration does not claim a battle-interface redesign or touch
acceptance for those controls.

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

For each changed milestone, review the 1920×1080 canvas, early fog, developed
territory, long labels, dense stacks, expanded inspectors, multiple orders and
urgent conditions. Exercise selection, dismissal, movement, cancellation,
regional boundaries, seasonal feedback and save/reload through visible controls.
Check picking after zoom/resize and ensure drag release never issues an order.

Judge whether the player can locate their realm, choose a useful action and
explain its result. Readability and unclipped controls are necessary but do not
prove that loop. Store stable captures directly in `docs/verification/`, follow
the shared capture/publish workflow, and record remaining limitations in the
[evidence index](verification/README.md).
