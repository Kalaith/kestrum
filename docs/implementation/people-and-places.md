# Provisional people, places and continuity rules

[Plan](../implementation-plan.md) · [Work packages](work-packages.md)

**P16–P23 are provisional implementation defaults**, not claims that the founding
drafts specified these thresholds. Sources: chapters
[06](../06-character-development.md), [07](../07-living-places.md),
[08](../08-generations-and-succession.md), [09](../09-history-and-content.md).
O10's actual-participation requirement and O16's preserved veterancy always apply.

## P16 — Participation, service and formation veterancy

The resolver emits facts for actual participating formation/person IDs: battle,
victory/defeat, survived_outnumbered, commander_wounded, assumed_command,
treated_wounded, encountered_troop_type, defended_anchor, captured_anchor,
retreated, and later valid training/mentorship/non-combat service. A report or
remote ally earns none of these. “Outnumbered” uses starting base power: enemy at
least 150% of own side. Treatment requires a participating surviving Medic person
or Medics formation and actual friendly casualties; at a supplied recovery site
it requires an assigned Medic and real recovery of headcount. Merely waiting in
an empty infirmary grants no treatment evidence.

A battle is meaningful for progression if own starting power faces at least 50%
of its strength, or the participant loses at least 10% headcount, or it captures or
successfully defends a strategic anchor. Credit one encounter per formation per
site/opposing faction per round. Also cap formation service XP at 4 per round:
2 for a meaningful battle, +1 victory, +1 survival while outnumbered. Ordinary
formation -> Seasoned at 8 total XP -> Veteran at 20. Recovery gives no XP and
never changes these earned tiers. Destruction discards XP and specialization.

Store compact per-person/formation counters and encounter-type facts separately
from narratives. Store per-formation seasonal participation summaries for the
latest eight rounds for retrospective emergence. They contain service-compatible
tags, date and site IDs, not detailed eternal battle logs. Idempotent consumption
uses the C03 queue/sequence and per-round eligibility state; no unbounded set of
every event ID is needed. After transfer, a person retains personal evidence;
joining a veteran formation does not inherit its old deeds.

## P17 — Emergence, disposition, traits and recognition

After meaningful participation, choose at most one emergence candidate per faction
per round: surviving formation with highest current-round service XP, then
highest veterancy, then lowest ID. In the people RNG, draw once against permille
chance `floor(150 * 64 / (64 + N*N))`, multiplied by 1250/1000 for Seasoned or
1500/1000 for Veteran, then by 2 if the formation both survived outnumbered and
captured/held an anchor; clamp at 500. N counts all living non-retired adult named
people of that faction, including recovering/site-assigned people. Dead people,
children and retirees do not count. This is a soft curve, never a 20/30-person cap.

An emerging person gets a new stable ID, human name from an authored reusable pool,
age 18–30 from the people RNG, Recruit class initially, and attachment to the
source formation. They are a tracked member of its existing headcount, not +1
troop. Appointing or transferring them does not change headcount either; named
support is abstracted separately after tracking. Their service start is the latest
of formation creation, eight rounds ago, and their seventeenth birthday. Only
formation participation on/after that date can ground their retrospective facts.
Use “served with…” for inferred service; never generate unsupported individual
kills, kinship or a detailed heroic act from aggregate troop data. Emergence's
distinguishing deed uses the qualifying current event.

Disposition has three hidden tendencies, each sampled once from -1/0/+1 at first
tracking: courage, care and curiosity. They adjust evidence thresholds slightly,
not the combat formula or access to nonexistent experiences. Do not expose them
as precise recruitment ratings. Traits require distinct meaningful occasions:
Bold from survived_outnumbered, Protective from defended_anchor or treated_wounded,
and Natural Commander from assumed_command or commanded_victory. Threshold is
`3 - relevant_tendency`, clamped 2–4; Bold uses courage, Protective uses care, and
Natural Commander uses curiosity. Pupil/command
facts come from actual assignments. Trait effect initially is eligibility/context,
not an unlisted army multiplier. Retain an earned trait permanently; supporting
stories may expire while compact evidence remains. No evidence decay initially.

Formal recognition requires three meaningful encounters plus at least one
assumed_command, survived_outnumbered, defended_anchor or treated_wounded fact.
Apply once; announce supported deeds and permit a data-authored epithet based on
one known site/event. A personal name, recognition, class and command appointment
are distinct. Existing named adults already contribute under P11; recognition
does not retroactively add phantom formation members or force a class change.

Record shared-service seasons and repeated opposing encounters as compact pair
facts only for tracked people actually present. After two mutual combats, an
observed opponent can be described as a recurring rival. Initially bonds/rivalry
affect biographies and AI objective tie breaks only (prefer a known rival when
all strategic scores tie), not extra damage or guaranteed revenge. Limit retained
pair facts to currently relevant living people and P23 memory budgets.

## P18 — Ordinary training and formation specialization

Eligibility is a conjunction of actual evidence, age/fitness, site access and
resources. UI lists missing requirements separately. No generic XP substitution
for a missing enemy type, no invented dragons, no advanced or magical classes.
Ordinary class courses require age 17+; younger site trainees can accumulate
mentorship evidence without field deployment or completing an adult class.
Training is an explicit action for a selected person at an owned supplied,
unbesieged site. It costs 20 Gold for ordinary Infantry/Archer/Scout/Medic,
40 for Cavalry, 50 for Officer; consumes two eligible seasonal steps. It pauses
while moving, cut off, wounded or missing its facility, resumes when eligible,
and cancels on death or owner loss with no refund after progress (P07 policy).
No simultaneous class courses. Previous evidence/traits survive switching;
only one current class provides capabilities. Lost infrastructure does not remove
an already earned class.

| Class | Required evidence | Local opportunity |
| --- | --- | --- |
| Recruit | Entry/emergence; no purchased promotion required | Any valid service entry |
| Infantry | 2 meaningful battles in Warriors/Spearmen or 2 seasons infantry mentorship | Training ground |
| Archer | 2 meaningful battles in Archers or 2 seasons archery mentorship | Training ground |
| Scout | Traverse 6 distinct physical routes during service | Training ground |
| Cavalry | 2 seasons riding practice/mentorship and participation in at least 1 battle with Riders | Stable, horse access |
| Medic | 2 treatment occasions under a qualified Medic or Medics formation | Infirmary |
| Officer | 3 meaningful encounters and 1 assumed_command/commanded_victory, or 2 seasons command mentorship plus 1 meaningful encounter | Training ground |

Riding practice is an explicit seasonal site assignment at a functional supplied
Stable with horse access; it supplies riding evidence only, never battle evidence.
Production founders are full lords from the beginning, with a generated name,
a seeded age of 18–24, and Officer as their starting profession. Their title and
modest command bonus do not require fabricated completed battles or recognition.
The Rosemarch prototype retains its age-24 Officer setup. One fit
adult attached person can be appointed commander per army; any such person may
command, while the Officer class supplies the extra P11 bonus. Commanded victory
requires that appointment before a real battle. Reassigning after a report earns
nothing. Command is not restricted to recognized heroes.

Formation specializations preserve identity, veterancy and capacity. Require a
supplied Training ground and explicit conversion for 30 Gold over two eligible
steps; one specialization per formation. Whole-formation conversion pauses/cancels
as training does. Unlock Shield Guard from Warriors after three defended_anchor
occasions; +100 permille resistance on defense only. Unlock Pikemen from Warriors
or Spearmen after three meaningful encounters against Riders; counter versus
Riders becomes 1750 (replaces 1500/1000). Unlock Light Cavalry from Riders after
six distinct traversed routes and two victories with a defeated retreating enemy;
movement allowance becomes 9. These are ordinary formation types, not new races
or named-character classes. Put the additions and references into the data schema.

## P19 — Development, occupation, refugees and capitals

Use independent habitation and fortification enums; civil identity is a tag set;
occupation and structural/fort damage are independent 0–100 values. Capital/HQ
are roles. Structural damage, geography, fortifications, income tier, population,
and political title must not overwrite each other. Unsettled and ruined are
distinct states; a ruin remembers its last habitation tier only while needed for
reclamation/current state, not as a permanent biography.

Initial populations: Camp 20, Outpost 50, Hamlet 100, Village 200, Town 500,
City 1000, Major City 1800. These are simulation units of untracked civilians, not individual entities.
Tracked household dependents sit outside that abstract pool: adoption/apprentice
entry transfers one unit from the pool into a tracked person; a newly born tracked
child adds that person once without another population-unit grant. Emergent
soldiers come from existing military headcount, not a second civilian deduction.
Normal aggregate growth/migration operates only on the abstract pool. This is a
light demographic model, not an exact census of all soldiers and civilians.
Outpost establishment draws up to 50 migrants from the nearest supplied friendly
inhabited site that can retain at least its tier's minimum, preferring path cost
then ID. If none is available, completion pauses with “No settlers available.”
Never create settlers by reopening completion. Initial HQs start at 250 so early
expansion is possible. Bandit-ruin reclamation uses the same order/migration rule.

Each round uses a snapshot of local conditions. Safe means no hostile presence
on-site/adjacent, no battle there this round, no local threat. Connected means
supplied. Trade means an intact improved route to a friendly inhabited neighbor.
Food means plains/river/valley geography and no structural damage >= 50.
Habitation caps: plains/river/valley/coast Major City; forest/hill Town;
marsh/pass/island Village. Terrain never changes merely because a tier rises.

Development pressure per round: +2 safe, +1 connected, +1 trade, +1 food,
+1 capital, +2 Growth focus; -3 battle this round, -2 cut off, -2 damage >= 50,
-2 occupation >= 50. Clamp each round's contribution to -6..+6. Accumulate signed
pressure bounded -24..24. At +24, upgrade at most one habitation tier if population
meets the next tier minimum and the geography cap permits; consume 24 pressure.
At -16, downgrade at most one tier and consume 16 negative pressure. Upgrading
does not manufacture population. If a positive transition is blocked, retain at
most 24 and show the blocking condition. Unsettled needs an outpost order; it
does not passively become inhabited. Fort layer is unchanged by tier transitions.

Safe supplied inhabited sites gain `max(1, floor(population/100))` population per
round up to 1.5 times their terrain's maximum-tier minimum. Damaged >= 75 or
occupied >= 50 sites displace `floor(population/20)` instead of natural growth.
Route displaced groups to the nearest safe friendly inhabited site via friendly
uncontested paths, up to three edges, then ID; destination room is capped by that
same population capacity. Apply departures and arrivals simultaneously. Unplaced
people remain at origin as displaced pressure; they do not disappear or duplicate.
Explicit Resettle moves up to 50 such people to a legal reachable destination for
10 Gold, observing capacity and conservation. No per-person refugee roster.

If a damaged >= 90 site falls below population 20 for four consecutive rounds,
it becomes Ruined with population unchanged, no habitation income/recruitment and
no functional facilities. Reclamation after clearing its threat uses a three-step
outpost order and 50-settler rule, resets structural damage to 50 and sets Outpost,
preserving site ID and extant fort damage. This neither restores old population
nor wipes current event references.

Occupation declines by 10 each safe round with a friendly army or functional
fort, otherwise 5 when safe, and never declines in a battle round. It blocks
passive growth while >= 50 through pressure; it does not create rebel factions.
Structural damage repairs 5 per safe supplied round. Fortification focus repairs
an additional 5 fort damage; default fort repair is 2 under those conditions.
Troop Training focus grants one training/mentorship progress step as normal but
does not double progress; its separate benefit is reducing local course Gold cost
by 25%, rounded up at order placement. Gold/Wood/Stone focus adds 25% to that site's
matching base income (Wood needs forest tag, Stone needs hill/quarry tag); use
disabled reasons for unsuitable focus. Growth is the pressure benefit above.
Fortification focus has no free military-tier upgrade.

Income site modifier is `(100 - floor(structural_damage/2))%`, then 50% while
occupation >= 50, then the matching focus multiplier; round down once after all
factors. Ruined/besieged sites earn zero. Record these P06 inputs once per boundary.

Rename is a 1–40 character nonempty label action and preserves all IDs. Move Capital
costs 100 Gold, destination owned Town+, safe/supplied, occupation < 25; changes
only the political role and future pressure. HQ remains independent. Relocate HQ
costs 80 Gold/40 Wood/20 Stone, to an owned unbesieged Village+ with a training
ground and a friendly army. It is immediate, once per four rounds; destination
need not be supplied by the old lost HQ. It becomes the single root and loses the
old HQ bonus/root. This explicit recovery path prevents a lost HQ from making
every later recruitment/recovery action permanently impossible. AI uses relocation
when its HQ is lost and a legal alternative exists.

## P20 — Age, injury, career change and death

Birth/service dates share the seasonal calendar. Dependents are sparse until a
real role is available. Field service begins at 17; ages 13–16 may be site trainees
only. Aging gives no combat XP. Initial humans: 17–40 normal field fitness; 41–55
use at most 8 personal movement while serving with Light Cavalry but
retain their class; 56+ become elder officers/mentors and contribute only when
appointed commander, not by stacking elder passengers. At age 56 show a career
choice; keep the current command until reassigned rather than silently deleting
it. No age-based penalty to a formation's abstract population.

Limited combat wounds require two eligible supplied, nonbesieged recovery steps
and block field contribution/training until healed. A person can remain attached
while recovering or be assigned to a local owned settlement. Healing is free in
this initial person model, has no headcount effect, and grants a Medic evidence
only if that Medic was actually assigned and present. Do not add permanent random
injury classes or disease simulation in this milestone.

Retire is an explicit action to an owned site. Retired people cannot command a
field army or count toward emergence N but can mentor or govern. A governor is
one fit/retired living adult assigned to one owned settlement and contributes +1
development pressure there; no extra unnamed global bonus. At age 70, automatic
field retirement occurs with a visible notice; transfer to the nearest owned
unbesieged inhabited site by graph distance, as an administrative career move
between rounds (not a military retreat or teleporting army). If none exists,
remain a retired displaced person at current site without strategic powers.

Old-age mortality is checked only on birthdays: under 60 none, 60–69 2%, 70–79
5%, 80–89 15%, 90+ 35%. Draw once per eligible birthday in stable person-ID order.
Death releases assignment, mentorship and commands exactly once; item/successor
handling follows P22. Existing illness/captivity/major-disaster death sources are
deferred rather than being silently implemented as extra periodic rolls.

## P21 — Mentorship and useful non-blood continuity

An alive age-26+ qualified person (including a retiree) can mentor one learner at
a time. A learner has one mentor. Qualification is the taught ordinary class or
relevant earned trait and at least four seasons of service in that discipline.
Both must share a physical site at round end: same army qualifies, nearby armies
in different internal sites do not. Youth trainees may learn at a settlement;
combat deployment still requires 17. Wounds, moving apart, death, capture of their
site or lost required facility pause/end the relationship with a reason.

Every eligible boundary grants one season of discipline evidence, capped at four
per course. Two seasons satisfy the P18 mentorship prerequisites; four record a
completed apprenticeship and a legacy tie. This supplies actual training only,
never “encountered cavalry/dragon” or unwitnessed battle participation. Teachers
do not copy their skills into a pupil. Tuition is free; subsequent class courses
still use P18 costs/opportunities. Non-blood pupils get exactly the same learning
rules as children. Teachers remain useful without front-line deployment.

AI assigns an available qualified elder/retiree to the lowest-ID compatible young
learner at their site, prefers a missing Officer/Medic capability, then Infantry;
uses the same validation, class course and appointment actions as the player.

## P22 — Households, children, service entry and succession

Keep relationships sparse and optional. Pairing is a player proposal between two
living unpartnered adults of the same faction at a safe owned site, age 18+,
without known parent/child/sibling relationship. Acceptance requires at least four
recorded shared-service/household seasons; otherwise explain missing familiarity.
One active partnership per person. This is a minimal household mechanic, not
political marriage diplomacy or forced guaranteed births.

For household continuity, eligible partnered adults both aged 20–40 and present
at the same safe settlement may choose Raise Child. Once per year there is a 20%
people-RNG chance to add one dependent, with at least eight seasons between new
children and at most two dependent children per household in this initial model.
The household action can opt out. Resolve the one annual attempt at the Spring
boundary, tracking the last attempted year; repeated clicks cannot reroll it.
Record parent/guardian context as explicitly
chosen biological/adoptive background; the simulation does not model genetics or
require a gender pairing. No automatic trait/stat/class inheritance. An explicit
Adopt Ward action instead adds a single age-8 dependent from that site's abstract
population, deducting one population unit once; same guardian/chronology checks,
no class privilege. Tests need not depend on randomly obtaining a birth.

At age 13, an existing dependent can take a local trainee assignment; at 17 they
can enter a valid local formation or site service as Recruit. Birth facts alone
grant no experience. A faction without dependents can invite one age-17 apprentice
at an owned supplied Village+ per year for 20 Gold, drawn from local population,
with one unit deducted and a truthful local-origin record. This keeps non-family
succession available and prevents an inactive dynasty from ending the people loop.

Designate at most one successor per distinct legacy category: command, item,
household/seat, institution/title. Supported relationships are blood, martial
(pupil/officer), religious (ordinary temple disciple), political (governor/adviser),
and adopted ward. Religious legacy is non-magical and uses a Temple/mentor.
Eligibility is category-specific: command requires fit adult local service and
an available army; item custody requires a living holder/site; household may use
a guardian for a minor; institutional role requires its physical site and training.

On retirement/death, release old duties first. A valid designated successor can
receive a vacant command only if already co-located with its army; otherwise
leave it vacant and notify. No skill, army, property or person is teleported or
duplicated. If a designation fails, select the oldest qualified local pupil for
command, then highest command evidence, then ID; if none, remain leaderless.
Blood is not the automatic tie break. Estate items stay with site custodian until
a valid holder collects them. Titles/house links can persist as context without
restoring an eliminated faction, paying resources or granting command expertise.

AI appoints legal available replacements, encourages one eligible household or
apprenticeship when it has fewer than two learners, and uses non-blood successors
when no suitable family exists. Never simulate every ordinary soldier's parents.

## P23 — Items, institutional memory, eras and retention

Named legacy equipment is one stable item record, initially with no stat bonus.
Each founding Officer starts with a mundane named weapon as a setup possession.
Transfer requires co-location or a site estate collection. Record holder and
retained custody deeds once. Inheritance moves that same item ID, never copies it.
Military traditions are a completed mentorship/army service link; memorials are
retained person/place labels. These provide supported context without inventing
crafting, loot tiers or a research tree.

Use the already delegated chapter 09 budgets in `history_rules.json`:

| Record | Limit |
| --- | --- |
| Detailed narrative | At most 40 rounds old and newest 10,000 events |
| Notable summaries per extant place, army or person | 12 entries, at most 80 rounds old |
| Unreferenced departed people | At most 2,000 and at most 80 rounds since departure |

Prune at round end by age then count with event-ID ties. Retain minimal identity
facts still required by living families, item custody, succession, knowledge and
retained events. Destroyed formations keep no formation archive. Per-person
progression counters, active relationships, resolved threat/order flags and
current place state do not depend on narrative records. Save checkpoint count
is not capped by this policy. Report viewing/forgetting never grants experience.

History offers filters by person/place/army, event type and season range, paged
50 entries at a time. A single fact can appear in several views without applying
again. Show dated labels and honest missing-history text. Observers use P10
visibility even in search and biographies. Retrospective service is identified
internally and never expanded into unsupported individual achievements.

Era labels are derived summaries: Founding until first declared war; named active
war periods by faction pair; Peace after four full rounds without active war;
renewed war creates a new dated period linked to the earlier pair if retained.
No predetermined story chapters or obligatory fifty-year ending. Anniversary
reminders every ten years of an extant person's service/site foundation use
compact valid dates, at most one per faction per round, ID tie break, recorded
last-reminded year. Never claim “few living soldiers remember” without evidence.

K18 must exercise 200 rounds (50 years) and 400 rounds (100 years). Remembering
every event for those durations is not required; preserving coherent people,
current places, valid relationships and earned capabilities is.
