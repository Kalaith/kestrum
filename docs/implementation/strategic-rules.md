# Provisional strategic rules

[Plan](../implementation-plan.md) · [Contracts](contracts.md)

All **P01–P10** choices below are provisional defaults for review and implementation.
Confirmed constraints are cited by their chapter 13 IDs. Values belong in JSON,
not Rust constants. The owning GDD chapters remain the source of design intent.

## P01 — Setup and the fixed world

Sources: [world](../02-world-time-and-control.md), [kingdoms](../03-kingdoms-and-economy.md).

Production setup asks for a trimmed, nonempty kingdom name (maximum 32 Unicode
characters), an emblem from eight authored botanical symbols, and **4–8 total
factions including the player**, default 4. Start with one normal difficulty and
no hidden AI income bonus. Starting year comes from existing presentation data.
Use a displayed seed; randomizing it is an explicit setup action before creation.

The production layout contains exactly 80 major locations. Eight are region hubs
with ten internal sites apiece; the other 72 are physical single sites. Thus the
initial production traversal graph has 152 physical sites, not 160 or 80. Assign
stable IDs during creation, store the instantiated graph, and never add/delete
sites/routes later. Generate initial contents and faction assignment from the
seed against authored geographic layout data. This initial generator does not
need unconstrained procedural terrain or arbitrary image recognition.

Author normalized atlas positions and explicitly traversable land connections in
`world_layout.json`. Review that markers and lines fit the actual atlas. Do not
connect across open water without an authored bridge/passable coastal route;
ports are ordinary land sites until naval travel is separately scoped. Every
major location is connected, every region has at least two mapped entrances, and
eight reserved HQ candidates are mutually separated by at least four physical
edges. Seed-shuffle those candidates and select the first requested faction count.
Reserve accessible expansion sites and material-producing terrain for each start.
Reject a bad layout; do not silently shrink the campaign or retry without a bound.

Each faction begins with one controlled Village headquarters/capital, one army
containing full Warriors, Spearmen and Archers, and one named founding lord
aged 18–24, attached to Warriors and appointed commander with the Officer
profession. Name and age are seeded; founding nobility is a persistent title,
independent of earned recognition. These initial troops/person are
setup grants, not paid recruitment or invented pre-campaign battle credit. Gold,
Wood and Stone start at 500/200/150 from `economy.json`. HQ has a training ground
and access to horses; other facilities must be built. Other sites begin neutral
except scenario-authored holdings; rivals start at Peace. One pair of ordinary
local threats near each HQ provides early activity without forcing instant war.

The prototype is a separate named scenario, not a production size override: four
HQ world sites plus one ten-site Rosemarch region, 5 major markers and 14 physical
sites. Region IDs: west_gate, milltown, orchard, bridge, high_fort, city,
east_gate, quarry, ruined_hold, hill. Undirected internal edges are
west_gate–milltown, milltown–orchard, milltown–bridge, orchard–hill,
hill–high_fort, bridge–high_fort, bridge–city, high_fort–city, city–east_gate,
city–quarry, quarry–ruined_hold, ruined_hold–east_gate. Two HQs connect to
west_gate, two to east_gate, via distinct external routes. The player holds its
HQ, the eastern opponent holds city/high_fort; the other two rivals hold their
HQs. Scenario diplomacy can begin at War to exercise battles. Scenario positions,
troops and resources are durable authored data; tests may construct smaller graphs
in memory. Example protagonists remain optional labels.

I09 adds a Fort layer to Hawthorn Headquarters in new Rosemarch prototype
campaigns, providing a defended siege target before K12's active rival policy.
Its Village and founding army are unchanged. Previously saved worlds retain their
saved layers; this authored challenge does not change production starting grants.

## P02 — Faction phases and round order

Sources: D03/O04 and [round resolution](../11-simulation-and-data.md#round-end-resolution).

Player first, then active independent NPC factions in stable ID order. Snapshot
the round's faction order and persist its acted set. Eliminated or subordinate
vassal factions have no independent turn; skip them without replaying/skipping
another faction. Commands and battles resolve immediately. `End Turn` ends only
the active faction's phase. NPC work advances one atomic command at a time with
visible progress and the C03 pause/transfer control.

After every eligible faction has acted, perform exactly this boundary:

1. Reconcile armies, hostile occupancy, control, siege participants and supply.
   Snapshot eligible supply paths, facilities and sites for this boundary.
2. For each faction, grant controlled-settlement income and eligible HQ bonus,
   then pay upkeep and set/clear deficit flags. Inactive factions earn nothing.
3. Progress eligible prepaid construction and active sieges once. Orders begun
   this round receive their first step now if eligible. A besieger leaving before
   the boundary grants no siege progress.
4. Apply formation/person recovery using the step-1 snapshot and post-upkeep
   balances. Construction completed in step 3 first helps recovery next round.
5. Apply development, occupation normalization, population movement and repairs
   from a shared pre-development snapshot; apply all node results together.
6. Increment `completed_rounds` once. Run birthday/lifecycle checks for the new
   date. Terminal deaths release assignments before new appointments.
7. Consume pending progression/recognition, training and mentorship opportunities
   from actual service this round. A dead person gains no new active role.
8. Reconcile victory/elimination, knowledge, and bounded history. Reset movement
   spent and per-round action flags for surviving formations/people. Initialize
   the next active-faction order/acted set. Attempt the distinct round checkpoint.

Steps 1–5 use the ending season's date, step 6 uses the new date, step 7 records
recognition at the new date while retaining the earlier evidence dates. The save
represents the fully resolved new season. A completed terminal campaign gets the
same checkpoint attempt and an end screen; it accepts no further strategic orders.
Frame time, report opening, NPC count and save retry cannot advance this sequence.
Seasonal weather modifiers and starvation attrition are disabled in this version.

## P03 — Control and nested boundaries

Sources: D06 and [control/anchors](../02-world-time-and-control.md).

Every external connection names both physical endpoints. For a region endpoint
this is a particular entrance site. Crossing pays that edge once; moving within
the region requires its internal edges. A collapsed world route preview expands
to the real physical route before confirmation. A world marker is never a free
shortcut between gates.

Unopposed legal arrival captures an unfortified neutral/hostile site immediately;
control persists after departure. At Peace, foreign controlled sites cannot be
entered. Peace does not mean military access. Occupancy by hostile sides is legal
only during an atomic encounter or persistent siege. Supply and political control
do not count contested sites as secure.

Region ownership uses a data expression: all required anchor sites securely held
and any one entrance with a supply path to that faction's HQ. Rosemarch requires
city and high_fort plus one such entrance. Evaluate after changes that affect
control/supply. A qualifying claimant immediately becomes political owner.
If nobody qualifies, retain the last owner as a historical claim but mark the
region contested; do not grant local control or income from that claim. Remaining
enemy armies and sites keep their real identities and controllers. A rival can
retake political ownership by satisfying the same expression. A local capital is
not automatically a faction HQ or its seat of government.

## P04 — Armies, movement and transfers

Sources: D01/O02/O05/O09/O16 and [armies](../04-armies-and-logistics.md).

An army has six optional formation slots. It must have at least one surviving
formation; remove an empty army after reassigning surviving people under P13.
Create a new army by transferring one existing formation to a new ID at the same
site. No hard army or friendly-stack cap. Whole formations transfer; do not add
partial-headcount splitting/merging before an explicit design extension. A person
may transfer independently to another surviving friendly formation at that site.

| Troop | Seasonal movement allowance |
| --- | ---: |
| Warriors, Spearmen, Archers, Medics | 6 |
| Riders | 8 |
| Siege Engines | 4 |

Army available movement is the minimum remaining allowance among its formations
and assigned people. An attached person uses their class movement allowance
(Cavalry 8, or 9 while serving in Light Cavalry and younger than 41; other ordinary
classes 6). Store spent movement per formation/person.
Every participating member pays each edge's cost. Transferring/splitting preserves
spent movement, so shuttling a fast unit cannot refresh a slow army. New recruits
start exhausted until the next boundary; founding armies start fresh.

Base edge costs: plains/coast/ordinary roadless crossing 2; forest/hill 3;
mountain pass/marsh 4. An intact improved road subtracts 1, minimum 1, for every
faction. Road damage of 50 or more disables the bonus. All edges are undirected
in the initial content. No zero-cost edges or naval traversal.

Use minimum-total-cost pathfinding with site-ID tie breaks. Show legal cost from
known state, remaining movement, and uncertain enemy contact; no exact enemy
strength forecast. Follow the confirmed route edge by edge. Insufficient movement
rejects that next step without spending it. Hostile contact interrupts travel and
resolves combat/siege; participating forces become exhausted for the round.
Defending does not require unused movement. A retreat costs all remaining movement
and follows P13. Transfer is free and has no safe-site or supply requirement.

## P05 — Supply and recovery

Sources: O03/O11/D14 and [supply](../04-armies-and-logistics.md#simple-supply).

One controlled, unbesieged HQ is the faction's supply root. Trace over connected
friendly-controlled, uncontested physical sites; no arbitrary distance cap.
Neutral, enemy, foreign-peace and besieged sites block transit. Road improvement
changes speed, not the existence of a supply link. Outposts can relay existing
supply but are never independent roots. If the HQ is lost, normal recovery stops
until it is retaken or P19's explicit relocation establishes a new HQ.

Besieged defenders are cut off. A besieger is supplied if it can reach a friendly
root through an adjacent usable site without crossing the besieged site itself;
that endpoint exception cannot supply units beyond the siege. Supply paths and
failure reasons are derived, invalidated after route/control/diplomacy changes,
and rendered without exposing unseen armies. Do not use hidden army composition
to determine whether a path exists.

Round-end recovery requires supply, no upkeep shortfall, and no active defending
siege. A formation can restore at most floor(capacity × 20 / 100) headcount and no
more than it is missing. For restoring `n`, cost in Gold is
`ceil(recruit_gold × 50 × n / (100 × capacity))`, with one final rounding. Choose
the largest affordable `n` within the cap; process faction, army ID, then slot.
Deduct that exact cost once. No Wood/Stone, experience gain, veterancy dilution,
automatic starvation loss, or revival of a zero-headcount formation.

Example: Warriors 21/100 with recruitment Gold 60 recover 20 for 6 Gold; if only
2 Gold is available they recover 6 for 2 Gold. Their veteran status is unchanged.
Medics can provide participation evidence through P16 but do not silently add a
second replenishment formula. Limited named-person wound recovery is P20.

## P06 — Economy, recruitment and facilities

Source: O11 and [authoritative defaults](../../assets/data/economy.json).

Consume every supported existing economy field. Full surviving formations pay
their listed upkeep regardless of headcount. No separate named-person upkeep,
debt, population recruitment deduction, automatic disbanding or refund for
disbanding. A shortfall blocks recruitment and recovery until a later boundary
fully pays upkeep. Prepaid construction may continue. Recruitment spends costs
only after every requirement passes, completes immediately, and creates a fresh
formation ID. Empty slots are allowed, a seventh formation is not.

Income comes from locally controlled, unbesieged habitation using the JSON tier
table. Contested sites give neither side income. A controlled unbesieged HQ grants
exactly one additional 40 Gold/15 Wood/10 Stone bonus. Capturing another faction's
former HQ does not duplicate that role/bonus. The capital title grants no separate
income. Apply P19's damage/occupation and focus modifiers with one final floor per
resource per site; the HQ bonus itself is unmodified.

Recruitment requires one's turn, owned supplied unbesieged Outpost or larger,
available army slot (or a new army), and no deficit. Warriors/Spearmen/Archers need
no extra facility. Riders require a local Stable and a horse-access tag; Medics a
local Infirmary; Siege Engines a local Workshop and Fort or better. All facilities
must be functional (site structural damage below 75). Access is local, not an
empire-wide unlock. Existing specialists retain their class if access is lost.
Recovered ordinary Riders do not consume a new horse stockpile in this version.

| New ordinary facility | Gold / Wood / Stone | Progress steps | Prerequisite |
| --- | --- | ---: | --- |
| Training ground | 60 / 40 / 20 | 2 | Outpost+, supplied |
| Stable | 100 / 60 / 40 | 2 | Village+, local horse access |
| Infirmary | 80 / 40 / 20 | 2 | Village+ |
| Workshop | 100 / 80 / 60 | 3 | Village+ |
| Temple | 80 / 60 / 40 | 3 | Village+; ordinary civic/religious mentoring, no magic |

Facility costs are new provisional JSON additions; they do not replace existing
outpost/road/fort costs. No abstract technology tree or extra stockpiled Food,
Horses, influence or research points. P19's conditions model food/trade abstractly.

Ordinary class training also needs a person assigned to the site or an army there;
an owned remote facility is not access. Moving a person out of field service into
a site role, or back into a formation, requires co-location. Person travel is
through a host formation; there is no free independent strategic movement order.
Death, forced retirement and estate handling have the explicit P20/P22 exceptions.

## P07 — Construction, roads and focus orders

Sources: O12/D12 and [outposts](../04-armies-and-logistics.md#outposts).

Pay prepaid costs from `economy.json` once when accepting an order. One active
site construction order and one active improvement order per route; focus is
separate. States: Active, Paused(reason), Completed, Cancelled. Never grant effects
from paused or partial work. Cancellation before progress refunds 100%; after any
progress refunds zero. Show the same duration/cost policy to player and AI.
A builder can support only one active or paused order at a time; reassign/cancel
it explicitly before starting another.

Outpost: controlled Unsettled/Camp site, a nonempty friendly builder army, supplied
and unbesieged. Three eligible round-end steps. Builder must remain at that site;
departure, lost supply or combat this round pauses progress without refund/reset.
Another co-located friendly army can be assigned as builder. Capture cancels the
old owner's order without refund or transfer of paid work. Completion establishes
Outpost habitation with population under P19, without changing node/route IDs.

Road: existing edge, both endpoints friendly-controlled/supplied, no siege at
either, builder at either endpoint; two steps, same pause/capture rules. Completion
sets improved=true and damage=0. It never creates a new connection. Repair a
damaged improved road for 20/15/0 over one eligible step. No targeted sabotage
command initially; P15 defines rarer road damage from serious combat.

Fort: owned supplied Outpost+, builder present, three steps using existing fort
cost. Set military layer to Fort without changing habitation. Higher Stronghold
and Citadel levels remain content extensions. Facility orders in P06 require a
builder only to place the order, then progress while owned/supplied/unbesieged.
Interruption rules match outposts except ongoing builder presence.

Set Focus chooses one of Growth, Fortification, Troop Training, Gold, Wood, Stone
and replaces the previous focus; no stacking. Existing cost is zero. P19 defines
effects. Changing focus does not spend population or grant an immediate tier.

## P08 — War, peace, defeat and vassal outcome

Sources: O07/O21/D04/D11 and [victory](../03-kingdoms-and-economy.md#victory-defeat-and-continuity).

**Review-sensitive scope choice:** vassalisation is a defeated-faction outcome,
not a full diplomatic economy. This provisionally resolves D04. Diplomacy otherwise
has War and Peace. Declare War is an explicit own-turn action with confirmation;
peaceful movement never silently declares war. A peace agreement has a four-round
non-aggression timer; declaration is disabled during it. No alliances or tribute.

Offer Peace once per pair per round. An AI accepts if it has no siege underway
against the proposer and either its total surviving formation capacity fraction
is below half the proposer's, or it has lost at least one inhabited site to that
proposer in the last four rounds. These negotiation totals are internal sovereign
evaluation; do not reveal them to the proposer. AI uses the same criteria to offer
peace; the player accepts/declines through a visible choice. Store recent loss
counters separately from expiring stories.

Peace lifts mutual sieges. Besiegers and other forces illegally on foreign sites
withdraw by P13 to a legal adjacent friendly/neutral site. If any would have no
legal destination, reject the agreement with “Forces must withdraw first”; do not
teleport/delete them. Neutral-site peaceful stacks can remain. No border trading.

Defeat eligibility requires **no controlled Outpost-or-larger site and no surviving
nonempty army**, not merely a lost capital. Capture/remove/reconcile first, then
evaluate this predicate. For a rival defeated by the player, offer Annex or Accept
Submission. Annex marks Eliminated permanently; Submission records a subordinate
vassal with no independent orders/armies/income and no revival in this version.
The victor already controls captured places; vassal status grants no extra economy
or magic transfers. AI chooses Annex for defeated rivals. Personal survivors
become retired/displaced records, not claimant factions or free player recruits.

Player defeat uses the same eligibility predicate and ends play; no restart as
another faction. Player victory occurs when every other initial faction is
Eliminated or a direct player vassal. A mutually destructive last battle defeating
the player is a player defeat. Resolved terminal status is persistent and read-only
except history/saves/menu. At least one complete conquest path must be verified.

## P09 — Initial AI policy

Source: [AI parity](../03-kingdoms-and-economy.md#ai-parity).

Use ordinary commands/validators and observer-filtered information. Start with
deterministic priorities and ID tie breaks, not a separate privileged simulator:
defend threatened owned anchors/HQ; retreat supplied-path-seeking damaged armies
(below 40% capacity); recruit legal affordable replacements (Warriors, Spearmen,
Archers, then missing support); complete useful construction; clear nearby known
threats; capture reachable neutral sites; attack a known hostile target if observed
own effective strength exceeds its last-known strength by 25%; otherwise approach
a border or end. Unknown strength is uncertainty, never permission to peek.

Recruit to a target of two armies with at least three formations apiece, then
fill those armies to six before creating another for an additional threatened
front. A recruit order can create an army at its site. Keep at least two rounds'
current upkeep in Gold after recruitment unless the HQ is threatened. Prefer
supplying/recovering existing forces over repeatedly creating small empty hosts.

Evaluate accessible targets by path cost, then strategic priority HQ > regional
anchor > other inhabited site > outpost candidate > wilderness, then site ID.
Retain a chosen objective for up to four rounds unless invalid or an HQ emergency
arises. During Peace, declare war after at least four peace rounds if no safe
neutral expansion target exists within two edges and the faction has two supplied
armies; choose an adjacent rival with the fewest publicly controlled inhabited
sites. No war while a non-aggression timer runs. Apply P08 peace evaluation.

Limit to 64 accepted strategic commands per faction phase, then End Turn. Never
retry an unchanged rejected command; stop zero-progress loops. Legal transfers
cannot multiply the allowance. UI can process commands across frames, but outcome
is independent of frame rate. No difficulty bonuses initially. K18 may tune policy
and values with evidence, without allowing illegal recruits, omniscience, or free
experience. AI must also use new people/place actions as their packages arrive.

## P10 — Observation and report filtering

Source: O21 and [intelligence](../03-kingdoms-and-economy.md#fog-of-war-and-intelligence).

Topology, terrain, public site name/controller and visibly built fortifications
are public. Before combat, a friendly army can see only “Hostile force present”
at its site or an adjacent site; no count, composition, strength, name, portrait,
recognition, supply status or class. Outside that observation range show no live
army marker. A route preview uses public legality and that limited presence cue.

Combat reveals the participating enemy formations, their start/end headcounts,
attached character names/classes actually present, and visible consequences to
the opposing participants. Store the observation's date and site. After contact,
it is last-known information even if the real entity later changes; never update
it from live enemy state. A character link says “last encountered” with those facts.
No hidden remote progress, current location, death or family data leaks via search,
notifications, biography links or end-turn reports. Historical reports retain what
their viewer was entitled to learn at the time.

AI strategic choice uses this same observation service. Diplomacy can internally
evaluate its own facts and the explicit P08 acceptance rule without granting the
opponent an intelligence screen. Scouts are ordinary classes and troops initially;
spies or expanded reconnaissance require a later explicit rule.
