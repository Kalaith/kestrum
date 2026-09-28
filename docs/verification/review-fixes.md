# Review fixes — 28 September 2026

## Implemented corrections

| Finding | Result | Commit |
| --- | --- | --- |
| Unfamiliar defending armies prevent first battles | Both attacks and objective travel use a bounded, configured unknown-strength estimate with the existing advantage margin. Hidden rosters do not inform the estimate. | `76543a3` |
| Emerging people inherit commanders' accomplishments | Personal command tags stay with their actual actors; retrospective emergence retains shared service only. | `7a97b36` |
| Medicine and Scouting lessons lack practical career connections | Lessons support reduced practical requirements, and eligible medical pupils assist actual recovery alongside their mentor. | `5ab1e66` |
| Dependents receive service anniversaries | Only serving people receive milestones; entering service starts their service clock. | `ddb54ec` |
| First-use teaching does not follow actions | Eight saved checkpoints follow actual actions, close on completion, and resume or restart through Help. | Guided-introduction change |

The focused AI, evidence, progression, specialist mentorship, succession and
legacy tests pass. Each correction was published with the default publisher
before its commit. The source-size gate passes with every Rust source below
the 800-line limit.

## Specialist careers

Five regression cases cover real person and formation recovery, unavailable
assistants, lessons without patients, completed apprenticeship/save continuity,
and actual scouting travel. Existing evidence, progression and mentorship suites
also pass. Formatting, strict all-target Clippy and the 800-line source gate pass.
Default Windows/WebGL publishing to Preview succeeds.

Replaced `ui_career.png` and `ui_career_minimum.png` through the shared capture
wrapper at requested 1920 × 1080 and 1280 × 720 window sizes. Both capture
processes exited. The dense career fixture retains identity and service above
course decisions; Scout and Medic show both evidence paths without truncation,
with prices and facilities beside their controls. Existing longer Cavalry and
Officer summary lines still truncate.

In the published browser, used visible controls from New Game through
headquarters > Armies > Orders > People > Career. Both specialist paths and
disabled-course feedback are readable at full screen. Resizing the in-app
browser's viewport to 1280 × 720 reported a correctly sized canvas but rendered
the game in a smaller upper-left area; native minimum-size captures are verified,
but this browser resize behavior remains a limitation. Physical touch hardware
was not available; interaction checks use the visible single-pointer controls.

## Guided introduction

Five regression tests cover old-save opt-in, out-of-order/idempotent receipts,
dismissal and restart, rejected commands, real movement and first-turn receipts,
save/reload, and campaign isolation. Progress is campaign state, separate from
simulation history. New campaigns start at headquarters; migrated saves remain
closed until opened from Help.

The shared capture wrapper exercised the actual UI action dispatcher through
headquarters, movement, region/world navigation, Career, household review, first
turn and Records. Assertions at the checkpoints verify advancement and final
dismissal. The blocked route keeps the movement lesson pending. Household review
does not confirm its proposed order.

Fourteen scenes were captured at requested 1920 × 1080 and 1280 × 720 sizes,
directly into the stable `ui_tutorial_*.png` names here. The normal captures are
1920 × 1061 after native window framing; minimum captures are 1280 × 720. Final
capture processes 8536 and 33356 both exited. Visual review covered:

| State | Normal capture | Minimum capture |
| --- | --- | --- |
| Headquarters | [Normal](ui_tutorial_headquarters.png) | [Minimum](ui_tutorial_headquarters_minimum.png) |
| Long headquarters name | [Normal](ui_tutorial_long_name.png) | [Minimum](ui_tutorial_long_name_minimum.png) |
| Army roster | [Normal](ui_tutorial_roster.png) | [Minimum](ui_tutorial_roster_minimum.png) |
| Movement group | [Normal](ui_tutorial_group.png) | [Minimum](ui_tutorial_group_minimum.png) |
| Blocked destination | [Normal](ui_tutorial_blocked.png) | [Minimum](ui_tutorial_blocked_minimum.png) |
| Route and cost review | [Normal](ui_tutorial_route.png) | [Minimum](ui_tutorial_route_minimum.png) |
| Regional return | [Normal](ui_tutorial_region.png) | [Minimum](ui_tutorial_region_minimum.png) |
| People | [Normal](ui_tutorial_people.png) | [Minimum](ui_tutorial_people_minimum.png) |
| Career | [Normal](ui_tutorial_career.png) | [Minimum](ui_tutorial_career_minimum.png) |
| Households | [Normal](ui_tutorial_households.png) | [Minimum](ui_tutorial_households_minimum.png) |
| Household requirements | [Normal](ui_tutorial_review.png) | [Minimum](ui_tutorial_review_minimum.png) |
| Rival turn | [Normal](ui_tutorial_turn.png) | [Minimum](ui_tutorial_turn_minimum.png) |
| Resume through Help | [Normal](ui_tutorial_help.png) | [Minimum](ui_tutorial_help_minimum.png) |
| Completion and restart | [Normal](ui_tutorial_complete.png) | [Minimum](ui_tutorial_complete_minimum.png) |

The map/roster stays dominant. Prompts fit the header band at both sizes, including
the long-name and blocked states; costs, missing requirements and action controls
remain readable. Close and Show HQ provide 48-pixel-high pointer targets. The
regional breadcrumb is hidden while the guide occupies that header, while World
Map remains visible.

In the published WebGL game, visible pointer controls verified a new campaign,
Close, Help > Resume, Show HQ, headquarters roster, movement group, a blocked
origin selection and a successful adjacent movement through Review Route and
Confirm Move. The lesson advanced only after confirmation. Dragging within the
guide did not pan the map or select an underlying marker. The final publication
was reloaded successfully to its title screen.

Minimum-size native captures pass. The in-app browser viewport override still
renders a smaller upper-left game after interaction despite a reported
1280 × 720 canvas; the override was reset. This does not establish minimum-size
WebGL acceptance. Physical-touch hardware was unavailable, so browser checks
establish pointer behavior only.

## Final validation and unresolved failures

All checks used this checkout on `master` and its real shared workspace.
`cargo fmt --check`, strict all-target Clippy through the shared Cargo wrapper,
the source-size gate and `git diff --check` pass. The final no-argument
`publish.ps1` builds Windows and WebGL and deploys successfully to Preview.

The complete shared-wrapper run, `cargo.ps1 test --all-targets --no-fail-fast`,
finishes with two failing targets; all other targets pass. The release profiling
test remains explicitly ignored in the ordinary run.

1. `campaign_scenarios::four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds`
   fails at `tests/support/campaign_scenarios.rs:118`: the unattended four-faction
   campaign (seed 180404) ends at round 268, before the required round 400. Its
   round-200 save/replay checkpoint passed. Allowing previously blocked rival
   battles changes the long-run scenario, but the precise terminal cause was
   not diagnosed. The continuity assertion remains unchanged; this run does not
   validate the round-400 milestone or the later eight-faction leg.
2. `k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`
   fails at `tests/k18_production_battle.rs:465`: the already documented seed-88
   scenario produces no production victory by round 120. Its deadline assertion
   remains unchanged.

These results leave long-run continuity/balance validation unresolved; they are
not reported as a passing full suite.
