# K18 review follow-up

This records fixes to the [implementation review](../implementation-review.md).
Release acceptance remains open until the platform and integrated checks are
complete; implementation fixes do not stand in for physical touch evidence.

## Private feedback and service dates — R01 / R05

The application now calls `engine::action_notices` for accepted outcomes. That
single boundary checks ownership for people, commanders, items and anniversary
subjects before resolving their names. A past enemy encounter does not grant
access to current private family events. Loading or drawing a view never calls
the notice producer.

Person transfers set the service date only when leaving Dependent or Trainee
status. Adult site apprentices already entered service when invited; later
site/formation transfers retain the original date, wound chronology, evidence
and spent movement.

Validation: 17 focused cases passed across `continuity_feedback`, `movement`,
`succession` and `code_standards`. The new regressions exercise the application's
actual notice function with player/NPC invitations and NPC retirement, and
repeat wounded transfers through serialized reloads for all three family
origins. Strict all-target/all-feature Clippy passed. Formatting passed.

The no-argument publisher passed Windows and WebGL release builds, Preview
deployment and Project Roost bookkeeping for this fix group.

## Rival threats and career continuity — R02 / R03 / R04 / R10

Threat objectives now score reachable adjacent staging places plus the final
Clear Threat edge. Distant armies can approach a threat observed by another
friendly army; ordinary travel still cannot enter or cross an unresolved threat.

The planner evaluates local adult courses, deploys trained adults and completed
command apprentices to an appropriate local formation, and uses the shared
transfer command. Command pupils can therefore earn the encounter prerequisite
for an Officer course. Governors, retirees, underage learners and unfinished
apprentices remain in local roles. Established classes are preserved; a serving
commander may make the justified upgrade to Officer. Adult people born into a
household are no longer incorrectly required by validation to remain Recruits.

Known rivalry breaks ties only after distance and strategic priority. It uses
the observer's retained encounter location and current public hostile-presence
cue. It never follows a private person's live location or rewrites their dated
encounter.

The former large progression/continuity routines now delegate to individual
training, deployment, household, estate and successor decisions. All functions
in `src/engine/ai/` and the new regression suite are within 100 physical lines.

Validation: 37 cases passed across AI, progression, mentorship, succession and
source-size suites. Five new review cases cover adjacent clearing/reload,
distant approaches and blocked/unknown threats, stable classes across repeated
seasons, real lessons followed by selected training/transfers and command
replacement, an actual subsequent threat encounter, and rivalry ordering.
The apprenticeship case separately exercises Infantry and Command teachers.
Fixture funding and intentionally stationary seasons isolate career choices;
these are not a substitute for ordinary multi-seed balance acceptance.
Strict all-target/all-feature Clippy and formatting passed.

The no-argument Windows/WebGL Preview publication passed for this AI group.

## Local training and peaceful familiarity — R06 / R07

Troop Training focus discounts the local course total by 25%, rounded upward.
Person and formation previews, command checks, AI affordability and screen
prices use the same calculation. Every new course stores its actual payment.
Unstarted cancellation, retirement, death and loss of the training site refund
that receipt; started courses refund zero. Focus changes and reloads cannot
reprice a receipt. Old saves without receipts migrate using the frozen original
class/specialization prices in `legacy_course_prices.json`; no past discount
is invented. Courses still advance once at an eligible seasonal boundary.

At each boundary, fit adults in active site or formation service at the same
owned, unbesieged physical site gain one shared-service season. This includes
stationary garrisons, governors and active adult household members. Dependents,
trainees, underage people, displaced/wounded people and retirees do not qualify.
Different internal sites do not share this credit. The stamp names the season
just completed, so movement, battle and stationary contact cannot count that
season more than once. Symmetric relationship caps still apply; familiarity
never grants battle evidence.

The career screen shows local prices before the longer prerequisite summary.
Course and specialization rendering is divided into identity/status/options
helpers. The current decision remains which local course to begin or cancel;
prices and progress sit beside those controls, with full biography deferred to
history. Normal and minimum layouts retain the existing two-column class list.

Validation: 56 focused cases passed across `training_focus` (6),
`stationary_familiarity` (5), AI review (5), progression (9), lifecycle (6),
mentorship (8), succession (9), persistence (7) and the source-size gate (1).
The new cases exercise actual charges, exact affordability, upward rounding,
legacy receipt migration, reload/focus changes, started cancellation, death,
retirement and captured-site cleanup. Four real stationary turns followed by
Form Household pass across repeated reloads; exclusions, movement deduplication
and bounded symmetric relationships also pass. Existing battle tests cover
battle/stationary deduplication.

Strict all-target/all-feature Clippy and formatting pass. The six replacement
career, active-course and specialization captures were inspected at actual
1920x1080 fullscreen and 1280x720; local prices remain visible and status text
stays clear of Cancel. The shared hidden capture processes exited. Native
captures verify composition; browser and physical-touch interaction remain in
the final platform pass. The no-argument Windows/WebGL Preview publication
passed on the final code, including Project Roost bookkeeping.

## Dated lives in shared history — R08

Accepted actions now record emergence, birth/adoption/local invitation,
recognition with its supported deed, class completion, mentorship start and
completion, household formation/end and its cause, adult service entry,
retirement and natural death. Combat deaths remain in their witnessed battle
receipt. Each life record has one stable ID, date, place, immutable participant
labels and only its owning faction as observer. Person, place and associated
army histories read that same record. Both present and released army membership
are retained for a departure. Recognition, emergence and class completion also
produce concise own-person notices through the R01 application boundary.

Old saves gain no invented past milestones. Loading and viewing never run the
transition recorder or notice producer. The existing detail/notable/departed
budgets remain in force; current class, progression and family state do not
depend on keeping narrative detail. Life-event notables validate their owner
visibility even after their detailed event expires.

The biography keeps its existing Events view: dated milestones are the main
content, with person/place labels beneath them and paging/filter controls below.
The new `history_life` capture derives an apprentice's five milestones through
real service, teaching, course and retirement commands. Its actual 1920x1080
and 1280x720 renders were inspected: five rows and navigation remain readable,
with no overlapping labels or controls. The shared hidden capture processes
both exited. Browser and physical-touch navigation are covered in the final
platform pass.

Validation: 64 focused cases pass across life history (7), progression (9),
legacy (10), knowledge (5), private feedback/service dates (2), lifecycle (6),
mentorship (8), succession (9), wounds (6), integrated Medic service (1), and
source size (1). The emergence regression now checks its real emitted date,
place, army and owner. New history regressions cover complete local lessons,
class feedback, retirement, recognition exactly once, rejected foreign access,
immutable viewing/reload, natural death, four-season familiarity/households,
adoption, a real spring birth, chosen household end, a ward's 36-season path to
adult service, and pruning without replay. Strict Clippy and formatting pass.
No-argument Windows/WebGL Preview publishing and Project Roost recording pass.

## Household and succession reviews (R11; affected R13 functions)

Every family action now opens a visible review, including unavailable actions.
The review gives full selected names and ages, recorded familiarity or the
selected legacy category/link, and the authoritative command rejection.
Confirmation rechecks current state. Back retains the list selections. Eligibility
comes from one engine query using the same phase and semantic validators as the
command; the duplicated UI eligibility predicates have been removed.

Five new regression cases compare review reasons with real command errors and
assert that queries/rejections preserve state. They cover missing/foreign
selections, off-turn orders, four real stationary seasons, age, active partners,
guardian capacity, child training/service entry, deficit, exact cost, annual
invitation limits, missing successor links and duplicate categories. All 27
focused cases pass (succession 14, life history 7, familiarity 5, source size 1).
Strict Clippy and formatting pass. A brace-aware source scan confirms each
household UI/review/query function is at most 100 physical lines.

Twelve replacement native captures cover households, succession, blocked and
ready reviews, successor review, and a 13-person list with long names at actual
1920x1080 and 1280x720. Names truncate only in list rows and appear in full in
review. Text and 48-pixel controls remain separate at both sizes. Capture
processes exited; pointer and physical-touch acceptance remain in the platform
pass. Evidence: `ui_household_blocked[_minimum].png`,
`ui_household_ready[_minimum].png`, `ui_succession_review[_minimum].png`,
`ui_household_dense[_minimum].png`, `ui_households[_minimum].png`, and
`ui_succession[_minimum].png`.

No-argument Windows/WebGL Preview publishing and Project Roost recording pass.

## Atlas and current teaching (R09/R12)

Reviewed all 80 world placements and 119 external route traces against the atlas.
Lakeward's former offshore graph now follows the mainland coast; the southern
Rosevale shore and Eastfold lake placement are corrected too. Thirteen simple
sites and the Lakeward region marker change position. Ten routes use authored
land waypoints around bays/lakes; nineteen routes show narrow bridge crossings.
The normal road and selected route share the same geometry. IDs, endpoints,
costs, regional entrances, HQ candidates and all 191 simulation edges remain
unchanged. Internal region coordinates remain in their separate map space.

`world.layout_revision` separates geography from rules/content version. New
production campaigns save revision 2 and their authored route geometry. Missing
revision means 1; the frozen previous marker positions validate old saves, whose
sites are never moved on load. Unknown revisions, mixed coordinates and altered
route geometry are rejected. Five regression cases cover current and old saves,
unchanged topology/costs, exact replay after reload, and invalid crossing data.
The topology, entrance, HQ-spacing, navigation, persistence, content and source
suites pass: 32 cases in total.

The production page now describes seeded 4–8-faction setup, careers, households,
succession, history and the training discount. Help teaches named regions from
the active map. Chapter 10 describes implemented screens and retains historical
verification limits. A fresh render exposed overlapping Help paragraphs; Help
now flows by wrapped height across 17 short topic pages with visible Previous,
Next and Back controls. Capture topic indices follow those pages.

Normal/minimum world, western coast, central river, introductory Help, careers
Help and lifecycle Help captures were reviewed at actual 1920x1080 and 1280x720.
The replacement coast removes ocean routes, shows supported crossings and keeps
dense markers distinct; zoom enlarges these groups. The central river is crossed
at visible bridge symbols. Help text clears its navigation controls. Evidence:
`ui_production_world[_minimum].png`, `ui_atlas_coast[_minimum].png`,
`ui_atlas_river[_minimum].png`, `ui_help[_minimum].png`,
`ui_help_service[_minimum].png`, `ui_help_lifecycle[_minimum].png`.

Formatting, strict Clippy, Windows release and WebGL release passed. The no-argument publisher deployed this revision to Preview successfully.

## Engineering contract (R13)

The four modules named by R13 now separate course controls, household reviews,
continuity decisions and training execution. A brace-aware physical-line audit
of those responsibilities found the following maximum function lengths:

| Module | Maximum function lines |
| --- | ---: |
| `engine/progression/commands.rs` | 87 |
| `engine/ai/progression.rs` | 53 |
| `engine/ai/progression/continuity.rs` | 48 |
| `engine/ai/progression/training.rs` | 77 |
| `ui/army/progression.rs` | 64 |
| `ui/army/progression/formation.rs` | 92 |
| `ui/army/progression/person.rs` | 88 |
| `ui/army/households.rs` | 71 |
| `ui/army/households/review.rs` | 80 |

The behavioral checks and captures for those extractions are recorded above.
The complete Rust file-size gate passed in the latest suite. This is a function
audit of the review's named responsibilities, not a claim that every older
function elsewhere was restructured.

## Final application routing correction

An ordinary eight-faction WebGL campaign exposed an application guard omission:
Armies → People → Households → Invite Apprentice showed an eligible review,
but Confirm returned “Close the open panel and return to the campaign first.”
The same guard also blocked career, mentorship and lifecycle commands issued
from the Armies panel.

`GameState::command` now admits those commands from their owning panel, including
formation courses and succession. The shared engine still validates ownership,
phase, costs and eligibility; unrelated overlays still block orders and an open
panel still blocks End Turn/NPC advancement. No simulation rules were changed.
The user ended testing before this final correction, so no new regression or
browser rerun is claimed. Formatting was applied and the final no-argument
publication result is recorded in [K18 closeout](k18-integrated.md).

## Testing scope change

On 2026-09-28 the user waived the physical-touch requirement and asked to end
further testing because the game may change substantially before balance review.
References above to a future platform pass describe the plan at the time of each
fix. That pass is now deferred; existing evidence retains its original scope.
