# 04 — Armies and logistics

[Documentation index](README.md) · [Combat](05-battles-and-sieges.md) · [Characters](06-character-development.md)

## Military identity and scale

A kingdom should field a handful of recognizable armies. Commander, composition, veterans, shared service, battles, and relationships give each army an identity. Exact army limits are open; the design prefers meaningful forces over dozens of disposable stacks.

The concept draft described persistent units of approximately six individuals, such as Briar Company: Captain Serai, Brother Edrin, two squires, and two temple neophytes. The kingdom draft instead specifies an army of six troop-formation slots, with named people supporting much larger bodies of troops.

**Working interpretation (D01):** use six troop formations per army as the current implementation baseline. Keep the earlier six-person company as a preserved concept example, not a second simultaneous army-size requirement. Individuals may be tracked within formations, but ordinary headcount is abstract.

## Army composition

Example Briar Host:

| Slot | Formation |
| --- | --- |
| 1 | Warriors |
| 2 | Archers |
| 3 | Riders |
| 4 | Clerics |
| 5 | Squires |
| 6 | Siege Engines |

Each slot holds a formation whose size depends on type:

| Source example | Full headcount |
| --- | ---: |
| Warriors | 100 |
| Archers | 80 |
| Heavy Cavalry | 40 |
| Siege Engines | 20 |
| Wyverns | 3 |
| Dragon | 1 |

These examples communicate scale; they are not final balance data. A dragon can occupy the same conceptual slot as a hundred infantry. Siege-engine counts and their accompanying crews need a consistent definition before casualty rules are implemented.

The player chooses troop types, character placement, support, specialists, siege equipment, and magical forces before combat. This preparation is a principal tactical choice. The source allows full composition control but leaves reorganization location, movement cost, empty slots, and transfer timing open.

**Proposal:** permit fewer than six occupied slots; prevent exceeding six. Transfers require a safe shared location and eligible characters. Validate both sides before moving anything. A character or formation may belong to only one army at a time.

## Named characters as force multipliers

An army can exist without a named leader and can keep operating after a leader dies or retires. It is substantially weaker. Roughly 50% effectiveness without a named character is a working tuning target, not a fixed formula.

Multiple important characters should improve an otherwise identical army. Concentrating them increases strength; distributing them improves coverage. Exact stacking, role compatibility, command appointment, and whether effects apply to one slot or the entire army are open. The first prototype must make the contribution visible and avoid runaway multiplication.

A newly emerging character may appear within a formation as “Squire Elian + Warriors.” Once sufficiently recognized or qualified, Elian can leave Commander Teresa's Briar Host to lead the Frostmarch Guard. That transfer intentionally reduces Briar Host's concentration of talent.

## Formation persistence

Formations maintain headcount across battles. Warriors at 100/100 may return at 72/100. They can recover if supply and other requirements permit, and may carry experience from their service.

Veterancy can progress from Warriors to Seasoned Warriors to Veteran Warriors. Restrained improvements may affect effectiveness, morale, resilience, discipline, and rout chance. Repeated survival provides more credible opportunities for individual emergence.

At zero headcount the formation is destroyed. Its active formation history and veteran identity are lost; a newly recruited formation of the same type is a new formation.

**Working interpretation (D05):** destruction prevents reuse of the formation's experience and active identity, while the world chronicle can retain that it existed and was destroyed. Historical records are not a mechanism to transfer its bonuses to replacements. How reinforcement dilutes veterancy is open.

## Formation specialization

Repeated service can create eligibility for a different troop type:

| Existing force | Evidence | Possible specialization |
| --- | --- | --- |
| Warriors | Repeated fortress defense | Shield Guard |
| Warriors | Repeated combat against cavalry | Pikemen |
| Riders | Extensive scouting and pursuit | Light Cavalry |

Experience creates eligibility; resources and facilities still determine whether the kingdom can support the change. Conversion cost, whether it is automatic or requested, and whether it changes capacity are open. Preserve identity and history when a surviving formation legitimately specializes.

## Bonds and shared service

Characters and troops that serve together may develop bonds. Initial effects should remain subtle. Later possibilities include morale, reduced rout chance, improved support, recovery, coordinated action, and one character saving another.

The purpose is army identity. A large relationship-management interface is outside the intended core. **Proposal:** begin with shared-service records and contextual biography entries before adding numerical bond effects.

## Movement

Armies use connected routes and a movement allowance. Composition, roads, terrain, and siege equipment affect speed; weather may matter later. Costs and movement amounts require prototyping.

**Proposal — movement order:** select army, preview destination and route, inspect movement cost and known supply consequences, confirm, then resolve the legal path. Hostile contact interrupts travel for encounter resolution. If a route becomes invalid, explain the interruption and retain the army at its last valid location.

Do not promise exact information beyond fog of war. The preview can identify known threats and uncertainty separately.

## Roads

Roads improve movement between nodes and help invaders as well as defenders. Damaging or destroying them is possible but should be harder than burning fields or destroying temporary structures.

Under the initial stable-graph interpretation, constructing a road improves an existing route. Road condition affects travel and potentially supply when that rule is defined. A later topological road-building system must explicitly revise graph validation and historical records.

## Simple supply

An army with a valid supply connection can recover health and troop strength normally. An isolated army cannot recover normally and can be worn down by successive attacks. Initial logistics avoids detailed food inventories and supply-wagon control.

**Proposal — connectivity model:** supply queries follow usable routes to an eligible friendly source. Hostile-controlled nodes, blockades, contested boundaries, and siege isolation must be evaluated consistently at both world and regional scales. Cache results only if control and route changes invalidate them correctly.

**Open:** define valid sources, treatment of contested nodes, range limits, neutral passage, supply through occupied territory, and exact recovery timing/cost. An outpost should extend an existing logistical position; treating every isolated new outpost as unlimited independent supply would undermine encirclement and requires a separate decision.

The v1 rule is restricted normal recovery. Starvation, automatic attrition, and detailed siege supplies are later possibilities. Show “Cut off: cannot replenish” at the selected army and identify a known broken route when possible.

## Outposts

Armies can establish outposts on suitable nodes. Approximately three turns is the source working target. With one complete round per season, **working interpretation:** construction requires approximately three seasonal progress steps, about nine months. This pacing must be tested.

Outposts create territorial presence, a forward recovery point, a future settlement seed, a defensive position, and supply extension. They are persistent places. They may become hamlets, villages, fortified settlements, and towns, or become abandoned, ruined, and occupied by bandits.

**Proposal — order states:** planned → under construction → complete; interruption can suspend or cancel the work according to an explicit policy. Show remaining seasons and why progress is blocked. Resolve cost, occupation requirement, refund, and interruption rules before shipping. The source's “Village → Fort → Town” example describes changing habitation and fortification together; those remain separate state layers.

## Stacking and reinforcement

Multiple friendly armies may occupy one node. V1 has no planned hard stacking limit or direct stacking penalty. The cost is the territory left undefended elsewhere.

The source does not settle whether every co-located army automatically participates in a battle. Define deterministic participation and relief rules in the [combat system](05-battles-and-sieges.md); do not hide an arbitrary one-army limit behind the six-slot rule.

## Proposed invariants

- Slot count is at most six; headcount stays between zero and capacity.
- Destroyed formations cannot recover, specialize, or return through a simple type match.
- A character transfer preserves identity, biography, equipment ownership, and relationships.
- A lost leader does not implicitly delete surviving troops or invalidate every army action.
- Supply, movement, roads, and entrance mappings use the same world geography.
- Construction advances once per season, regardless of the number of factions.
- Roads and physical facility loss affect the player and AI through the same rule checks.
