# Map playability implementation plan

Updated 2026-10-01. **Status: M01 complete with recorded validation limits; M02 is next and unstarted.**

Kestrum needs a map that explains the kingdom, presents useful decisions, and
shows their consequences. This is the active implementation sequence following
the map review. It replaces the completed K01–K18 and B01–B07 delivery sequences
as the starting point for new work. Their implemented mechanics remain the
baseline, subject to the changes explicitly described here.

The user requested this plan and the documentation reconciliation. M01's
implementation decisions are recorded below and in the owning chapters;
[verification](verification/kingdom-overview.md) records its scoped acceptance
and inherited limits. M02–M05 are unstarted. Ordinary design tuning is resolved
during implementation and recorded in the owning chapter.

## Outcome

A player should be able to see what they own, identify a useful next action,
issue an order, and explain what changed after a season while staying oriented
on the map. Kingdom growth, military pressure and remembered people should be
visible through places and armies.

Use Stellaris as a reference for territory, useful map symbols, contextual
selection and attention management. Keep Kestrum's seasonal faction turns,
physical route graph, automatic battles, restricted enemy knowledge and
generational continuity. The existing simulation supplies most of the needed
facts; this work connects them to play and improves the geography.

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

- Keep 80 major markers and 4–8 factions. Initially reuse the 152 physical sites
  and their identities; varied connections and control objectives can produce
  distinct regions without increasing the content count.
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
- New diplomacy systems, races, classes, combat rules and a larger world are
  outside this plan. Existing balance defects remain visible in the status ledger.

## Target screen brief

| Question | Target behavior |
| --- | --- |
| Current decision | Choose a place to secure or improve, or a force to redirect, based on visible opportunity and pressure. |
| Dominant focus | The known kingdom, its frontier, useful places and active forces. |
| Primary action | Issue the selected object's relevant order; show its cost, access and consequence beside it. End Turn stays separately reachable. |
| Supporting information | Gold/Wood/Stone, actual seasonal income and upkeep, calendar, selected force condition, urgent known conditions and remaining orders. Label actual receipts separately from any forecast. |
| Deferred information | Full accounts, rosters, tactics, biographies, household administration, historical filters and utilities. |
| Layout and camera | At 1920×1080 and 1280×720, the map remains dominant. Collapse the attention list before reducing labels or touch targets. Fit the useful known realm and preserve camera orientation across selection. |
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
starting the next major change. M01 is complete; M02 is the next unfinished
milestone and has not started.

| Milestone | Result | Dependency | Status |
| --- | --- | --- | --- |
| M01 | Readable kingdom overview and truthful map information | Reconciled baseline `3073c18` | Complete; [evidence and limits](verification/kingdom-overview.md) |
| M02 | Orders, common actions and seasonal consequences stay connected to the map | M01 | Not started |
| M03 | Distinct regional geography with coherent terrain and compatible saves | M02 | Not started |
| M04 | Opening play teaches a complete strategic loop | M03 | Not started |
| M05 | Integrated early and developed campaign acceptance | M01–M04 | Not started |

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

### M02 Map orders and consequences

Keep the selected place or army visible beside a compact inspector. Bring common
recruitment, construction, focus and diplomatic entry points into that context;
reuse existing commands and their blocked reasons. Detailed composition,
tactics and histories retain dedicated views with a reliable return path.

Show persistent route/order status, arrivals, idle or blocked own forces and
supply problems. End Turn remains available during selection. Provide a compact
seasonal outcome list whose entries focus their affected places or people.
Prioritize action-required outcomes; informational entries can collapse or be
dismissed. Keep unresolved conditions until resolved. Store sufficient receipt
and acknowledgement state to survive save/reload without duplicate notices;
derive current warnings from authoritative state. Do not replay commands to
reconstruct feedback or turn every historical event into an alert.

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

Implement the compatibility boundary below before changing topology. Prototype
three contrasting regions, then complete all eight using the same validated
content model. Keep an authored reason for each route, resource, entrance and
control objective. At least two meaningful internal approaches must exist in
each region; a choke point can restrict a key objective within a branching map.

These are authoring briefs, adjustable while keeping their strategic distinction:

| Region | Strategic decision |
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
on the terrain they describe. Extend asset/schema validation where needed.
Keep settlement tiers and condition overlays driven by campaign state. Art must
support picking, visibility and map reading at the minimum size.

Likely boundaries: `world_layout.json`, world/layout schemas and validators,
`src/state/validation/world.rs`, generation, map navigation, atlas assets and
their registry, `src/ui/world/routes.rs`, and `tests/atlas_revision.rs`.

**Acceptance:** all eight regions offer distinct route/control choices and no
longer share one chain/layout. Three contrasting regions demonstrate why an
approach matters in actual play. Every 4–8 faction start retains a viable first
action, reachable resources and fair access; AI can enter, operate and leave.

**Five behavioral cases:** all regional graphs and anchors reachable; meaningful
alternative paths and geographically valid crossings; all supported starts and
NPC boundary travel; historical topology/order roundtrip; new topology movement,
supply, construction and siege integration. Use table-driven cases across regions.

#### Save and topology compatibility

Current campaigns embed their physical world and validate it against authored
content. Layout revisions 1–3 cover earlier geometry and terrain-tag differences. A new topology
cannot be handled by changing marker positions or loosening validation.

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

An agent walkthrough verifies interaction paths. Record a first-time human
playtest when available; until then, retain that limitation instead of calling
the onboarding proven. No blanket new physical-touch requirement is added to
the existing waiver. Visible touch-capable controls and supported browser sizing
remain implementation requirements.

## Validation and completion

Follow [delivery and validation](12-delivery-and-validation.md), the shared
engineering rules and `UI_STYLE.md`. Use the actual checkout, shared Cargo pool,
normal publish command and supported capture wrapper. Test affected behavior,
then perform broader integration for changed projections, saves or topology.
Keep useful regressions and strongly target five cases per major feature.

For each milestone update its status, source/docs changed, tests actually run,
normal/minimum captures, browser interactions, publish result, known blockers
and commit. Store screenshots directly in `docs/verification/` using stable
screen/state names. Replace equivalent captures and confirm capture games exit.

The last recorded production-victory test fails at its 240-round cap; minimum
WebGL scaling and several dense battlefield presentation issues are also recorded.
See the [evidence index](verification/README.md). Keep those failures visible,
rerun relevant checks when affected, and do not weaken tests or invent a clean
workspace to claim acceptance. Their historical existence does not halt unrelated
map implementation. M05 must report remaining failures explicitly.

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
