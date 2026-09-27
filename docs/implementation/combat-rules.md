# Provisional combat and siege rules

[Plan](../implementation-plan.md) · [Strategic rules](strategic-rules.md)

**P11–P15 are provisional**, deliberately small explainable models. The source
does not prescribe this mathematics. Implement and test it first; tune it through
recorded playtesting. Sources: [armies](../04-armies-and-logistics.md),
[battles and sieges](../05-battles-and-sieges.md), D01/D05/O05/O06/O13.

## P11 — Participation and strength

An ordinary encounter has two hostile factions. The moving army attacks; other
armies at its origin do not teleport into the encounter. An explicit group move
may include any selected co-located friendly armies that can all afford the edge.
Every army of the defending faction already at the target participates. Friendly
stacks have no cap or hidden stacking penalty. Keep each army and formation's
identity throughout calculation and casualty allocation.

No tactical positioning/minigame or six-person party model. Each side's combat
roster is sorted by army ID then formation slot. Distinguish an empty slot from a
destroyed formation. Siege engines' capacity 20 means twenty engine-and-crew teams,
not twenty named soldiers; headcount loss removes whole teams. Other human types
count people. Medics are support troops, not forty free named characters.

| Formation | Attack per counted element | Resistance | Initial role |
| --- | ---: | ---: | --- |
| Warriors | 10 | 10 | General infantry |
| Spearmen | 9 | 11 | Anti-cavalry infantry |
| Archers | 12 | 7 | Ranged support |
| Riders | 24 | 18 | Fast, concentrated force |
| Medics | 4 | 8 | Support and treatment evidence |
| Siege Engines | 20 | 16 | Assault support |

For a named character to contribute they must be alive, age 17+, fit for field
duty, and attached to a participating formation. Count each once. The army-wide
leadership permille is `500 + floor(1000*n/(n+2))`, where `n` is that count.
Examples: zero gives 500, one 833, two 1000, four 1166; concentration improves
strength with diminishing returns. A valid appointed Officer commander adds 100
permille. No named person is required for legal army operation.

Formation veterancy multiplies attack and resistance by 1000/1100/1200 permille
for ordinary/seasoned/veteran, earned under P16. An earned specialization adds only
the specific P18 modifier. Terrain modifies defender resistance: ordinary 1000,
forest/hill 1100, bridge/pass 1200. Fort/siege advantage is an additional factor
specified in P14, used only when walls protect that side. No numerical bond or
hidden aptitude bonus in this first resolver.

## P12 — Bounded automatic exchanges

Resolve up to eight exchanges. Snapshot living rosters at each exchange start;
calculate all attacks from that snapshot and apply both sides' casualties
simultaneously. Thus an element alive at the start still attacks if killed in the
same exchange. One exchange is not a strategic season.

For source formation index `i`, target `(i + exchange_index) mod enemy_roster_len`
in the enemy's stable living roster. Sum attacks assigned to each target before
computing its losses. No retargeting overkill during the exchange. This is an
automatic targeting baseline, not a player-controlled formation-position system.

Attack contribution uses current source headcount × base attack × leadership ×
veterancy × counter modifier, keeping the permille denominators until one final
division. Counters: Spearmen versus Riders 1500; Riders versus Archers 1250;
Archers versus Spearmen 1250; all others 1000. A specialized modifier replaces or
adds exactly as P18 states. Medics do not resurrect casualties mid-battle.

For each target, effective resistance is base resistance × veterancy × terrain ×
wall factors divided by their permille denominators. Compute losses as
`floor(total_effective_attack / (10 * effective_resistance))`, minimum one when
positive attack was assigned, capped at target headcount. Use fixed rational
integer arithmetic with wide intermediates and specified final rounding; no
floating-point or random damage draws. Reject invalid zero resistance data.

After simultaneous losses, remove zero-headcount formations. If a side has no
surviving formations, it loses. If both are empty, the encounter is a mutual
destruction; an empty site becomes uncontrolled unless surviving site defenders
remain in a siege context. Otherwise a side routs if its remaining base-power sum
(`headcount × base attack`, before temporary modifiers) is at most 40% of its
starting sum. If both meet this condition simultaneously, attacker withdraws and
defender keeps the site. Routed survivors use P13.

After exchange eight, compare remaining base power as a percentage of each side's
own starting power. If one exceeds the other by at least 100 permille, that side
wins; otherwise it is a stalemate and the attacker withdraws. This threshold
comparison uses cross multiplication to avoid different rounding. Defender
control is retained on stalemate. No unresolved infinite encounter state.

All participating formations and people are exhausted for the strategic round,
including defensive survivors. This never prevents defending again. Attackers
cannot repeatedly assault with unused allowance. Apply casualties, withdrawals,
control, person outcomes, events and evidence in one transaction. An encounter
report explains leadership, troop counters, terrain/walls, headcounts and retreat;
it cannot promise a different result because it is reopened or animated.

**Worked arithmetic fixture:** ordinary Warriors 100 with two contributing people
and no Officer bonus have leadership 1000 and attack 1000. Against an ordinary
Warrior target with resistance 10 and no terrain/walls, their assigned first
exchange causes 10 losses. With no named people the same force attacks for 500
and causes 5. In a symmetric fixture both sides lose those casualties at once.

## P13 — Retreat, destruction and limited person consequences

Legal retreats are adjacent, traversable sites that are neutral or controlled by
the retreating faction, unbesieged, free of hostile armies/local threats, and not
the enemy's current attacking origin. Peaceful foreign territory is not access.
For a defeated attacker prefer its valid origin; otherwise choose an eligible
supplied site, then lowest site ID. Defenders use supplied preference then ID.
The engine chooses automatically and shows the destination in the report. Each
army retains its own ID. Peace withdrawal uses the same location eligibility but
has no hostile attacking-origin exclusion.

A surviving losing army with no legal route is destroyed in the initial resolver.
This is an explicit encirclement risk; no off-map teleport, prisoners or surrender
army state. Friendly survivors can share destinations without a cap. A voluntary
Retreat command before initiating combat is a normal adjacent move; a trapped
besieged force uses Escape under P14. Retreat never starts a chained encounter.

At zero headcount delete the formation and its service/veteran record. Record only
permitted world event labels. For attached named people, process ascending person
ID and draw once from the combat RNG: 25% die; survivors become wounded. A wounded
survivor attaches to the lowest-ID surviving friendly formation in that encounter
and shares its final location; if none exists, escapes to the lowest-ID legal
adjacent controlled, unbesieged friendly settlement and becomes recovering there.
If no such refuge exists, the person dies. No troop formation is recreated to
carry an escaping hero. Survivors have witnessed their actual encounter only.

For a surviving formation, people normally live. To support the leadership story
without a full injury simulator, once per side per encounter, if its commander
is present and that commander's formation lost at least 20% of starting headcount,
draw a 10% wound chance. If wounded, another fit adult named member of that army
with the lowest ID assumes command for the report and earns evidence; a leaderless
army still operates if none exists. This event does not retroactively change the
already resolved attacks. A wounded person has no field contribution until P20
recovery; there is no per-hit named-person death roll.

Retreat, destruction and leadership outcomes are committed once. Founder mortality
is not immunity; future balance work should preserve meaningful careers through
the specified tunable chances rather than secret resurrection.

## P14 — Persistent siege, escape and relief

Fort or better with hostile defending armies enters a siege on hostile arrival.
An undefended fort is captured immediately. The besieger pays entry movement and
becomes exhausted; the defender stays inside the same physical site. Siege records
partition occupying armies into defending and besieging sides. This partition is
not an extra node or an illegal duplicate location. The site is contested for
supply and income while defending control remains.

Start with advantage 1500 permille. At an eligible seasonal step add one elapsed
step and 5 lasting fort-damage points (clamp 0–100). Wall resistance factor is
`max(1000, 1500 - 75*elapsed_steps - 3*fort_damage)`. Siege weakening never grants
automatic victory or automatic headcount loss. An absent besieger cannot progress
the siege. Removing a siege never repairs persistent fort damage.

If attacking Siege Engines survive in an assault, subtract 200 from the wall
factor, floor 1000, for that assault; multiple engine formations do not multiply
this reduction. Damaged forts with fort_damage >= 100 provide no wall bonus until
repaired but remain sites with a recoverable Fort layer.

| Order | Preconditions and result |
| --- | --- |
| Maintain | Default; no extra action/progress immediately; boundary advances once |
| Assault | Active attacker, at least one participating army with unused movement; all selected besiegers fight all defenders with wall bonus; each participant exhausted |
| Withdraw | Active besieger moves to a legal adjacent friendly/neutral site; last besieger leaving lifts siege |
| Sortie | Active defender with unused movement; all chosen defenders attack all besiegers in a field encounter without walls; a defeated defender falls back inside if any defender survives, siege remains |
| Escape | Active defender with unused movement and at least one legal exit; fight besiegers without walls as an escape encounter; surviving defender force reaches chosen legal retreat destination only if it wins; otherwise it falls back inside; casualties persist |
| Relief | Incoming friendly army attacks all besiegers in a field encounter; all fit defending armies join automatically, without wall bonus; resolve as one joint encounter with distinct army IDs |

On a failed relief, surviving original garrison remains inside; surviving incoming
relief armies retreat by P13. On a successful relief, besiegers retreat and siege
lifts; relief joins defenders at the site. On mutual destruction reconcile actual
survivors and control. Stalemate in sortie/escape leaves garrison inside; in relief
the relief force retreats and siege remains. Exception context must be explicit in
the battle outcome, not achieved by restoring dead formations. Sortie/escape
failure does not run a second battle during fallback.

I09 resolves the earlier contradictory Escape table in favor of this explicit
stalemate rule: escape requires victory. This is a delegated implementation
decision, not a newly confirmed author rule.

New friendly besieging armies join the existing besieging side. Reinforcing a
garrison must resolve Relief; no free walk through the siege. One besieging faction
per siege initially. A third hostile faction arriving first fights the incumbent
besiegers in the open field; if it wins and remains hostile to the garrison, it
becomes the new besieger with elapsed_steps=0 but existing fort damage. It cannot
automatically ally with either side. If a third faction lacks the required hostility
for this interaction, reject entry under the access rules.

Revalidate/lift after every retreat, capture, elimination or peace change. Spent
movement cannot be refreshed by entering/leaving a siege record. The siege UI must
offer the side's real legal actions, blocked reasons, elapsed seasons, supply, and
known advantage without revealing hidden enemy details before combat.

## P15 — Ordinary threats and lasting damage

Local threats are a persisted site occupant distinct from a sovereign faction:
Bandits, Wildlife, or Bandits in a ruined hold. Use the same resolver with
data-defined formations and neutral 1000 leadership. Initial examples are 60
Bandits (attack 8, resistance 8) or 20 Wildlife (attack 15, resistance 8). They
defend their site, have no faction turns, and cannot retreat. They do not count
toward the chosen 4–8 kingdoms. No magical/dragon content is implied.

Clear Threat is an explicit attack from an adjacent legal route. Victory marks
that threat ID cleared, grants its authored resource reward once (default 30 Gold
and 10 Wood for bandits, 10 Gold for wildlife), and records real participants and
encounter tags. A cleared ruin becomes eligible for reclamation/outpost work;
it does not instantly become a city or generate a new graph site. Do not respawn
cleared threats in the initial version. A newly ruined site may create one new
bandit threat after eight uninterrupted lawless rounds, at most once per distinct
ruination transition; store that transition ID and creation flag.

Every field battle on an inhabited site adds 5 structural damage. An assault adds
10 structural and 10 fort damage. Capture of inhabited hostile territory sets
occupation=100 and adds another 10 structural damage once. Clamp each to 100.
Only an assault that destroys at least half the starting defender base power
damages one incident improved road by 10 (lowest route ID). Roads are therefore
harder to destroy than fields. No separate Raid/Scorch command in this scope.

P19 consumes structural damage and occupation as conditions; it must not apply
the same battle damage again. Casualty reports separate troop losses from site
and road damage. No plague, loot crafting, detailed supply consumption, torture,
captivity or scripted guaranteed heroic escape is required by these defaults.
