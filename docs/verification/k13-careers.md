# K13 — Grounded emergence, traits and ordinary careers

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `7dc61c3` (K12). Package work was implemented in this checkout.

## Gameplay and save behavior

Surviving formations with meaningful current-season service can produce one
evidence-grounded person per faction and round. The candidate order, roster-sensitive
soft chance and people RNG follow P17. Emergence assigns a stable ID and human name,
retrospective service bounded by creation/history/age, a source formation and a
current-event deed when supported. The person joins existing headcount. Traits and
one place-based recognition require their factual evidence; prior service and traits
remain personal through transfers.

Person evidence unlocks Recruit-to-ordinary-class training, while riding practice
earns riding evidence without inventing battle participation. Training is local,
priced, two eligible seasonal steps, and can pause, resume, or cancel under the
documented refund rule. Fit attached adults can be appointed as commanders. Shield
Guard, Pikemen and Light Cavalry convert eligible existing formations without
changing formation identity, capacity or earned service. NPC planning uses the same
career and specialization option queries and the same validated commands.

K12 saves that lack all person career records receive deterministic defaults from
the saved people RNG in PersonId order. Mixed old/new career records are rejected.
Current saves round-trip without reconstructing progression. K13's choices and
migration details are recorded as delegated implementation decisions in
[I12](../13-decisions-and-open-questions.md#i12--grounded-careers-and-specialization).

## Behavioral checks

The new `tests/progression.rs` has nine cases:

- Soft emergence remains possible with twenty named adults and repeats by seed.
- Genuine deeds award the supported traits and one factual recognition.
- All six ordinary class routes expose eligible and missing-evidence states.
- An NPC selects and pays for a legal class from the same evidence options and
  command validation as a player.
- A wound pauses training; completed progress resumes, and cancellation refunds
  only before progress.
- Rival labels require two actual mutual combats.
- Specialization preserves formation identity, service and capacity.
- Light Cavalry's nine-point allowance survives combat validation and movement
  exhaustion rules.
- K12 save migration creates repeatable K13 defaults without changing the other
  random streams.

All **136 tests** pass through the actual workspace launcher, including the 800-line
source-size gate with no exceptions. `cargo fmt -p kestrum -- --check` and strict
all-target/all-feature Clippy with `-D warnings` pass. A legacy knowledge fixture
now removes the K13 career group when modeling a pre-K13 payload; its relationship
links are removed when the fixture itself deletes a person.

## Interface review

The Career screen presents evidence, cost, local facility, short missing-reason
labels, course progress/cancellation, riding practice and commander appointment.
Formation Training presents specialization prerequisites, effects, conversion
availability and course progress. Available-training, in-progress and formation
states were reviewed at 1920×1080 and 1280×720. All career, riding, appointment,
conversion, cancellation and Back targets are at least 48 logical pixels. The
shared capture wrapper ran with its hidden-window default; both capture processes
exited normally.

| Screen | 1920×1080 | 1280×720 |
| --- | --- | --- |
| Available careers | [Capture](ui_career.png) | [Capture](ui_career_minimum.png) |
| Active course | [Capture](ui_career_training.png) | [Capture](ui_career_training_minimum.png) |
| Formation specialization | [Capture](ui_specialization.png) | [Capture](ui_specialization_minimum.png) |

The review found no clipping or overlapping actions at either native size. The
screens use the normal UI pointer/tap hit regions, but K13 did not include a
browser interaction run or physical-device touch test. Physical touch remains
unverified; the known minimum-WebGL campaign scaling defect remains an explicit
K18 check. Native captures do not substitute for those checks.

## Publication

No-argument `publish.ps1` passed from this checkout. Windows and WebGL release
builds and packages completed, the Windows/WebGL Preview deployment and catalogue
refresh succeeded, and Project Roost recorded `rust_kestrum` in one tracker.

K14–K18 remain required for integrated lifecycle, household, history, production
world and platform acceptance.
