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

This slice does not complete M02: persistent directed routes and local work,
supply and seasonal-outcome marks remain the next slice. Browser acceptance is
deferred to that completed implementation. No publication was run.
