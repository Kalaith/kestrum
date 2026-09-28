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
