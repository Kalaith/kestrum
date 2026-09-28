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
