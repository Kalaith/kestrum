# Kestrum

## High Concept

**Kestrum** is a node-based war simulation focused on emergent campaigns and emergent characters.

The player begins from a headquarters and expands across a world represented as connected strategic nodes. Armies move between cities, forts, settlements, passes, and other locations.

Some important locations are not single nodes. They contain their own internal network of nodes, creating a second strategic scale for invasions, defence, and prolonged campaigns.

Armies are built from small units containing persistent characters. Most characters begin as ordinary squires, recruits, neophytes, or apprentices rather than finished heroes.

Through battle, assignments, mentorship, equipment, resources, success, failure, and survival, these ordinary characters gradually develop identities, abilities, classes, reputations, and histories.

The same systems apply to enemy factions.

Kestrum is therefore not about following a fixed cast through a scripted war.

It is about watching a war create its own cast.

---

# Design Pillars

## 1. War Happens Across Connected Spaces

The world is represented through strategic nodes rather than free movement.

Positioning, routes, chokepoints, reinforcement paths, and ownership matter.

The player must consider where armies are committed rather than simply whether an army is strong enough to win a battle.

---

## 2. Important Locations Become Campaigns

Major regions can contain their own internal node maps.

Invading a province, city, fortress complex, or strategically important location may therefore involve several connected engagements rather than one battle.

A world-map node can become an entire local front.

---

## 3. Characters Are Made Through Play

Most characters are not recruited as complete personalities or predefined classes.

The player gives characters opportunities.

Their experiences gradually shape:

- capabilities
- class options
- personality
- reputation
- relationships
- specialisations
- historical importance

The player influences who someone becomes without directly selecting every outcome from a menu.

---

## 4. Heroes Emerge From Ordinary People

Named characters represent people who have become significant through events.

A legendary enemy commander may once have been an unknown squire.

A famous plague cleric may once have been an ordinary temple neophyte assigned to a diseased settlement.

Heroic status is an outcome of the simulation rather than the starting state.

---

## 5. Enemy Factions Follow Similar Rules

Enemy characters can:

- develop classes
- gain experience
- survive defeats
- become commanders
- form reputations
- develop relationships
- become famous
- become recurring opponents

The player should be able to encounter someone early in the campaign as an insignificant enemy and later recognise them as a major figure.

---

# Strategic World

## World Map

The main game world is represented as a network of connected strategic nodes.

Example:

```text
                  Northern Fort
                       |
Capital ---- Farmland ---- Rosemarch ---- Eastern Pass
                       |
                  River Settlement
```

Nodes can represent:

- cities
- villages
- forts
- castles
- mountain passes
- bridges
- ports
- temples
- resource locations
- headquarters
- regional capitals
- wilderness locations
- strategic positions

Connections represent valid movement routes.

Armies do not freely travel anywhere on the map.

Control of routes therefore becomes strategically important.

---

# Headquarters

The player begins from a headquarters or equivalent core territory.

The headquarters acts as the initial source of:

- military units
- recruits
- supplies
- promotions
- training
- command structure

Expansion radiates outward through connected territory.

The player gradually establishes additional safe locations and logistical positions.

---

# Nested Node Maps

## Concept

Some world nodes represent locations complex enough to contain their own node network.

Example:

```text
WORLD MAP

Player Territory ---- Rosemarch ---- Enemy Territory
                         |
                     Southern Road
```

Opening Rosemarch might reveal:

```text
                   Northern Gate
                        |
                  Hawthorn Fort
                   /          \
            Milltown        Old Road
                |               |
          Western Gate ---- Rosemarch City
                                |
                           Southern Pass
```

Rosemarch exists simultaneously as:

- one strategic location on the world map
- an internal regional campaign map

---

# Regional Warfare

Entering a region does not immediately capture it.

Armies physically enter the internal map.

Multiple friendly and enemy units may therefore operate inside the same region.

Example:

```text
Western Gate
     |
1st Company
     |
Milltown ---- Enemy Scouts
     |
2nd Company
     |
Hawthorn Fort ---- Enemy Army
```

The player may choose to:

- concentrate forces
- split armies
- hold chokepoints
- advance through multiple routes
- bypass positions
- reinforce threatened nodes
- withdraw
- attack supply routes
- defend captured territory

---

# Multiple Fronts

Because armies physically occupy nodes, wars naturally generate fronts.

The player may simultaneously:

- invade an enemy province
- defend a border
- suppress resistance
- reinforce a fortress
- counterattack somewhere else

Example:

```text
WEST

Enemy Army
    |
Ashford
    |
Player HQ


EAST

Player Army
    |
Hawthorn Province
    |
Enemy Capital
```

The player must decide where limited experienced characters and units are most valuable.

---

# Regional Entry Points

Regions may connect to surrounding world nodes through specific internal nodes.

Example:

```text
WORLD

Northern Kingdom
      |
  Rosemarch
   /      \
West     South
```

Internally:

```text
          North Gate
              |
          High Fort
          /       \
   West Gate    Capital
                   |
               South Pass
```

Entering Rosemarch from the north places armies at North Gate.

Entering from the west places armies at West Gate.

Geography therefore affects campaign structure.

---

# Territorial Control

Regions should support partial ownership.

Example:

Rosemarch:

- Western Gate: Player
- Milltown: Player
- High Fort: Contested
- Rosemarch City: Enemy
- Southern Pass: Enemy

A region is therefore not simply:

**PLAYER**

or

**ENEMY**

during an active campaign.

---

# Strategic Anchors

The player does not necessarily need to capture every node.

Important regions may contain strategic anchors.

Example requirements:

```text
Rosemarch Control

Required:
- Rosemarch City
- High Fort
- At least one external supply route
```

Once sufficient strategic anchors are controlled, political ownership may change.

Enemy forces may still remain within isolated areas.

This allows:

- pockets of resistance
- surrounded armies
- guerrilla activity
- later counterattacks
- negotiated withdrawals

without requiring tedious capture of every minor node.

---

# Army Structure

Armies consist of smaller persistent units.

A unit may contain approximately six characters.

Example:

## Briar Company

- Captain Serai
- Brother Edrin
- Squire
- Squire
- Temple Neophyte
- Temple Neophyte

Named and unnamed characters coexist within the same formation.

Early in the game, only a small number of characters may already be significant.

Others develop through service.

---

# Character Philosophy

Characters are not primarily selected from a roster of pre-generated personalities.

The game should avoid systems where recruitment looks like:

```text
Squire Tomas

Aggression: High
Learning: Slow
Courage: 72
Loyalty: 61
```

Instead, the player initially knows relatively little.

Example:

```text
Tomas
Squire
Served: 2 campaigns
```

Personality and capability become clearer through actions and events.

---

# Character Development

Character development is influenced by five broad forces:

## Disposition

Characters may begin with subtle underlying inclinations.

Examples:

- slightly courageous
- naturally cautious
- physically gifted
- empathetic
- ambitious
- curious
- stubborn

These should influence behaviour without determining destiny.

---

## Experience

Characters remember important situations they participate in.

Examples:

- held a gate while outnumbered
- survived a retreat
- fought cavalry repeatedly
- served during a plague
- participated in a siege
- defeated a notable officer
- was wounded protecting another soldier
- survived a dragon attack

Experiences contribute toward future development.

---

## Mentorship

Serving alongside experienced characters affects growth.

A squire assigned to an accomplished cavalry officer may gain:

- riding knowledge
- cavalry familiarity
- confidence
- access to related classes

A neophyte assigned to a battlefield healer may develop differently from one serving within a temple.

---

## Opportunity

The player influences development by creating opportunities.

Examples:

- giving a recruit command responsibilities
- assigning someone to a dangerous front
- providing specialist equipment
- sending a neophyte into a plague-stricken city
- assigning someone to cavalry
- placing them under a particular mentor
- entrusting them with wounded soldiers
- allowing them to participate in a siege

Opportunity replaces direct trait selection.

---

## Consequences

Success and failure both shape characters.

A failed defence may create:

- caution
- determination
- fear
- hatred
- defensive expertise

A successful charge may contribute toward:

- confidence
- aggression
- prestige
- cavalry specialisation

The same event should not always create the same person.

Existing disposition and context influence the result.

---

# Evidence-Based Traits

Traits should emerge from repeated behaviour and experiences.

Example:

Tomas participates in several battles.

During them he:

- holds position against superior forces
- volunteers for a counterattack
- takes command after an officer is wounded
- protects retreating soldiers

Over time the game may recognise traits such as:

- Bold
- Protective
- Natural Commander

These traits were not selected during recruitment.

The player witnessed the behaviour that produced them.

---

# Recognition

Recognition represents the point at which an ordinary character becomes historically important.

Junior characters gradually accumulate recognition through meaningful events.

Possible sources include:

- surviving major battles
- leading troops after an officer falls
- defeating important enemies
- successfully commanding detachments
- saving other characters
- capturing important locations
- participating in famous campaigns
- becoming repeatedly wounded
- surviving disasters
- performing extraordinary healing
- forming notable rivalries
- earning major promotions

---

# Becoming a Named Character

Once sufficient recognition has accumulated, a character may become formally significant.

Example:

```text
Tomas
Squire
```

becomes:

```text
Tomas of Hawthorn
Knight

Known for:
- Holding Hawthorn Gate
- Leading the Milltown Counterattack

Traits:
Bold
Protective
Natural Commander
```

The important distinction is that the player does not decide who Tomas is.

The player helped create the circumstances that revealed who Tomas became.

---

# Named Character Status

Named status can unlock greater simulation importance.

A named character may gain:

- unique portrait treatment
- biography
- command ability
- relationships
- rivalries
- epithets
- political importance
- event involvement
- greater battlefield influence
- historical records

Their death or capture becomes strategically and emotionally meaningful.

---

# Emergent Class Development

Classes should be connected to lived experience rather than primarily level thresholds.

A character may become eligible for a class because their history supports it.

---

# Example: Dragon Knight

Dratanus begins as:

```text
Dratanus
Squire
```

During his career he:

- trains with cavalry
- uses polearms
- serves under a mounted commander
- fights dragonkin
- survives a dragon attack
- receives specialised riding training
- bonds with a drake

Eventually:

```text
Dragon Knight
```

becomes available.

Possible requirements:

```text
Dragon Knight

Foundation:
- Martial training
- Riding experience
- Polearm proficiency

Experience:
- Encountered dragonkin

Plus one:
- Survived dragon attack
- Defeated dragonkin officer
- Bonded with drake

Opportunity:
- Drake access
- Dragon Knight mentor
- Specialist stable
```

The class therefore reflects the character's history.

---

# Example: Plague Cleric

Mira begins as:

```text
Mira
Temple Neophyte
```

She is assigned to a recently conquered city suffering from disease.

Through several events she:

- treats infected civilians
- performs battlefield triage
- learns herbalism
- works with an apothecary
- survives infection
- studies poison and disease

Her progression may become:

```text
Temple Neophyte
        |
   Field Cleric
        |
   Plague Cleric
```

The player did not recruit a Plague Cleric.

The campaign created one.

---

# Resources as Development Opportunities

Character resources should generally create opportunities rather than directly modify abstract attributes.

Instead of:

```text
Book of Courage
+5 Courage
```

the game should favour systems such as:

```text
Warhorse
Provides cavalry experience opportunities
```

```text
Apothecary Access
Provides medicine and disease-related experiences
```

```text
Command Assignment
Provides leadership opportunities
```

```text
Master Spearman Mentor
Provides polearm development
```

Resources shape possible futures rather than immediately defining characters.

---

# Enemy Character Development

Enemy factions use the same broad character-development model.

An enemy character may initially appear as:

```text
Unknown Squire
```

Later:

```text
Dratanus
Knight
```

And eventually:

```text
Dratanus the Ashen
Dragon Knight
Commander of the Eastern Host
```

The player may recognise that this is the same character who escaped an earlier battle.

---

# Emergent Rivals

Enemy characters can become significant partly because of their encounters with the player.

Example:

The player destroys Dratanus's company.

Dratanus survives.

Possible consequences:

- gains survivor experience
- develops hatred toward the player's faction
- becomes more cautious
- gains prestige for surviving
- seeks revenge
- receives promotion

Years later, he may command an invasion against the player.

The campaign has effectively created its own antagonist.

---

# Character Formula

At a high level:

```text
Character Development =
Starting Disposition
+ Experiences
+ Mentorship
+ Opportunities
+ Consequences
```

The goal is to avoid:

```text
Character =
Random Stats Generated At Recruitment
```

---

# Relationship Between Strategic and Character Systems

The strategic map creates the circumstances in which characters develop.

Examples:

## Chokepoint defence

A junior officer repeatedly holds bridge nodes.

Possible development:

- defensive specialist
- disciplined
- shield-focused classes
- command recognition

---

## Deep offensive

A unit spends months advancing far from supply.

Possible development:

- endurance
- independence
- aggressive doctrine
- scouting expertise

---

## Disease-stricken region

Clerics and support characters work under severe medical pressure.

Possible development:

- plague expertise
- triage
- herbalism
- emotional consequences

---

## Encircled army

Characters survive while cut off from friendly territory.

Possible development:

- survival skills
- loyalty
- desperation
- leadership
- trauma
- reputation

The node simulation is therefore not separate from character progression.

It generates character progression.

---

# Campaign Memory

Kestrum should record enough history that important characters feel connected to actual events.

A character biography could eventually contain entries such as:

```text
Year 2
Joined Briar Company as a squire.

Year 3
Fought at Hawthorn Gate.

Year 3
Assumed command after Captain Serai was wounded.

Year 4
Promoted to Knight.

Year 5
Led the defence of Rosemarch.

Year 6
Became known as Tomas of Hawthorn.
```

The biography emerges automatically from gameplay.

---

# Botanical Identity

Kestrum's setting may use floral and botanical references throughout its cultures and military traditions.

This is thematic rather than literal.

Plants are not the combatants.

Instead, flowers and plants influence:

- heraldry
- noble houses
- military orders
- regions
- fortresses
- classes
- titles
- historical conflicts

Examples:

- Roseguard
- Blackthorn Company
- Laurel Captain
- Hawthorn Keep
- Ashmere
- White Rose Gate
- The War of Three Blooms
- The Red Laurel Campaign

Different factions may visually associate themselves with particular botanical symbols.

This gives territory and armies recognisable identities on the strategic map.

---

# Example Campaign Story

The player invades Rosemarch through the western entrance.

Briar Company captures Milltown.

The enemy reinforces Hawthorn Fort.

The player divides their army:

- Captain Serai attacks Hawthorn Fort.
- Tomas and several inexperienced soldiers defend Milltown.
- Mira tends wounded troops behind the front.

An enemy counterattack reaches Milltown.

Tomas successfully holds the node after his commander is wounded.

Mira treats soldiers affected by contaminated weapons.

Later:

Tomas gains leadership recognition.

Mira develops disease-related expertise.

The enemy commander escapes the fall of Hawthorn Fort.

Several campaigns later:

- Tomas has become a recognised knight.
- Mira is progressing toward Plague Cleric.
- the surviving enemy officer has become a major rival.
- Rosemarch remains partially contested.

None of these outcomes were scripted as story missions.

They were produced by the strategic and character simulations interacting.

---

# Early Prototype Scope

The first playable prototype should prove the interaction between the node system and character system.

## World

Create:

- player headquarters
- several simple world nodes
- one nested region
- one enemy headquarters

---

## Nested Region

Create approximately:

- 8 to 12 internal nodes
- 2 entry points
- 1 regional capital
- 1 fortress
- several minor locations

Allow partial ownership.

---

## Armies

Support:

- several six-character units
- movement between connected nodes
- friendly and enemy occupation
- engagements
- retreat
- unit destruction
- reinforcement

---

## Characters

Support initially:

- named characters
- junior characters
- basic classes
- persistent identity
- experience records
- recognition
- limited trait emergence
- class eligibility

---

## Character Events

Prototype a small number of reusable experiences:

- participated in battle
- won battle
- lost battle
- survived while outnumbered
- commander wounded
- assumed command
- treated wounded
- fought specific enemy type
- defended strategic node
- captured strategic node

These should be enough to test whether interesting characters begin emerging naturally.

---

# Prototype Success Criteria

The prototype is successful if, after playing for a while, the player can naturally say things like:

> "I'm keeping this squire with Serai because he's turning into a good cavalry officer."

> "Don't send Mira there. I need her dealing with the outbreak in Rosemarch."

> "That enemy knight again? He survived the last campaign."

> "I can't abandon Hawthorn because Tomas has been holding it for three battles."

At that point the simulation is producing attachment through history rather than authored story.

---

# Current Core Identity

Kestrum is a war simulation where:

**The map creates campaigns.**

**Campaigns create experiences.**

**Experiences create characters.**

**Characters create history.**

The player's story emerges from those systems interacting rather than from a predetermined narrative.