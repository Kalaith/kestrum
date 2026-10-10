# Map orders and consequences

[Verification index](README.md) · [M02 roadmap](../map-playability-plan.md#m02-map-orders-and-consequences)

## 2026-10-10 — Contextual orders

Owned places expose Recruit, Build and Focus in the compact map inspector.
Recruitment can create a new army or add a formation to the selected army;
the selected army's movement card opens recruitment for that exact army.
Build, road and focus choices use the existing previews, costs, blocked
reasons and authoritative commands. Construction review and builder paging
stay on the map. Detailed management, armies, history and diplomacy remain
available through their existing views.

The inspector retains the selected country marker or regional site. Back
returns to that selection; changing place, closing selection or ending the
player turn clears unfinished choices. Successful recruitment immediately
changes the army banner and resource totals. Successful construction and focus
leave their result in the selected-place inspector.

The runtime capture harness exercises actual rendered press/release controls
with synthetic pointers. It checks paging, a successful recruit and exact
resource spending, a stale enabled confirmation rejected with the campaign
unchanged, recruitment into the exact selected ArmyId with its saved route
unchanged, construction target/kind/cost, builder paging, focus, Back, Close
and End Turn. Facility work correctly does not retain a builder after placement.
These checks are native application actions; they are not hardware touch tests.

Reviewed native 1920×1080 evidence:

- [Owned-place controls](ui_context_place.png)
- [Selected-army recruitment](ui_context_army_recruit.png)
- [Recruitment result](ui_context_recruit.png) and [stale rejection](ui_context_recruit_blocked.png)
- [Construction result](ui_context_build.png) and [review](ui_context_build_review.png)
- [Construction choices](ui_context_build_choices.png) and [builder page](ui_context_builders.png)
- [Focus choice](ui_context_focus_selected.png) and [End Turn / dismissal](ui_context_focus.png)

The initial capture assumptions that required an eligible recruit on the final
page and a retained builder for facility work were corrected against the engine.
No gameplay rule was changed to make those captures pass. Visual review corrected
a clipped recruitment result and an oversized confirmation label.

Validation passed on the frozen source:

- `cargo fmt -p kestrum -- --check`.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`.
- `..\rust_management\cargo.ps1 test -p kestrum --test code_standards --test asset_registry --test content`: 9 passed. The source-size gate was rerun after subsequent harness and UI fixes; unchanged asset/content results were reused.
- Hidden pooled native build and ten individual capture processes exited 0.
  `scripts/capture_ui.ps1 -Scenes @('<scene>') -WindowWidth 1920 -WindowHeight 1080 -Frames 12`
  built the current executable; subsequent unchanged captures used `-SkipBuild`.
- Full diff, source ownership, screenshot and `git diff --check` review.

No tests were added. No full suite was required for this presentation and
application-navigation change; loading, save schemas, route planning and
simulation rules were not changed.

At this first-slice checkpoint, persistent directed routes and local work,
supply and seasonal-outcome marks remained for the following slice. Browser
acceptance was deferred to that completed implementation. No publication was run.

## 2026-10-10 — Routes and local consequences

Saved movement plans draw their remaining path with direction arrows in the
working map bands, including after selection is dismissed. Selected routes are
emphasized and preview routes draw above them. Overview retains aggregate force
marks and suppresses unselected route lines. The movement card reports the
remaining steps and supply warning alongside its existing destination and
blocked reason.

Open player construction orders show progress beside their known target, with
the authoritative pause reason when blocked. Overview groups work at a shared
anchor and uses a paused order as its representative when one exists. Known
player places outside supply get a separate cutoff badge. Work opens existing
management; supply opens place context. Neither badge changes simulation rules.

Eligible recent, unread or active receipts group beside their known place or
route, using saved typed snapshots when needed. Active and urgent receipts take
priority. Map eligibility is derived before the existing event rail is truncated,
so rail overflow does not remove local feedback. Off, History Only, dismissal,
hide cutoffs and discovery remain effective; Observer gets no player receipt
badges. Faction-wide receipts without a location remain in the event system.

Tapping anywhere on an outcome badge opens the existing event card, with
Next/Previous across the location group. Completed construction and old battle
receipts can still use Show on Map after their live record has disappeared,
within current discovery. Stale construction receipts do not restore a command
against a missing order. Feedback is projected from existing state and receipts;
commands are never replayed to reconstruct it.

Drawing, hit testing, gesture blocking and label avoidance share one status
layout. It reserves the inspector, event card/rail, Attention, tutorial, resource
strip, feedback, map controls, places and army marks. Displaced badges have a
leader to their anchor; route anchors use the same authored polyline as the
drawn road. If no collision-free location fits, zoom or use Attention/the event
rail. Map Key explains the new marks. This is a density limit, not evidence that
every status is visible at every zoom.

The runtime harness uses existing authored captures for queued movement and
construction, with accepted player commands and scripted rival End Turn passes.
Its grouped-place examples are synthetic typed receipts used to isolate mixed
category ordering and delivery controls. Work completion and supply-blocked
receipts come from real seasonal engine transitions. These are deterministic
application checks, not evidence of a human campaign or hardware touch session.

Engineering checks on the frozen second slice passed:

- `cargo fmt -p kestrum -- --check`.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`.
- `..\rust_management\cargo.ps1 test -p kestrum --locked --test code_standards --test asset_registry --test content`: 9 passed after the final grouped-work fix.

Compiler feedback removed unused exports/fields, simplified the work grouping
value and fixture arguments, and corrected a slice conversion. The initial
native audits were corrected to respect the authored fixture's paused rivals
and use its known legal replacement route. These were harness assumptions;
no movement or seasonal rule was changed to obtain a pass.

Reviewed native 1920×1080 evidence:

- [Saved route after dismissal](ui_m02_routes_dismissed.png).
- [Two selected armies and an overview route](ui_m02_routes_overview.png).
- [Construction progress, supply pause and cutoff](ui_m02_construction_supply.png).
- [Grouped local seasonal outcomes](ui_m02_receipts_map.png).
- [Completed/pruned work returns to its known place](ui_m02_receipt_fallback.png).
- [Map Key](ui_m02_map_key.png).

The receipt audit checks the urgent/information group with Next/Previous,
read/dismiss, whole-badge pointer handling, overlay suppression and stale
delivery actions. The other place's receipt is inspected in the same campaign
after focusing that place; the dense layout did not fit both badges in one
view. The final image restores the home group. Turning delivery off preserves
the separate current-condition supply badges, threats and Attention.

The fallback audit starts and completes ordinary Fort and Stable orders through
accepted commands, then checks the older pruned Fort's saved destination. An
unknown typed destination cannot focus the map through either the UI or direct
application action. The pruned-battle fallback is source-reviewed only.
Final review also aligned route-receipt Show on Map with the UI's first known
endpoint, avoiding an enabled button that failed when only the second endpoint
was discovered.
Overview work grouping was also corrected so the displayed paused order's site
drives its tap action when several sites share one marker.
An isolated presentation-only projection places an earlier active Site5 order
before the real paused Site10 order at their shared marker. Its pointer action
opens Site10, then the harness restores the original projection and view.

All six individual hidden capture processes exited 0 at 1920×1080 with 12
frames. Map Key and overview captured the final "Steps left: 1" copy. The
construction/supply capture rebuilt the final grouped-work source and passed
the representative-site audit. Other accepted images were retained where
subsequent changes did not affect their scene. Formatting, strict Clippy and
the source/content gates were rerun after the endpoint and grouped-work fixes.
The three focused gates passed again after the grouped-work fix. No tests were
added and no full suite was required: save schemas, content loading, generation, route planning
and portraits did not change.

Browser acceptance used the final pooled release build:
`..\rust_management\cargo.ps1 build -p kestrum --locked --release --target wasm32-unknown-unknown`.
It exited 0. The slot-1 artifact was 9,499,236 bytes, modified
2026-10-10 07:24:35 UTC. An earlier successful build was superseded by the
grouped-work fix and was not used for acceptance.

Headless Chromium loaded that exact WASM and current checkout assets through an
in-memory localhost server. The existing generated `dist/webgl` shell normally
displayed a 1200×675 canvas. Test-host CSS expanded its container without
fullscreen: both CSS bounds and drawing buffer measured 1920×1080, DPR 1.
This verifies the game at its acceptance canvas, not a newly packaged or
published shell. The delegate hit its usage limit during browser setup; the
primary agent completed the browser work and closed the validation services.

In a normal four-kingdom campaign, seed 260926, browser mouse controls opened
the capital's recruitment context. Emulated touch selected and recruited
Warriors: 500/200/150 resources became 440/190/150 and the force mark showed
two armies. Touch also reviewed and placed a Fort for 100 Gold, 80 Wood and
100 Stone, leaving 340/110/50 and an immediate local `Fort · 0/3` badge.
Tapping the badge's text opened the correct place's management view.

Three ordinary End Turns ran the live rivals and progressed the Fort through
completion. Its local outcome group opened with a touch on the badge text;
touch Next and mouse Previous changed the selected receipt and persisted its
read state. [Browser outcome card](ui_m02_browser.png) shows the real local
receipt group, two armies and accounts at the completed third season.

The visible Save Game/Create Save and Load Game controls restored the campaign.
Saving a new copy after loading produced an identical entire campaign JSON
payload in the toolkit's browser localStorage catalogue. Page reload, waiting
for storage readiness, Continue and another visible save produced the same
payload again: accepted sequence 55, completed seasons 3, resources 370/167/86,
three receipts and their acknowledgement flags unchanged. No repeated spending
or receipt insertion occurred. No browser page errors were recorded.

Browser movement-plan reload and blocked-supply interactions were not repeated;
the native route roundtrip and supply-action audits above cover those cases.
Hardware touch, a broad human campaign, native on-disk Continue, the full suite
and publication remain unrun. Existing unrelated TODOs and older failed-capture
logs were preserved; 16 diagnosed logs created by this M02 task were removed.
