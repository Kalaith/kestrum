# Campaign notifications

Planned on 2026-10-02. **Implementation in progress.** Durable receipt foundations
are being validated; UI and forecast acceptance remain outstanding. See
[scoped verification](verification/portraits-events.md).
This specifies the notification part of [M02](map-playability-plan.md#m02-map-orders-and-consequences).
The other M02 map-order and inspector work keeps its existing scope.

## Player experience

Keep a compact row of event icons below the map header. A new hero, a settlement
about to decline, or a captured holding should be noticeable even when the
player is looking elsewhere. Tap an icon to read a small detail card, inspect
the affected person or place, dismiss the message, or turn off that message type.
The world stays visible and usable around the card.

The supplied Stellaris image establishes the reference: several top-of-screen
notifications with details opened deliberately. Stellaris' developers also
describe different urgency levels and message settings accessible from a message
or the menu in [Dev Diary #295](https://store.steampowered.com/news/posts/?appids=281990&enddate=1682593307&feed=steam_community_announcements).
Use that interaction pattern with Kestrum's seasonal turns and visual style.
The user's copied settings instructions are reference material, not requirements
to reproduce Stellaris' menus, hotkeys or event simulation.

“Terrain changes” means changes to controlled places: habitation, control,
occupation, damage, ruin, supply, fortification and construction. Physical
geography is currently static. New biome transformations or disasters would
need their own gameplay feature and could later use this notification channel.

### Screen brief

| Question | Planned behavior |
| --- | --- |
| Current decision | Does this change need attention, and where should I act? |
| Dominant focus | The world or region map and the player's current objective. |
| Primary action | Inspect the event; use Show on map or an explicit subject/action link if useful. |
| Supporting information | Event type, urgency, unread state, season and affected subject. |
| Deferred information | Explanation, before/after values, related events, history and notification settings. |
| Layout and camera | One shallow icon row and at most one compact inspector; camera stays put until Show on map. |
| Input and feedback | Visible tap controls for reading, dismissal, overflow, settings and navigation; no required hover, right-click or hotkey. |

### Layout and interaction

- Use Kestrum's existing **1920×1080** canvas contract. Begin with a 64-pixel
  rail at y=96, below the header/tutorial band; buttons have at least 56×56
  logical-pixel targets. These dimensions are tuning defaults, subject to review.
- Show up to eight event buttons, then **More (N)** and **Notifications**.
  Symbols distinguish people, places, work and danger. A count shows grouped
  events; an unread dot and a shape/text cue accompany semantic color.
- Prefer urgent actionable changes, then warnings, then useful information;
  use occurrence order within each priority. Freeze button positions during a
  pointer gesture so an arriving event cannot change its target before release.
- Tap opens a roughly 480-pixel-wide card below the rail, capped at 620 pixels
  high. Long content scrolls within it, with visible page/scroll controls and a
  reachable footer. No fullscreen scrim, fullscreen notification sheet or stack
  of overlapping event windows.
- The card uses the existing inspector space. Opening it collapses Attention
  and temporarily replaces the selected-object inspector. Close restores the
  previous inspection context when valid. Preserve camera and queued orders;
  clear any armed destination-picking mode to prevent accidental movement.
- Close/Back dismisses the card before leaving the map; Escape may supplement
  the visible Close control. The first event teaches: “Tap an event icon for
  details. Use Notification settings to choose which messages appear.” Keep
  that instruction in reopenable Help after it is dismissed.
- Only **Show on map** changes camera/scope. It closes the card, focuses the
  current eligible subject through navigation, and opens its inspector.
  **View person**, **Manage place** and **Records** deliberately enter their
  existing views. Returning restores context, without re-arming an order.
- Reserve card/rail rectangles in label placement, camera framing and pointer
  hit testing. UI gestures are consumed before map picking, pan, zoom or orders.
  Outside the card the map remains interactive; selecting a map object closes
  the card. End Turn, NPC Pause/Step/Resume and Menu remain reachable.
- During battles, required decisions and management sheets, collect events
  without covering that view. Display them on return to the map. An incoming
  event never changes the screen, camera, simulation speed or pause state.
- An already-open card remains a dated event snapshot while AI actions run.
  Refresh its current-status note and revalidate every action before execution.
  Notification inspection itself does not pause the simulation.

Illustrative arrangement; labels below stand in for icon buttons:

```text
WORLD / REGION                  Spring, Year 16                    Menu
  [Hero •] [Growth 2] [Work 3] [Danger •] [More (4)] [Notifications]

  ┌ A new hero emerges ───────────── Close ┐
  │ Elara · Spring, Year 16               │       MAP REMAINS VISIBLE
  │ Recognized through service in ...     │       AND INTERACTIVE
  │ Army / place · relevant change       │
  │ [Show on map] [View person]           │
  │ [Dismiss] [Notification settings]     │
  └──────────────────────────────────────┘

  Resources / seasonal accounts          Attention             End Turn
```

Examples are proposed presentation copy, not claims about existing characters.

## What to notify

Start with the player's own people and places. Ownership at the time of the
event matters: losing a place must still notify its former owner. Political
claims alone do not mean every site in a region is controlled. Participant
events and already-observed facts can be included where the existing visibility
rules permit them; hidden rival development and private careers stay private.

| Category / type | Trigger | Default priority and useful detail |
| --- | --- | --- |
| People: new hero | A person emerges or first gains recognition through service | Information; name, army/place, role and recorded reason. Merge emergence and immediate recognition of the same person into one card. |
| People: career and continuity | Class completion, arrival, retirement, death, succession or vacant command | Information, or urgent for an unresolved vacant command; preserve existing life/continuity notices and explain next available action. |
| People: earned opportunity | A service/prerequisite change newly opens a supported career path for an owned person | Information; group newly available paths per person, link to Career, and revalidate eligibility on selection. Ordinary affordability fluctuations do not repeat the notice. |
| Places: growth approaching | An eligible owned place is one seasonal step from its next tier under known current conditions | Information; current → expected tier, conditional timing and relevant causes. |
| Places: decline or ruin risk | Known conditions put an owned place one seasonal step from downgrade or ruin | Warning; risk, causes and available interventions. Ruin or endangered capital can take urgent priority. |
| Places: actual change | Habitation changes, ruin/reclamation, capital/HQ relocation or fort/facility functional status changes | Information or warning for harm; dated before → after, recorded cause and subject link. |
| Territory and security | Control gained/lost, contested/occupation state entered or cleared, relevant siege or known local threat started/ended | Urgent on loss or immediate danger, otherwise information; distinguish physical control from regional claim. |
| Orders and supply | Route arrived or blocked; an owned force loses or regains supply | Warning for blockage/loss, information for arrival/recovery; destination and legal next action. |
| Military outcomes | An owned force retreats or is destroyed; a player-relevant battle resolves outside the currently reviewed encounter | Warning or urgent; safe outcome, survivors/retreat destination and report link. Do not duplicate the battle aftermath the player is already reviewing. |
| Construction | Work completes, becomes blocked, resumes or is lost | Information or warning; work/site and reason. Batch routine completions. |
| Economy and diplomacy | Unpaid upkeep begins/clears, or a player-relevant diplomatic outcome/decision arrives | Urgent if action is required, otherwise information; existing rules and confirmations remain authoritative. |
| Remembrance | Existing anniversary and legacy-transfer notices | History only by default; named settings allow top-bar delivery. |

Do not notify every population increment, repair tick, pressure change, routine
income receipt, AI move or construction progress step. Keep immediate accepted
command feedback and save/load feedback brief. A richer background event should
have one delivery path, so a hero does not also produce a duplicate text toast.

### Advance warnings

1. Evaluate after accepted state changes and at the start of the player's turn,
   early enough to act before **End Turn**. Reuse a pure rules query; do not
   resolve an extra season, consume RNG or inspect future AI decisions.
2. Share eligibility and threshold logic with development/construction. Respect
   habitation caps, population requirements, pressure, ruin streaks, pending
   construction and the actual round order. Construction precedes natural
   development; a raw pressure-plus-contribution check is insufficient.
3. Say **“Expected next season if conditions hold”**, **“At risk next season”**
   or **“Work expected to complete next season if uninterrupted”**. Include
   the forecast's season and conditions. Never guarantee a change that later
   warfare, lost supply, capture or another order could prevent.
4. Use the observer-safe development view. If contribution/safety is unknown,
   withhold a precise tier forecast and show uncertainty only when an existing
   known risk justifies it. Do not reveal enemies through a warning or its causes.
5. Create one warning when entering an imminent state. Update it in place as
   evidence changes; a meaningful escalation may make it unread again. When
   conditions recover, ownership is lost, or the change occurs, close the active
   warning and retain its dated result. Renewed risk gets a new episode identity.

Damage that happens immediately in battle can only be reported afterward unless
an existing observed siege/risk supports an earlier warning. Notifications do
not invent advance knowledge or delay game rules to guarantee warning time.

## Noise controls and message lifecycle

**Notification settings** is a visible action on every card and in
**Menu → Settings → Notifications**. The top-bar **Notifications** control
opens the same compact surface with **Recent** and **Settings** tabs. Settings
can be reached even when every category is muted and the event rail is empty.
Use broad categories with expandable individual message types, so “growth
approaching” can be disabled while “settlement declined” remains enabled.

Delivery choices:

- **Top bar + history:** persistent event button until dismissed or aged out;
  all types above use this by default except Remembrance.
- **History only:** off for attention-grabbing delivery; retained in Recent
  without rail badges, sounds or automatic opening.
- **Off:** suppress delivery and remove this type from the notification view.
  It does not delete authoritative campaign history or change gameplay.

Provide **All off** and **Restore defaults**, with their resulting state visible.
Changing a type to History only/Off removes existing buttons immediately.
Changing it back applies to future events; it does not replay old messages.
Preferences persist locally across campaigns and reloads using the existing
preferences store. No auto-open, auto-pause or notification sound in the first
delivery. Optional toasts/sounds can follow only if play review demonstrates a
need; the durable rail already solves missed events.

Lifecycle rules:

- Opening a card marks that event read, but leaves its button until dismissed.
  Opening a group marks only the entries actually opened; **Mark all read** is
  explicit. Closing the card is different from dismissing the message.
- **Dismiss** removes that message from the rail; **Dismiss read** clears read
  informational entries. Neither action resolves a gameplay condition.
- Attention remains the home of current unresolved conditions. Link both views
  to the same subject and condition identity; do not add a second active-problem
  list. A muted event can still have a condition visible in Attention or on its
  affected object. Muting does not bypass required battle/diplomacy decisions.
- Group repeated same-type events from one season into a counted button, with
  explicit item navigation. Keep distinct people/sites and their details.
  Do not collapse a new hero or a lost holding into a generic seasonal summary.
  Grouping is presentation only and never substitutes one event for another.
- Use stable event/transition IDs, not rendered text, entity name or season
  alone, for deduplication. Warnings use a subject/type/episode identity. Several
  genuine events for the same subject in one season remain possible.
- **Recent** is a bounded inbox, not a second permanent history archive.
  Initial tunable limits: 200 receipts and eight completed seasons. Prune old
  dismissed/read items first; at the hard bound summarize omitted entries with
  an explicit count, including unread ones, and a Records link. Active condition
  warnings are bounded separately by subject/type and remain discoverable in
  Attention. Validate and store these limits as presentation data.
- Retain safe event-time names and before/after facts while the receipt survives.
  If an entity disappears, keep the explanation and disable its live navigation
  with a reason. If longer-lived Records were pruned, say so; never recreate them
  or imply Records is an unlimited archive.

## Implementation design

### Current foundations and gaps

| Existing source | Use and required change |
| --- | --- |
| `src/engine/history/life.rs`, `src/engine/notices.rs` | Life events already identify emergence/recognition. Convert typed facts to targetable receipts instead of concatenated strings. |
| `src/game/campaign.rs`, `src/game/storage.rs` | The shared five-second notice is overwritten by subsequent outcomes/save feedback. Separate background inbox delivery from transient command/storage feedback. |
| `src/engine/development/progress.rs`, `src/state/development.rs` | Reuse `DevelopmentReceipt` for actual habitation/ruin changes. |
| `src/engine/development/query.rs`, `conditions.rs` | Extend pure observer-safe queries for conditional warnings; preserve unknown safety/contribution. |
| `src/engine/construction/progress.rs`, `src/engine/siege.rs` | Reuse typed work and siege transitions, filtering progress-only changes. |
| `src/engine/actions.rs`, `actions/types.rs`, `round.rs` | Integrate at accepted transaction boundaries, with explicit transition receipts where missing. Handle NPC, battle and nested continued-movement results exactly once. |
| `src/engine/overview.rs`, `src/ui/overview.rs` | Reuse Attention's current-condition targets and observer rules; extend relevant conditions without a parallel rules implementation. |
| `src/ui.rs`, `src/game/composition.rs`, `src/game/world.rs` | Add a map-local rail/card, input guards and reserved map rectangles. Current `Overlay` sheets dim the full canvas and suspend NPC processing, so they are unsuitable for the notification card. |
| `src/state.rs`, `src/game.rs`, campaign compatibility and storage | Extend local `Preferences`; persist campaign receipts, acknowledgement and warning episode state with additive compatibility. |

New cohesive modules can be `state/notifications.rs`, `engine/notifications.rs`
with receipt/forecast/projection children, `game/notifications.rs` and
`ui/notifications.rs` with rail/details/settings children. Use named module files,
keep every Rust file below 800 total lines, and split existing large dispatchers
by responsibility as needed. This is a boundary proposal, not a requirement to
create empty scaffolding.

### State and transaction contract

- `StrategicCampaign` owns bounded receipt facts, monotonic notification IDs,
  source identities, read/dismissed state and warning episodes. `Game` owns the
  open card, selected item, paging and return context. Global preferences own
  delivery choices. UI reads projections and emits explicit intents.
- A receipt contains a kind, occurrence season/order, source ID, observer-safe
  subject references/snapshots, before/after facts, priority and lifecycle state.
  Store structured values and JSON text keys; format text in projection.
  Receipts can reference history, but must not require history to remain present.
- Collect typed changes inside the accepted transaction. Preserve pre-change
  ownership for loss/capture and ownership snapshots for development. Deduplicate
  nested results by source identity and finalize before checkpoint serialization.
  Preview, rejected commands, redraw and save retry must not append receipts.
- A single owner must perform collection; do not append both in nested engine
  actions and again in `handle_campaign_result`. Audit every early-return command
  path and automatic progression path before choosing the final integration seam.
- Global delivery preferences filter the presentation, not simulation or RNG.
  Off types may still have bounded internal receipts, but stay absent from the
  notification view. Do not persist different simulation outcomes due to settings.
  Track presentation delivery eligibility/cutoffs separately for each campaign
  and type when preferences change or a campaign is loaded, so enabling a type
  cannot promote its suppressed backlog into new top-bar events.
- Compare controller, contestation, supply and functional-status transitions at
  accepted state boundaries where no typed event exists. Emit a threshold/state
  change once; never discover events by scanning the whole history every frame.
  Career-opportunity notices compare earned eligibility changes through existing
  progression queries; group simultaneous unlocks and exclude routine cost
  fluctuations. Battle receipts provide retreat/destruction outcomes, subject
  to visibility and suppression of already-reviewed aftermath.
- Existing development history can have a broad audience. Apply relevance and
  event-time ownership in addition to history visibility. Protect private people,
  enemy rosters and unknown causes in counts, groups, text and navigation alike.
- Observer mode keeps a separate session with notifications off initially;
  explicitly selecting observed kingdoms may be a later extension. Leaving it
  must restore the human campaign's inbox and never leak observer-only facts.

### Persistence and shared tooling

Add defaulted campaign fields through the existing strict compatibility path.
Old saves load with an empty event inbox; baseline current warning conditions
without announcing old heroes or old captures. Current risks remain available
in Attention; subsequent new/escalated risks notify normally. Save/read/dismiss
state travels with a saved campaign, while delivery preferences stay local.
Loading an older manual save intentionally restores that save's unread state.

Mark notification presentation changes dirty for the next durable save. Ensure
checkpoint and named-save snapshots include them, and use existing storage
recovery for failures. Do not silently overwrite a named save merely to persist
a click. Reload guarantees apply to successfully saved state; the UI must not
claim unsaved acknowledgements are durable. New campaign and failed-load paths
must respectively reset and preserve the correct inbox.

The toolkit's `NotificationManager` already provides transient toasts and bounded
text/type history. It lacks campaign IDs/targets, categories, read state and
observer filtering, so typed campaign receipts belong in Kestrum. Reuse toolkit
buttons, scrolling, viewport/pointer helpers, JSON loading and persistence;
consider any needed generic rail/card widget as a toolkit addition before a local
alternative. Kestrum owns relevance, priorities and forecast rules. No new
dependency is expected.
Put message copy, category definitions and tunable limits in validated JSON under
`assets/data/`, loaded through toolkit APIs.

## Delivery and acceptance

Deliver within M02 as useful, separately validated commits. These are planned
packages, not completed milestones:

1. **M02-N1: durable events and compact UI.** Add receipts, additive save support,
   rail/card, Recent, per-type settings and own-hero details. Migrate all existing
   background `action_notices` types to the new path without duplicate toasts.
   Preserve transient command/save feedback. Acceptance: a new hero is noticed,
   can be located, stays readable through later events, and obeys mute/reload.
2. **M02-N2: controlled places and advance warnings.** Add actual development,
   construction, control and condition transitions plus conditional development,
   ruin and work forecasts. Connect active risks to Attention. Acceptance: an
   imminent downgrade can be inspected before End Turn; recovery clears that
   warning; actual change or ownership loss produces the correct dated receipt.
3. **M02-N3: complete seasonal integration and review.** Cover remaining relevant
   career opportunities, retreat/battle, route, supply, siege, economic and
   diplomatic outcomes; finish batching,
   overflow, tutorial/help and dense-state review. Acceptance: the player can
   explain a busy season from the compact feed and follow its affected subjects
   without losing map orientation or receiving hidden information.

Strongly target five cohesive behavioral cases for the notification feature,
using table-driven inputs and extending existing regression suites where useful:

1. Accepted player/NPC/nested actions produce the right ordered receipts once;
   preview, rejected actions, save retries and history pruning do not duplicate them.
2. Conditional forecasts match eligible rule states, clear/recur by episode and
   preserve uncertainty; cover population/cap/construction/ruin boundary cases.
3. Own-event relevance, former-owner loss, hidden people/sites, observer isolation
   and safe counts/targets hold across all projection modes.
4. New/old saves, preferences, read/dismiss state, retention, missing subjects and
   failed load/storage preserve their documented semantics.
5. App-action guards support open/close/focus/group/mute flows, pointer consumption
   and safe return through sheets/battle/turn changes without issuing stray orders.

Keep tests under `tests/`. Preserve useful life-history, continuity, development,
construction, persistence and observer coverage; justify distinct cases beyond
the target instead of deleting regressions.

### Visual and interaction review when implemented

Review the actual supported 1920×1080 native and browser canvas. It is both the
normal and smallest supported design canvas; smaller hosts retain the existing
scaling/letterbox behavior rather than acquiring a new layout promise. Verify
pointer mapping after host resize and report physical target limits if scaled.
Keep the existing physical-touch waiver explicit; synthetic taps are not hardware
touch verification.

Use stable captures directly in `docs/verification/`: `ui_notifications.png`,
`ui_notification_details.png`, `ui_notification_warning.png`,
`ui_notification_settings.png` and `ui_notifications_dense.png`. Replace each
state in place. Cover early fog, tutorial coexistence, long names, eight icons
plus overflow, simultaneous heroes/place changes, urgent conditions and a selected
army. Exercise all visible controls, disabled links, map pan/zoom, NPC progression,
End Turn, save/reload and returning from battle. Confirm the card never obscures
the entire world or primary controls and notifications do not steal input/focus.

Delegate validation with relevant edits frozen. Use the actual checkout, shared
Cargo launcher and hidden capture wrapper; verify launched games exit. Run
formatting, strict Clippy, the source-size gate, affected tests and the required
no-argument `publish.ps1` after meaningful implementation changes. Record any
inherited failures or unverified browser/touch coverage explicitly. Commit each
validated package on `master` under the repository's commit rules.

The original planning revision had only documentation checks. Implementation
checks and outstanding acceptance work are recorded in the linked verification
document; this plan alone does not establish runtime or visual acceptance.
