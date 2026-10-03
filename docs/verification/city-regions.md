# City-created regions

Date: 2026-10-03. Scope: the requested region-free opening, deliberate city
investment, city spacing, local countryside views and capital tutorial.

## Design and ownership

New production campaigns retain all 152 physical sites and their routes, with
one country-map marker per site. No region is unlocked at the start. The old
eight-cluster layout supplies authored geography; Rosemarch remains a separate
regression scenario. Derived geography is validated against its deterministic
source rather than permitting arbitrary saved topology.

A Village or Town may receive paid city development. The shared command and
projection enforce ownership, safety, supply, usable conditions, affordability,
and no directly neighboring City or Major City. Investment preserves population.
Natural growth cannot bypass the paid City transition. The AI uses the same
command and retains its upkeep reserve.

A functioning city unlocks a local view containing the center and connected
countryside. These are existing physical identities with unchanged ownership,
income and movement costs. The region closes if the center falls below City or
becomes ruined; no sites or saved references are removed. The local view must
use observer-visible state and must not disclose hidden neighbors.

The opening tutorial must use the ordinary paid action on the capital, then
enter that specific region and return to the country map. Management costs and
blocked reasons remain visible, and completing the guide dismisses its prompts.

## Validation

The first slice covers production generation, save validation and city-development
engine behavior. The second covers controls, local rendering and tutorial
integration; that second slice is not yet validated.

The initial full integration run used
`..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --no-fail-fast`.
Wall time was 564.21 seconds, including a 3 minute 56 second cold-slot compilation.
The long `campaign_scenarios` target passed in 174.42 seconds. City development
(5), region membership/projection (3), generation (5), regional starts (5), and
the source-size gate (1) passed. Strict all-target/all-feature Clippy passed.

The full run retained the known seed-88 victory failure and found stale region
fixtures in `world_navigation` and `observer_visibility`, plus two midgame setup
failures. The former tests now retain their authored-region checks on Rosemarch.
The midgame diagnostic confirmed a real opening battle whose detailed report
expired under the existing 40-round retention rule. It also exposed a scripted
patrol grouping learners from different locations. Focused verification of these
fixture corrections then passed using
`..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --test midgame --test world_navigation --test observer_visibility --test code_standards --no-fail-fast`:
midgame 5/5, navigation 2/2, observer visibility 6/6 and source size 1/1.
Wall time was 45.29 seconds (29.81 seconds compilation; midgame 14.66 seconds).
The repaired patrol uses each learner army's actual route evidence and ordinary
movement; the round-60 save still needs ten adults, eight trained heroes and
positive persistent battle/meaningful-encounter evidence. No game rule or test
round limit was relaxed. Formatting, diff checks and strict Clippy passed again
(Clippy wall time 5.37 seconds). Unaffected full-suite results were reused.

`scripts/capture_ui.ps1 -Scenes production_world,move_arrived -WindowWidth 1920 -WindowHeight 1080`
passed through the hidden shared wrapper in 2.55 seconds; its process exited.
The [opening country map](ui_production_world.png) shows separate physical nodes
with no unlocked region; the [arrival state](ui_move_arrived.png) preserves the
movement receipt and newly discovered neighbors. Both 1920×1080 images were
inspected. The arrival panel's heading overlaps the notification strip; the next
UI slice must address that alongside city controls. These captures establish
the opening/arrival states, not acceptance of the unfinished city-region UI.

The Kestrum-specific workflow in [PROJECT_AGENTS.md](../../PROJECT_AGENTS.md)
requires local pooled builds and hidden verification; external publication is
deliberately unrun. The inherited seed-88 round-240 production-victory failure
is an authorized baseline exception, not a passing check. The existing physical
touch waiver remains in force; synthetic input does not establish device touch.
