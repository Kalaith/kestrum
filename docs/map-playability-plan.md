# Map playability implementation plan

Updated 2026-10-04. **Status: M01, M01A and city-created regions are implemented
under their recorded scope and verification; M02–M05 remain unstarted.**

The 2026-10-03 request supersedes the eight fixed production regions assumed
below. New campaigns show all physical nodes on the country map, with no region
unlocked. Paid development makes a Village or Town a City; cities cannot be
direct neighbors. A functioning city opens its connected countryside as a local
view of the existing graph. The tutorial develops and enters the capital first.
The [world chapter](02-world-time-and-control.md#city-regions) owns this rule.
The [city-region evidence](verification/city-regions.md) records engine checks,
native tutorial completion and browser investment/region navigation at 1920×1080.
M03's earlier requirement to complete eight authored regional graphs is
superseded by local city geography. The remaining map-order and readability
work stays in scope.

### City development screen brief

| Question | Current decision |
| --- | --- |
| Decision | Which eligible settlement should receive the resources to become a city? |
| Dominant focus | Country geography during selection; the selected settlement during investment. |
| Primary action | Develop City, with gold/wood/stone cost and spacing or condition blockers beside it. |
| Supporting information | Existing habitation, ownership, supply and neighboring cities. |
| Deferred information | Regional detail appears only after city development; management remains in its sheet. |
| Layout and camera | The existing 1920×1080 canvas, independent country and local cameras, visible return control. |
| Input and feedback | Tap Manage, review and confirm development, then Enter Region; tutorial receipts follow successful actions. |

Kestrum needs a map that explains the kingdom, presents useful decisions, and
shows their consequences. This is the active implementation sequence following
the map review. It replaces the completed K01–K18 and B01–B07 delivery sequences
as the starting point for new work. Their implemented mechanics remain the
baseline, subject to the changes explicitly described here.

The user requested this plan and the documentation reconciliation. M01's
implementation decisions are recorded below and in the owning chapters;
[verification](verification/kingdom-overview.md) records its scoped acceptance
and inherited limits. The subsequent request for a more spacious world adds
M01A before M02. M01A implementation and its scoped validation are recorded in [spatial scale verification](verification/spatial-scale.md). M02–M05 remain unstarted.
Ordinary design tuning is resolved during implementation and recorded in the
owning chapter.

## Outcome

A player should be able to see what they own, identify a useful next action,
issue an order, and explain what changed after a season while staying oriented
on the map. Kingdom growth, military pressure and remembered people should be
visible through places and armies. Ordinary play should show one useful part
of a world that continues beyond the viewport. A deliberate overview provides
orientation across the known kingdom.

Use Stellaris as a reference for territory, useful map symbols, contextual
selection and attention management. Keep Kestrum's seasonal faction turns,
physical route graph, automatic battles, restricted enemy knowledge and
generational continuity. The existing simulation supplies most of the needed
facts; this work connects them to play and improves the geography.

## Spatial scale review

The follow-up review inspected source at `ce22c6a` and the then-existing developed
world and minimum-size regional captures. It did not run a new gameplay session.
These findings describe the pre-M01A implementation that motivated the work:

| Behavior at the review | Effect on perceived scale |
| --- | --- |
| `navigation.rs` uses the same 1280×720 extent for the UI and normalized map positions; zoom is limited to 1–3. | At minimum zoom, the entire atlas occupies one screen and cannot pan. |
| Developed-save loading calls `frame_discovered`; focus uses fixed world/regional zoom values. | Loading compresses the discovered realm into view, while recentering can abruptly magnify a small neighborhood. |
| Settlement disks stay 48 logical pixels across; army banners stay 196×56. | Large symbols and cards dominate when many places are visible; zooming out alone packs them closer together. |
| `game.rs` renders the 1280×720 logical UI through the virtual viewport. | At 1920×1080 it scales by 1.5: a settlement disk becomes 72 pixels and an army banner 294×84. |
| Every regional view shows the same ten-site, two-row chain against the continental atlas. | A region reads as a complete diagram rather than a landscape with approaches and distance. |

Sources: [camera and picking](../src/navigation.rs),
[focus and banners](../src/navigation/armies.rs),
[load behavior](../src/game/saves.rs), [atlas](../src/ui/atlas.rs),
[symbols](../src/ui/world/symbols.rs), [virtual canvas](../src/game.rs), and
[regional layout](../assets/data/world_layout.json).
Existing images: [developed world](verification/ui_midgame_map.png) and
[regional minimum](verification/ui_overview_region_minimum.png).

### Resolution contract

The user's correction on 2026-10-01 sets **1920×1080 as the sole design and
acceptance resolution**, including the logical UI canvas. It replaces the
previous 1920×1080/1280×720 target pair. The former 1280×720 logical layout must
be migrated and recomposed. Increasing the window size while scaling the old
layout by 1.5 would preserve the oversized controls and cramped composition.

Design and verify native and published browser gameplay at an actual usable
1920×1080 canvas. Window borders, browser chrome and embedded host padding do not
count toward that area. Other host sizes may uniformly scale or letterbox this
same composition through toolkit viewport conversion; they do not introduce
another layout or acceptance spec. Keep historical captures and their recorded
limitations as evidence of earlier builds.

### Proposed scale and information contract

Separate map coordinates from the 1920×1080 logical UI viewport. Keep readable
text and controls while giving the existing geography a larger navigable extent.
Start tuning with the full world about 2.5–3 viewport widths across at ordinary play
scale, preserving the authored aspect ratio. For regional play, start around
1.5–2 viewport widths. These are initial tuning targets, not measured acceptance
results. Judge the visible neighborhood and useful terrain gaps at 1920×1080.

Use three presentation bands within the existing world and regional scopes:

| Band | What the player sees and does |
| --- | --- |
| Kingdom overview | Known territory, major names, capital, frontier and urgent activity. Minor places become quiet marks and nearby forces aggregate. Tap a crowded group to focus it. |
| Campaign view, the default | A home area or active frontier with nearby objectives and connecting terrain. Places use compact symbols; armies have concise identity/status marks. Select an exact place or force to act. |
| Local detail | More place names, relevant facilities, precise route costs and individual forces where the scope permits. Full troop/order details belong to the selected force. Enter Region still opens the internal graph explicitly. |

Zoom changes the amount of information as well as geographic spacing. Use smooth
transitions and separate entry/exit thresholds to prevent labels flickering near
a boundary. Preserve capital, selection and actionable known danger at every
useful scale. Aggregated forces must still distinguish a regional grouping from
armies sharing one physical site.

Provide a visible Overview action that fits known land and can restore the prior
working view. Keep +/− and Recenter; Recenter focuses the selected force or home
at a useful working scale. Initial play and older-save loading focus home and
nearby routes. Selection, dismissal, End Turn and returning from another screen
preserve the player's view unless they explicitly request a different focus.
Retain independent world/region contexts. Camera persistence across application
reload is optional presentation work, with additive defaults if introduced.

The landscape should occupy the gaps: valleys, woodland, rivers, coasts and
approaches give places separation. M01A reuses the current graph and normalized
geometry. M03 authors distinctive regional terrain and routes. Keep geographic
distance separate from travel cost; the authoritative route rules still determine
how far an army can move. A larger drawing must not silently lengthen journeys.

## Historical starting point

The review inspected source at `b6978a3`, the supplied midgame image, existing
captures, and the published browser's early campaign map, settlement and army
interfaces. The following findings describe that pre-M01 review, not the current
implementation. It did not establish full campaign balance or physical-touch usability.

| Baseline behavior | Finding at the initial review |
| --- | --- |
| Political claims, physical control and contested regions | Ownership appeared mainly as faction initials and colored rings; territory had little visible shape. |
| Population, income, facilities, damage, growth and construction | Places looked similar and their purpose was mostly hidden in management sheets. |
| Direct destination taps, immediate partial travel and saved routes | Army banners showed a generic label/count; order state disappeared from the ordinary overview. |
| Observed threats and hostile presence | Region markers did not aggregate their internal visible danger badges. |
| Eight regions with ten sites each | Every internal network is the same nine-edge chain with the same two-row placement and anchor pattern. |
| World and regional map scopes | Both reuse the continental atlas; local terrain and routes do not explain local geography. |
| Seasonal economy and many durable consequences | Feedback lacks a concise map-linked account of the changes that matter next. |
| Action-driven tutorial | Careers and households precede the first End Turn; it teaches screens before a useful strategic payoff. |

Source entry points: [world rendering](../src/ui/world.rs),
[route rendering](../src/ui/world/routes.rs), [map HUD](../src/ui/atlas.rs),
[visible campaign](../src/engine/projection.rs),
[production geography](../assets/data/world_layout.json),
[tutorial](../src/state/tutorial.rs), and [round effects](../src/engine/round.rs).

## Scope and implementation decisions

- Keep 4–8 factions and the 152 physical site identities. The city-region change
  exposes one country marker per site and replaces the earlier 80-marker limit.
  M01A expands their presentation space; varied connections and control
  objectives follow in M03.
- Keep the world/region distinction for this delivery. Give regions coherent
  local terrain and a visible breadcrumb. A seamless zoom transition can be
  considered later if these views still break orientation.
- Build the normal map around the world, one contextual inspector, and quiet
  supporting controls. Add a compact attention list and resource strip with
  clear priorities; avoid permanent panels for every simulation system.
- Preserve direct movement: select an army, then tap a destination. Show costs
  and warnings before or beside the target; ordinary movement must not regain
  a mandatory confirmation screen. Battle preparation and consequential
  spending/diplomacy confirmations retain their existing rules.
- Keep six formation slots and one named member per formation. Show relevant
  commander identity on the map; keep composition and biographies on demand.
- Expose known facts only. An observed enemy marker does not authorize live
  enemy strength, plans, treasury, unseen territory, or hidden tactical details.
- Keep balance values and authored layout in validated JSON. Evaluate shared
  toolkit support before adding generic layout, picking or drawing utilities.
- New diplomacy systems, races, classes, combat rules and additional world
  locations are outside this plan. More navigable presentation space is in scope.
  Existing balance defects remain visible in the status ledger.

## Target screen brief

| Question | Target behavior |
| --- | --- |
| Current decision | Choose a place to secure or improve, or a force to redirect, based on visible opportunity and pressure. |
| Dominant focus | The known kingdom, its frontier, useful places and active forces. |
| Primary action | Issue the selected object's relevant order; show its cost, access and consequence beside it. End Turn stays separately reachable. |
| Supporting information | Gold/Wood/Stone, actual seasonal income and upkeep, calendar, selected force condition, urgent known conditions and remaining orders. Label actual receipts separately from any forecast. |
| Deferred information | Full accounts, rosters, tactics, biographies, household administration, historical filters and utilities. |
| Layout and camera | At the sole 1920×1080 spec, a useful neighborhood fills the dominant map area and geography continues beyond it. Overview explicitly fits known land. Collapse secondary information before reducing readable text or touch targets; preserve orientation across selection. |
| Input and feedback | Tap selects and reveals actions; close restores the view; drag/pinch and visible zoom/recenter controls work. Orders and results remain readable after transient effects end. |

### Visual language

Use subtle territorial fill and boundaries for political claims. Show occupied
sites and contested land separately; a filled region must never imply that every
internal site is secure. Clip political display to known geography and meaningful
land boundaries. Use a distinct capital symbol and readable kingdom labels.

Give settlement tiers recognizable silhouettes, with small symbols for relevant
resources or facilities. Show growth, construction, damage and siege state only
where they change a decision. Keep labels dependent on zoom and importance,
rather than abruptly hiding all ordinary names after a marker-count threshold.

Army banners show a short identity, owned strength and a readable order/status
symbol. Selection reveals exact composition, allowance, supply and route details.
Display relevant queued routes, destination and blockage after selection closes;
provide a focused order view when many routes overlap.

The default view communicates politics and urgent activity. A visible control
opens contextual supply or development information; these overlays need legends
and tap equivalents. Color is always paired with shape, symbol or text.

## Delivery sequence

Each milestone is independently useful. Finish its validation and commit before
starting the next major change. M01 and M01A are complete under their recorded
scope. M02 is the next unfinished milestone, adding map actions and route detail
to the implemented camera and information hierarchy.

| Milestone | Result | Dependency | Status |
| --- | --- | --- | --- |
| M01 | Readable kingdom overview and truthful map information | Reconciled baseline `3073c18` | Complete; [evidence and limits](verification/kingdom-overview.md) |
| M01A | Spacious navigable geography, useful camera defaults and detail appropriate to scale | M01 | Complete; [evidence and limits](verification/spatial-scale.md) |
| M02 | Orders, common actions and seasonal consequences stay connected to the map | M01A | Not started |
| M03 | Distinct regional geography with coherent terrain and compatible saves | M02 | Not started |
| M04 | Opening play teaches a complete strategic loop | M03 | Not started |
| M05 | Integrated early and developed campaign acceptance | M01, M01A, M02–M04 | Not started |

The [hero portrait generator](hero-portrait-generator-plan.md#remaining-delivery)
adds G01-G04 as a planned companion graphics workstream after M01A. It starts with
stable saved appearance and People/Career portraits, then expands authored parts
and eligible identity contexts. M02 remains next; portraits neither replace these
milestones nor become a new M05 prerequisite. Review portrait recognition in M05
if that work has shipped. Detailed asset, compatibility and acceptance TODOs live
only in the portrait plan.

### M01 Readable kingdom overview

**Complete, 2026-10-01.** Native normal/minimum review, observer/movement checks,
strict Clippy, formatting, source-size checks, publishing and published browser
gameplay/reload checks are recorded in [M01 verification](verification/kingdom-overview.md).
The unchanged seed-88 victory failure, minimum WebGL rendering limitation and
physical-touch waiver remain explicit; no broader platform/balance acceptance
is implied.

Extend the observer projection with the public and owned facts needed by the
map. Build political boundaries, capital/settlement symbols, zoom-aware labels,
owned army banners, resources and a small attention list. Selecting a warning
focuses the affected place or army. Aggregate observed threats, sieges and
hostile contacts from regional sites onto their world marker without revealing
unobserved occupants.

Implementation boundaries: observer projection and `src/engine/overview.rs`,
`src/game/projection.rs`, `src/ui/overview.rs`, cohesive layers under
`src/ui/world/`, selection and navigation helpers, and validated
`map_presentation.json`. Political rendering reads control; it cannot change it.

**Acceptance:** a developed realm can be read without opening a management
sheet. The player can locate their capital, a frontier, their forces, an urgent
known condition and their economic position. Neutral, occupied, contested and
unknown territory remain distinct.

**Five behavioral cases:** observer secrecy; mixed regional control; aggregate
known danger; development/ownership changes update summaries; picking and label
selection remain correct across zoom and supported sizes. Visual review covers
early fog, the developed realm, long names, eight factions and dense contacts.

#### Implementation decisions

- Observer summaries contain discovered sites/claims, owned capital and army
  supply, owned troop totals/orders, and existing visible danger facts. A region
  with undiscovered internal sites withholds its political claim; known neutral
  land remains distinct. No enemy strength, orders or undiscovered control enters
  a summary.
- Political fill and boundaries are clipped to discovered geography; the world
  view also uses authored atlas land/water masks. Regional fill retains local
  coordinates. These are presentation data, with no new routes, terrain
  rules, control rules or topology revision. Local control rings, occupation and
  contested marks remain independent of the political fill.
- Settlement silhouettes and separate fort marks replace generic site initials.
  The owned capital is crowned. Work, damage and Wood/Stone cues appear at local
  or zoomed scales. Labels use zoom, strategic importance and collision placement
  rather than a total-marker cutoff.
- A single owned banner shows its name; a stack shows its army count. Both show
  combined surviving troops and the highest priority state: Siege, Cut off,
  Route or Idle. Exact composition and path details remain on selection. M02
  retains responsibility for persistent destinations/routes and richer inspectors.
- The lower accounts strip separates current Gold/Wood/Stone from the last actual
  completed season's income, upkeep paid/due, recovery spending and shortfall.
  Before the first receipt it says no season has completed. No forecast is added.
- Attention starts collapsed beside the lower accounts, expands upward with
  three rows per page and visible navigation, and collapses during selection.
  This placement preserves upper-map capitals; inspectors end above the strip.
  It focuses exact observed sites or owned
  armies without issuing travel. Region warnings aggregate already-known internal
  threats, participant sieges and hostile-contact sites; counts describe known
  conditions rather than enemy forces.

The capture harness uses `production_world` for early fog and includes
`overview_urgent`, `overview_region`, `overview_attention`, `overview_siege`
and `overview_deficit`, each with a `_minimum` variant. The deficit scene shows
an actual unpaid-upkeep receipt after ordinary seasonal resolution. The
eight-faction stress scene authors supported conditions
and validates its campaign; its economy receipt comes from ordinary full-round
commands. Existing `midgame_map`/`midgame_frontier` supply the separately earned
developed campaign. The nine refreshed states at both sizes and the published
browser checks are recorded in [M01 verification](verification/kingdom-overview.md),
including the minimum browser limitation.

### M01A Spatial scale and map navigation

**Complete, 2026-10-02, under the recorded headless review scope.**

Implementation uses a 1920×1080 logical canvas, a 5280×2970 world and
3360×1890 regional extent at working zoom 1.0. The JSON presentation settings
validate aspect ratio, extents and separated band thresholds. Overview enters
at 0.68 and exits at 0.78; Detail enters at 1.45 and exits at 1.30. The maximum
zoom is 2.4. Authored normalized geometry and physical route rules are unchanged.

Overview/Return View remembers the previous camera separately for each scope.
New and loaded campaigns focus the physical home, opening its region when needed. Selection, management,
title return and seasonal actions retain context; Recenter and Attention focus
explicitly. Compact places retain 48-pixel targets. Crowded places and nearby
forces provide shared visible focus targets; regional force counts distinguish
regional aggregation from co-located armies. Exact force details stay selected.

The HUD, title, inspector and accounts are composed directly at 1920×1080.
Management content keeps its pixel sizes in centered sheets with matching
pointer translation. Map Key holds symbol meanings and navigation guidance.
Historical smaller-size captures and inherited battlefield limitations remain
recorded. See [verification](verification/spatial-scale.md) for measured results.

Formatting, strict Clippy, source-size checks and the final 14-scene native
capture batch passed. All captures measure 1920×1080. The full suite retains
one inherited production-victory failure and one intentionally ignored profiling
test. Automated native actions, background browser gameplay and publishing/reload
passed, including a complete 1920×1080 browser capture. Hardware touch and native
operating-system rapid-click timing remain unverified; no broad human playtest
is claimed.

The following sequence defines the milestone's acceptance scope:

Deliver the scale contract above before adding more permanent map information.
Implement in this order within the milestone:

1. **Establish the 1920×1080 canvas.** Update native defaults, logical rendering,
   UI bounds, text, overlays, pointer conversion and capture fixtures together.
   Recompose the HUD at this resolution, with readable type and controls sized
   for the new canvas. Audit management, tutorial and battle screens that share
   the same viewport so they retain access and correct picking. Keep historical
   720p evidence; new acceptance uses the sole 1080p spec.
2. **Separate map extent and viewport.** Reuse toolkit `CameraTransform`, bounds,
   viewport and gesture helpers. Add validated presentation settings for world
   and regional extents and scale bands. Keep normalized authored positions and
   graph identities. Update background, routes, masks, territorial fill, fog,
   labels, banners, selection and inverse pointer projection together. Bounds
   must work when a whole map fits and when only part is visible. Cull and clip
   routes correctly even when both endpoints lie outside the viewport.
3. **Establish navigation defaults.** Replace mandatory discovered-world framing
   on load with useful home framing; keep fit-to-known as the explicit Overview
   action. Preserve working cameras across selection, turns, management screens
   and region return. Focus into the unobscured map area beside an inspector.
   Keep zoom anchored to the pointer or touch midpoint and provide concise first-use
   instructions for pan, zoom, Overview, Recenter and region entry/return.
4. **Reduce symbol and card dominance.** Tune ordinary unselected glyphs initially
   around 20–28 pixels on the 1920×1080 canvas, enlarging selection and critical
   cues as needed.
   Replace persistent full army cards with concise markers; selection exposes
   exact troop counts, orders and costs. Retain at least 48-pixel interaction
   targets and controls. When those targets overlap at overview scale, offer a
   group focus or visible chooser; a tap on a group never issues movement.
   Drawing and picking must use the same displayed grouping. Precise destinations
   retain direct movement without a new confirmation step.
5. **Recompose supporting UI.** Keep one inspector, compact resources, Attention
   and End Turn. Move the permanent long legend and repeated instruction line
   into visible Map Key/help disclosure. Keep the current selection's costs and
   urgent state beside its action. Use the existing Attention path for offscreen
   problems; add only a selected-target/destination edge cue if needed. Every cue
   must use observed or owned information. Avoid a permanent minimap unless
   navigation review shows Overview and Recenter leave a specific gap.
6. **Validate the actual play loop.** Pan to a connected area beyond the initial
   viewport, select a force, issue a precise order, inspect a known problem,
   zoom to overview and return to the working view. Run these at 1920×1080 with
   early fog and the developed campaign before completing the milestone.

Likely boundaries: `src/main.rs`, shared UI composition and capture setup,
`src/navigation.rs`, `src/navigation/armies.rs`,
`src/navigation/exploration.rs`, `src/game/world.rs`, `src/game/saves.rs`,
`src/ui/atlas.rs`, the layers under `src/ui/world/`, labels and movement picking,
plus `map_presentation.json` and its schema/validation. Split cohesive modules
before reaching the 800-line limit. Consider shared toolkit additions for any
missing general camera/input capability before adding a local replacement.

The existing atlas can establish the first camera pass. Check texture quality
at the proposed scale and reuse layered terrain rendering where useful; reauthor
art, masks and paths together if needed. Preserve aspect ratio and alignment.
Scale the existing fog geometry consistently so zoom or a change in presentation
extent cannot reveal new places or change discovered state. Keep drawing work
bounded to visible detail and reuse existing profiling when investigating a
regression; a larger map should not imply a larger full-resolution intermediate
render target or new per-frame scans of campaign history.

**Acceptance:** normal play presents a readable working area with connected
geography beyond the screen. Overview gives a quieter understanding of the
known kingdom and returns reliably to the previous view. Terrain visibly
separates compact places; full army cards and labels do not cover every gap.
At 1920×1080, the player can select precise destinations, recover an
offscreen force and operate the inspector without shrinking required controls.
Moving the camera never changes travel, discovery, supply or control.

**Five behavioral cases:** projection/picking at 1920×1080 across scale bands, edges
and scopes; gesture suppression and crowded-group selection without accidental
orders; hidden-information invariance and unchanged discovery; view retention,
load fallback and direct/queued movement through navigation; old-save graph and
deterministic outcome preservation. Extend useful existing navigation,
exploration, overview and movement tests rather than duplicate them.

Review early home, developed frontier, explicit overview, regional detail and
dense selected forces at actual 1920×1080 native and published browser canvases.
Replace equivalent captures at their stable paths; add stable state names only
for genuinely new scenes. Check actual drag, zoom, overview return, recenter,
selection, movement and resize behavior. Record texture clarity, label stability,
target access and whether land feels continuous beyond the viewport. Preserve
the physical-touch waiver and distinguish click checks from hardware testing.
Run formatting, strict Clippy, source-size and relevant tests through the required
workflow, then no-argument `publish.ps1`. Investigate any host scaling failure
that affects the supported 1920×1080 canvas; historical 720p failures remain
recorded without creating a second resolution requirement.

### M02 Map orders and consequences

Keep the selected place or army visible beside a compact inspector. Bring common
recruitment, construction, focus and diplomatic entry points into that context;
reuse existing commands and their blocked reasons. Detailed composition,
tactics and histories retain dedicated views with a reliable return path.

Show persistent route/order status, arrivals, idle or blocked own forces and
supply problems using M01A's scale bands. Emphasize the selected route; aggregate
other orders at overview scale so the expanded map does not become covered in
lines and cards. End Turn remains available during selection.

The [notification plan](notification-plan.md) specifies M02's seasonal feedback:
a compact top event rail, clickable details within one small map inspector,
hero emergence and controlled-place changes, conditional advance warnings and
per-type delivery controls. M02-N1–N3 are its planned delivery packages; they do
not mark other M02 work complete. Attention remains the current-condition view.
Preserve receipt and acknowledgement state in saves, derive current warnings
from authoritative observer-safe state, and avoid duplicate background toasts.
Do not replay commands to reconstruct feedback or turn every historical event
into an alert. Notification windows never cover the whole game view.

Likely boundaries: `src/ui/movement/`, `src/game/movement.rs`, army/settlement
inspectors, `src/game/campaign.rs`, `src/engine/notices.rs`, round outcomes and
additive persistence compatibility. Existing detailed systems need no rewrite.

**Acceptance:** the player can issue a route, dismiss selection, see its purpose
and progress, react to a blockage and inspect a seasonal consequence in place.
Common actions show their cost and update the affected map object immediately.

**Five behavioral cases:** direct partial movement and queued continuation;
cancel/replace/block orders; save/reload without repeated commands or alerts;
contextual spending rejected atomically when circumstances change; ordered
seasonal receipts and persistent urgent conditions. Include app-action guards,
not only direct engine tests. Review multiple armies and simultaneous outcomes.

### M03 Distinct regions and local terrain

Implement the compatibility boundary below before changing physical topology.
Prototype city regions in three contrasting landscapes, then extend their
geographic variety across the country using the same validated content model.
Regions remain unlocked by city investment at eligible nodes; do not restore
eight fixed region bundles. Keep an authored reason for each route, resource,
crossing and control objective. Branching approaches should offer meaningful
choices around major objectives, with choke points that reward local planning.

These are authoring briefs, adjustable while keeping their strategic distinction:

| Geographic area | Strategic decision |
| --- | --- |
| Northwatch | Defend a wooded approach or take a longer route to bypass its fort. |
| Alder Vale | Protect a productive basin while choosing between exposed fast roads and a safer outer route. |
| Westmere | Hold a short bridge crossing or invest movement in a secondary river approach. |
| Riverfold | Choose a valley route or higher ground, with a genuine crossing at its named bridge. |
| Lakeward | Defend a coastal road and connected inland holdings with limited reinforcement approaches. |
| Rosevale | Commit to a marsh causeway or take a longer supply-safe route. |
| Eastfold | Control a mountain pass while guarding a second approach that prevents a single linear campaign. |
| Southplain | Spread forces across open routes or concentrate around its settlement and supply hub. |

Give each regional scope terrain that agrees with its site geography and routes.
Author bends and crossing geometry for internal roads; place bridge/pass symbols
on the terrain they describe. Carry M01A's spacious working scale into these
layouts: cluster related settlements and leave coherent terrain between them,
with entrances and approaches that can continue beyond the initial viewport.
Show all sites together only through deliberate overview framing. Extend
asset/schema validation where needed.
Keep settlement tiers and condition overlays driven by campaign state. Art must
support picking, visibility and map reading at 1920×1080.

Likely boundaries: `world_layout.json`, world/layout schemas and validators,
`src/state/validation/world.rs`, generation, map navigation, atlas assets and
their registry, `src/ui/world/routes.rs`, and `tests/atlas_revision.rs`.

**Acceptance:** the country offers distinct route/control choices beyond its
repeated authored chains. City regions in three contrasting landscapes
demonstrate why an approach matters in actual play. Every 4–8 faction start
retains a viable first action, reachable resources and fair access; AI can
enter, operate and leave.

**Five behavioral cases:** all regional graphs and anchors reachable; meaningful
alternative paths and geographically valid crossings; all supported starts and
NPC boundary travel; historical topology/order roundtrip; new topology movement,
supply, construction and siege integration. Use table-driven cases across regions.

#### Save and topology compatibility

Current campaigns embed their physical world and validate it against authored
content. Layout revisions 1–3 cover earlier geometry and terrain-tag differences;
revision 4 derives country markers for each physical site without changing route
endpoints or costs. A new graph cannot be handled by changing marker positions
or loosening validation.

- Introduce an explicit topology revision separate from presentation geometry
  and the existing strictly validated rules `content_version`. Changing the
  latter also requires an explicit compatibility path. Preserve the recognized
  old authored graph as durable compatibility
  content, with strict validation for each supported revision.
- New campaigns use the revised graph. Existing campaigns retain their saved
  sites, routes, endpoints, costs, entrances, anchors and mechanical geography.
  Do not transplant new connections into a live war or silently reroute orders.
- Keep site/marker identities stable. Give genuinely new connections new route
  identities; never reuse an old route identity with a different meaning in the
  same supported topology.
- Render old campaigns with suitable presentation for their saved topology.
  Map readability and inspector improvements should still work in those saves.
- Test historical saves with queued movement, road work, sieges, partial control,
  knowledge and pending battles. Unknown/mixed revisions remain errors with a
  recoverable load path. Capture genuine existing state as durable test evidence
  only where needed; do not fabricate history or modify the user's save catalogue.

The topology work depends on passing these checks. It does not depend on clearing
unrelated deferred platform or long-campaign balance work.

### M04 Teach the opening through play

Replace the introductory screen tour with a contextual sequence: recognize the
home settlement and force; choose a useful nearby opportunity; move or recruit;
resolve a threat or secure a holding; advance the season; see the economic/local
result; invest in a road, force or settlement; encounter a rival frontier.

Use actual start state, legal commands and known opportunities. Guidance should
adapt if a player takes a different valid route. Show the reason for a suggested
action and a visible way to dismiss/resume help. Careers, mentorship and household
prompts follow relevant service or lifecycle events. Existing campaigns keep
their guide state without being forced through a new introduction. The current
guide serializes completed enum values: preserve their decoding or explicitly
migrate versioned progress, retain dismissal, and do not invent completion of
new lessons from an unrelated old step.

**Acceptance:** a new player completes a useful action and sees its consequence
within the first few turns, then faces a choice about expansion or defense. The
10–15 minute target is a playtest target, not an established measurement or forced
timer. No tutorial step requires opening an irrelevant system to advance.

**Five behavioral cases:** alternate legal opening choices; blocked action with
recovery; genuine consequence advances guidance once; dismiss/resume and old-save
compatibility; emerging career/household prompt respects relevant state. Verify
browser input through visible controls and include at least two starting regions.

### M05 Integrated playability acceptance

Review the opening and an established realm through the published build. Include
the existing developed campaign for old-save coverage and a new campaign on the
revised geography. A headless pass or attractive still image is insufficient.

Record observed answers to these questions without directing the player to a menu:

1. What land and forces do you control, and where is your capital?
2. Which known problem or opportunity would you address next, and why?
3. What does the selected order cost, where will it go, and what might stop it?
4. What changed after End Turn, and where did it happen?
5. Which place or person matters because of something that happened in play?
6. Where does the connected world continue beyond this view, and can you visit
   that area and return home without losing your bearings?

An agent walkthrough verifies interaction paths. Record a first-time human
playtest when available; until then, retain that limitation instead of calling
the onboarding proven. No blanket new physical-touch requirement is added to
the existing waiver. Visible touch-capable controls and supported browser sizing
remain implementation requirements.

### Campaign review cases

The [early Stellaris review](stellaris-release-lessons.md) motivates these five
M05 scenarios. They extend the questions above, not the feature scope or the
deferred balance/platform work. Record observed failures and follow-up ownership
without assuming the comparison establishes a Kestrum bug.

1. **Established realm:** observe four consecutive seasons of a developed
   campaign, including consolidation without new conquest. Ask for a worthwhile
   next objective, an alternative use of resources and the consequence of a
   choice. Record why the player waits when they choose to wait. Four seasons
   are a bounded review sample, not proof of long-campaign balance.
2. **Preparation and explanation:** before an encounter, record a formation's
   intended role; afterward, ask which witnessed action or condition explains
   the result and what the player would change. Follow losses, retreat and
   control back to the map. Keep unobserved enemy details private and report
   inherited battlefield limitations separately from map acceptance.
3. **Interrupted commitment:** encounter changed access or an invalidated order,
   inspect its reason, take a legal recovery action and reload. Confirm a clear
   remaining order or resolved state without duplicate costs or outcomes.
4. **Dismissed information:** dismiss a consequential seasonal receipt, then
   retrieve its retained facts and inspect the subject after reload. Include
   simultaneous outcomes and a changed, absent or forgotten subject; current
   conditions must stay understandable within the documented retention limits.
5. **Continuity and planning:** use a campaign with service, retirement or
   succession evidence. Ask which existing person/place now matters, why, and
   which legal next step advances the player's goal. Distinguish known career
   requirements from uncertain opportunities. A fixture tests access to this
   state; only played progression establishes its pacing and emotional payoff.

Use the sole 1920×1080 canvas and existing visible-control requirements. Record
campaign seed/save provenance, elapsed rounds, actions, the player's explanations
and the limits of the sample. Separate human observation from an agent walkthrough
and an engine-only simulation. This documentation review executes none of these
scenarios or changes a milestone to complete.

## Validation and completion

Follow [delivery and validation](12-delivery-and-validation.md), the shared
engineering rules and `UI_STYLE.md`. Use the actual checkout, shared Cargo pool,
normal publish command and supported capture wrapper. Test affected behavior,
then perform broader integration for changed projections, saves or topology.
Keep useful regressions and strongly target five cases per major feature.

For each milestone update its status, source/docs changed, tests actually run,
1920×1080 captures, browser interactions, publish result, known blockers
and commit. Store screenshots directly in `docs/verification/` using stable
screen/state names. Replace equivalent captures and confirm capture games exit.

The last recorded production-victory test fails at its 240-round cap; historical
720p WebGL scaling and several dense battlefield presentation issues are also recorded.
See the [evidence index](verification/README.md). Keep those failures visible,
rerun relevant checks when affected, and do not weaken tests or invent a clean
workspace to claim acceptance. Their historical existence does not halt unrelated
map implementation. M05 reports remaining failures and distinguishes historical
out-of-spec results from failures at the current 1920×1080 spec.

## Documentation handoff

Start with this plan and the current [README](../README.md). Read the relevant
system chapter and source for the milestone being implemented. The
[decision register](13-decisions-and-open-questions.md) holds fixed constraints;
the [interface chapter](10-interface-and-accessibility.md) owns the screen design.
Completed packets and dated verification are reference material, not a new queue.

During delivery, update those owning documents in the same commit as behavior.
Remove superseded instructions from active sections; link historical evidence
instead of copying old pass counts or unresolved-question lists. No new umbrella
plan, duplicated milestone ledger or approval checkpoint is required to begin.
