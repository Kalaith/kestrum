# Kestrum: Living World & Generational Design

## Purpose

Kestrum’s world should not feel like a static board on which armies move.

The map itself should have a history.

Settlements should grow, decline, burn, recover, change purpose, and sometimes outlive the people who founded them. Characters should age, retire, die, mentor successors, create families, and leave behind reputations that shape later generations.

The intended effect is that after several decades, the player can look at the world and recognise not just territory, but history.

A plain where the campaign began may now be a capital.

A once-important fortress may be ruins.

A famous knight may be dead, while their child, apprentice, or former squire now leads an army.

The campaign should feel less like completing a sequence of missions and more like participating in the history of a country.

---

# Design Pillars

## 1. Places Change

Nodes are persistent locations, not disposable battle markers.

A node may move through states such as:

```text
Plain
  ↓
Outpost
  ↓
Hamlet
  ↓
Village
  ↓
Fortified Town
  ↓
Regional Capital
```

But development is not always upward.

```text
Fortified Town
      ↓
    Sacked
      ↓
     Ruins
      ↓
 Bandit Hold
      ↓
 Reclaimed Fort
```

War leaves scars.

Peace creates growth.

Strategic importance changes over time.

---

## 2. People Change

Characters age alongside the world.

A squire in Year 3 might become:

- a knight in Year 10
- a commander in Year 20
- a retired governor in Year 35

or they may never survive that long.

Characters should have careers rather than static levels.

---

## 3. History Continues Through Succession

When a major character leaves the world, the game does not simply replace them with a randomly generated equivalent.

Their role may continue through:

- children
- siblings
- apprentices
- squires
- political heirs
- adopted successors
- religious disciples
- trusted officers

Legacy therefore matters even when bloodline does not.

---

## 4. Long Wars Are Made of Smaller Campaigns

A conflict may last decades without armies fighting continuously for thirty years.

Wars should contain:

- offensives
- stalemates
- winter pauses
- rebuilding periods
- border raids
- political interruptions
- temporary truces
- renewed invasions

This makes long conflicts believable while still giving the player meaningful pacing.

---

# Recommended Time Scale

A useful default is:

## 1 strategic turn = 1 season

This gives:

- 4 turns per year
- 40 turns per decade
- 80 turns across 20 years
- 160 turns across 40 years

That is long enough for characters and settlements to visibly change without making every action consume years.

A campaign might typically span:

**20 to 50 years**

with especially long campaigns potentially reaching:

**60 to 100 years**

The game should support longer histories, but hundreds of years should not be required for normal play.

This avoids the problem where every starting character dies long before the campaign becomes emotionally meaningful.

---

# Seasons

Seasons can matter lightly without becoming a weather simulator.

Possible effects:

## Spring
- easier recruitment
- improved movement
- agricultural recovery

## Summer
- campaigning season
- higher supply production
- increased travel

## Autumn
- harvest
- strong logistics
- preparations for winter

## Winter
- slower movement
- harsher sieges
- increased attrition
- fewer major offensives

This naturally creates rhythm in long wars.

---

# Node States

Every node should have several layers of state.

## Geographic State

The underlying geography rarely changes.

Examples:

- plains
- forest
- coast
- river crossing
- hill
- mountain pass
- valley
- marsh
- island

Geography affects what can develop there.

---

## Settlement State

This represents habitation and development.

Possible stages:

```text
Unsettled
Camp
Outpost
Hamlet
Village
Town
City
Major City
Capital
```

Not every site can or should reach Capital.

---

## Military State

Separate from settlement size.

Examples:

```text
None
Watchpost
Palisade
Fort
Stronghold
Citadel
```

A small settlement may be heavily fortified.

A large trade city may be poorly defended.

---

## Civil Identity

Over time, places gain character.

Possible identities:

- agricultural
- military
- religious
- commercial
- administrative
- arcane
- industrial
- academic
- frontier
- criminal

These influence what the node provides.

---

## Condition

Examples:

- prosperous
- stable
- strained
- occupied
- damaged
- devastated
- abandoned
- ruined
- lawless

Condition can change rapidly during war.

---

# Node Growth

Growth should mostly come from conditions rather than upgrade buttons.

The player creates favourable circumstances.

The simulation determines the result.

A settlement might grow because it has:

- safe roads
- surplus food
- nearby population
- military protection
- political importance
- trade connections
- religious importance
- long-term stability

The player can influence this through high-level actions.

Examples:

- establish outpost
- fortify location
- encourage settlement
- invest in trade
- move administration
- establish temple
- secure road
- resettle refugees

The player should not manually place houses and blacksmiths.

Kestrum is not a city builder.

---

# Development Pressure

Each node can accumulate abstract development pressure.

Example factors:

```text
Safety             +++
Trade Access        ++
Food Supply         ++
Population          +
Administrative Role ++
War Damage          ---
Isolation           --
Raiding             ---
```

When conditions remain favourable long enough, the node can evolve.

This allows development to feel earned rather than purchased.

---

# Example Growth Story

Year 1:

```text
Red Plain
Plains
Unsettled
```

Year 4:

```text
Red Plain Outpost
Military frontier site
```

Year 9:

```text
Red Plain Hamlet
Population growing around the garrison
```

Year 16:

```text
Redplain
Fortified town
Regional supply centre
```

Year 29:

```text
Redplain City
Major military and trade centre
```

Year 42:

```text
Redplain
Capital of the Northern March
```

This progression happened because the frontier moved outward and Redplain became increasingly safe and central.

---

# Decline

Growth must be reversible.

Reasons for decline may include:

- repeated attacks
- loss of trade routes
- famine
- plague
- depopulation
- political abandonment
- relocation of the frontier
- destruction of infrastructure

Example:

```text
Hawthorn City
     ↓
Besieged for three years
     ↓
Population collapses
     ↓
Hawthorn Ruins
     ↓
Bandits occupy outer district
```

Later:

```text
Hawthorn Ruins
     ↓
Military expedition clears bandits
     ↓
Hawthorn Hold
     ↓
Slow resettlement
```

The rebuilt version should retain its history.

---

# Ruins

Ruins should remain meaningful.

They can become:

- bandit bases
- monster dens
- military camps
- archaeological sites
- memorials
- religious sites
- reclaimed settlements

A ruined city should not simply disappear from the map.

Its past is part of the world.

---

# Historical Memory of Places

Each important node can maintain a compact chronology.

Example:

```text
Hawthorn Gate

Year 6
Outpost founded by Captain Serai.

Year 11
Expanded into permanent fortress.

Year 17
Site of the First Hawthorn Siege.

Year 18
Tomas assumed command after Serai was wounded.

Year 32
Settlement expanded around the fort.

Year 41
Sacked by the Eastern Host.

Year 47
Reclaimed by the Crown.
```

This should be generated automatically.

---

# Renaming

Names can evolve.

For example:

```text
Red Plain
↓
Red Plain Outpost
↓
Redplain
↓
Redplain Citadel
```

Some places may be renamed after:

- famous commanders
- battles
- noble families
- religious events
- faction changes

A conqueror might rename a city, while locals still remember the old name.

---

# Capitals

Capital status should be dynamic.

The starting capital does not need to remain the permanent capital.

A new capital may emerge because it becomes:

- safer
- wealthier
- more central
- politically important
- better connected

A player may eventually move the seat of government.

This can produce a strong historical contrast:

> The original village still exists, but the empire is now ruled from the fortress that once guarded its eastern frontier.

---

# War Damage

Battles should create local consequences.

A node may accumulate:

- structural damage
- population loss
- supply disruption
- damaged roads
- destroyed fortifications
- refugee pressure
- unrest

Repeated warfare should visibly degrade a region.

A border contested for twenty years should look different from untouched heartland.

---

# Occupation

Taking a node does not instantly make it culturally stable.

A newly occupied settlement may suffer:

- unrest
- resistance
- low supply
- sabotage
- population flight

Long-term occupation can gradually normalise control.

This gives conquest a cost beyond winning one battle.

---

# Refugees and Population Movement

Major wars can move people between nodes.

A devastated settlement may push population into nearby safe areas.

This can cause:

- growth in neighbouring cities
- labour shortages
- unrest
- cultural mixing
- new military recruits

A ruined town may indirectly help another city become important.

That gives destruction secondary consequences.

---

# Map Density Over Time

The early world should be visually sparse.

Many nodes can begin as:

- empty land
- crossroads
- woods
- hills
- ancient ruins
- tiny settlements

As decades pass:

- more roads become important
- new settlements appear
- fortifications rise
- regional capitals emerge
- ruins accumulate

The map gradually becomes denser and more historically layered.

This makes progression visible without relying entirely on numbers.

---

# Aging

Characters should have biological and career age.

Example life stages:

## Childhood
Not generally active in military rosters.

## Youth
Possible:
- page
- novice
- apprentice
- trainee

## Young Adult
Main entry point for:
- squire
- recruit
- neophyte
- apprentice mage

## Prime
Typical period of strongest military service.

## Veteran
Greater skill and reputation, potentially reduced physical ability.

## Elder
Often:
- commander
- mentor
- governor
- priest
- strategist
- retired figure

---

# Approximate Age Ranges

These should remain flexible.

Example:

```text
0–12   Child
13–16  Page / Novice / Apprentice
17–25  Young recruit
26–40  Prime
41–55  Veteran
56+    Elder
```

Fantasy settings can adjust this depending on culture and species.

---

# Aging Effects

Aging should not simply be:

```text
Age 45
-2 Strength
```

Instead, the impact should be role-based.

Older characters may lose:

- endurance
- recovery
- mobility

but gain:

- command
- judgement
- mentorship
- political influence
- reputation
- training ability

This makes aging transformation rather than pure decline.

---

# Retirement

Characters should sometimes retire before death.

Reasons may include:

- age
- injury
- personal choice
- political appointment
- religious calling
- family obligations

Retired characters can remain useful as:

- governors
- trainers
- advisers
- mentors
- diplomats

This keeps important figures in the world after frontline service ends.

---

# Death

Death should be possible from:

- combat
- illness
- age
- major events

But characters should not die so frequently that long-term attachment becomes impossible.

The system should favour meaningful careers rather than constant replacement.

---

# Injury

Permanent or long-term injury can affect career direction.

Example:

A knight suffers a severe leg injury.

They may no longer be ideal cavalry.

They might become:

- commander
- trainer
- governor
- tactician

This can create new character paths rather than simply making them useless.

---

# Families

Important characters may form households and families.

This should emerge through:

- proximity
- relationships
- shared service
- social compatibility
- politics
- player encouragement

The player may influence relationships without directly controlling every pairing.

---

# Deliberate Pairing

The player may have limited ability to encourage or arrange partnerships.

This can serve:

- political alliances
- family continuity
- succession
- faction stability

But it should never reduce characters to breeding statistics.

The interesting question should be:

> What does this relationship mean for these people and the faction?

not:

> Which two characters maximise offspring strength?

---

# Children

Children should inherit context more strongly than raw stats.

They may inherit:

- surname
- social standing
- family reputation
- access to mentors
- political obligations
- property
- cultural background

They may also inherit mild aptitude tendencies.

But their actual identity still emerges through experience.

---

# Example

Two famous cavalry officers have a child.

That child might receive:

- easy access to horses
- cavalry mentors
- prestige
- expectations

But they may eventually become:

- priest
- infantry commander
- merchant
- mage
- politician

The family creates opportunity, not destiny.

---

# Heirs

Succession can operate through multiple systems.

## Blood Heir

A child or close family member.

## Martial Heir

A squire or officer trained by the character.

## Religious Heir

A disciple or junior cleric.

## Political Heir

A trusted administrator or appointed successor.

## Adopted Heir

A ward or orphan incorporated into the household.

This allows legacy without forcing every famous character to reproduce.

---

# Mentorship

Mentorship should be one of the major bridges between generations.

An aging hero may train junior characters.

Mentorship transfers:

- techniques
- class access
- traditions
- doctrine
- reputation
- relationships

This makes retired or aging characters strategically useful.

---

# Character Legacy

When a character dies or retires, they may leave:

- family
- students
- named equipment
- titles
- political consequences
- memorials
- military traditions
- class unlocks
- settlement history

A character can therefore remain relevant after death.

---

# Named Equipment

Equipment can also become historical.

Example:

```text
Serai's Spear
Used at Hawthorn Gate
Passed to Tomas in Year 22
```

Later generations may treat the weapon as:

- heirloom
- relic
- symbol of command

This reinforces continuity.

---

# Settlements and Families

Families may become tied to particular places.

Example:

```text
House Hawthorn
Founded by Tomas of Hawthorn
Seat: Hawthorn Keep
```

If the keep later becomes a city, the family may rise with it.

If the city falls, the family might become refugees, exiles, or claimants.

Places and families can therefore evolve together.

---

# Generational Recruitment

Later generations should not simply spawn at recruitment screens.

Young characters can emerge from:

- military families
- temple communities
- settlements
- apprenticeships
- refugee populations
- noble houses

This allows the roster to reflect the world.

---

# Roster Control

A generational system can explode in size, so the game needs limits.

Only a small proportion of people should become fully simulated named characters.

Most people remain abstract population.

Characters receive deeper simulation when they become relevant through:

- lineage
- military service
- mentorship
- recognition
- politics

This keeps the system manageable.

---

# Character Promotion to Simulation

A child may initially exist simply as:

```text
Child of Serai and Tomas
Age 8
```

At age 15:

```text
Eligible for training
```

If they enter service:

```text
Elara
Page
```

Now they become a fully simulated character.

The game does not need detailed simulation for every child from birth.

---

# Multi-Generational War

Wars can survive individuals.

Example:

## Year 3
The First Rose War begins.

## Year 11
Temporary truce.

## Year 19
Border conflict resumes.

## Year 28
Original commanders are aging.

## Year 34
Second generation begins leading armies.

## Year 41
The war finally ends.

That feels much more believable than one army spending 38 continuous years attacking the same castle.

---

# Sieges

Major sieges can last multiple seasons or years.

A siege becomes a persistent state.

Effects may include:

- supply consumption
- disease
- civilian suffering
- reinforcements
- relief armies
- fort damage
- negotiation

The player decides whether continuing the siege is worth the cost.

---

# Long-Term Fronts

A border may remain contested for decades.

The actual frontline shifts over time.

Example:

```text
Year 5
Border follows River A.

Year 12
Player captures Hawthorn.

Year 18
Enemy retakes northern forts.

Year 24
Front stabilises around Redplain.

Year 31
Player launches second eastern campaign.
```

This naturally creates famous regions.

---

# Historical Eras

The game can divide long campaigns into informal eras.

These do not need to be scripted chapters.

Examples:

- Founding Years
- First Eastern War
- Long Peace
- Plague Years
- War of Three Blooms
- Second Expansion

Era names can be generated from major events.

This helps the player mentally organise long histories.

---

# Generational Tone

The game should occasionally remind the player how much time has passed.

Examples:

> Hawthorn has now stood for 25 years.

> Mira enters her 30th year of service.

> The children born during the First Siege are now reaching military age.

> Few living soldiers remember when Redplain was only an outpost.

These small touches reinforce scale.

---

# Example 50-Year Campaign

## Years 1–5

The player controls:

- one settlement
- one small fort
- sparse surrounding nodes

Characters:
- 3 named founders
- 6 junior followers

Red Plain becomes an outpost.

---

## Years 6–15

First major war.

Hawthorn Gate becomes fortified.

Several squires become named characters.

A plague creates the first Plague Cleric.

Children begin appearing among important families.

---

## Years 16–25

The frontier expands.

Red Plain becomes Redplain Town.

Founding characters begin reaching veteran age.

Several younger characters enter service.

One original commander dies during a siege.

Their squire inherits command.

---

## Years 26–35

Second-generation officers now lead units.

Hawthorn is sacked.

Refugees strengthen Redplain.

The capital begins losing strategic importance.

---

## Years 36–45

Redplain becomes a major city.

The government relocates there.

An enemy commander who first appeared as a young squire decades earlier becomes the central military rival.

---

## Years 46–50

The original founders are mostly retired or dead.

Their descendants, students, and political heirs dominate the world.

The map bears visible traces of fifty years of war.

The campaign ends with a world that looks nothing like the one that existed at the start.

---

# Interaction With Character Development

The long time scale makes the existing character system stronger.

A squire can now genuinely have a career:

```text
Age 17
Squire

Age 22
Knight

Age 29
Captain

Age 38
Named Commander

Age 49
Marshal

Age 56
Governor and Mentor
```

Their life becomes a story rather than a level curve.

---

# Interaction With Nested Maps

Nested regions should also change.

A region initially containing:

```text
Plains
Village
Fort
```

may decades later contain:

```text
City
Citadel
Market Town
Monastery
Ruined Fort
New Road
```

The internal campaign map therefore evolves with the world map.

This is important.

Nested regions must not become static tactical boards frozen at campaign start.

---

# Interaction With Class Development

Age and experience can influence class progression.

Younger characters may have greater access to:

- physically demanding classes
- apprenticeships
- long training paths

Older characters may naturally transition toward:

- commander
- strategist
- priest
- mentor
- governor

This creates meaningful career arcs.

---

# Important Design Constraint

Do not make aging punitive.

The player should not think:

> This character became old, so now they are bad.

They should think:

> This character's role has changed.

The knight who once led charges may become the person training the next generation.

That transformation is the entire point.

---

# Important Design Constraint

Do not make reproduction mandatory.

A character should be able to leave a meaningful legacy without children.

Legacy through mentorship should be equally valuable.

This prevents the game from becoming excessively dynasty-focused unless the player wants it to be.

---

# Important Design Constraint

Do not make every node grow automatically.

Some places should remain:

- tiny villages
- remote shrines
- empty wilderness
- ruins

The world becomes interesting through uneven development.

Not every dot should eventually become a metropolis.

---

# Important Design Constraint

History should be visible without drowning the player in logs.

The game should surface history contextually.

When selecting Hawthorn:

> Founded Year 6  
> Survived 3 sieges  
> Last captured Year 41

When selecting Tomas:

> Served at Hawthorn Gate  
> Mentor to 4 knights  
> Founder of House Hawthorn

The player can dig deeper if desired.

---

# Core Loop Across Decades

At the largest scale:

```text
Explore
↓
Establish
↓
Defend
↓
Develop
↓
Expand
↓
Fight
↓
Rebuild
↓
Age
↓
Succeed
↓
Repeat
```

The world should never return completely to zero.

Every cycle leaves something behind.

---

# Core Fantasy

The player should eventually be able to look at a late-game map and think:

> That city used to be empty plains.

> That ruined fortress held the border for twenty years.

> That commander was trained by one of my original knights.

> The enemy king's father fought me in the first war.

> This whole region exists because of decisions made thirty years ago.

That is the experience the changing-map and aging systems should serve.

## Summary

Kestrum’s world operates on three parallel timelines:

**People age.**

**Places evolve.**

**Wars leave history behind.**

The player is not merely conquering a map.

They are watching a civilisation accumulate.
