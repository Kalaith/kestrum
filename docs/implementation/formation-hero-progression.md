# Formation recruits earn Hero status

Status: active implementation, 2026-10-08. Stages A-D are committed. Stage E's
threshold tuning, replacement lifecycle scenario and production replay are
recorded in [verification](../verification/formation-hero-progression.md). The
earned path works, but the replay has Hero-bearing members in only 6 of 13 armies
at round 120, so the user's midgame majority target remains open. Full-project
validation also retains five failing targets listed in that report. This plan's
source investigation used master commit 5ed5429; implementation decisions follow
the current checkout.

## 1. Outcome and scope

The user's clarified goal is:

> An army has a warrior unit; a hero apprentice comes up through that unit and
> reaches Hero rank after a few engagements.

Deliver the visible progression:

**Warriors -> Apprentice Name + Warriors -> Hero Name + Warriors.**

The named person is one of the formation's existing troops. They keep their ID,
portrait identity, birth date, history and assignment as they progress. This path must work for
human and AI kingdoms, including Observer mode, without a settlement invitation,
a local teacher, a purchased class course or an AI collection expedition.

The reported symptom concerns army portraits and commanders, not just the Log
feed. The previous investigation into settlement apprentices explains a separate
AI weakness; it is not the implementation direction for this task. Leave that
recruitment/collection work outside this slice.

Retain the existing distinction between profession, Hero recognition and command
appointment. A Recruit can earn Hero status without becoming an Officer. Becoming
a Hero does not automatically take command from a living eligible commander.

## 2. Evidence and limits

Read-only investigation used:

- Observer log: C:/Users/Kalai/AppData/Local/kestrum/observer_logs/observer-260926-1791439477020.jsonl.
- Seed 260926, four factions, 3,585 observer steps, rounds 0 through 198
  (49.5 elapsed years), 74 resolved battles.
- The log lists 31 apprentice invitations and 13 births. Only two of those 44
  recorded arrivals entered formations and one became a commander. None of the
  31 invited apprentices was transferred into a formation.
- Five additional person IDs appear in commander actions without a corresponding
  arrival entry. This is consistent with the unlogged emergence path; the old
  audit cannot establish their exact origin, names, ages or recognition status.

Do not treat 44 arrivals as the total number of people created, or conclude that
no battle-emerged heroes existed. The old log serializes ActionOutcome.new_people,
which currently covers succession arrivals/births but omits emergence. It also
omits full person snapshots and serializes life changes only as history IDs in
debug text. Old logs therefore cannot prove the formation-to-Hero conversion rate.

Initial implementation gaps (addressed in Stages B-D):

1. emergence::advance chooses only the highest-service surviving formation per
   faction per season and rolls a roster-dependent chance. Its base probability
   is 15 percent before modifiers; unrelated living, non-retired named adults at
   settlements also reduce it. This is not a dependable service progression.
2. Candidate selection does not require an empty named-person slot. create uses
   available_person_formation, which can choose a different local army/formation
   and can fall back to a site assignment. Source service and actual placement
   can therefore diverge.
3. Recognition requires three meaningful encounters plus a particular special
   deed. create can grant it immediately from retrospective formation evidence;
   it does not require three later engagements fought by the tracked apprentice.
4. A named unrecognized person has a portrait but no explicit Apprentice stage
   or simple personal progress toward Hero in the current presentation.

These describe the starting checkout, not the current contract. The open gap is
campaign pacing: too few active armies receive a later qualifying battle after
their Apprentice appears, as measured in Stage E.

## 3. Planned behavior

The exact numbers below are **recommended tuning defaults selected by this plan**,
not numbers previously specified by the user. Implement them as validated JSON
data. Adjust them only with relevant replay evidence and record the change.

| Rule | Initial implementation default |
| --- | --- |
| Formation produces a named apprentice | Two qualifying engagements while its named-person slot is vacant |
| Apprentice becomes a Hero | Three further qualifying engagements personally participated in after emergence |
| Candidate scope | Each eligible surviving formation, in stable formation-ID order |
| Named capacity | Preserve one named person per formation and six formation slots per army |
| Timing | Resolve through accepted seasonal evidence consumption; events appear at that boundary |
| Randomness | Keep seeded identity/disposition generation; no random failure after earning a threshold |
| Roster pressure | Settlement people and other formations' members do not block this formation's earned progression |

These were starting values, not a user-specified cadence. Stage E records any
replay-driven tuning and its effect below; the final JSON thresholds are the
implemented contract.

This is a deliberate planned replacement for the old random faction-wide emergence
curve and the mandatory special-deed recognition gate. The old 20-30-person soft
roster target was a tuning direction, not a hard capacity rule. Formation capacity,
actual combat opportunities, casualties and age limit this new path. Report its
observed roster growth; do not silently retain a hidden gate that defeats earned
progression or introduce an unrelated hard cap to force the old target.

### Qualifying service

Reuse the existing meaningful-participation classifier: substantial opposing
strength, actual casualty pressure or a qualifying strategic defense/capture.
Keep its established duplicate protection per formation/person, place, opponent
and round. Do not count idle seasons, travel, recovery, uncontested moves,
training purchases, previews, repeated inspection or replaying the same receipt.
Two distinct qualifying encounters in one season may count if existing credit
rules regard them as distinct. XP caps must not accidentally become encounter caps.

Credit only the formation and people actually present in the accepted battle
report. Use personal participation/fitness records, including wound and death
outcomes, rather than everyone assigned to the army at the later boundary.
A destroyed formation cannot produce a recruit. A dead or retired person cannot
be promoted to Hero by delayed evidence consumption.

### Formation vacancy and emergence

- Accumulate compact progress for the current vacant named-person slot. Keep
  formation veterancy and lifetime history separate from replacement progress.
- When the threshold is earned, create one age-valid Recruit directly in the
  source formation. Do not call the generic local-slot allocator for this path.
- Recheck surviving troops and vacancy before insertion. Never evict an existing
  named member, add troop headcount, add another slot or park this recruit at a site.
- An occupied formation must not monopolize candidate selection or suppress a
  different vacant formation that has earned its recruit.
- A staffed formation does not bank replacement credit. Retirement, death,
  departure or transfer-out opens a fresh opportunity requiring new service.
  Recruitment/disband/reinforcement/transfer/army merge must not duplicate or
  refresh earned progress incorrectly. Moving the same formation to another army
  preserves its identity and valid progress; destroying it discards that progress.
- Decide vacancy at the time of credited service, not only from end-of-season
  state. Example: a commander fights twice then retires at the boundary; those
  already-staffed encounters must not instantly create their replacement.
- Preserve truthful age-compatible retrospective background as biography/career
  evidence where already supported. Keep it separate from the apprentice's new
  personal Hero-progress counter, which starts at zero.

### Personal Hero progression

- The emergence-triggering encounter and earlier unit history cannot satisfy the
  one later personal engagement. The whole batch consumed before
  creation is retrospective; the new person did not appear in those battle reports.
- Store compact durable personal progress. A transfer preserves earned progress,
  but does not import the receiving formation's old history. Save/load, pruning
  narrative history and UI inspection cannot add, reset or duplicate credit.
- Award recognition once at the first qualifying subsequent encounter. Keep
  portrait identity, profession and host troop kind unchanged.
- Special deeds still support traits and specific epithets. They are not an extra
  mandatory gate for basic Hero rank. Add a truthful general battle-service
  recognition reason if no specific deed fits. Never fabricate an assumed command,
  captured anchor, healing action or outnumbered survival just to fit an enum.
- Review existing Recognition/EpithetFact/history/notification schemas and every
  match arm affected by the general recognition reason. Do not hide the new rule
  in display-only code or repurpose an unrelated existing deed.
- Existing already-recognized figures and founding Lords keep their identity and
  titles. Other unrecognized tracked people may earn the same recognition through
  their own subsequent field service; settlement invitations alone confer none.

### Aging and command succession

Use existing retirement/death cleanup and command eligibility rules. A newly
emerged person is already present in the army and can become a legal local
replacement. Ensure the AI actually considers eligible attached people when a
command is vacant, without requiring a settlement invitation or relocation.
Do not invent automatic Officer qualifications, teleport successors or displace
an eligible current commander. A peaceful army with no new engagements need not
produce a battle-earned Hero merely because years pass.

### Persistence and atomicity

Persist bounded formation-vacancy and personal progress in the authoritative
campaign state, with semantic validation. Keep creation, RNG allocation, progress
consumption and emitted receipts inside accepted command transactions. A failed
preview/action cannot consume a recruit opportunity or change identity rolls.

New fields need an explicit load policy. Existing recognized people keep their
awards. Do not infer new personal Hero credit from an old person's retrospective
formation counters, or bank a veteran formation's entire past for an immediate
replacement. Safe zero-progress defaults are acceptable if the resulting save is
valid and the limitation is documented. If an older save cannot be supported
cleanly, follow the prototype policy: show a clear recoverable error. Do not build
migration machinery solely for this feature or overwrite the user's catalogue.

## 4. Source map

Paths are relative to D:/WebHatchery/RustGames/kestrum.

| Responsibility | Starting files |
| --- | --- |
| Service classification and duplicate credit | src/engine/evidence.rs; src/engine/evidence/participation.rs; src/state/evidence.rs |
| Emergence, retrospective evidence and recognition | src/engine/progression/emergence.rs; src/engine/progression.rs |
| Rules and validation | assets/data/progression.json; src/data/progression.rs |
| Person progress, chronology and invariants | src/state/people/career.rs; src/state/people/validation.rs; src/state/military/roster.rs |
| Seasonal sequencing and accepted outcomes | src/engine/actions.rs; src/engine/round.rs |
| Retirement, casualties and command replacement | src/engine/lifecycle.rs; src/engine/person_combat.rs; src/engine/succession.rs; src/engine/ai/progression.rs |
| Typed life history and notices | src/state/history/life.rs; src/engine/history/life.rs; src/engine/notifications/collect/life.rs |
| Observer audit and headless replay | src/game/observer_log.rs; examples/observe_campaign.rs |
| Portrait/name and formation presentation | src/ui/components.rs; src/ui/army/formation_slots.rs; src/ui/army/people.rs; src/ui/army/progression/person.rs |
| Player-facing data | assets/data/game_text.json; assets/data/notifications.json; assets/data/human_names.json |
| Existing regression seams | tests/progression.rs; tests/evidence.rs; tests/formation_people.rs; tests/life_history.rs; tests/observer.rs; tests/ai_review.rs |
| Existing visual fixtures | src/game/founder_capture.rs; src/game/history_capture/life.rs; scripts/capture_ui.ps1 |

Inspect file lengths before editing. Every Rust file has an 800-total-line hard
limit; extract cohesive modules where needed, with no new mod.rs files. Reuse
existing JSON loading, seeded identity/portrait allocation and UI components.
No new shared toolkit capability or external name service is needed for this fix.

## 5. Commit-sized delivery sequence

Read AGENTS.md, PROJECT_AGENTS.md and CODE_STANDARDS.md first. Read UI_STYLE.md and
relevant MACROQUAD_TOOLKIT.md sections before presentation work. Declare owned
files/checks per slice, preserve other work, validate frozen sources, review full
and staged diffs, then commit each coherent slice on master before the next.
Follow the project's required Luna delegation for implementation/validation.

### A. Make the existing path observable and establish a baseline

Extend the existing replay and observer audit to distinguish:

- Newly emerged people, invited apprentices and births.
- Formation service opportunities, eligibility, current member and why emergence
  did not occur; in the old baseline include roster count, chance and selected source.
- Each emerged person's actual source formation, assigned formation/army, age,
  personal qualifying participation and recognition date/reason.
- Living attached apprentices, attached Heroes, retirees, deaths and unstaffed
  formations/leaderless armies, per faction at useful checkpoints.

Record typed life events or equivalent structured facts, rather than debug-text
parsing alone. Discover new people from accepted before/after state or a complete
creation receipt; do not rely only on the presently incomplete new_people list.
Use one canonical event source to prevent duplicate arrivals/recognitions.
Include enough diagnostics at boundaries/events to explain progress without
serializing an ever-growing history on every action. Update the audit format
version if its contract changes. Preserve ordinary human-observer secrecy.

Run seed 260926 with four factions to round 200 before changing simulation rules.
Summarize rounds 40, 80, 120, 160 and 200. This is a fresh deterministic baseline
for the actual checkout, not an exact reconstruction of an unknown earlier binary.
Commit useful observability and its focused verification independently.

### B. Make emergence belong to the source formation

Add validated data thresholds and compact vacancy-service state, then replace
faction-wide probabilistic selection with the rules in section 3. Keep the existing
identity allocator, lawful age/service dates and single-member invariants. Add
regressions for earned appearance, an occupied high-XP unit not starving another
unit, correct source assignment, fresh replacement service and unchanged headcount.
Document the changed cadence as planned above. Validate and commit this slice.

### C. Make Hero recognition follow personal engagements

Add durable post-tracking progress; keep retrospective evidence separate. Remove
immediate recognition at emergence, grant Hero exactly once after the configured
personal threshold, and support a truthful general service reason. Cover personal
participation, transfers, duplicate consumption, death/retirement and save/load.
Update life events/notifications and relevant design rules together. Validate
and commit before beginning presentation work.

### D. Show the apprentice, progress and earned Hero status

Use existing Army Details, person detail and event surfaces. Record the brief in
the relevant README/GDD section before changing them:

| Screen question | Required answer |
| --- | --- |
| Current decision | Identify who is serving with a formation and whether they are becoming a replacement leader |
| Dominant focus | The selected army's existing six formation rows, with its troop type and named member |
| Primary action | Tap the formation/person to inspect; existing army orders continue play |
| Supporting information | Same portrait, Apprentice/Hero title, fitness, troop kind and personal progress |
| Deferred information | Qualifying encounter dates/places and special deeds in person/history detail |
| Layout and camera | Existing 1920x1080 layout; map remains dominant during map play and its camera is retained |
| Input and feedback | Visible tap controls; one arrival event and one later recognition event with inspectable history |

Show Apprentice Name + Warriors after emergence and Hero Name + Warriors after
recognition. Apprentice here is the battle-progression stage, not a requirement
for an active mentorship assignment. Keep the Recruit profession separate.
A compact detail such as 'Hero progress: 0 / 1 engagements' must count personal
qualifying service and explain the criterion through visible detail/help.
Preserve founding Lord precedence and established portrait aging/identity.

Correct event wording: emergence introduces an apprentice; Hero recognition is
a separate later event. Inspect NewHero, PersonRecognized and immediate-merging
logic so a recruit is not already announced as a completed Hero. Verify the
Observer Log includes both transitions for every faction. Map commander portraits
must reflect actual command appointments, not all formation members.

Use real accepted progression in at least one capture fixture and interaction
sequence. Existing formation_emerged/formation_members fixtures seed state directly
and therefore establish layout only unless updated to run the real transitions.
Verify long names, six occupied rows, a vacant command and an aging replacement.
Validate and commit this presentation slice.

### E. Verify generations and close the handoff

Run the integrated replay with the final build and report the changes against A.
For every threshold-reaching eligible formation/person, reconcile the expected
transition against the recorded outcome; explain every missing transition. Report
aggregate birth/invitation counts separately from formation emergence.

Add one real-command lifecycle scenario: troop-only formation -> apprentice ->
one later qualifying engagement -> Hero -> legal vacant-command appointment;
then retirement/death opens a slot, and fresh formation service produces a new
identity. Reload during the sequence and compare deterministic continuation.
A scripted fixture proves the rule; a production replay proves opportunities and
pacing. Do not substitute one for the other or claim a Hero should appear during
forty completely peaceful years.

Use seed 260926/four factions through round 200 as primary evidence. Use a second
fixed seed or an eight-faction replay only to address remaining roster-growth or
candidate-starvation risk; record exact arguments and stop conditions. Do not
chase an arbitrary total Hero count without checking actual qualifying exposure.
For the user's midgame pacing goal, require more than half of the campaign's
active armies to contain at least one recognized Hero at round 120. Count
Apprentices separately; do not treat them as Heroes. The current 6/13 result
does not pass, so keep the handoff open and investigate post-emergence exposure
or distribution without crediting service a person did not witness.

Update docs/06-character-development.md, docs/08-generations-and-succession.md,
docs/implementation/people-and-places.md and the relevant decision register/README
sections to describe the implemented contract. Preserve historical verification
records. Add the final evidence to docs/verification/formation-hero-progression.md
and its index. Remove completed work from todo.md; retain genuine open checks.

## 6. Acceptance and test plan

Group related inputs into focused behavioral cases; extend existing useful
coverage instead of deleting it or seeding counters as the only proof.

1. **Emergence and placement:** real accepted qualifying service produces one
   apprentice in its own troop formation at the threshold; occupied candidates,
   other armies and settlement population do not interfere. Headcount and six-slot
   capacity stay unchanged; identity and age/service dates are valid.
2. **Earned recognition:** zero personal progress at emergence, then one Hero
   event after the first later personal engagement; keep portrait/ID/host unit,
   with no compulsory special deed or inherited commander credit.
3. **Evidence boundaries:** trivial/duplicate/nonparticipating service gives no
   progress; meaningful personal participation does; same-season batching,
   preview, inspection, history pruning, transfer and reload cannot double-credit.
4. **Replacement:** death/retirement/transfer frees a slot without old-service
   instant respawn; later real service creates a different young person; formation
   destruction eliminates its opportunity; existing legal AI command succession works.
5. **Presentation and audit:** Apprentice and Hero appear in their formation with
   consistent progress/history, one event each, complete Observer records and no
   leakage into normal fog-limited enemy views. Verify dense and long-name states.

Relevant existing targets include progression, evidence, formation_people,
lifecycle, life_history, succession, ai_review, observer, observer_visibility,
notifications, portraits and persistence. Run affected targets at coherent
checkpoints, not this entire list blindly before every change. Expand to the full
project suite at the state/simulation integration boundary because persistent
progress and seasonal processing have broad impact.

Commands below run from the real Kestrum checkout. Use the wrapper for build,
test, Clippy and replay; do not copy the project or bypass the shared pool:

~~~powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 test -p kestrum --locked --test code_standards
..\rust_management\cargo.ps1 test -p kestrum --locked --test hero_progression --test notifications --test progression --test evidence --test formation_people
..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 run -p kestrum --locked --release --example observe_campaign '--' 260926 200 4
..\rust_management\cargo.ps1 test -p kestrum --locked
..\rust_management\cargo.ps1 build -p kestrum --locked --release
..\rust_management\cargo.ps1 build -p kestrum --locked --release --target wasm32-unknown-unknown
~~~

The focused command is a starting group, not a substitute for related changed
coverage. The full suite is an integration check, not a per-edit loop. The existing
seed-88 round-240 production-victory failure is an authorized baseline exception,
never a pass. Diagnose any new failure; do not weaken tests to fit the new output.

For UI work, use scripts/capture_ui.ps1 and its hidden default at actual 1920x1080.
Existing scenes include formation_emerged, formation_members, formation_long_name
and founder_army; inspect the current harness and add a targeted earned-Hero scene
if required. Capture directly to stable files in docs/verification/, use -SkipBuild
only with the verified matching output, inspect the images and verify game exit.
Build WASM once and reuse it for relevant headless browser taps/inspection. Never
invoke fullscreen or seize the user's desktop. Keep screenshots and scripted
interactions distinct from human playtesting.

Do not run publish.ps1, push or deploy: PROJECT_AGENTS.md authorizes local pooled
builds and headless verification instead of external publication.

## 7. Tooling note — earlier blocker resolved

The planning session observed PowerShell and `cargo.ps1` stalls before any game
edits or replay. The implementation session rechecked the normal shell: pooled
tests, strict Clippy, the production replay and the hidden capture wrapper all
ran through their established tools. The earlier malformed `%SystemDrive%` cache
directory was verified removed. This historical blocker does not prevent the
remaining pacing work; do not bypass the shared pool or clear caches.

## 8. Completion report required from the implementation agent

Provide the slice commit hashes, final thresholds and any justified tuning,
replay seed/rounds/faction count, emergence-to-Hero/replacement evidence, screenshots
and interactions actually verified, full/focused check outcomes and honest remaining
limits. Finish with git status --short and preserve any unrelated changes.
The plan is complete only when the battle-earned path works and is visible and
the round-120 majority pacing target passes; extra settlement invitations or an
improved log alone do not satisfy it.
