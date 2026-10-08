# 08 — Generations and succession

[Documentation index](README.md) · [Character development](06-character-development.md) · [History](09-history-and-content.md)

## Careers across a changing world

**Agreed direction (O17):** typical campaigns target roughly 20–50 years, with aging careers, changing roles and retirement. Biological age, service dates, birthday mortality, local wound recovery, retirement and governors are implemented. [lifecycle.json](../assets/data/lifecycle.json) holds current age thresholds and probabilities; campaign duration and mortality remain balance targets, not established acceptance results. The source's squire-to-knight-to-governor story is illustrative; the current ordinary roster uses Recruit, Infantry, Archer, Scout, Cavalry, Medic and Officer.

A longer example follows a character from squire at seventeen, knight at twenty-two, captain at twenty-nine, named commander at thirty-eight, marshal at forty-nine, and governor and mentor at fifty-six. These are illustrative milestones, not required ages for promotions.

## Life stages

| Approximate age | Stage | Source roles |
| --- | --- | --- |
| 0–12 | Child | Generally outside military rosters |
| 13–16 | Youth | Page, novice, apprentice, trainee |
| 17–25 | Young adult | Squire, recruit, neophyte, apprentice mage |
| 26–40 | Prime | Typically strongest period of military service |
| 41–55 | Veteran | Greater skill and reputation, potentially reduced physical ability |
| 56+ | Elder | Commander, mentor, governor, priest, strategist, retired figure |

The initial roster is human; other species and their age ranges are later content under O24. Birth dates and dates of entry into service must use the same [seasonal calendar](02-world-time-and-control.md). Calendar passage does not itself award combat experience.

## Aging as a change in role

Older characters may lose endurance, recovery, and mobility while gaining command, judgment, mentorship, political influence, reputation, and training ability. A blanket “Age 45: -2 Strength” does not express the intended design.

Current age and fitness checks affect movement, field participation, apprenticeship, command and automatic retirement. Adults can take governor or qualified mentor roles. Strategists, priests, diplomats and nonhuman age curves are future content, not implied active professions. Source life-stage labels explain the intended arc; concrete thresholds come from current lifecycle and household data.

**Presentation requirement:** explain what the person can now contribute and which assignments fit them. Give the player time to prepare a successor and move the veteran into a useful role. Existing age rules need understandable opportunities and warnings; the map plan must not imply new abilities or disable current roles silently.

## Injury, retirement, and death

A severe or long-term injury can change a career. A knight with a serious leg injury may become a commander, trainer, governor, or tactician. The injury can create a new path rather than simply remove all value.

Current retirement can be ordered explicitly or follow the automatic age threshold. Governor assignment and qualified mentorship preserve useful roles for older people. Religious calling, family-driven retirement events, advisers and diplomats are source possibilities for future systems; do not award benefits for roles that are not implemented.

Combat death, wounds, recovery and birthday mortality are implemented. Death frequency must leave room for attachment and meaningful careers. The [person-combat rules](05-battles-and-sieges.md#casualties-and-character-survival) govern encounter outcomes; a broader illness, permanent-injury or captivity simulation remains future scope.

**Implemented lifecycle contract:** person status, current assignment, retirement and site role are separate facts. Death clears active roles and invokes existing succession and custody handling once. Current validation prevents a dead person returning through stale transfer or training orders. Capture has no implied implementation.

## Households and relationships

**Implemented direction (O20):** households, children, wards, apprentices, mentorship and several successor links support continuity; legacy does not require children. Household commands validate age, local presence, shared service and existing relationships. Formation/end-of-household actions, optional childraising, adoption, trainee assignment and entry into service are implemented through [household data](../assets/data/household_rules.json) and [succession commands](../src/engine/succession/commands.rs).

An active household needs a friendly, inhabited home. Capture or abandonment of
that settlement ends the household and stops childraising, while retaining its
family records. Seasonal decline to Unsettled counts as abandonment even when
the kingdom still controls the site. This local loss does not end the campaign;
kingdom defeat follows the separate [defeat rules](03-kingdoms-and-economy.md#victory-defeat-and-continuity).
See the [Observer household regression](verification/household-abandonment.md).

Limited encouraged or arranged partnerships may support alliances, continuity, succession, and stability. The intended question is what the relationship means for those people and the faction. It should not become a roster of breeding statistics.

The household workflow already supplies player agency and current eligibility explanations. Political marriage, diplomatic alliances and a full social-compatibility simulation remain future scope; they do not block using or improving existing family and succession controls.

## Children and inherited context

The source envisions children inheriting surname, social context, reputation,
mentor access, political obligations, property and cultural background. Current
family identity, training opportunities, successor links and mundane heirlooms
cover part of that goal. Heritable aptitude, broad property rights and political
obligations remain future possibilities; experience still determines capability.

Two famous cavalry officers' child may have horses, mentors, prestige, and expectations, yet become a priest, infantry commander, merchant, mage, or politician. Family provides opportunity without fixing a class.

**Implemented baseline:** dependents and trainees have sparse persistent person/family records, age-valid assignments and contextual service entry. Current defaults allow local training from thirteen and service from seventeen. “Child of Serai and Tomas” remains a source illustration, not a mandatory relationship or a requirement to simulate every abstract civilian.

## Five routes to succession

| Heir type | Source |
| --- | --- |
| Blood heir | Child or close relative |
| Martial heir | Trained squire or officer |
| Religious heir | Disciple or junior cleric |
| Political heir | Trusted administrator or appointed successor |
| Adopted heir | Ward or orphan incorporated into the household |

Siblings, apprentices, trusted officers, and other successors also support continuity. Every significant character can leave a legacy without reproduction. A blood heir should not automatically become equally skilled or qualified for command.

**Implemented contract:** successor designations distinguish legacy categories and relationship links; eligibility and appointment validate actual identity, age, role and local circumstances. Mundane item custody also persists separately. Inheritance never copies a predecessor's class or guarantees a command appointment. Multiple claims, civil war and political disputes remain future possibilities.

## Mentorship between generations

Older heroes can pass techniques, class access, traditions, doctrine, reputation, and relationships to juniors. This makes veterans and retirees strategically useful.

**Implemented mentorship:** records identify teacher, learner, discipline, progress and pause reasons. The current default requires a qualified mentor aged at least twenty-six with four relevant service seasons, permits one learner, and requires four apprenticeship seasons. Both people need valid local contact, fitness, an owned supplied site and the discipline's usable facility; Riding also requires horse access. Separation, wounds or lost opportunity pause progress. [Qualification rules](../src/engine/mentorship/qualification.rs) and lifecycle data are authoritative; proximity alone is insufficient.

Mentorship should be at least as valuable a legacy route as family. A founder's former squire can carry an army's traditions even if no descendant enters service.

## Legacy, institutions, and equipment

A person can leave family, students, named equipment, titles, political consequences, memorials, military traditions, class access, and settlement history. Their relevance can continue after retirement or death.

Source example:

```text
Serai's Spear
Used at Hawthorn Gate
Passed to Tomas in Year 22
```

Current mundane heirlooms already have stable IDs, person or site-estate custody and dated transfer history. Founders receive a Muster Sword; Serai's Spear remains a source example. Custody and succession do not grant magical effects or duplicate the previous owner's class or skill. Additional named equipment is content work.

Families can become linked to places: House Hawthorn, founded by Tomas of Hawthorn, has its seat at Hawthorn Keep. The house can rise with the settlement or become displaced when it falls. Refugees, exiles, and claimants are possible personal outcomes; restoration or claimant factions remain future scope.

## Recruitment and simulation size

Later recruits should reflect military families, temples, settlements, apprenticeships, refugee populations, and noble houses. The world supplies their context.

Most people remain abstract population. Deeper simulation is reserved for relevance through lineage, military service, mentorship, recognition, politics, or exceptional events. Sparse dependents, tracked juniors, active recognized figures, and historical figures need distinct treatment.

**Confirmed direction (O23):** historical memory may be bounded and forgotten, including stories about famous dead people. Keep living people's state and facts required by current relationships, succession, and progression; prune unneeded narrative records under the [history policy](09-history-and-content.md#bounded-history-and-forgetting). Narrative retention and formation emergence are separate controls. Each vacant formation slot earns a Recruit after two distinct meaningful engagements; the rule has no roster-size gate or hard named-person cap. Campaign replays measure the resulting roster growth and pacing.

## Generational acceptance cases

1. A founder ages into a useful non-frontline role while a trained junior can take responsibility.
2. A child, apprentice, or adopted ward can enter service with valid dates and contextual opportunities.
3. Death or retirement removes active assignments once while preserving relationships and history.
4. A legacy can continue through mentorship with no children or family requirement.
5. A multi-decade campaign maintains a manageable active roster and bounded, searchable retained history without breaking current relationships or progression.

## Connection to the map plan

The [map playability plan](map-playability-plan.md) keeps these systems intact.
Show a relevant vacancy, retirement, successor opportunity or lost local
facility where it changes an army or settlement decision. Detailed households
and legacy records stay available on selection. More permanent family panels
would not resolve the current map's missing territorial and military information.
