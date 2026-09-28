# Kestrum

Kestrum is a generational strategy game about kingdoms, armies, people shaped by service, and places changed by decades of war and peace.

## Current milestone

The production campaign is implemented in Rust, Macroquad, and Macroquad Toolkit.
The game opens on a Kestrum title screen with Continue, New Game, Settings, How to
Play, Credits, and native Quit Game. New Game opens production setup for a kingdom
name, botanical emblem, 4–8 factions and displayed seed. A confirmed setup creates
the authored 80-major / 152-physical-site campaign over the illustrated atlas.
Rosemarch remains a separate small scenario for regression tests and captures.

K13–K17 are complete. K18 integrates production campaigns, long-run continuity,
retention, save reload and native screen review. A public-observation player
policy now wins a real 4-faction production campaign at round 60, including
actual AI turns, player-vs-rival combat, and a terminal save/reload check. The
published browser route is reachable at `http://127.0.0.1/games/kestrum/`.
At a 1280×720 page viewport, the saved-campaign Continue flow redraws across
the full 1200×675 host canvas without the reported upper-left shrink; Full
Screen expands it to 1920×1080. K18 remains in progress because the exact
1280×720 WebGL canvas and physical-touch review still need device acceptance,
and the isolated 8-faction phase-timing tail remains above its provisional
target. See [K18 verification](docs/verification/k18-integrated.md) for results
and remaining acceptance work.

The map fills the entire logical canvas. Ordinary play keeps only the map name,
season/round, active faction, Menu, zoom/recenter controls, compass, and End Turn
visible. End Turn passes to the next faction; rivals recruit, build, expand and
fight using the same campaign rules. Pause,
Step and Resume control rival progression between atomic actions. A full round
advances one season and saves the campaign. The menu contains manual save/load,
settings, help, and the route back to the title. Continue restores the current
campaign or its disk/browser save. Starting over requires confirmation before
production setup.

The save catalogue keeps each round checkpoint and each new named save. It offers
explicit overwrite/deletion and retries failed writes without replaying the round.
Enter Region opens the selected region's ten connected sites. Six-slot armies, recruitment,
disbanding, income, upkeep, movement, transfers and supplied recovery are playable.
Hostile field encounters resolve automatically, with lasting casualties, retreat,
person wounds and recorded reports. Persistent fortified encounters, escape and joint relief are playable.
Terrain labels are distinct from selectable place markers.

Production setup uses the displayed seed to choose neutral headquarters from eight
spaced Village candidates, name every founding faction, and place local threats.
Region names and faction seals keep the 80-node world legible; select any marker
for its full name and facts. Each region's ten detailed sites appear when entered.

### K01 — Typed content and Rosemarch

K01 is complete. Startup and the graphics-free `GameData::load()` library API load
and validate presentation, the existing economy, setup rules, and the authored
Rosemarch scenario through toolkit JSON APIs. The scenario has five major markers,
four headquarters, ten regional sites, twelve internal edges and four external
routes with reciprocal entrance mappings. City and High Fort plus either supplied
entrance form its anchor expression. Faction identity, local control, founding
grants, diplomacy and normalized positions are durable JSON content.

The economy's balance values are unchanged. All existing fields are typed and
validated, including unsupported policies, duplicate table IDs and invalid costs.
Scenario resource grants reference its 500 Gold / 200 Wood / 150 Stone starting
table; full formation grants reference its troop capacities. K02 instantiates
factions and the graph. The original `kestrum_campaign_v1` bytes remain readable
through Open Old Atlas and are never overwritten by a strategic campaign.

[K01 verification](docs/verification/k01-content.md) records the five new behavioral
tests, 13 preserved regressions, native captures and Windows/WebGL Preview publish.
Project Roost tracking was unavailable for K01 and succeeded during K02 publishing.
Browser fullscreen and physical touch limitations remain open.

### K02 — Campaign ownership and seasonal phases

Strategic state owns stable graph/faction IDs, founding resources, four persisted
toolkit RNG streams, the current faction, round order, acted set and command
sequence. Accepted commands commit atomically; rejection preserves state. Player
observation exposes public places and the player's own faction resources.

The season stays fixed while the player and each eligible rival take their turns.
Four rounds make one year. At K02, rivals passed; K12 adds their decision policy.
All active factions receive
income and pay upkeep at the common boundary. Menu overlays and errors hold automatic progression.
Manual saving is available during the player's phase. Old atlas campaigns are
read-only, with no invented strategic history.

K02 passes 23 tests, formatting, strict Clippy, both-size native visual review and
Windows/WebGL Preview publishing. Browser checks cover phase controls and save
restoration after reload. [K02 evidence](docs/verification/k02-campaign.md) records
the remaining fullscreen/physical-touch limitations. K18 integrated/platform
acceptance remains.

### K03 — Recoverable campaign saves

Saved Campaigns opens from the title or Menu. The paged list has no game-imposed
slot limit. New Save keeps a separate named copy; Overwrite and Delete require
confirmation. Naming uses a shared touch keyboard with optional physical typing.
Automatic saves retain each completed round. Retry uses the exact failed snapshot;
Continue Unsaved returns to the current campaign without repeating its effects.

The shared toolkit writes fresh payloads and publishes catalogue references through
a recoverable journal. Native file locks and browser Web Locks serialize writers.
Storage errors remain visible. A second game window can browse committed entries
and retry for writer ownership. Campaign IDs and save identities do not use game RNG.
Open Old Atlas preserves the shell; Import Earlier Campaign copies the K02 save
into the catalogue with a new identity. Both source slots stay intact.

K03 passes all 28 game tests, formatting, strict Clippy, both-size native review,
browser catalogue/reload/recovery checks and Windows/WebGL Preview publishing.
See [save verification](docs/verification/k03-saves.md) for shared commits, failure
coverage and the remaining platform limitations.

### K04 — Selectable geography and territorial claims

Tap a world marker or regional site to inspect its local controller, political
claim, contested state, supply and connections. Enter Region and World Map retain
the respective cameras; Close dismisses the inspector. Gates name their external
headquarters connections. The world marker summarizes the same physical sites.

Secure anchors and a supplied entrance determine a region's political owner.
Losing an anchor retains the previous claim as contested, while every physical
site keeps its actual controller. Supply follows secure friendly physical routes.
Earlier v2 saves gain these derived fields without changing their original bytes.

All 33 tests, formatting, strict Clippy, native review at both supported sizes and
Windows/WebGL Preview publishing pass. [K04 evidence](docs/verification/k04-geography.md)
records browser navigation checks and the inherited minimum-browser display issue.
K05 extends this foundation below. K18 integrated/platform acceptance remains
before the full release.

### K05 — Armies, recruitment and round economy

New campaigns begin with four authored armies, twelve formations and four Officer
founders. Each army has six slots. Armies shows headcounts, leadership, commander,
supply, upkeep and available resources. Recruit or an empty slot opens six troop
choices with costs and missing requirements. Confirm Recruit creates a full new
formation that becomes ready next round. New Army creates a separate roster at
the same site. Previous/Next pages through co-located armies.

Disband requires confirmation, gives no refund and preserves named people at the
site or in another friendly formation. Removing the last formation removes the
empty army. A full round grants secure local income and one eligible HQ bonus,
then pays full formation upkeep. A shortfall clamps Gold to zero and blocks
recruitment until a later boundary pays upkeep in full. The roster shows the
actual last-round income, paid/due upkeep and shortfall.

Earlier strategic saves retain their date, resources and RNG without invented
troops or founders; legal recruitment starts their army roster. The five K05
behavioral cases and all 38 game tests pass. [K05 evidence](docs/verification/k05-armies.md)
records visual, browser, save and Windows/WebGL Preview verification and limitations.

### K06 — Movement, composition and connected recovery

Orders opens group movement, formation transfers, local People and disbanding.
Choose co-located armies, select a physical destination, review the route and
Confirm Move. Every member pays actual edge costs; the group stops at its last
legal site when a later edge is blocked. Route details can close without cancelling
the order. K07 added field combat; K10 extends fortified arrival into persistent sieges.

Transfer whole formations or people between local friendly rosters, or split a
formation into a new army. Spent movement is preserved. Opening Armies pauses a
rival phase so these transfers remain available between rival actions.

After income and upkeep, supplied surviving formations recover up to 20% of their
capacity, limited by missing troops and affordable Gold. Orders shows the current
forecast and actual last-round recovery. Cut supply and unpaid upkeep block it.
All 48 tests, strict Clippy, formatting, source-size checks, native visual review,
browser movement/transfer/reload and Windows/WebGL Preview publishing pass.
[K06 evidence](docs/verification/k06-logistics.md) records the checks and remaining
platform limitations. K18 integrated/platform acceptance remains.

### K07 — Automatic battles and recorded consequences

Move a selected army group into a hostile unfortified force to fight every enemy
army at that site. Up to eight simultaneous exchanges account for surviving troop
counts, named leadership, troop counters and defensive terrain. Participants spend
their remaining movement. Defeated survivors retreat through legal adjacent routes;
trapped forces are destroyed. Inhabited battlefields and hostile captures retain
structural damage, with occupation recorded after capture.

Battle Reports opens after an encounter and remains available through Menu > Records.
Outcome, Forces, People and Factors show the recorded result, separate formation
losses, destinations, wounds, command succession and the strength factors used.
Previous/Next pages through long rosters; Older/Newer changes encounters. Reports
preserve what the participating faction witnessed and do not rerun the battle.

People from destroyed formations can die or escape wounded through the documented
combat rolls. A surviving commander can also be wounded; another fit adult in the
same army takes command when available. Wounded people cannot contribute leadership
and recover after two supplied seasonal boundaries. Earlier saves gain no invented
encounters, injuries or occupation. All 58 tests, strict Clippy, formatting,
source-size checks, both-size native review, published browser battle/recovery/
reload and Windows/WebGL Preview publishing pass. [K07 evidence](docs/verification/k07-combat.md)
records the checks and remaining platform limitations. K18 integrated/platform
acceptance remains.

### K08 — Service and known histories

Records opens from Menu with People, Places and Own Armies. It includes Battle
Reports and a touch keyboard for known-person search. A selected army, formation,
place or person opens contextual Overview, Events and date/kind Filters. Back
returns to the previous view. Enemy-person links show dated encounter snapshots.

Progression consumes genuine participation at the seasonal boundary. Meaningful
battles earn 2 XP, victories add 1 and surviving while outnumbered adds 1, capped
at 4 per formation per round. Seasoned begins at 8 XP and Veteran at 20. Their
attack and resistance factors are 110% and 120%. Recovery preserves earned service;
a destroyed formation cannot transfer its veteran identity to a replacement.
Detailed history is bounded separately from persistent gameplay evidence.

All 68 tests, formatting, strict Clippy and source-size checks pass. Forty-two
normal/minimum native captures and published browser service, search, filters,
report links and reload were reviewed. Windows/WebGL Preview publication passed;
Project Roost tracking was unavailable. [K08 evidence](docs/verification/k08-service.md)
records the checks and remaining platform limitations. K18 integrated/platform
acceptance remains.

### K09 — Persistent construction

Select an owned place and open Manage for population, facilities and work orders.
Build and Roads list improvements with prepaid costs, prerequisites and duration.
Review a choice, select a local builder and Confirm Build. Details shows
progress and interruptions; field work offers Replace Builder, while Cancel
shows the exact refund before confirmation. Facilities release their placing
builder. Active and paused field orders keep their builder reservation.

Outposts need three eligible seasonal boundaries and transfer up to fifty real
settlers from a supplied friendly donor with available population. Roads improve
movement after two boundaries; road repair takes one. Forts and local facilities
become persistent site layers. Leaving, combat or lost supply pauses work; losing
control cancels it. Unstarted work refunds its prepaid cost, while progressed
work gives no refund. Focus stores a replaceable choice; its development and
income effects were added in K11.
Saves and owner-visible histories retain actual orders and outcomes.

[K09 verification](docs/verification/k09-construction.md) records 81 passing tests,
strict Clippy, native visual review and published browser construction/reload checks.
Windows and WebGL Preview publishing passed. Minimum-browser sizing, cache
invalidation and physical-touch checks remain open for K18. Integrated/platform
acceptance is the remaining package.

### K10 — Persistent sieges and relief

New Rosemarch campaigns have a defended Fort at Hawthorn Headquarters. Existing
saves keep their saved site layers.

Entering a defended Fort establishes a siege with the garrison inside and
besiegers outside at the same physical site. The garrison keeps control while
the site is contested and cut off. Walls weaken once per season; lasting fort
damage remains when the siege ends. A supplied friendly approach can feed the
besiegers without carrying supply through the contested site.

Open Siege from a participating place or army, select your armies, and Choose
Siege Order. Besiegers can Maintain, Assault or Withdraw; defenders can Sortie or
Escape. Exit orders show legal adjacent destinations before Confirm Siege Order.
A failed or drawn assault leaves surviving attackers outside. Sortie/escape
failure leaves surviving defenders inside; escape reaches its exit only on victory.
All combat casualties and exhausted movement persist.

Move outside friendly armies to a besieged garrison to initiate relief. The
garrison joins the actual battle; failed relief retreats incoming armies while
surviving original defenders remain inside. Route review explains the context,
and reports separate troop losses, structural damage, fort damage and road damage.
Siege histories reveal no undiscovered enemy rosters. Only real combat earns
assault, defense, sortie, escape or relief service evidence.

K10 is complete; [verification](docs/verification/k10-sieges.md) records 96 passing
tests, publication and the remaining platform checks.
K18 integrated/platform acceptance remains as the final required package.

### K11 — Living places and local threats

Safe connected settlements grow gradually, while battles, isolation, damage and
occupation reduce their prospects. Population, habitation, walls, damage, civic
identity and kingdom roles remain separate. Seasonal income records its actual
site inputs; Growth, resource and Fortification focuses affect the relevant rules.
Displaced inhabitants migrate along friendly routes with conserved population,
and Resettle moves up to fifty for 10 Gold where capacity permits.

Use the existing Manage screen's Local Actions to rename a place, resettle people,
move the capital or relocate headquarters. HQ relocation creates the kingdom's
new single supply root and has a four-round cooldown. The settlement overview
explains its pressure, population limits and blocked development.

Nearby Bandits and Wildlife appear as local threat cues. Clear Threat selects
adjacent armies and resolves actual combat, casualties and a single-use reward.
Occupants block travel, supply and building. Clear Ruined Hold before reclaiming
it with the normal three-step Outpost order. Ruins retain their site and walls;
a later distinct ruination can create one new bandit occupation after eight
unpoliced rounds. Older saves acquire no invented occupants or development history.

All 111 tests, formatting, strict Clippy, source gate, representative native review
and Windows/WebGL Preview publishing pass. [K11 verification](docs/verification/k11-living-places.md)
records browser seasonal growth and reload, decisions and platform limitations.
K18 integrated/platform acceptance remains required.

## Screen brief

### Production campaign setup

| Question | Current answer |
| --- | --- |
| Current decision | Choose a kingdom name and emblem, how many total factions to face, and the world seed. |
| Dominant focus | One focused setup sheet above the atlas. |
| Primary action | Create Campaign is enabled for a trimmed name of 1–32 Unicode characters. It uses the displayed seed and normal difficulty. |
| Supporting information | Faction count includes the player; every start has equal troops/resources and two nearby local threats. The seed randomizer is explicit. |
| Deferred information | Geography and site names appear on the atlas; deeper scenario controls do not appear in production setup. |
| Layout and camera | At 1280 × 720, choices appear together. Name entry opens the visible touch keyboard on its own step; the atlas remains visible behind the sheet. |
| Input and feedback | Tap an emblem, − / +, or Randomize Seed. Tap Enter a Name for the touch keyboard. The displayed 4–8 count, selected emblem and seed show each change. |

### History item inspector

| Question | Current answer |
| --- | --- |
| Current decision | Inspect an owned heirloom and transfer it to a living local person, or collect it from a local estate. |
| Dominant focus | The selected item, its current holder and place, and the eligible local recipient. |
| Primary action | A visible Transfer button names its recipient; the same control collects an estate item. The action has no resource cost and requires same-faction local custody. |
| Supporting information | The item is a named Muster Sword with no combat bonus. Its creation season and current holder/place stay on the overview. |
| Deferred information | Custody deeds, war timeline and reminders appear in Events or filtered chronicles; detailed deeds expire after 40 seasons. |
| Layout and camera | The existing history overlay sits above the atlas. At the 1280 × 720 minimum, the item details and one local transfer fit together; additional recipients use overview paging. |
| Input and feedback | Tap the recipient's 48-logical-pixel Transfer button. The resulting deed can be opened from the item's, person's or site's Events view; an estate item is collected by choosing a local person. |

The normal and minimum item screens are captured in
[item overview](docs/verification/ui_history_item.png) and
[minimum overview](docs/verification/ui_history_item_minimum.png). The linked deed
is shown in [the chronicle](docs/verification/ui_history_item_deed.png) and its
[minimum view](docs/verification/ui_history_item_deed_minimum.png).

### Strategic map

| Question | Current answer |
| --- | --- |
| Current decision | Inspect a place, manage armies or construction, or confirm a costed route for a travelling group. |
| Dominant focus | The connected strategic map over the full-bleed illustrated atlas. |
| Primary action | Enter Region opens a regional marker; World Map returns to the previous world camera. End Turn remains separate from selection. |
| Supporting information | A dismissible inspector shows local control, regional claim and anchor requirements. The top edge shows season, round and active faction. |
| Deferred information | Settings, saves, help, and credits appear only when opened. No unimplemented system gets a panel. |
| Layout and camera | 1280 × 720 logical canvas, scaled to 1920 × 1080; minimum supported landscape canvas is 1280 × 720. Camera stays within the atlas at 1–3× zoom. |
| Input and feedback | Tap markers to inspect; Close dismisses selection. Drag to pan; pinch, wheel, or visible + / - to zoom; Recenter restores the current map. Targets and controls are at least 48 logical pixels. Help names the controls. |

Approximately 80% of the minimum canvas sits between the shallow edge controls;
the terrain continues beneath them. No framed central map widget or persistent
sidebar exists. A selected place adds one inspector opposite its map position.
The 80-marker production world lowers route-line weight and shows region names,
faction seals, stationed-army counts, observed local threats and the selected
location. Selecting a marker reveals its full name and facts. Regional maps show
all ten internal site names.

### K12 — Rival kingdoms, diplomacy and endings

Rivals now recruit legal forces, conserve upkeep reserves, build useful works,
clear known threats, capture territory and respond to danger. Planning uses their
own observation and dated enemy encounters. Unknown forces do not reveal private
numbers to the planner. Pause and Step expose one actual order at a time.

Open Kingdom from Menu or a rival place. Declare War explicitly, or Offer Peace.
A negotiated peace lasts for at least four rounds and withdraws armies from
foreign territory through legal adjacent exits. An impossible withdrawal rejects
the whole agreement. Incoming offers and defeated rivals pause play for your
choice; these pending decisions can be saved and resumed.

A kingdom falls only when it has no functioning Outpost or higher and no army.
Choose Annex or Submission for a rival you defeat. Submitted kingdoms remain
inactive. Losing a capital alone does not end the campaign. Once every rival is
eliminated or your vassal, the kingdom milestone ends in victory. Losing your own
last base and army ends in defeat. The saved result preserves the final battle's
service and allows Records, Save, Menu and New Game. Simulation stops there.

K17 completes the production world and setup. K18 integrated and platform
acceptance remains required for the full generational release.

[K12 verification](docs/verification/k12-kingdoms.md) records 127 passing tests,
Windows/WebGL Preview publishing, native review and the browser war/AI/save/reload
check. Minimum WebGL campaign scaling and physical-touch acceptance remain open.

### K13 — People earn their careers through service

Combat, movement, recovery and seasonal evidence can now ground a named person
who emerges from a surviving formation. The new person joins existing headcount,
gets a recent service history and begins as a Recruit. Evidence, hidden dispositions,
earned traits, one factual recognition and actual shared-service or rival records
are durable; recognized deeds do not grant unlisted combat bonuses.

The Career view shows ordinary class requirements, course prices and required local
facilities. Infantry, Archer, Scout, Cavalry, Medic and Officer courses use two
eligible seasons; riding practice supplies riding evidence only. A fit adult attached
to an army can be appointed as commander. Shield Guard, Pikemen and Light Cavalry
convert an existing formation while preserving its identity, experience and capacity.
Rivals use the same evidence options and validated course/appointment commands.

[K13 verification](docs/verification/k13-careers.md) records nine progression cases,
136 passing tests, strict Clippy, formatting, the source-size gate, normal/minimum
screen review and Windows/WebGL Preview publishing. K17 is complete; K18 remains
required, including minimum-WebGL campaign scaling and physical-touch acceptance.

### K14 — Elders keep serving in new ways

Birthdays now drive age thresholds, service chronology, fixed mortality checks,
two-step wound recovery and age-70 field retirement. Light Cavalry people lose
their extra movement at 41; from 56, elders contribute to an army only as its
appointed commander and can move into mentorship or local governance. A governor
adds one development pressure at their owned settlement.

Mentorship uses qualified living adults, four real service seasons, one learner
per teacher and shared local contact at a supplied safe site. Injury, separation,
site or facility loss pauses lessons with a reason; players may resume or end a
paused link. Two seasons open a K13 class alternative, while four complete the
dated apprenticeship without battle or encounter evidence. Rivals use the same
validated options and aging rules.

[K14 verification](docs/verification/k14-lifecycle.md) records 151 passing tests,
strict Clippy, formatting, the source-size gate, twelve reviewed 1920×1080 and
1280×720 captures, and Windows/WebGL Preview publishing. Browser interaction, physical
touch and minimum-WebGL campaign scaling remain K18 checks.

### K15 — Households and succession

Local adults with four recorded shared-service seasons may form an optional
household at a safe friendly settlement. Households can opt into sparse yearly
Spring births; players can adopt a named ward, train dependents from age thirteen,
and enter local service at seventeen. A separate age-seventeen apprentice can
join once a year for 20 Gold and one unit from local population when no dependents
are present. Births, wards and apprentices start as Recruits without copied
traits or evidence.

Command, household, item and institutional successor designations record their
real blood, adoption, mentorship or political link. A command heir already has to
serve with that army. Retirement or death uses the eligible designation, then an
oldest local pupil or demonstrated command evidence; without one the army stays
leaderless. Household capture/death and faction defeat clean up active duties
without moving people or restoring an eliminated faction. Rivals use the same
validated continuity commands.

[K15 verification](docs/verification/k15-succession.md) records nine focused
succession cases, 160 passing tests, full validation and native captures.
Browser interaction and physical-touch checks remain part of K18.

### K16 — Heirlooms, chronicles and eras

Every founding Officer carries a named Muster Sword with no combat bonus. A living
person of the same faction may transfer the same item only at the holder's site;
after death, a qualified local Item successor inherits that item, otherwise it
remains at the death site for local collection. Retirement leaves custody intact.
The same dated deed appears in item, participant and place histories, visible only
to its faction. Help and the item overview distinguish the known creation season
from later custody changes.

The history overlay adds item and memory filters, item inspection, factual war and
peace era labels, known site foundation dates, and one non-repeating anniversary
reminder per faction at each completed boundary. Retained detail is bounded by age
and count; pruning a dead person's full record also clears stale succession and
mentor references while preserving earned service counters. Old saves receive
current-round starting items without invented deeds or dates; active wars whose
start is unknown say so.

[K16 verification](docs/verification/k16-heirlooms.md) records the focused and
full-suite results, both-size native review, publisher outcome and any remaining
browser/touch limitations. K17's production campaign is complete; K18 integrated
platform acceptance remains.

### K17 — Production worlds and seeded kingdoms

New Campaign now opens a focused setup for the kingdom name, botanical emblem,
total faction count and displayed seed. The authored production world contains 80
major locations, 152 physical sites and 191 routes, with eight separated neutral
Village headquarters candidates. Four to eight kingdoms receive equal starting
forces, distinct human names and emblems, Peace relations and two local threats.
The seed determines their starts, settlement tiers and names. Rosemarch remains
available as the small, fixed regression scenario.

[K17 verification](docs/verification/k17-production.md) records five production
generation cases, full project validation, normal/minimum setup, touch-keyboard,
world and region captures, and Windows/WebGL Preview publishing. K18 integrated
campaign performance, browser play and platform acceptance remain.

### K18 — Integrated campaigns and release acceptance

K18 adds save-continuous Rosemarch combat and character development, isolation,
siege and relief, non-blood succession, layered settlement changes, a real
production-world battle and defeat against actual AI, and 200/400-round
4/8-faction replay and retention checks. A 20-screen native review at
fullscreen and 1280 × 720 found and fixed a wrapped-text overlap in the dense
army roster.

The locked all-features suite, formatting, strict Clippy, all-target check, and
no-argument Windows/WebGL Preview publish pass. K18 remains **In progress**:
the published Preview route is reachable and the saved-campaign Continue flow
passes in the WebGL page. At a 1280×720 page viewport the shared shell sizes the
canvas to 1200×675 under its header; the reported upper-left campaign shrink was
not reproduced, and Full Screen showed the full map and End Turn control at
1920×1080. The exact 1280×720 WebGL canvas and touch-only navigation still need
device review. Physical-touch hardware is unavailable, and isolated 8-faction
NPC phase p95 is 277.5 ms against the provisional 250 ms review target.
[K18 verification](docs/verification/k18-integrated.md) records the evidence
and open checks.

## Development

Kestrum is a registered member of the real shared Cargo workspace. The toolkit
path is `../macroquad-toolkit`; Macroquad remains pinned to `=0.4.16`.

```powershell
..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --all-features
cargo fmt -p kestrum -- --check
.\scripts\capture_ui.ps1 -Fullscreen
.\scripts\capture_ui.ps1 -Scenes title_minimum,gameplay_minimum,help_minimum -WindowWidth 1280 -WindowHeight 720
.\publish.ps1
```

The capture harness isolates verification from campaign saves, writes directly to
`docs/verification/`, and exits its hidden game process. Native full-screen
captures use the actual monitor dimensions. The root thumbnail is the title
capture, with no additional image processing.

## Code and artwork

- `src/data.rs`, `src/data/`: combined content, typed schemas and semantic validation.
- `src/state.rs`, `src/state/`: title/campaign transitions and authoritative state.
- `src/engine.rs`, `src/engine/`: atomic commands, projections and faction rounds.
- `src/navigation.rs`: bounded map camera, reused by mouse and touch.
- `src/game.rs`: input routing, action dispatch, assets, and toolkit persistence.
- `src/ui/`: atlas, menus, typography, and toolkit plaque rendering.
- `assets/data/game_config.json`: presentation copy and terrain labels.
- `assets/art/kestrum_atlas.png`: generated original atlas, OpenAI ImageGen.
- `assets/fonts/`: Cinzel (SIL OFL) and DejaVu Sans (Bitstream Vera license).
- `assets/data/economy.json`: validated recruitment, upkeep, income and recovery defaults; K05 consumes recruitment and seasonal economy.
- `assets/data/campaign_rules.json`: supported setup policy and eight botanical emblems.
- `assets/data/scenarios/rosemarch.json`: versioned small scenario, topology and founding grants.
- `assets/data/world_layout.json`: production topology, 80 major markers, 152 physical sites and eight reserved starts.

## Documentation and verification

Start with the [design index](docs/README.md). The founding drafts remain preserved
under `docs/reference/`. Historical references describe the earlier documentation
milestone; this README and the [UI verification record](docs/verification/initial-map.md)
describe the implementation now present.

For future development, use the [implementation plan](docs/implementation-plan.md).
It supplies 18 ordered work packages, concrete provisional rules, state/data/save
contracts, behavioral acceptance cases, a complete system coverage ledger, and a
reusable prompt for implementing one package at a time. K18 has integrated code
and test coverage but remains open for browser and physical-touch acceptance and
the measured phase-timing tail; see the acceptance ledger.

Follow [AGENTS.md](AGENTS.md), [CODE_STANDARDS.md](CODE_STANDARDS.md), and
[UI_STYLE.md](UI_STYLE.md). Shared guidance remains owned by `rust_management/docs/`.
