# 05 — Battles and sieges

[Documentation index](README.md) · [Armies](04-armies-and-logistics.md) · [Living places](07-living-places.md)

## Player control

Combat is primarily automatic. The player prepares composition, character assignments, geography, logistics, and the decision to attack, wait, retreat, or relieve a siege. They do not manually command individual soldiers during the resolution.

Formation combat implements troop type and headcount, attached people and abilities, veterancy, terrain, fortifications, tactics, morale, targeting, activations and reactions. Values are adjustable defaults in [combat_rules.json](../assets/data/combat_rules.json) and [battle_tactics.json](../assets/data/battle_tactics.json). The resolver and its deterministic receipt are the current baseline; richer bond effects remain future content.

## Implemented formation and tactics expansion

The 2026-09-30 request led to preparation-led formation combat, now implemented
through milestones B01–B07. The supplied
[battle notes](reference/battle-system-notes.txt) describe six troop units,
embedded heroes, three front and three rear slots, ordered conditional tactics,
automatic activations and reactions, morale and visible formation gaps.
The subsequent [battlefield mockup](reference/battle-system-mockup.png) establishes
opposing troop groups fighting in a shared landscape. The default execution view
prioritizes that scene, with compact condition labels and contextual inspection.

The [battle system implementation plan](battle-system-implementation-plan.md)
records the seven delivery milestones and their verification. The implementation
keeps six slots per army in grouped encounters, distinguishes reaction rules such
as Brace from activation rules, and separates authoritative campaign resolution
from battlefield playback. Preparation includes positioning and ordered tactics;
the illustrated battlefield projects the saved encounter sequence. Content uses
the existing human troop roster. Continuous tactic edits during execution and
commander interventions remain outside the implemented scope.

This expansion supersedes P11's non-positional roster and P12's simultaneous
rotating-target exchanges. Retreat, destruction, person consequences, sieges,
threats and participant-only history are integrated. Damage and morale values
remain tunable. The [fresh battle review](verification/battle-review.md) and
[implementation README](../README.md) record later fixes and the remaining
production-victory balance failure; earlier milestone passes are historical.

## Encounter trigger

Opposing armies meeting at the same node trigger combat during sequential faction turns. For example, an enemy moving onto a player-occupied Node A in its turn initiates an encounter then.

**Implemented rule:** “opposing” means hostile under current diplomacy. Friendly stacks do not fight, and peaceful foreign borders block ordinary hostile movement. Fortified encounters can enter a siege instead of an immediate field battle. Encounter validation, rather than a visual map overlap, determines participation.

**Encounter contract:** identify participants, validate diplomacy and positions, determine terrain and siege context, resolve one encounter, apply results once, then continue the active turn. A movement order cannot continue through an unresolved encounter. Playback controls project the receipt and never repeat or alter campaign consequences.

## Resolution stages

1. **Snapshot:** take participating armies, formations, characters, relevant conditions, terrain, and information available to each faction.
2. **Prepare:** derive combat roles and effective strength from data-defined rules. Preserve actual headcounts separately from computed effectiveness.
3. **Resolve:** advance bounded automatic formation rounds, ordered tactics, activations, reactions and morale through the deterministic resolver.
4. **Conclude:** determine victory, defeat, withdrawal, rout, or stalemate. Each outcome needs a defined control effect.
5. **Apply:** update casualties, surviving formations, character consequences, locations, node damage, and control atomically.
6. **Record:** issue battle events for experience, recognition, intelligence, and history. Avoid duplicate credit when reopening a report or loading a save.

These stages summarize the implemented integration. The recorded event sequence supports playback, inspection and history without recomputing an outcome.

## Casualties and character survival

Formation losses persist between battles; zero headcount destroys the formation. Preserving veterans matters because destruction ends its military continuity and accumulated advantage.

V1 avoids per-hit death checks for named people. If their containing formation survives, they generally survive. If it is wiped out, they may die. This is a simple initial rule, not immunity from all later illness, old age, or major events.

Wounds, recovery and “commander wounded / assumed command” consequences are implemented. Formation destruction can kill the attached person or leave a wounded survivor in an available surviving formation or legal refuge. No refuge is an explicit lethal outcome. Captivity and a broader injury/disease simulation remain future scope.

Probabilities, wound duration and commander-loss thresholds are current combat data. [Person combat](../src/engine/person_combat.rs) applies them once after casualties and retreat. Each formation has at most one attached named person; old multiple-person examples are superseded by that roster rule. Meaningful careers remain a balance criterion.

## Retreat and rout

A surviving army can retreat to a connected node. This preserves experienced forces, recurring enemies, and accumulated relationships. Defenders under siege may attempt escape, which can succeed or fail.

**Implemented retreat:** destinations are adjacent, uncontested, free of active threats/sieges and hostile armies, and either uncontrolled or held by the retreating faction. The resolver prefers its eligible preferred destination, then supplied sites, then stable ID. Battle consequences consume participants' movement. A no-route outcome is explicit. [Retreat rules](../src/engine/retreat.rs) are shared with person refuges and siege contexts.

Planned map feedback should show the battle/retreat location, changed force state and relevant control consequence after returning from playback. Additional voluntary field-withdrawal or captivity systems are future scope, not prerequisites for the map plan.

## Persistent siege

**Implemented direction (O13):** attacking a fortified location can establish a persistent siege. Defensive advantage weakens with seasonal progress. Assault, sortie, escape, withdrawal and joint relief use current siege commands and [siege_rules.json](../assets/data/siege_rules.json). Their balance is adjustable without reopening their implementation as an approval question.

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

Fort damage and local consequences already persist. Disease, detailed consumable supplies and narrative negotiation remain future depth. Relief is part of the implemented light siege system.

## Siege state contract

A siege refers to the fortified node, participating factions and armies, start date, accumulated progress, current defensive advantage, and relevant damage. It remains identifiable between turns and saves.

Implemented transition outline:

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

Joint relief combines eligible relieving and breakout forces in the defined encounter context. Participating armies keep their identities and six-slot formations, and results apply casualties and retreat per participant. [Siege commands](../src/engine/siege/commands.rs) and [combat context](../src/engine/combat/context.rs) define eligibility; preserve those contracts when displaying region-level siege warnings.

## Local threats and peace-time combat

Initial local threats are human bandits, ordinary wildlife, and occupied ruins. Clearing them gives participation experience, formation veterancy, recognition, resources, or settlement opportunities. Monster nests and dragon-related rewards are future content under O24.

Source examples, including deferred fantasy content:

- A wyvern nest near Frostmarch is cleared. A veteran formation gains experience, a character distinguishes herself, and a recovered dragon egg creates a possible Dragon Knight path.
- A cleared Bandit Hold can become an outpost.
- A destroyed monster nest can expose a rare resource site.
- Defeating bandits in a ruined fort can permit reclamation.

These are source event chains, including future fantasy examples. Current ordinary threats keep persistent identities, active/cleared state and recorded outcomes; reopening a result cannot repeat rewards. Threat progression uses the implemented conditions and tuning in [threats.json](../assets/data/threats.json).

## Damage beyond the battle

War may burn farms and fields, damage settlements, sabotage roads, destroy outposts, and weaken fortifications. Population losses, refugees, supply disruption, and unrest feed [living places](07-living-places.md). A contested border should visibly diverge from peaceful heartland.

The casualty report should separate military losses from lasting local effects. Map smoke, damage and occupation cues are planned representations of actual state, not evidence that a new raid or scorched-earth command exists. Adding such commands is outside this presentation plan.

## Reports and acceptance criteria

A battle report should explain the result with known contributing factors, surviving headcounts, destroyed formations, character consequences, retreat destination, control change, and noteworthy experiences. Keep full calculation detail optional. Record sufficient facts for later biographies.

Acceptance should cover an ordinary encounter, a survivable defeat and retreat, a formation wipeout, a persistent siege and assault, and relief or escape with several armies. Add distinct coverage only where those cases cannot safely cover a necessary rule. Save/load must not repeat casualties or rewards.
