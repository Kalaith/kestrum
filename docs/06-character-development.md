# 06 — Character development

[Documentation index](README.md) · [Generations](08-generations-and-succession.md) · [History](09-history-and-content.md)

## Current baseline

Emergence, evidence-based recognition and traits, ordinary careers, formation
specialization, training, qualified mentorship and rival progression are
implemented. [progression.json](../assets/data/progression.json) and the
[progression engine](../src/engine/progression.rs) define their working rules.
This chapter preserves broader source examples as future possibilities; dragons,
magical classes and a detailed disease system are outside the current scope.
Earlier provisional defaults are implemented tuning choices, not pending approval.

## Identity through experience

Characters become recognizable through what happens to them. The player initially sees limited information, such as “Tomas — Squire — served two campaigns,” rather than a recruitment card exposing aggression, learning rate, courage, and loyalty as precise scores.

The five forces shaping development are:

| Force | Source examples and purpose |
| --- | --- |
| Disposition | Subtle courage, caution, physical aptitude, empathy, ambition, curiosity, or stubbornness influence responses without fixing destiny |
| Experience | Holding an outnumbered gate, surviving retreat, fighting cavalry, plague service, sieges, defeating an officer, protecting someone while wounded, surviving a dragon |
| Mentorship | Mounted officers teach riding and cavalry confidence; field healers and temple service create different clerical opportunities |
| Opportunity | Command responsibility, a dangerous front, equipment, cavalry assignment, a mentor, caring for wounded, or participation in a siege |
| Consequences | A failed defense can produce caution, determination, fear, hatred, or defensive skill; a successful charge can produce confidence, aggression, prestige, or cavalry specialization |

The same event need not produce the same person. Context, prior history, and disposition matter. The combined model is starting disposition + experiences + mentorship + opportunities + consequences, rather than identity fixed by random recruitment statistics.

## Levels of simulation and recognition

**Confirmed decision (D02):** ordinary troop members begin abstractly. Someone who distinguishes themselves becomes a visible character attached to the formation, with a plausible recent history generated when needed. This does not require tracking every soldier from birth. Sparse juniors, heirs, and trainees may still be tracked when relevant to the generational design.

**Implemented representation:** distinguish levels of simulation and prominence:

1. **Abstract population or troop member:** represented by population and formation headcount, with shared formation history.
2. **Tracked person:** a junior, trainee, emerging soldier, heir, or other relevant individual with a persistent ID and age.
3. **Recognized figure:** a tracked person whose meaningful deeds justify greater historical and political prominence.

An ordinary personal name is not the same as recognized status. Tomas can be named in a roster before becoming “Tomas of Hawthorn.” Emergence from abstraction and formal recognition are separate recorded facts. Named people can strengthen armies and later move into leadership through implemented fitness, assignment and appointment checks; recognition does not create an extra formation slot.

## Emergence from formations

A warrior formation can become “Apprentice Elian + Warriors” after relevant
service. This is an earned battle-progression stage, not a profession or active
mentorship assignment. The recruit remains inside the formation's existing
headcount and keeps the same identity as they advance.

This is a required roster representation: a named person appears with their
current host formation, not only in an army Commander summary. An archer emerging
from Archers remains `Elian + Archers`; formal recognition adds the Hero title,
giving `Hero Elian + Archers`. The starting full lord appears as `Lord Name +
Warriors` from the first turn. Neither emergence nor recognition grants an extra
troop or formation slot. Membership, multiple people, transfers and display
overflow follow [the formation-slot contract](04-armies-and-logistics.md#characters-belong-inside-formation-slots).

**Implemented battle emergence (O14):** each surviving formation with troops and a vacant named-person slot earns one named Recruit after one meaningful engagement accepted while the slot is vacant. Progress uses the established participation classifier and resolves at the seasonal boundary. Each eligible formation is evaluated in stable ID order; no faction-wide candidate lottery, chance roll, veterancy multiplier, or named-roster pressure can block earned service. Seeded people RNG still selects the recruit's identity and disposition. This threshold is data-validated and is not a hard roster cap; tune against observed campaigns.

**Implemented personal Hero progression (P17):** an emerging Recruit starts with
zero Hero progress even when their formation has older service evidence. One
later meaningful engagement personally fought while fit grants recognition.
Progress survives save/load and transfers, and never imports the receiving
formation's history. Specific supported deeds can ground an epithet; ordinary
qualifying service is enough for the general Battle Service epithet. Recognition
preserves the person, portrait, Recruit profession and host troop kind, and does
not appoint them as commander. Army Details labels emerging members Apprentice
and recognized members Hero. The person's Career detail shows personal Hero
service as qualifying engagements fought while fit. AI battle selection can favor
their army when a legal hostile opportunity exists; it does not fabricate
participation. Recognized Heroes can be fielded from a site into uncovered armies,
and surplus Heroes can move between colocated friendly armies under transfer
rules. The seed-260926 replay meets the midgame goal with Heroes in 11 of 14
active armies at round 120; see [verification](verification/formation-hero-progression.md)
for later checkpoints and limits.

### Army and person presentation brief

| Question | Decision |
| --- | --- |
| Current decision | Identify who serves in each formation and whether an apprentice is nearing Hero recognition and could later replace a vacant commander. |
| Dominant focus | Keep the selected army's six formation rows, troop type and named member prominent; the map remains dominant during map play. |
| Primary action | Tap the visible People control from Army Details, then open the person's Career detail; army orders stay available from the same screen. |
| Supporting information | Keep the same portrait, Apprentice/Hero title, fitness, troop kind and data-defined personal engagement progress beside the person. |
| Deferred information | Keep qualifying dates, places and specific deeds in the person's History detail. |
| Layout and camera | Preserve the existing six-row Army Details layout and map camera at the sole supported 1920×1080 canvas. |
| Input and feedback | Use visible touch-sized controls and distinct emergence and later recognition receipts; both transitions remain in history. |

## Retrospective grounding

An emerging person can receive a plausible background from formation history, recruiting place, recent battles, commander, settlement chronology, current age, and culture. No detailed simulation of every soldier from birth is required.

Example: Elian is nineteen. His Frostmarch formation was recruited three years ago and fought at Redplain and Hawthorn. A plausible biography says that he joined in Year 22, fought at Redplain in Year 23, and distinguished himself at Hawthorn. The timeline must support his birth, service, and presence; it cannot claim a battle fifty years before his birth.

A future retrospective family narrative may connect to known history when dates and recorded relationships permit. It must not automatically assign an emerging person as a founder's child. Current households and family records have their own commands and chronology checks. Prefer unknown context over contradicting parents, locations, ages or deaths.

## Evidence-based traits

Repeated actions can justify traits. Tomas holds against superior forces, volunteers for a counterattack, takes command after an officer is wounded, and protects a retreat. The game may recognize Bold, Protective, and Natural Commander.

**Implemented baseline:** compact evidence and data-defined thresholds support earned traits and career explanations. Narrative retention is separate from lasting earned capability. The player should understand why a trait exists without seeing every hidden inclination. More elaborate contradictory traits or evidence decay are future extensions, not missing prerequisites for the current ordinary careers.

## Recognition

Meaningful recognition can come from:

- Surviving major battles or disasters.
- Leading after an officer falls or commanding a detachment successfully.
- Defeating significant enemies or capturing important sites.
- Saving other people, extraordinary healing, or being repeatedly wounded.
- Participation in famous campaigns.
- Notable rivalries or major promotions.

Recognized figures may gain distinctive portrait treatment, a biography, command ability, relationships, rivalries, epithets, political importance, event involvement, battlefield influence, and historical records. Their capture or death carries greater strategic and emotional weight.

**Implemented baseline:** recognition uses supported personal evidence and a
data-defined engagement threshold, then records its result once. The presentation
should show personal progress, name any supported deed, and keep role separate. The
[planned portrait generator](hero-portrait-generator-plan.md) assigns stable
appearance to tracked people before recognition; becoming a Hero preserves the
face. Portraits are not implemented. Additional political benefits and captivity
remain future possibilities rather than automatic consequences of recognition.

## Class eligibility and opportunities

**Implemented direction (O15):** experience and opportunity create class eligibility, supported by resources and infrastructure in the world. The current ordinary career rules define prerequisites, costs, courses and class changes; mentorship has separate seasonal progress. Meeting history requirements does not supply missing facilities or resources. Future dragon training would require its own relevant infrastructure and content rules.

**Confirmed decision (O10):** eligibility uses the experiences and encounter types the unit actually participated in. Generic experience points cannot replace a missing encounter: a unit that has never seen a dragon cannot unlock dragon flying. A formation member can draw on events during their service in that formation, not battles before they joined or events experienced only by a remote ally. Compact participation facts survive narrative-history pruning under O23.

**Implemented scope (O24):** humans and the ordinary classes Recruit, Infantry, Archer, Scout, Cavalry, Medic, and Officer. Dragon Knight, Plague Cleric, magical specialists, and other races remain later content. The examples below preserve that future design without adding it to the current campaign or map plan.

Resources create possibilities:

| Resource | Opportunity |
| --- | --- |
| Warhorse | Cavalry practice and mounted service |
| Apothecary access | Medicine, poison, and disease experience |
| Command assignment | Leadership experience |
| Master spearman mentor | Polearm development |
| Dragon egg | A possible future bond and access to dragon-related development |

A “Book of Courage: +5 Courage” illustrates the abstract direct-stat approach the design wants to avoid.

### Dragon Knight: Dratanus — future content

Dratanus begins as a squire. He trains with cavalry, uses polearms, serves a mounted commander, encounters dragonkin, survives a dragon attack, receives specialist riding training, and bonds with a drake.

Source eligibility example:

- **Foundation:** martial training, riding experience, polearm proficiency.
- **Experience:** encountered dragonkin.
- **At least one:** survived a dragon attack, defeated a dragonkin officer, or bonded with a drake.
- **Opportunity examples:** drake access, Dragon Knight mentor, specialist stable.

The kingdom draft additionally suggests suitable named status and dragon infrastructure such as a Dragon Hatchery. Exact combinations of egg, drake, hatchery, stable and mentor are deferred with this fantasy content. They do not block current implementation or imply a required approval step.

### Plague Cleric: Mira — future content

Mira begins as a temple neophyte in a conquered city suffering disease. Treating infected civilians, performing battlefield triage, learning herbalism, working with an apothecary, surviving infection, and studying poison and disease can create the path:

```text
Temple Neophyte -> Field Cleric -> Plague Cleric
```

The campaign creates the specialist. Current Medic progression already records actual treatment experience; the wider disease and magical career narrative remains a future example.

## Player nurturing and occasional direction

Defensive service in forts under a defensive commander can encourage protective expertise. Assaults, vanguard service, and difficult positions create offensive opportunities. Dragon encounters, an awarded egg, a hatchery, and a mentor can encourage a dragon-related career.

Occasional explicit choices are allowed: Teresa may ask whether to hunt dragons or learn to ride one. Such an event provides direction. The preferred path creates opportunities through play: slay a dragon, recover an egg, award it to Teresa, and support her development through suitable experience and infrastructure. A direct “Become Dragon Knight” choice should not replace that chain.

## Enemy careers and rivalries

An enemy can progress from Unknown Squire to Dratanus, Knight, then Dratanus the Ashen, Dragon Knight and Commander of the Eastern Host. The same persistent person may be identified more clearly over time as intelligence improves.

If the player destroys his company and he survives, he may gain survivor experience, hatred, caution, prestige, ambition for revenge, or promotion. He may later lead an invasion. These outcomes are possible contextual developments, not a guaranteed revenge script or a free resurrection.

## Geography as a development source

| Situation | Possible experience and development |
| --- | --- |
| Repeated bridge defense | Discipline, shield classes, defensive specialization, command recognition |
| Deep offensive far from supply | Endurance, independence, scouting, aggressive doctrine |
| Diseased region | Triage, herbalism, plague expertise, emotional consequences |
| Encirclement | Survival, loyalty, desperation, leadership, trauma, reputation |

## Participation vocabulary

**Implementation baseline for O10:** record participation in battle, victory, defeat, survival while outnumbered, commander wounded, assumed command, treated wounded, fought an enemy type, defended a strategic node, captured a strategic node, training, and non-combat encounters. Tag the relevant troop/creature type, terrain, role, and activity. Additional event kinds can be added as content needs them.

Store only facts supported by actual participation or a valid retrospective service history. Seeing a dragon could satisfy a future encounter prerequisite but would not grant flying, riding skill, a mount or infrastructure. Current content uses ordinary human training and service tags; its recognition thresholds and class expressions are implemented in progression data.

**Invariants:** every experience has valid participants and a date; recognition is applied once; retrospective history fits age and presence; class eligibility can explain missing requirements; identity survives transfers and save/load; enemies follow the same development constraints.

## Planned map connection

The [map playability plan](map-playability-plan.md) should make an army's relevant
leader and assignment identifiable where that army acts, and make sites with
useful training facilities recognizable. Surface an earned opportunity or loss
where it affects the next decision; keep biographies, household detail and full
course lists in contextual inspection. This connects existing people to their
places and campaigns without adding another permanent character dashboard.
