# Movement plans — 2026-10-01

Long orders spend the group's available movement and save the remaining physical
route. A fully exhausted group can confirm a destination without a movement
error or a fabricated movement fact. The order resumes on its next faction turn,
after movement refreshes, and remains queued over further seasons as needed.
New orders replace overlapping plans; Cancel Route stops the selected plan.
Splits, transfers, lost armies and sieges reconcile invalid groups.

Foreign territory requires an actual war with its controller. A peaceful border
anywhere on the requested route rejects the whole order atomically, even beyond
the current allowance. Every continuation uses the normal movement command and
rechecks diplomacy, threats, battles and sieges. Encounters pause other plans
until their decision resolves. NPC one-edge planning requires an affordable step
so it cannot repeatedly submit an exhausted order.

Plans default to empty in old saves. Automatic travel after the seasonal boundary
records new-turn facts through the normal command pipeline. A boundary sequence
permits the catalogue checkpoint to include that travel; subsequent actions
invalidate that exception. Reload retains location, spent movement and pending
facts without repeating seasonal effects or movement.

## Screen composition and visual review

The current decision is where the selected army should travel. The atlas remains
dominant; the existing order card shows remaining movement, destination, current
turn stopping site, and Confirm Move. A saved route uses that same card for its
destination and Cancel Route. Route detail remains behind Review Route, and
rosters behind Army Details. Future affordable-season steps use the route's gold
colour; actual blocked borders use orange and disable confirmation.

The shared hidden capture wrapper wrote directly to the stable files below.
Requested normal size is 1920 × 1080 (native client image 1920 × 1061), with the
minimum 1280 × 720. Orders, zero-movement confirmation, saved destination,
cancellation, arrival, peaceful borders, route detail, dense army names and help
were visually reviewed at both sizes. Help wording was shortened after review
found its last paragraph touching the navigation controls.

| State | Normal | Minimum |
| --- | --- | --- |
| Departure with enough movement | [Departure](ui_world_move_exit.png) | [Departure](ui_world_move_exit_minimum.png) |
| Affordable prefix and remaining route | [Partial](ui_world_move_partial.png) | [Partial](ui_world_move_partial_minimum.png) |
| Saved destination and Cancel Route | [Plan](ui_world_move_queued.png) | [Plan](ui_world_move_queued_minimum.png) |
| Arrival on the next faction turn | [Arrival](ui_world_move_continued.png) | [Arrival](ui_world_move_continued_minimum.png) |
| Peaceful border rejection | [Border](ui_world_move_peace.png) | [Border](ui_world_move_peace_minimum.png) |
| Optional route detail | [Review](ui_world_move_review.png) | [Review](ui_world_move_review_minimum.png) |
| Dense names and multiple armies | [Stack](ui_world_move_stack.png) | [Stack](ui_world_move_stack_minimum.png) |
| Exhausted-army instructions | [Exhausted](ui_move_exhausted.png) | [Exhausted](ui_move_exhausted_minimum.png) |
| Movement help | [Help](ui_help_movement.png) | [Help](ui_help_movement_minimum.png) |

The harness selects the actual projected army banner with press/release input,
dispatches real destination and confirmation actions, cancels and replaces a
saved route, and advances a completed season through ordinary commands. It
asserts gate arrival, a zero-step confirmable preview, the persisted remainder,
and next-turn arrival with the correct remaining allowance.

## Published browser review

The Preview WebGL build at `http://127.0.0.1/games/kestrum/` was exercised with
visible click controls. Rose Host moved from Riverfold Green to Riverfold Bridge
and back, spending all six movement points. Confirming Bridge again saved a
route at zero movement. Cancel Route cleared both the plan and its feedback;
re-confirmation restored the plan. Dismissing the order card retained the route.
End Turn advanced to Summer / Year 1, Round 2, and the host travelled to Bridge
with three movement points left. Its checkpoint saved without a recovery error.
Reload and Continue restored Summer, the Bridge location and the same three
remaining points. No browser warnings or errors were observed; temporary
viewport overrides were reset after the review.

[Queued browser route](ui_move_plan_browser.png) and
[automatic browser arrival](ui_move_plan_arrived_browser.png) retain evidence.

The in-app browser's 1280 × 720 viewport override again rendered a smaller image
in the upper-left with black remainder, as documented in earlier map reviews.
Minimum browser acceptance remains unverified. Native minimum-size review and
click/press-release equivalents do not establish physical-touch acceptance;
physical touch and pinch hardware were not exercised.

## Validation

All validation uses this project checkout and the real shared workspace. The
focused movement, plan, persistence, AI, diplomacy, siege and source-size targets
passed all 34 checks, including five new movement-plan cases and a catalogue
checkpoint/reload regression. The full suite passed 327 checks, with one ignored
release profiling test and the existing production victory assertion failing at
its 240-round cap (`k18_production_battle`). Its balance blocker is already
recorded in the README and earlier battle review; the assertion remains intact.
Four- and eight-faction campaigns retained and replayed through round 400, and
all developed-midgame-save checks passed.

The final AI eligibility short-circuit retained cheap rejection before preview;
its AI, movement-plan, persistence and source-size rerun passed all 19 checks.
`cargo fmt -p kestrum -- --check`, whitespace checks, and strict all-target
Clippy (`..\rust_management\cargo.ps1 clippy -p kestrum --all-targets '--' -D warnings`)
passed. Every Rust file remains within the 800-total-line limit. All capture
wrapper runs completed and their launched game processes exited; no native
Kestrum process remained after the final help captures.

Final no-argument `.\publish.ps1` succeeded: Windows and WebGL release builds,
packages, assets and the root catalogue thumbnail deployed to Preview at
`\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`. Project Roost recorded
the Preview publish, and the local catalogue was updated.
