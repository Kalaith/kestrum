# 05 — Battles and sieges

[Documentation index](README.md) · [Armies](04-armies-and-logistics.md) · [Living places](07-living-places.md)

## Player control

Combat is primarily automatic. The player prepares composition, character assignments, geography, logistics, and the decision to attack, wait, retreat, or relieve a siege. They do not manually command individual soldiers during the resolution.

Source factors include troop type and headcount, named characters, class abilities, veterancy, terrain, fortifications, bonds, and army condition. Exact mathematics, targeting, troop counters, rounds, and randomness remain open. Do not present a numerical model in implementation as settled design without recording the choice.

## Encounter trigger

Opposing armies meeting at the same node trigger combat during sequential faction turns. For example, an enemy moving onto a player-occupied Node A in its turn initiates an encounter then.

**Working interpretation:** “opposing” means hostile under the current diplomatic rules. Friendly stacks do not fight. Neutral access and trespass rules are open. Fortified encounters can enter a siege state instead of resolving an immediate ordinary field battle.

**Proposal — encounter contract:** identify participants, validate diplomacy and positions, determine terrain and siege context, resolve one encounter, apply results once, then continue the active turn. A movement order must not keep executing while its army is awaiting an unresolved encounter.

## Proposed resolution stages

1. **Snapshot:** take participating armies, formations, characters, relevant conditions, terrain, and information available to each faction.
2. **Prepare:** derive combat roles and effective strength from data-defined rules. Preserve actual headcounts separately from computed effectiveness.
3. **Resolve:** advance bounded automatic exchanges or another chosen resolver, with explicit termination and deterministic random draws where used.
4. **Conclude:** determine victory, defeat, withdrawal, rout, or stalemate. Each outcome needs a defined control effect.
5. **Apply:** update casualties, surviving formations, character consequences, locations, node damage, and control atomically.
6. **Record:** issue battle events for experience, recognition, intelligence, and history. Avoid duplicate credit when reopening a report or loading a save.

This is an integration proposal, not a required combat formula. A first resolver should be explainable before it becomes elaborate.

## Casualties and character survival

Formation losses persist between battles; zero headcount destroys the formation. Preserving veterans matters because destruction ends its military continuity and accumulated advantage.

V1 avoids per-hit death checks for named people. If their containing formation survives, they generally survive. If it is wiped out, they may die. This is a simple initial rule, not immunity from all later illness, old age, or major events.

Wounds, captures, heroic escapes, and rare exceptional deaths are later combat extensions. The concept also needs a “commander wounded / assumed command” experience to prove emergent leadership. **Working interpretation:** a limited wound event can be prototyped for that purpose without claiming a complete injury or captivity simulation.

**Open:** probabilities and outcomes after formation destruction, treatment of multiple attached people, and the survival of a character whose host army retreats must be decided. Meaningful careers take priority over constant arbitrary replacement.

## Retreat and rout

A surviving army can retreat to a connected node. This preserves experienced forces, recurring enemies, and accumulated relationships. Defenders under siege may attempt escape, which can succeed or fail.

**Proposal:** compute legal retreat destinations from the current graph and control state. Preserve survivors and reduce their remaining action capacity according to a documented rule. A no-route outcome must be explicit; neither teleportation nor an infinite retreat loop is acceptable. Preview known retreat risks before a voluntary attack when the player could reasonably know them.

**Open:** who chooses the destination, whether retreat can be ordered before combat, retreat costs, pursuit losses, destination conflicts, and surrender/capture at encirclement.

## Persistent siege

Attacking a fortified location can establish a persistent siege. Defenders begin with a strong defensive advantage that gradually weakens as the siege continues.

The illustrative sequence is:

| Continued siege step | Narrative state |
| --- | --- |
| 1 | Strong walls and full preparation |
| 2 | Defenses weakened |
| 3 | Supplies strained |
| 4 | Fortifications damaged |
| 5 | Defender increasingly vulnerable |

This sequence illustrates direction, not a fixed five-turn capture timer. Under the seasonal clock, major sieges can last several seasons or years. The attacker must weigh delay against losses, opportunity cost, and relief forces.

| Attacker actions | Defender actions |
| --- | --- |
| Maintain the siege | Wait and hold |
| Assault early | Sortie against besiegers |
| Wait for defenses to weaken | Attempt escape |
| Withdraw | Receive reinforcements or coordinate relief |

Sieges can eventually involve disease, supply consumption, civilian suffering, fort damage, negotiation, and relief. Disease and negotiation need their own scope decisions; they are not all required for the first light siege mechanic.

## Proposed siege state contract

A siege refers to the fortified node, participating factions and armies, start date, accumulated progress, current defensive advantage, and relevant damage. It remains identifiable between turns and saves.

Suggested transitions:

```text
Hostile arrival -> Siege established -> Maintain across seasons
                         |                   |
                         +------ Assault ----+-> Battle result
                         +------ Sortie / relief / escape -> Battle result
                         +------ Withdrawal or diplomacy -> Siege lifted
```

Revalidate participants after every battle, move, elimination, or diplomatic change. An absent besieger cannot keep weakening a fort. Assault, relief, and maintenance must not each advance the seasonal siege timer again. Fortifications should not regenerate fully merely because the siege record was removed; lasting damage belongs to the node.

## Relief and multiple armies

A relief force can attack the besieger from outside while defenders may break out. This supports multi-army siege situations without a separate tactical siege game.

**Open:** whether relief and breakout are one joint encounter or ordered encounters, which friendly stacks participate, how slots from multiple armies are represented, casualty allocation, and retreat priority. Preserve each participating army's identity and supply state. Select a single documented rule before accepting a battle resolver.

## Local threats and peace-time combat

Bandits, wolves, wild beasts, monster nests, and dangerous ruins can occupy nodes. Clearing them gives combat experience, formation veterancy, recognition, resources, settlement opportunities, or class opportunities.

Source examples:

- A wyvern nest near Frostmarch is cleared. A veteran formation gains experience, a character distinguishes herself, and a recovered dragon egg creates a possible Dragon Knight path.
- A cleared Bandit Hold can become an outpost.
- A destroyed monster nest can expose a rare resource site.
- Defeating bandits in a ruined fort can permit reclamation.

These are possible event chains, not guaranteed loot rewards. **Proposal:** store the actual threat, resolution, and reward so that repeated visits cannot grant the same clearance event. Respawning threats, if added, should be new documented events with limits that prevent effortless recognition farming.

## Damage beyond the battle

War may burn farms and fields, damage settlements, sabotage roads, destroy outposts, and weaken fortifications. Population losses, refugees, supply disruption, and unrest feed [living places](07-living-places.md). A contested border should visibly diverge from peaceful heartland.

The casualty report should separate military losses from lasting local effects. Whether the player can deliberately raid or scorch territory is a command-design decision; the source establishes destructibility but not a complete order list.

## Reports and proposed acceptance criteria

A battle report should explain the result with known contributing factors, surviving headcounts, destroyed formations, character consequences, retreat destination, control change, and noteworthy experiences. Keep full calculation detail optional. Record sufficient facts for later biographies.

Acceptance should cover an ordinary encounter, a survivable defeat and retreat, a formation wipeout, a persistent siege and assault, and relief or escape with several armies. Add distinct coverage only where those cases cannot safely cover a necessary rule. Save/load must not repeat casualties or rewards.
