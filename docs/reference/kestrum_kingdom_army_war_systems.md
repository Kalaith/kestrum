# Kestrum: Kingdom, Army & War Systems

This document captures the current design decisions for Kestrum's kingdom layer, army structure, combat, expansion, economy, AI, and troop-to-character progression.

It is intended to sit alongside the core node/character design and the living-world/generational design. These documents may later be merged into a single full GDD.

---

# High-Level Direction

Kestrum is a generational, node-based war simulation about watching history emerge through:

1. war and territorial strategy
2. characters developing through experience
3. settlements changing over time
4. kingdoms growing, competing, and surviving
5. families and succession supporting the longer timeline

The main emotional goal is to look back over decades and recognise how wars, people, armies, and places have changed.

Key inspirations include:

- **Stellaris** for world scale and competing powers growing from small starts
- **Total War: Warhammer** for sequential faction turns, armies, and light siege concepts
- **Suikoden** for named characters acting as figureheads within larger military forces
- **Unicorn Overlord** for army composition and automatic battle resolution

Kestrum should not become a heavy city builder, political simulator, or manual tactical battle game.

---

# Campaign Start

The player begins at the formation of a new kingdom. Other AI-controlled kingdoms are also forming around the world.

Future multiplayer may allow human players to control additional kingdoms, but the initial design is single-player.

Initial player setup is deliberately simple:

- kingdom name
- emblem
- number of rival factions

Recommended faction count:

- minimum: 4
- maximum: 8

More elaborate origins, traits, cultures, or starting bonuses may be added later. For the initial version, kingdom identity should mostly emerge through play.

---

# Victory Condition

The initial victory condition is conquest-based.

The player wins when every other faction has either:

- been conquered
- been vassalized

Additional victory conditions may be added later.

---

# Strategic Map Scale

The world map should be similar in conceptual scale to a small Stellaris galaxy.

The top-level map contains major strategic regions. These regions act somewhat like star systems in Stellaris.

Inside a major region may be a smaller sub-map containing local strategic nodes.

```text
WORLD MAP
    |
    +-- Region
    |      |
    |      +-- City
    |      +-- Fort
    |      +-- Plains
    |      +-- Crossing
    |      +-- Shrine
    |
    +-- Region
    |
    +-- Region
```

Top-level nodes are fixed at world generation.

Subnodes may also be generated at world generation and then remain fixed. Their state can change dramatically over time, but the graph itself should generally remain stable.

---

# Fog of War and Information

The player has little information about rival factions.

Enemy army details should not be freely inspectable. Information is mainly gained by:

- encountering enemy armies
- fighting them
- observing territory
- possibly later through spies or scouting systems

A newly encountered force may initially reveal little more than its presence and rough size.

After combat, the player may learn:

- troop types
- army composition
- commander identity
- important named characters
- approximate strength

Future intelligence systems may expand this.

---

# Kingdom Pressure

A player who stays small is not punished by an artificial timer. The pressure comes from rival kingdoms continuing to grow.

If the player does not expand:

- rivals gain territory
- rivals gain resources
- rivals produce more armies
- rivals develop settlements
- rivals generate stronger characters

The strategic pressure is simple:

> Grow enough that your kingdom can continue to exist.

---

# Economy

The initial economy should remain simple.

Primary resources:

- Gold
- Wood
- Stone

These resources support:

- recruitment
- army upkeep
- settlement development
- roads
- outposts
- fortification
- specialised facilities

Population may also act as a soft limit on military growth.

The economy supports war and development rather than becoming the main game.

---

# Settlement Development

Settlement management stays light.

A settlement may have one active development focus instead of a large building grid.

Possible focuses:

- growth
- fortification
- troop training
- gold production
- wood production
- stone production

More advanced troop types can require specific infrastructure.

Examples:

- cavalry requires horses and appropriate facilities
- mages require an arcane institution
- clerics require a temple or religious centre
- Dragon Knights require dragon-related infrastructure such as a hatchery

The player influences development without micromanaging individual buildings.

---

# Resource-Based Military Unlocks

Kestrum should avoid a conventional abstract technology tree where possible.

Advanced options become available because the kingdom actually possesses the required infrastructure, resources, experience, and characters.

Example:

```text
Dragon Knight

Requires:
- suitable named character
- dragon-related experience
- dragon-related opportunity
- access to a Dragon Hatchery
- appropriate martial/riding background
```

If the kingdom loses its only Dragon Hatchery, training new Dragon Knights becomes impossible until another is built or captured.

Military capability therefore exists physically in the world.

---

# Turn Structure

Kestrum uses sequential faction turns inspired by Total War.

```text
PLAYER TURN
    ↓
NPC FACTION 1 TURN
    ↓
NPC FACTION 2 TURN
    ↓
NPC FACTION 3 TURN
    ↓
...
    ↓
NEXT PLAYER TURN
```

Each faction completes its actions before the next faction acts.

---

# AI Rules

AI factions should broadly obey the same simulation rules as the player.

AI kingdoms use:

- the same resources
- the same troop requirements
- the same settlement systems
- the same movement rules
- the same supply rules
- the same character development systems
- the same class requirements
- the same siege rules

Difficulty should not normally create impossible units or bypass world requirements.

Higher difficulty may instead increase values such as:

- gold income
- resource income
- recruitment efficiency

The simulation should remain believable.

---

# Armies

A faction should field relatively few armies, each with a strong identity.

The player should recognise armies by:

- history
- commander
- troop composition
- veteran status
- battles fought
- relationships between characters

Kestrum should favour a handful of meaningful armies over dozens of disposable stacks.

---

# Army Structure

Each army contains six troop slots.

Example:

```text
Army: Briar Host

1. Warriors
2. Archers
3. Riders
4. Clerics
5. Squires
6. Siege Engines
```

Each slot contains a troop formation. Headcount depends on troop type.

Examples:

```text
Warriors        100 / 100
Archers          80 / 80
Heavy Cavalry    40 / 40
Siege Engines    20 / 20
Wyverns           3 / 3
Dragon             1 / 1
```

Large and powerful troop types contain fewer individuals.

A single dragon may occupy the same conceptual slot as one hundred infantry.

---

# Named Characters and Armies

Armies do not require named characters to exist.

An army may continue operating if its leader dies or retires. However, an army without a named leader is substantially weaker.

Current working direction:

> An army without a named character may operate at roughly 50% effectiveness.

Exact values are tuning targets, not final rules.

Named characters are force multipliers rather than the army itself.

An army may contain more than one named character. Two strong named characters in one army should outperform an otherwise identical army with only one.

This creates a central strategic choice:

```text
Concentrate heroes:
One exceptionally strong army

OR

Split heroes:
Two weaker armies able to cover multiple fronts
```

---

# Character Emergence from Troop Formations

Most future heroes begin inside ordinary troop formations.

Example:

```text
Warriors
100 / 100
```

After relevant experiences, a person inside that formation may distinguish themselves.

The slot might become:

```text
Squire Elian + Warriors
```

Elian has now emerged as a distinct character.

He was not necessarily fully simulated before this moment. The game creates him when the simulation decides somebody in the formation has become noteworthy.

---

# Retrospective Character Grounding

When a character emerges, the game may create a plausible recent history based on:

- troop formation history
- recruitment location
- recent battles
- army commander
- settlement history
- current age
- faction culture

This should never create impossible history.

Example:

Elian is 19.

His troop formation was recruited in Frostmarch three years ago and fought at Redplain and Hawthorn.

A plausible generated history might be:

> Elian of Frostmarch joined the Frostmarch Warriors in Year 22. He fought at Redplain in Year 23 and distinguished himself during the defence of Hawthorn.

The system must never claim Elian fought in a battle fifty years before his birth.

Age and timeline consistency always take priority.

---

# Generated Family Context

When appropriate, character background may borrow from existing world history without requiring every relationship to have been simulated beforehand.

Example:

- Frostmarch was founded by Thomas
- Elian emerges from a Frostmarch troop formation
- timeline and ages make the relationship plausible

The game might create:

> Elian, son of Thomas of Frostmarch

This relationship does not need to have been tracked in detail before Elian emerged.

Generated relationships should remain conservative and chronologically plausible.

---

# Veteran Troop Formations

Troop formations gain their own experience.

A formation can become more effective through repeated survival and combat.

Example:

```text
Warriors
↓
Seasoned Warriors
↓
Veteran Warriors
```

Veterancy may provide restrained benefits such as:

- combat effectiveness
- morale
- resilience
- discipline
- reduced rout chance

---

# Veteran Units and Hero Emergence

Veteran formations are more likely to produce emerging named characters.

```text
Surviving formation
    ↓
Veterancy
    ↓
More history
    ↓
Greater chance somebody distinguishes themselves
    ↓
Named character emerges
```

New heroes therefore tend to emerge from meaningful military history rather than appearing randomly.

---

# Dynamic Hero Emergence Rate

Named characters should appear semi-rarely.

The emergence rate should depend partly on how many named characters a faction already has.

Example philosophy:

```text
Faction has 1 named character:
A new hero may emerge after only a few meaningful fights.

Faction has 10 named characters:
Emergence becomes less frequent.

Faction has 20 named characters:
A new hero may require an entire major war.

Faction near 30 named characters:
New emergence becomes rare.
```

The current target is roughly 30 important named characters at most.

This should be implemented as a soft probability curve rather than a rigid cap where possible.

---

# Troop Type Evolution

Troop formations may develop into specialised types through repeated use.

Examples:

```text
Warriors
    ↓
Repeated fortress defence
    ↓
Shield Guard
```

```text
Warriors
    ↓
Repeated fighting against cavalry
    ↓
Pikemen
```

```text
Riders
    ↓
Extensive scouting and pursuit
    ↓
Light Cavalry
```

Experience creates eligibility, but the kingdom may still need the physical resources or facilities required to support the new troop type.

---

# Unit Destruction

Troop formations use headcount and take losses over multiple battles.

Example:

```text
Warriors
100 / 100

After battle:

Warriors
72 / 100
```

A formation reduced to zero headcount is destroyed.

Its formation history is lost.

A future formation of the same troop type is a new formation, not a continuation of the destroyed unit.

This makes veteran formations worth preserving.

---

# Named Character Survival

Individual named characters should not be checked for death on every hit.

Character consequences matter at significant moments.

Current simple rule:

> If the troop formation containing the character is completely wiped out, the named character may die.

If the formation survives, the character generally survives.

Later systems may include:

- wounds
- captures
- heroic escapes
- rare deaths during major events

V1 should avoid excessive random character loss.

---

# Army Composition

The player controls full army composition.

Before combat, the player decides which six troop formations make up an army.

This includes:

- troop type
- named character placement
- support troops
- specialist formations
- siege equipment
- magical troops

Composition is one of the player's primary tactical decisions.

Combat itself does not require manual tactical control.

---

# Army Bonds

Troops and characters that serve together may develop bonds.

For now, bond effects should remain subtle.

Possible later effects include:

- morale
- reduced rout chance
- improved support behaviour
- better recovery
- greater chance of one character saving another
- improved coordination

The bond system should reinforce persistent army identity without becoming a large relationship-management interface.

---

# Splitting Characters Across Armies

A character who emerges inside an army does not have to remain there.

After reaching sufficient recognition or class status, the player may move that character to lead or support another army.

Example:

```text
Briar Host
- Commander Teresa
- Squire Elian
```

Later:

```text
Briar Host
- Commander Teresa

Frostmarch Guard
- Commander Elian
```

This weakens the original concentration of talent but creates another useful army.

This is intentional.

---

# Character Nurturing

The player should shape future characters mostly through circumstances rather than direct stat assignment.

Examples:

To encourage defensive development:

- place the troop unit in a fort
- order it to hold important territory
- expose it to repeated defensive battles
- assign it to a defensive commander

To encourage aggressive development:

- use it in assaults
- send it against difficult positions
- place it in vanguard formations
- give it offensive opportunities

To encourage dragon-related development:

- send it against dragons
- reward the emerging character with a dragon egg
- provide access to a dragon hatchery
- assign an appropriate mentor

The player creates conditions. The simulation produces the person.

---

# Character Direction Events

Character growth is mostly indirect, but occasional explicit choices are allowed.

Example:

> Teresa asks whether she should dedicate herself to hunting dragons or learning to ride one.

The player may provide direction.

However, the more interesting path is often systemic.

Example:

```text
Teresa slays a dragon.
        ↓
The army recovers a dragon egg.
        ↓
The player awards the egg to Teresa.
        ↓
Teresa gains dragon-related opportunities.
        ↓
Dragon Knight becomes possible.
```

World-grounded transformations should be preferred over menu-based class selection.

---

# Movement

Armies have movement speed.

Movement can be affected by:

- troop composition
- roads
- terrain
- siege equipment
- possibly weather later

Exact movement values should be tuned during prototyping.

---

# Roads

Roads improve movement between nodes.

This makes road development strategically valuable.

However:

> Roads help invaders too.

A kingdom with excellent roads can reinforce quickly, but an enemy that breaks through the border may use those same roads.

Roads may be damaged or destroyed, although doing so should be harder than burning fields or destroying temporary infrastructure.

---

# Destructible Terrain and Economic Damage

War should affect the world.

Examples:

- farms can be burned
- fields can be destroyed
- settlements can be damaged
- roads can be sabotaged or degraded
- outposts can be destroyed
- fortifications can be damaged

This connects warfare directly to the living-world system.

---

# Supply Lines

Supply should remain simple.

Armies with a valid supply connection can recover troop headcount and health normally.

Armies cut off from supply:

> Cannot recover health or troop strength normally.

This allows enemies to chip away at isolated forces over time.

The initial system should avoid detailed food inventories or supply-wagon micromanagement.

---

# Outposts

Armies may establish outposts on suitable nodes.

Current working rule:

> Establishing an outpost takes approximately 3 turns.

An outpost can provide:

- territorial presence
- forward recovery point
- future settlement seed
- defensive position
- supply extension

Outposts are persistent world objects rather than temporary RTS camps.

Over time an outpost may become:

```text
Outpost
↓
Hamlet
↓
Village
↓
Fort
↓
Town
```

Or:

```text
Outpost
↓
Abandoned
↓
Ruins
↓
Bandit Hold
```

The player may not know what an outpost will become decades later.

---

# Army Stacking

Multiple friendly armies may occupy the same node.

There is no planned hard stacking limit or direct stacking penalty for v1.

The natural downside is strategic concentration.

If every army is placed on one node, the kingdom may be vulnerable elsewhere.

Example:

```text
All armies concentrated in the east.

Enemies attack:
- northern frontier
- western road
- southern settlement
```

The player must decide whether concentration is worth leaving other territory exposed.

---

# Battle Trigger

Combat occurs when opposing armies occupy the same node.

Example:

```text
Player turn:
Player army moves to Node A.

NPC turn:
Enemy army moves to Node A.

Result:
Combat begins and resolves.
```

Sequential faction turns simplify encounter handling.

---

# Combat Resolution

Combat is primarily automatic.

The player controls army composition and strategic preparation.

When two armies clash, the result is determined from factors such as:

- troop types
- troop headcount
- named characters
- class abilities
- veterancy
- terrain
- fortifications
- bonds
- army condition

The player does not manually control individual soldiers during combat.

The inspiration is closer to Unicorn Overlord than to a direct tactical RPG.

---

# Retreat

Armies do not need to be destroyed in every defeat.

A surviving force may retreat to a connected node.

This helps preserve:

- veteran armies
- named characters
- rivalries
- long-term history

A defender under siege may also attempt to escape.

Escape may succeed or fail.

---

# Sieges

Fortified settlements use a light siege system inspired by Total War.

When an attacking army surrounds a fortified location, a persistent siege begins.

The defender receives a strong defensive advantage at the start.

Each continued round/turn of siege gradually reduces that advantage.

Conceptually:

```text
Turn 1:
Strong walls and full defensive preparation

Turn 2:
Defences weakened

Turn 3:
Supplies strained

Turn 4:
Fortifications damaged

Turn 5:
Defender increasingly vulnerable
```

The defender can:

- attack the besiegers
- wait
- attempt escape
- receive reinforcements

The attacker can:

- maintain the siege
- assault early
- wait for defences to weaken
- withdraw

Exact siege mathematics can be developed later.

---

# Siege Relief

Friendly armies may come to the aid of a besieged settlement.

A relief army can attack the besieger from outside.

The defenders may also attempt to break out.

This creates multi-army siege situations without requiring a separate tactical siege game.

---

# Peace-Time Activity

Peace should remain useful without becoming a second game.

During periods without major wars, the player can:

- develop settlements
- gather resources
- build roads
- establish outposts
- train and reorganise armies
- move characters between armies
- prepare borders
- clear local threats
- nurture future characters

The world should continue changing during peace.

---

# Bandits, Wild Animals, and Local Threats

Non-faction threats may occupy nodes.

Examples:

- bandits
- wolves
- wild beasts
- monster nests
- dangerous ruins

These provide peace-time military activity and can grant:

- combat experience
- veterancy
- character recognition
- resources
- settlement opportunities
- class opportunities

Example:

```text
Wyvern nest appears near Frostmarch.
        ↓
Army clears nest.
        ↓
Veteran troop gains experience.
        ↓
Character distinguishes herself.
        ↓
Dragon egg recovered.
        ↓
Future Dragon Knight path becomes possible.
```

---

# Wild Threats and World Change

Defeating local threats should be able to alter the map.

Examples:

```text
Bandit Hold
↓
Cleared
↓
Outpost
```

```text
Monster Nest
↓
Destroyed
↓
Rare resource site
```

```text
Ruined Fort
↓
Bandits defeated
↓
Reclaimed frontier fort
```

Local combat should feed back into territorial development.

---

# Faction Elimination

For v1:

> A defeated faction is gone.

There are no initial systems for:

- restoration wars
- exiled governments
- claimant factions
- breakaway successor states

These remain future-scope ideas.

---

# Kingdom Splintering

Kingdom fragmentation is future scope.

Possible later systems include:

- civil war
- breakaway provinces
- succession disputes
- rebel kingdoms
- separatist commanders

The initial version should keep factions stable until conquered.

---

# Diplomacy

Initial diplomacy is intentionally simple.

States:

- War
- Peace

Future systems may include:

- alliances
- marriages
- tribute
- vassals
- guarantees
- negotiated borders
- prisoner exchanges

These are not required for the first functional version.

---

# Named Character Count

The game should avoid roster overload.

A rough upper target is approximately 30 important named characters.

Most people remain part of troop formations and population abstractions.

Only people who become important through:

- combat
- leadership
- lineage
- mentorship
- recognition
- exceptional events

need full character simulation.

---

# Current Core Strategic Tensions

## Concentrate or spread

Do I keep Teresa and Elian together in one devastating army?

Or split them so I can defend two fronts?

## Build roads or protect the frontier

Roads make reinforcement easier.

They also make invasion easier.

## Preserve veterans or take risks

Veteran formations are stronger and more likely to produce heroes.

Sending them into dangerous battles may create legends.

It may also erase years of military history.

## Expand or consolidate

An outpost pushes the frontier forward.

But every new border creates another place that may need defending.

## Assault or siege

Attack a strong fort immediately and accept heavy losses?

Or spend several turns weakening it while enemy relief forces gather?

## Promote or retain

Elian is extremely useful as Teresa's subordinate.

Moving him to command another army makes Teresa's force weaker.

But the kingdom gains another competent front-line commander.

---

# Current System Identity

The kingdom layer should remain light enough that the player spends most of their attention on:

- armies
- characters
- fronts
- territory
- changing places

The guiding hierarchy remains:

1. Watching history emerge
2. War and territorial strategy
3. Developing characters through opportunities
4. Growing a kingdom
5. Dynasty and family systems

New systems should be judged by whether they reinforce this hierarchy.

---

# Prototype Loop

The first playable version should prove that this loop is enjoyable:

```text
Recruit troops
↓
Build army composition
↓
Move across node map
↓
Fight
↓
Troops gain experience
↓
Veteran formations emerge
↓
Characters distinguish themselves
↓
Characters improve armies
↓
Expand territory
↓
Establish outposts
↓
Settlements and borders change
↓
Fight larger wars
↓
Repeat across decades
```

If that loop creates memorable armies, characters, and places without requiring a scripted story, the core design is working.

---

# Summary

Kestrum's war system is built around a small number of persistent armies.

Each army contains six troop formations. Those formations have real headcounts, gain veterancy, specialise through experience, and may produce future named characters.

Named characters strengthen armies but are not required for armies to exist.

The player develops characters indirectly by placing troops in situations that encourage particular forms of growth.

Roads, outposts, supply lines, sieges, and settlement infrastructure make geography strategically meaningful.

AI factions follow the same simulation rules.

Over time, anonymous troops become veterans, veterans produce heroes, heroes lead new armies, outposts become cities, and old battlefields become part of the kingdom's history.

Kestrum should make war feel less like moving disposable counters and more like watching institutions, places, and people accumulate history.
