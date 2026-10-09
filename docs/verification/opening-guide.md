# Opening guide — 2026-10-08

The first-100-turn review found that the guide taught screens rather than a
strategic loop: it never mentioned claiming land, the bandits beside the
capital, a second army or rival aggression, and it reached careers and
households before the first End Turn.

## Lesson order

| Step | Lesson | Completed by |
| --- | --- | --- |
| 1–4 | Capital, city investment, region, Country Map | unchanged |
| 5 | Movement; the prompt now says moving into an unclaimed place claims it | an accepted move |
| 6 | Clear Threat: bandits or wildlife near the capital | an accepted Clear Threat order, or automatically once no threat is visible |
| 7 | Raise Army: a second force so one can claim while one guards | recruiting with New Army (filling an existing army does not count) |
| 8 | End Turn; rivals attack weaker neighbors, so keep armies strong | End Turn |
| 9–11 | Career, Household, Records | unchanged, now after rivals have acted |

Show Capital is offered for the Raise Army step. Older saves keep their
completed steps; a partly completed guide resumes at the first missing lesson.

## Validation

`tests/tutorial.rs` pins the new order and checks that Clear Threat and New
Army record their lessons while filling an existing army does not. The hidden
native capture walk-through (Xvfb, 1920×1080, 30 frames) ran every changed
scene through its assertions and was inspected:
[route / threat prompt](ui_tutorial_route.png),
[raise army](ui_tutorial_raise_army.png),
[first turn](ui_tutorial_first_turn.png) (new scene),
[rival turn](ui_tutorial_turn.png), [people](ui_tutorial_people.png),
[career](ui_tutorial_career.png), [households](ui_tutorial_households.png),
[review](ui_tutorial_review.png) and [complete](ui_tutorial_complete.png).
The capture records the Clear Threat receipt directly; the expedition itself is
covered by the test. Minimum-size, browser and physical-touch checks were not
run.
