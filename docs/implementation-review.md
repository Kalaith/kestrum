# Full implementation review — 2026-09-28

[Design index](README.md) · [Plan](implementation-plan.md) ·
[Acceptance ledger](implementation/acceptance.md) ·
[K18 evidence](verification/k18-integrated.md)

## Conclusion

Reviewed `96385e4` on `master`, against K01–K18, contracts C01–C07,
provisional rules P01–P23 and the recorded I01–I20 implementation decisions.
The campaign has substantial working simulation and integration coverage, but
it does **not yet satisfy the whole implementation plan**. In addition to the
known K18 platform and performance checks, the review found missing behavior
in AI, training, continuity, notifications and history, plus presentation gaps.

The earlier K01–K17 delivery commits remain valid historical records. Their
Done labels must not be read as evidence that the findings below are resolved.
Close these follow-ups through K18, or record an explicit accepted amendment,
before declaring full release acceptance. No gameplay fixes or rule changes
were made by this review.

## Review method and limits

- Compared every package's outputs and acceptance cases with its rule packets,
  implementation entry points, tests and existing verification record. Traced
  commands through validation, application, round resolution and player feedback.
- Reviewed the current observer boundary, save catalogue/migrations, production
  generation, AI policy, careers, family/service chronology and history producers.
- Inspected existing native minimum captures for production world, households and
  dense army management. These are existing evidence, not fresh rendered captures.
- Opened the deployed browser title and inspected the host page. At a fresh
  1280×720 page viewport, the DOM reported a 1200×675 canvas at
  `(32.5, 103.0625)`, ending at y=778.0625. The page also still advertises a
  Rosemarch campaign. No new campaign or player save was created for this review.
- Findings R01–R08, R10 and R13 are source-traced defects or omissions, not newly
  executed reproductions. Each specifies the regression evidence still needed.
  R09 and R11 also use existing screenshots; R12 uses the live host page and
  presentation data. Passing existing tests does not cover an absent assertion.
- This is a full plan-coverage review, not a claim to have manually played every
  branch or audited every source line. Physical touch/pinch, exact minimum WebGL
  canvas gameplay and browser frame timings remain unverified.

## Package coverage

“Present” means the implementation and relevant regression suites were located;
it does not close the follow-ups or outstanding platform acceptance.

| Package / rules | Implementation and evidence reviewed | Assessment / follow-up |
| --- | --- | --- |
| K01 — typed content; C05/P01/P06 | `data.rs`, data validators, JSON scenario/economy, `tests/content.rs` | Present: toolkit loading, reference/policy checks and Rosemarch topology. |
| K02 — ownership/rounds; C01–C04/P02 | `state/campaign.rs`, `engine/actions.rs`, `engine/round.rs`, strategic campaign tests | Present: candidate-state commits, persisted RNG/order and one boundary. Feedback visibility needs R01. |
| K03 — saves; C06 | `state/persistence/`, `game/saves.rs`, shared indexed-store integration, `tests/persistence.rs` | Present: distinct identities, journal failure tests, explicit overwrite/delete, invalid-load preservation and retry. Keep browser acceptance below open. |
| K04 — geography; P03/C07 | `state/world/`, movement paths, navigation/world tests | Present: physical gates, anchors, partial control and camera tests. Production atlas alignment needs R09. |
| K05 — armies/economy; P05/P06/P11 | recruitment, economy, military state, economy tests | Present: six slots, setup grants, local access, exact upkeep and removal. |
| K06 — movement/recovery; P04/P05 | movement, transfer, recovery, movement/recovery tests | Present: costed routes, spent allowances, pause/off-turn transfers and recovery. Personal chronology is not preserved by every transfer: R05. |
| K07 — combat; P11–P13/P15 | combat, retreat, person outcomes, combat/wound tests | Present: deterministic simultaneous exchanges, retreat, persistent consequences and report replay. I20's wounded-assignment change is recorded; no additional combat change is proposed here. |
| K08 — evidence/knowledge; P10/P16/P23 | evidence, knowledge, history, evidence/knowledge tests | Present: participation, veterancy, report snapshots and bounded narrative. Application notices bypass that boundary: R01. |
| K09 — construction; P06/P07 | construction commands/progress, construction/data/history tests | Present: costs, pause/refunds, builders, settlers and fixed routes. Training focus remains unfinished after later packages: R06. |
| K10 — sieges; P14 | siege arrival/commands/projection, siege/integrity/history tests | Present: establishment, weakening, assault, escape/sortie, relief and save continuity. Browser side-by-side interaction checks remain open. |
| K11 — places/threats; P15/P19 | development/migration, threats, development/threat tests | Present: layered condition, growth, refugees, roles and one-time threats. R06 affects focus; R02 prevents normal AI threat selection. |
| K12 — AI/diplomacy/endings; P08–P10 | AI planners, diplomacy, AI/diplomacy tests and production ending tests | Present: legal commands, peace/defeat choices and durable endings. Policy execution gaps: R02–R04; knowledge gap: R01. |
| K13 — careers; P17/P18 | progression, class/specialization options, progression tests | Present: emergence, evidence gates, courses and conversions. R04/R06/R08/R10 cover remaining policy, cost, feedback and rivalry work. |
| K14 — lifecycle/mentorship; P20/P21 | lifecycle, mentorship, lifecycle/mentorship tests | Present: birthdays, wounds, retirement, governors and lessons. NPC local students need a complete progression path: R03. |
| K15 — households/succession; P22 | succession/family commands, relationship validation, succession tests | Present: optional households, dependents, apprentices and category designations. R03/R05/R07/R11 affect practical continuity. |
| K16 — legacy/history; P23 | legacy custody, history retention/eras, legacy tests | Present: single item custody, estate fallback, filters, pruning and reminders. Generational chronology/announcements remain incomplete: R08. |
| K17 — production world; P01/C07 | `world_layout.json`, generation/setup, generation tests, K17 captures | Present: 80 markers, 152 sites, 191 routes and seeded 4–8 starts. Atlas placement and production-facing text need R09/R12. |
| K18 — integrated acceptance | campaign scenarios, production battle/ending, isolation/siege/people/place suites, K18 verification | In progress. Existing end-to-end proofs are useful; R01–R13 and A01–A04 below remain follow-ups. |

## Required implementation changes

Priority P1 means a core advertised behavior is bypassed or leaks information;
P2 means a required behavior is missing or incorrect; P3 is lower-impact plan or
presentation completion. IDs are local to this review, not new design decisions.

### R01 — P1: Filter people and succession notifications by observer

**Contract:** C07/P10; K08/K15: no hidden births, careers, deaths or family facts
through notifications.

**Evidence:** [`game/campaign.rs`](../src/game/campaign.rs),
`handle_campaign_result` lines 94–153, resolves every `outcome.new_people` and
`outcome.succession` ID against the authoritative campaign and builds a player
toast. Neither branch filters ownership. NPC Invite Apprentice returns an ID
through [`succession/commands.rs`](../src/engine/succession/commands.rs), annual
births return all created IDs through `succession/family.rs`, and
[`succession::notices`](../src/engine/succession.rs) scans all armies. These results
reach the same handler from automatic NPC processing. Item and anniversary
notifications already have explicit player filters in `engine/actions.rs`.

**Impact/change:** an unseen rival's new child/apprentice name, commander departure,
successor and army name can be announced to the player. Produce observer-safe
notifications at a single boundary; keep enemy knowledge dated to actual contact.

**Acceptance:** NPC birth, apprentice invitation, retirement/death and replacement
produce no private player toast, even for a previously encountered rival. Own
events still announce once, including after checkpoint/reload. Cover application
notification generation, not only `engine::project`.

### R02 — P1: Make AI threat selection reach the Clear Threat command

**Contract:** P09/K12 explicitly require AI to clear nearby known threats; P01
places two early threats beside every production headquarters.

**Evidence:** [`ai/military.rs`](../src/engine/ai/military.rs), `clear_threat`
(line 142), feeds threat sites into `targets`. In
[`ai/travel.rs`](../src/engine/ai/travel.rs), `targets` requires a path to the
destination (line 171), but `transit` immediately rejects any visible threat
site (line 57), including the destination. An army cannot legally already stand
on the unresolved threat. Thus ordinary initial selection never reaches
`clear_threat_at`, despite that helper containing the legal command.

**Impact/change:** rivals leave their authored early threats unresolved, losing
rewards, encounters and expansion options. Score a legal adjacent staging site
and the final Clear Threat edge; keep ordinary movement through threats blocked.
Also allow approaching a more distant threat without treating it as normal entry.

**Acceptance:** with higher-priority needs satisfied, an NPC selects and clears
an adjacent known threat, approaches a reachable distant threat, avoids unknown
or unreachable threats, and pays/grants effects once across save/reload.

### R03 — P2: Let NPC local learners train and enter field service

**Contract:** K14/K15/P21/P22 require rivals to train replacements and maintain
non-blood continuity through the same legal actions as the player.

**Evidence:** [`ai/progression.rs`](../src/engine/ai/progression.rs) skips every
non-formation person before considering class courses (lines 32–35). Its
`EnterService` intent always passes `formation: None` (line 301). Invited
apprentices start at a site, and the planner never proposes `TransferPerson`;
[`ai/intent.rs`](../src/engine/ai/intent.rs) has no corresponding intent mapping.
`age_trainees` correctly returns an adult trainee to Dependent, so aging itself
is not the problem.

**Impact/change:** NPC apprentices/children can learn at home but cannot complete
the planner's course-to-army-to-command path. Evaluate eligible site courses and
assign a fit adult to a real co-located formation when a field role is needed.
Keep governors, retirees and underage trainees out of inappropriate deployments.

**Acceptance:** an NPC invites an unrelated apprentice, supplies real lessons and
encounter evidence, completes a useful class, transfers locally and appoints a
qualified replacement after the founder leaves. Assert actual choices, not merely
that a manually supplied NPC command passes the shared validator.

### R04 — P2: Prevent AI career oscillation and loss of needed roles

**Contract:** K13/K14/P09/P21 call for useful training and replacement capabilities.

**Evidence:** [`career_options`](../src/engine/progression/career.rs) lists
Infantry, Archer, Scout, Cavalry, Medic and Officer in that order. Eligibility
is evidence-based and does not exclude the current class. The planner tries
every eligible entry in list order; the command validator rejects only the same
class or an active course. An Officer with infantry evidence can therefore be
sent to Infantry; with two qualifying classes, completion makes the previous
class eligible to buy again. There is no desired-role or benefit check.

**Impact/change:** valid commands can repeatedly consume Gold and erase the
commander's Officer bonus. Select a needed role, preserve useful current roles,
and reconsider completed training only when circumstances justify a switch.
Do not forbid intentional player class changes.

**Acceptance:** an Officer with ordinary infantry evidence retains command class;
a dual-qualified person does not alternate courses across repeated seasons; a
missing capability still triggers a justified course using legal prerequisites.

### R05 — P2: Preserve the original service date during later transfers

**Contract:** C02/P16/P22/P23 preserve personal service, evidence and chronology
when a person moves between formations.

**Evidence:** [`transfer.rs`](../src/engine/transfer.rs), lines 158–160, resets
`service_start_round` whenever `families.contains_key(person)`, regardless of the
source assignment. This includes an already-serving born/adopted/local apprentice
moving between two formations, not only first entry into adult service.

**Impact/change:** a veteran's service biography and anniversary clock restart.
For a still-wounded person, moving the start date past `since_round` also violates
[`validate_person_status`](../src/state/people/validation.rs) and rejects an
otherwise legal transfer. Set service start at initial service entry only; keep
it on subsequent formation/site changes.

**Acceptance:** later transfers preserve the service date, personal evidence,
anniversary state and movement spent for each family origin. Include a wound
from an earlier round and repeated save/reload transfers.

### R06 — P2: Implement Troop Training focus's course discount

**Contract:** P19 requires a 25% local course Gold discount, rounded up at placement,
without double progress; K11 must complete actual focus effects.

**Evidence:** `Focus::TroopTraining` is selectable, but no engine consumer applies
it. [`progression/commands.rs`](../src/engine/progression/commands.rs) validates,
charges and refunds the base course/specialization prices (lines 89, 172, 217,
254, 265 and 287). `career.rs` also reports the base price. Presentation still
says no course is available (`focus_effect_training`).

**Change:** put the discount in typed JSON rules and one cost calculation shared
by player preview, AI, validation and application. Store the amount actually
paid so cancellation, capture/death cleanup and later focus changes cannot refund
a different price. Define migration for existing course records.

**Acceptance:** a 20-Gold ordinary course costs 15 at the focused site; unaffected
sites retain base cost; odd prices use the specified ceiling; insufficient-Gold
checks use the displayed price; unstarted cancellation refunds exactly what was
paid, and started work refunds zero. Progress remains one step per eligible round.

### R07 — P2: Count genuine stationary shared service and household familiarity

**Contract:** P17/P22 allow actual shared-service/household seasons to establish
the four-season familiarity needed for a partnership.

**Evidence:** [`progression/relationships.rs`](../src/engine/progression/relationships.rs)
only calls `update_shared` for movement receipts and faction battle participants.
There is no seasonal producer for people serving together at a site, stationary
garrison or household. Partnership validation and NPC pairing nevertheless rely
on `shared_service_seasons`. Capture fixtures seed relationship counts directly.

**Impact/change:** adults serving together peacefully can remain strangers forever;
repeated travel becomes the workaround. Credit real qualifying co-located service
once per season. Specify which site roles qualify, and preserve the distinction
between familiarity and combat evidence.

**Acceptance:** four qualifying stationary seasons enable a non-kin partnership;
different internal sites, pre-service/underage cases and absent people do not
receive that credit; movement plus battle plus stationary contact in one season
cannot count the same season repeatedly. Verify a household through real commands.

### R08 — P2: Complete generational milestone history and feedback

**Contract:** P17 explicitly requires recognition to be announced once; K16/P23
require contextual biographies/chronicles and factual continuity across generations.

**Evidence:** [`HistoryKind`](../src/state/history.rs) contains battle, military,
construction, place/diplomacy, custody and anniversary records, but no emergence,
recognition, class completion, mentorship, household, service-entry, retirement
or natural-death event kinds. `progression/emergence.rs::update_tracked` and course
completion mutate current fields without emitting such records. Emergence also
does not return IDs to the application's `new_people` notice. Existing career
fields, completed mentorship records and death status do preserve some facts;
they do not supply the missing dated event stream or recognition announcement.

**Change:** record the significant implemented life transitions once, with date,
place, supported cause and observer visibility, and reuse them in person/place/
army views. Add concise own-person emergence/recognition/class feedback. Keep the
existing retention budgets; do not implement every illustrative story in chapter 09.

**Acceptance:** real emergence → recognition → training → mentorship → departure
is traceable through retained events; each relevant view reads the same record;
viewing/loading never repeats effects or notices; pruning preserves current
capabilities; enemy milestones remain private under R01/P10.

### R09 — P2: Align the production land graph with its atlas

**Contract:** P01/K17 require reviewed geographic placement and prohibit open-water
connections without an authored bridge/passable coastal route; naval travel is later scope.

**Evidence:** the existing [minimum production capture](verification/ui_production_world_minimum.png)
puts Lakeward Hollow (site 40, normalized 0.035/0.674) and Lakeward Field
(site 43, 0.048/0.757) over western water. Route 99 joins them as an ordinary
cost-2 edge; route 103 continues east from Field. The
[`authored layout`](../assets/data/world_layout.json) repeats these coordinates in
marker and physical-site records. Count/connectivity tests cannot establish that
a land route follows the artwork.

**Change:** review all 80 placements and external routes against the actual atlas.
Move misplaced land sites/routes or explicitly author and depict supported
crossings. Preserve stable identity and make any saved-layout/content compatibility
decision explicit; do not silently change an existing campaign's graph.

**Acceptance:** normal/minimum world captures and selected route traces show
believable supported land connections, including the western coast and central
river. Re-run topology, entrance, HQ-spacing and save compatibility checks.

### R10 — P3: Implement the specified known-rival tie break

**Contract:** P17 says repeated rivals influence AI objective tie breaks when
strategic scores otherwise tie, without extra damage or hidden intelligence.

**Evidence:** relationships record mutual combats, but
[`ai/travel.rs::targets`](../src/engine/ai/travel.rs) sorts only by path cost,
strategic priority and site ID. The AI planners never consume mutual-combat
relationship facts for target selection.

**Change/acceptance:** use only retained, legitimately observed rival information
after strategic scores tie and before the final stable ID tie break. A higher
priority objective must still win; unknown or moved enemies must not disclose
their live location. Record an explicit rule amendment if this behavior is removed.

### R11 — P2: Explain disabled household and succession actions

**Contract:** C07, package definition of done, UI_STYLE §5: missing requirements
must be visible through tap controls.

**Evidence:** [`ui/army/households.rs`](../src/ui/army/households.rs) duplicates
eligibility into booleans and disables Form Household, Adopt Ward, Invite Apprentice
and Designate Successor without rendering their specific failing requirement.
`designation_eligible` returns only `bool`. The existing
[household capture](verification/ui_households_minimum.png) illustrates the muted
controls; its general help does not tell the player which constraint failed.

**Change/acceptance:** expose observer-safe eligibility reasons from shared queries
and show the selected action's missing familiarity, age, funds, yearly limit,
local role or existing designation. Verify each reason at both sizes with taps,
including a populated list and a long name. Keep the map/selected decision dominant.

### R12 — P3: Replace obsolete prototype teaching and page copy

**Contract:** C07/K17 require accurate ordinary content and contextual Help.

**Evidence:** [`game_page.json`](../game_page.json) still says “Begin a Rosemarch
campaign”; the live host repeats it. `game_config.json` says focus has no current
bonus (`focus_success`, `help_construction_refund`), says no course is available
(`focus_effect_training`), and instructs users to select Rosemarch
(`help_region`), which is absent from the normal production region names.

**Change/acceptance:** describe the production setup and existing career/legacy
controls, correct focus/training text after R06, and teach region navigation with
the actual selected region or a generic visible label. Check Help from a fresh
production campaign and the published page. Update the chapter 10 prototype-era
screen notes without rewriting historical verification claims. This review corrects
the stale current-plan status paragraph and adds links; presentation assets remain
follow-up work.

### R13 — P2: Bring oversized functions within the engineering contract

**Contract:** C01 and CODE_STANDARDS §4.1 set a 100-line maximum per function,
separately from the 800-line source-file limit.

**Evidence:** [`ui/army/progression.rs`](../src/ui/army/progression.rs) starts
`person` at line 17 and the next function at line 350. In
[`ai/progression.rs`](../src/engine/ai/progression.rs), `continuity` spans lines
131–373. [`ui/army/households.rs`](../src/ui/army/households.rs) and
[`progression/commands.rs`](../src/engine/progression/commands.rs) also contain
well-over-100-line eligibility/drawing/dispatch functions. The existing source
gate checks file length, so its passing result does not establish this rule.

**Change/acceptance:** extract cohesive responsibilities such as course controls,
household actions and individual continuity decisions while fixing these systems.
Share eligibility queries where appropriate (R11), preserve command order and
presentation, and check function lengths as well as file lengths. Avoid unrelated
restructuring or compressing formatting. Run behavioral and visual checks for the
affected responsibilities.

## Remaining release acceptance

| ID | Required work | Evidence needed to close |
| --- | --- | --- |
| A01 — minimum WebGL and host | Resolve or explicitly accept the embedded canvas sizing/occlusion issue in the shared host, then review an actual 1280×720 WebGL canvas. | Record canvas dimensions separately from page dimensions; exercise selection/picking after Continue, resize, zoom, display scaling and fullscreen entry/exit. Review dense stacks, routes/transfers, battle/siege sides, history, failures and ending screens. The fresh title measurement above does not close campaign acceptance. |
| A02 — touch | Perform the full supported-size touch-only flow, including setup/name entry, pan/pinch, drag-release suppression, regional navigation, off-turn transfer, Back/Close, long lists, Help, fullscreen and save recovery. | Device/browser/version, actual canvas size and interaction results. Physical pinch requires hardware; mouse use and existing captures cannot establish it. |
| A03 — performance | Measure native and browser release gameplay frame/input latency, individual NPC work, boundary work, save/load and catalogue growth. Profile before selecting a fix; distribute bounded work across frames if necessary without changing deterministic order. | Existing isolated baseline: 8-faction phase p95 277.5 ms versus provisional 250 ms; individual action p95 41.1 ms, max 263.8 ms. One action per frame alone does not prove 60 Hz. Record percentiles, hardware/build/browser, representative late state and steady-state memory; capture-process peaks are not gameplay memory. Reassess after AI fixes. |
| A04 — integrated play and balance | Revisit production campaigns after R01–R13 and close the full acceptance scenarios through real controls. | The seed-88 victory proves one legal conquest path at 60 rounds/15 years. The defeat test deliberately disbands the player army before invasion. The 200/400-round runs pass the player's turn and test continuity/replay. These are complementary tests, not proof of the typical 20–50-year experience, balanced AI resistance, or all generational decisions in an ordinary campaign. Record multi-seed 4/8-faction play and the review-sensitive combat, emergence, growth and aging outcomes. |

The plan's limited vassal defeat outcome, ordinary human scope and bounded memory
are deliberate. Advanced/magical classes, other races, tribute/alliances,
restoration, civil war, multiplayer, naval travel, detailed disease/starvation and
portrait canvases below 1280×720 remain later scope. This review does not turn
those into release blockers.

## Suggested delivery order

1. Close the notification leak and personal chronology regression (R01/R05), with
   focused regressions through the real application/command boundaries.
2. Repair AI threat, student and career policy (R02–R04), then its small rivalry
   omission (R10); replay ordinary production campaigns before judging balance.
3. Finish training focus and peaceful familiarity (R06/R07), including persisted
   costs/dates and interrupted/reloaded cases.
4. Add factual life-history feedback and actionable UI reasons (R08/R11).
5. Correct atlas alignment and teaching/page copy (R09/R12), then complete
   A01–A04. Each meaningful game change requires normal checks and no-argument
   publishing from this checkout, with its own reviewable commit. Apply R13's
   cohesive function splits alongside the affected fixes.

## Validation of this review

This section records fresh checks separately from the historical K18 baseline.

- `cargo fmt -p kestrum -- --check` passed on the actual checkout.
- Source inventory: 290 Rust files, none over 800 physical lines; largest is
  `src/ui/army/households.rs` at 768. The existing test gate has no exceptions.
- `..\rust_management\cargo.ps1 test -p kestrum --locked --all-targets
  --all-features` passed, including the 200/400-round scenarios and production
  ending regressions. These runs validate existing assertions, not the new
  acceptance cases proposed above.
- `..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets
  --all-features '--' -D warnings` passed.
- Documentation checks passed: 211 local links/anchors across seven changed
  documents, coverage of all 18 packages, 13 findings and four acceptance areas.
  `git diff --check` passed.
- No new publication or capture batch was needed for documentation-only changes.
  Prior Windows/WebGL Preview publishing remains historical evidence in K18.
