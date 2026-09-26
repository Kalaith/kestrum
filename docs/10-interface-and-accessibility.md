# 10 — Interface and accessibility

[Documentation index](README.md) · [Shared UI authority](../UI_STYLE.md) · [Validation](12-delivery-and-validation.md)

## Status and goals

This entire chapter is a **proposed screen plan**, applying the shared UI rules to the source game design. It is not evidence of implemented Kestrum screens. The supplied template's panels, technical labels, action demo, grid, and captures must be reconsidered when implementation begins.

The strategic map and current decision dominate ordinary play. Give no more than two or three areas strong visual emphasis. A selected object's short inspector can support the map; quiet navigation leads to kingdom, roster, history, and settings views as needed.

## Viewport targets

Proposed review sizes, expressed as actual usable canvas dimensions:

- Normal desktop: 1280 × 720.
- Minimum landscape: 800 × 450.
- Minimum portrait touch: 360 × 640.

These are design targets to validate, not claims of existing support. Review native and browser rendering, including embedded canvas sizes. At smaller sizes, use a bottom selection sheet or a dedicated full-screen detail view. Preserve a useful map region and readable action controls before reducing typography. Long collections scroll within deliberate bounds.

Proposed minimum tap target is 44 × 44 logical pixels after effective scaling; the actual device review determines whether more space is needed. Long names and dense army stacks must remain selectable without relying on precision clicking.

## Screen briefs

### New campaign

- **Current decision:** name and identify the new kingdom and choose the campaign's faction count.
- **Dominant focus:** compact setup form and emblem preview.
- **Primary action:** Start Campaign, beside any invalid setup explanation.
- **Supporting information:** single-player scope and clearly labelled rival/total count once decided.
- **Deferred information:** future culture, origin, and difficulty details until supported.
- **Layout and camera:** form fills the useful area; no map camera needed.
- **Input and feedback:** visible text input, emblem controls, and count adjustment; validate and acknowledge campaign creation.

### World and region map

- **Current decision:** where to commit a force or develop a place this turn.
- **Dominant focus:** connected strategic map, readable army markers, and selected route or front.
- **Primary action:** context action such as Move or Establish Outpost; End Turn remains visible and distinct from object orders.
- **Supporting information:** selected force condition, known supply, movement allowance, current season, relevant cost, urgent threats.
- **Deferred information:** full kingdom accounts, genealogy, biography, detailed battle math, and save management.
- **Layout and camera:** map gets most space; desktop selection inspector can sit beside it, while compact layouts use a dismissible sheet. Enter Region and World Map preserve orientation.
- **Input and feedback:** tap army, tap destination, inspect preview, tap Move; drag map or use visible pan controls; visible Zoom In, Zoom Out, Reset View, and Close controls. Selection and resulting movement remain clear after animation ends.

### Army composition

- **Current decision:** which formations and people should serve together or split into another army.
- **Dominant focus:** six formation slots with current/capacity headcounts and attached people.
- **Primary action:** confirm the chosen transfer or composition change, with requirements and consequences beside it.
- **Supporting information:** veterancy, role, leader contribution, supply, and the receiving force where relevant.
- **Deferred information:** full biographies, relationship networks, and hidden personality values.
- **Layout and camera:** dedicated comparison view; stack slots vertically and scroll on a phone. No map camera required.
- **Input and feedback:** select slot, choose replacement or destination, then confirm. Dragging is optional; all transfers have tap controls. Explain unavailable options.

### Battle result

- **Current decision:** understand losses and the resulting strategic position.
- **Dominant focus:** outcome and surviving forces.
- **Primary action:** Continue to Map; use a visible retreat choice only if the combat model permits one at this stage.
- **Supporting information:** casualties, destroyed formations, noteworthy character events, retreat destination, and control changes.
- **Deferred information:** full exchange logs and formulas behind a Details action.
- **Layout and camera:** readable report with linked places and people; long results scroll without hiding Continue.
- **Input and feedback:** tap a person or formation for details, Close to return, then Continue. Do not require watching an animation to learn the result.

### Siege

- **Current decision:** maintain, assault, withdraw, sortie, attempt escape, or provide relief, according to side and available forces.
- **Dominant focus:** the besieged location and forces around it.
- **Primary action:** the selected legal order, with risks and known defensive advantage beside it; equally valid choices can share emphasis.
- **Supporting information:** elapsed seasons, current fort condition, supply, and known relief threats.
- **Deferred information:** civilian chronology, biography, and detailed calculations.
- **Layout and camera:** keep relevant entrances and relief approaches visible; compact view can open a focused siege sheet.
- **Input and feedback:** visible labelled actions and clear disabled reasons; show the changed siege state after the action.

### Settlement development

- **Current decision:** choose a development focus or feasible investment.
- **Dominant focus:** selected settlement, condition, and the factors currently helping or blocking it.
- **Primary action:** Set Focus or confirm a specific supported order with its cost and duration.
- **Supporting information:** current focus, military state, supply/access, relevant population and damage.
- **Deferred information:** all other settlements, full history, and future building possibilities.
- **Layout and camera:** selection inspector or focused view; keep the location identifiable and avoid a building grid.
- **Input and feedback:** select one focus and confirm; show why growth remains constrained. Investment must not visually promise an instant tier change when the simulation does not.

### Character and legacy

- **Current decision:** where this person can contribute and what opportunity to provide.
- **Dominant focus:** identity, current role, relevant capability, and a few supported deeds.
- **Primary action:** Assign, Mentor, or a supported career choice with prerequisites.
- **Supporting information:** age, service, current army, known traits, eligibility and missing opportunities.
- **Deferred information:** full genealogy, chronology, hidden disposition, and irrelevant class paths.
- **Layout and camera:** dedicated detail view or expanded inspector; history opens separately rather than crowding every action.
- **Input and feedback:** visible role and relationship links with Close/Back controls; show reassignment or eligibility changes clearly.

### History and campaign end

- **Current decision:** review what happened and follow an interesting person, place, army, or era.
- **Dominant focus:** selected chronology; after victory or defeat, campaign outcome and a concise historical summary.
- **Primary action:** inspect a linked event or choose a supported continue/new-campaign/menu action.
- **Supporting information:** date, participants, location, outcome, and visibility confidence.
- **Deferred information:** unrelated event streams and technical save data.
- **Layout and camera:** filterable timeline with bounded scrolling and a visible Back control.
- **Input and feedback:** tap filters and links; retain navigation context after closing a biography.

## Map visual language

Use botanical emblems and coherent faction colors, supported by shapes, labels, and borders so color alone is not required. Distinguish control from partial regional ownership, known from unknown territory, and current observation from last-known information.

Settlement size, fortification, damage, ruins, roads, and supply breaks should be visible where relevant. Avoid placing all history as permanent labels on the map. Dense army markers can open a selectable stack list. World/region breadcrumbs and a return action explain scale without exposing implementation concepts.

## First-use teaching flow

Proposed prompts use the exact visible action labels:

1. “Tap your headquarters to inspect it.”
2. “Tap Briar Host to select the army.”
3. “Tap a connected destination, then tap Move.”
4. “Tap Enter Region to inspect Rosemarch's routes.”
5. “Tap Army to review the six formation slots.”
6. “Tap End Turn when your orders are complete.”
7. After combat: “Tap the highlighted person to see what they became known for.”

Show each only when the supporting feature exists. Dismiss completed prompts. A visible Help control reopens instructions. Hover and shortcuts can supplement every step, but cannot be required. Drag scrolling must not trigger an underlying action when the player releases their finger.

## Feedback, interruption, and recovery

Place costs and missing requirements by the action. A lost supply route belongs beside the affected army. A high-priority unresolved choice may interrupt End Turn, but ordinary army availability should not force a confirmation after every turn.

Save, load, settings, and debug controls belong behind clear navigation. Failed saves or invalid loads need an understandable message and a usable route back to play. Enemy turn progress should remain visible; the exact pause/cancel policy requires implementation decisions. Critical outcomes remain in state or history after transient notifications fade.

## Required visual and interaction review

At normal and minimum viewports, inspect first use, ordinary play, selected/expanded objects, dense late-game map and roster, long names, large values, siege urgency, lost supply, and failure/recovery states that actually exist. Verify every required interaction using touch alone: setup, map pan and zoom, selection, regional navigation, orders, composition, report dismissal, Help, menu, save/load, and recovery.

Check focus, map size, readable text, target sizes, clipping, overlap, hidden controls, drag-release behavior, and picking after resize/zoom/display scaling. Store captures directly in `docs/verification/` under stable scene names and replace equivalent states. Use the shared capture wrapper with its hidden-window default, wait for completion, and confirm the game exits. Browser touch checks supplement captures; neither compilation nor a clean screenshot proves usability.

No such Kestrum UI review has been performed in this documentation-only milestone.
