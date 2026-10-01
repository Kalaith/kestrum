# 04 — Armies and logistics

[Documentation index](README.md) · [Combat](05-battles-and-sieges.md) · [Characters](06-character-development.md)

## Military identity and scale

A kingdom should field a handful of recognizable armies. Commander, composition, veterans, shared service, battles, and relationships give each army an identity. The current system has no arbitrary kingdom-wide army cap; six formation slots, resources and strategic coverage constrain force composition.

The concept draft described persistent units of approximately six individuals, such as Briar Company: Captain Serai, Brother Edrin, two squires, and two temple neophytes. The kingdom draft instead specifies an army of six troop-formation slots, with named people supporting much larger bodies of troops.

**Confirmed decision (D01):** each army has six discrete troop-formation slots with variable headcounts. The earlier six-person company remains a preserved concept example. Individuals may emerge within formations, but ordinary headcount is abstract.

## Army composition

Source example Briar Host (includes deferred troop types):

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

These are source examples of scale, not the current recruitment menu. Fantasy formations are future content under O24. Current human formations and their abstract headcounts, including Siege Engines, use [economy.json](../assets/data/economy.json); casualties and recovery already operate on those capacities. No separate engine-crew simulation is required by the present rules.

The current base roster is Warriors, Spearmen, Archers, Riders, Medics and Siege
Engines. Squires describe a source character role, not an additional current
formation kind; magical Clerics remain deferred.

The player chooses troop types, character placement, support, and siege equipment before combat. This preparation is a principal tactical choice. Magical forces and advanced classes are later content.

**Confirmed decision (O09):** formations and named characters may transfer whenever their armies share the same physical node. A region's world-map marker does not make different internal sites the same node. A safe settlement, supply connection, or fresh movement allowance is not required.

**Implemented baseline:** transfers cost no movement or time and are not restricted to the active faction's phase. Apply them between atomic actions, never partway through a resolving battle or round-end calculation; revalidate co-location when applied. Allow empty slots while enforcing six slots, ownership, and unique membership. Formations and people retain their own spent movement, so transferring never refreshes an allowance.

## Named characters as force multipliers

**Confirmed decision (O06):** an army can exist without a named character and can keep operating after a leader dies or retires. It is substantially weaker. Current leadership tuning starts at 500 permille without a contributing named person; this is an implemented adjustable default, not an unanswered design question.

Multiple eligible people improve an otherwise identical army with bounded contributions. Campaign leadership rules, formation leader snapshots, class abilities and appointment validation are implemented. Their values live in [campaign_rules.json](../assets/data/campaign_rules.json), [combat_rules.json](../assets/data/combat_rules.json) and [battle_tactics.json](../assets/data/battle_tactics.json). Concentrating talent trades strength for coverage; preserve the one-person-per-formation rule below.

A newly emerging character may appear within a formation as “Elian + Warriors” and can later leave Commander Teresa's Briar Host to lead the Frostmarch Guard. That transfer reduces Briar Host's concentration of talent. Appointment validates the person's actual assignment, life status, fitness and eligibility; transfers follow O09's shared-node rule.

### Characters belong inside formation slots

Each formation slot permits **one named character plus troops, or troops alone**.
Two named characters can never share a slot. Multiple characters in an army
occupy different formations; distribute them across armies when local formations
are available. Transfers and entry into service reject an already staffed slot.

The roster must show a formation's attached named character together with its
troops. An army-level Commander label alone does not communicate this membership.
The founding lord begins inside Warriors, so the first slot displays, for example,
**Lord Catrin Cairn + Warriors**. A named character serving in an Archers slot
can read **Hero Elian + Archers** once formally recognized.
Before recognition it reads **Elian + Archers**. Profession, founding nobility,
recognition and army command are independent of the host formation's troop type.

| State | Formation label and behavior |
| --- | --- |
| No attached named people | Troop type alone, such as Warriors. |
| One named member | Name and any established title, followed by `+` and the troop type. |
| Another named person tries to join | Reject the order and ask for a troops-only formation slot. |
| A different commander is appointed | Formation labels keep showing their actual members; appointment alone does not move anyone. |
| A person transfers | Remove them from the source label and include them in the receiving formation immediately. Their class does not choose the troop type. |
| Wounded but still attached | Keep their membership visible; fitness and leadership contribution remain separate rules. |
| Site duty, retirement away from troops, displacement or death | Do not show the person as a current member of their former formation. |

Named members use the formation's existing slot. Tracking, recognition,
appointment or transfer does not add another slot, troop, capacity or upkeep
charge. `Lord + Warriors, 100/100` still represents the same 100-capacity Warriors
formation. Show current/capacity headcount separately from named membership;
veterancy and remaining/spent movement remain visible on the same row.

Keep the troop type and headcount readable when names are long.
Shorten the name if needed; the paged People view preserves full
identity, assignment, condition, Career and History controls. The army Commander
summary states who commands the whole army, while the formation row states where
that person serves. Save/reload restores labels from actual person assignments,
including earlier saves; no new people or honorary status are inferred.

Earlier stacked saves retain the commander in their existing formation and move
other members into troops-only formations at the same physical site, preferring
the least staffed army. If no local formation remains, people wait at that site.
This migration preserves people, troops, movement, service and historical reports;
it neither teleports people to distant armies nor grants free formations. Newly
emerging people likewise use a local troops-only slot or remain at the site.

## Formation persistence

Formations maintain headcount across battles. Warriors at 100/100 may return at 72/100. They can recover if supply and other requirements permit, and may carry experience from their service.

Veterancy can progress from Warriors to Seasoned Warriors to Veteran Warriors. Restrained improvements may affect effectiveness, morale, resilience, discipline, and rout chance. Repeated survival provides more credible opportunities for individual emergence.

**Confirmed decision (D05):** at zero headcount the formation is destroyed. Its formation history and veteran identity are gone; a newly recruited formation of the same type is a new formation.

The wider world can still remember battles involving the lost formation, but there is no requirement to retain its own service archive. Those world events cannot restore the formation or pass its veteran benefits to replacements.

**Confirmed decision (O16):** reinforcement has no effect on a surviving formation's veterancy or specialization. Veteran Warriors at 21/100 remain fully veteran when restored to 100/100. Replacements neither dilute nor grant experience. At zero headcount, D05 still destroys the formation; recruitment cannot revive it.

## Formation specialization

Repeated service can create eligibility for a different troop type:

| Existing force | Evidence | Possible specialization |
| --- | --- | --- |
| Warriors | Repeated fortress defense | Shield Guard |
| Warriors | Repeated combat against cavalry | Pikemen |
| Riders | Extensive scouting and pursuit | Light Cavalry |

Experience creates eligibility; a validated player or AI course order supplies the change. Costs, facilities, required evidence and specialization definitions are implemented in progression data and [specialization.rs](../src/engine/progression/specialization.rs). Preserve the surviving formation's identity, headcount and history according to those rules; missing prerequisites must be explained rather than bypassed.

## Bonds and shared service

Characters and troops that serve together may develop bonds. Initial effects should remain subtle. Later possibilities include morale, reduced rout chance, improved support, recovery, coordinated action, and one character saving another.

The purpose is army identity. Shared-service and relationship records are implemented; a large relationship-management interface and new numerical bond effects are outside this map improvement scope.

## Movement

Armies use connected routes and member-owned movement allowances. Composition, roads, terrain and siege equipment already affect travel. Edge cost applies authored terrain cost and the current road bonus; a group's legal progress respects every member's remaining budget. Weather remains future scope.

**Current interaction:** tap an Army banner, inspect remaining movement and nearby costs, then tap a valid destination to issue the order immediately. Longer routes move as far as affordable and retain their remainder for later rounds; Review Route and Cancel Route are available. World orders include physical regional travel. Hostile contact, sieges, peaceful borders and discovery restrictions constrain travel. A changed route stops at the last legal location and reports the interruption.

**M01 overview feedback:** each owned army banner shows its shortened name,
current surviving troop count and an Idle, Route, Siege or Cut off state.
World stacks show their owned army count, combined troops and the most urgent
state; aggregation does not make different internal sites the same physical
location. Siege takes precedence, followed by lack of supply, queued route and
idle state. These are current owned facts, independent of an open order card.
The attention list includes unsupplied owned armies and focuses the selected
force through the existing movement interface.

Exact composition, movement allowance and route detail remain available on
selection. M02 of the [map playability plan](map-playability-plan.md) owns
persistent destination/path presentation after dismissal, clearer interrupted
orders and common actions in the map inspector. M01 preserves the existing
destination-tap command path and does not infer enemy strength or orders.

Do not promise exact information beyond fog of war. The preview can identify known threats and uncertainty separately.

## Roads

Roads improve movement between nodes and help invaders as well as defenders. Damaging or destroying them is possible but should be harder than burning fields or destroying temporary structures.

Under confirmed decision D06, constructing a road improves an existing route. Road damage can disable its movement bonus; it does not delete the physical graph edge. Supply uses secure connectivity, not a requirement for every route to be improved. The current design keeps the graph fixed.

## Simple supply

**Agreed direction (O03):** an army with a valid supply connection can recover health and troop strength normally. A cut-off army cannot normally recover health or headcount and can be worn down by successive attacks. Initial logistics avoids detailed food inventories and supply-wagon control.

**Implemented connectivity model:** supply starts at the faction's controlled headquarters and follows secure friendly physical sites. Contested sites and active local threats block the connection. The same physical graph spans both map scales; siege isolation is reflected through current control/conflict state. [Supply queries](../src/state/world/supply.rs) remain authoritative for recovery and planning.

Recovery resolves at round end; its rate and cost are in [economy defaults](03-kingdoms-and-economy.md#provisional-economy-defaults). An outpost extends a headquarters-connected position; an isolated outpost is not an unlimited independent supply source. Planned overlays should expose these existing connections and known breaks rather than introduce a new logistics model.

The v1 rule is restricted normal recovery. Starvation, automatic attrition, and detailed siege supplies are later possibilities. Show “Cut off: cannot replenish” at the selected army and identify a known broken route when possible.

## Outposts

Armies can establish outposts on suitable nodes. **Implemented default (O12):** an accepted order needs three seasonal progress steps, about nine months, with valid control, supply and other requirements. [Construction rules](../assets/data/construction_rules.json) define duration and conserved settlers; order validation and progress determine blocking and cancellation. Balance remains adjustable.

Outposts create territorial presence, a forward recovery point, a future settlement seed, a defensive position, and supply extension. They are persistent places. They may become hamlets, villages, fortified settlements, and towns, or become abandoned, ruined, and occupied by bandits.

Construction orders have persistent progress, blocked reasons, completion and cancellation receipts. Show remaining seasons and the actual blocking reason. Current cost/refund, supply, control-loss and interruption rules are implemented. The source's “Village → Fort → Town” example describes changing habitation and fortification together; those remain separate state layers.

## Stacking and reinforcement

**Agreed direction (O05):** multiple friendly armies may occupy one node without a hard stacking limit. V1 has no planned direct stacking penalty. The cost is the territory left undefended elsewhere.

Encounter context determines participating armies and joint relief through the [combat system](05-battles-and-sieges.md). Every participating army retains its six-slot identity in grouped encounters; the six-slot rule is not a combined encounter limit.

## Invariants

- Slot count is at most six; headcount stays between zero and capacity.
- Destroyed formations cannot recover, specialize, or return through a simple type match.
- A character transfer preserves identity, biography, equipment ownership, and relationships.
- A lost leader does not implicitly delete surviving troops or invalidate every army action.
- Supply, movement, roads, and entrance mappings use the same world geography.
- Construction advances once per season, regardless of the number of factions.
- Roads and physical facility loss affect the player and AI through the same rule checks.
