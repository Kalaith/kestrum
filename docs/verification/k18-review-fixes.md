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
