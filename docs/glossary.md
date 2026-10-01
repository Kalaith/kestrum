# Kestrum glossary

[Documentation index](README.md)

Definitions describe the implemented campaign as of 2026-10-01 unless marked as
design scope. The map improvement plan can change presentation and new-world
content without redefining saved identities or inventing campaign history.

| Term | Meaning in this documentation |
| --- | --- |
| Army | Persistent field force with six optional formation slots and at least one surviving formation; empty slots are legal and an army is removed when no formation remains |
| Formation | One troop body in an army slot, with type, headcount, experience, and identity; zero headcount destroys it and its formation history |
| Unit | Ambiguous source term: can mean an early six-person company or a troop formation; prefer the explicit term |
| Company / Host | In-world formation or army names; a name alone does not define mechanical scale |
| Troop type | Reusable military category such as Warriors, Riders, or Wyverns |
| Headcount | Current individuals or type-specific counted elements in a formation, relative to its capacity |
| Veteran formation | Surviving formation with accumulated experience; replenishment does not dilute its veterancy or specialization |
| Abstract person | Population or troop member without a full individual simulation record |
| Tracked person | Individually persistent junior, emerging soldier, heir, or other relevant person |
| Named / recognized character | Individually persistent person with condition, career and witnessed history; eligible people can hold army command or formation leadership |
| Emergence | A person becomes individually important or tracked through service or other relevance |
| Recognition | Significance earned through meaningful events; can formalize a tracked person's importance |
| Disposition | Subtle underlying inclination influencing a person's response to experience |
| Trait | Recognized pattern supported by behavior and experience |
| Class eligibility | History and foundations support a career path; opportunities may still be missing |
| Opportunity | Equipment, assignment, access, resource, mentor, or event that permits development |
| Bond | Relationship arising from shared service; initially subtle in effect |
| Legacy | Lasting family, students, roles, objects, institutions, or history after a person's active career |
| Heir | Successor by blood, martial training, religion, politics, adoption, or another supported relationship |
| Node | General design term for a place in a connected graph; specify world marker or physical site when discussing code or orders |
| World marker | Selectable world-map representation of one physical site or a whole region; a regional marker is not an army's physical location |
| Physical site | Persistent location used by armies, settlements, construction, supply and saved orders; its contents evolve while its identity remains stable |
| Region | World marker containing a connected graph of physical sites, entrance mappings and a political-control expression |
| Entrance | Mapping between an external route and a specific internal regional node |
| Route / edge | A valid connection for movement subject to current traversal rules |
| Movement plan | Saved army group and remaining sequence of physical sites; seasonal continuation rechecks movement allowance, access and encounters |
| Layout revision | Authored geography version used to validate a saved world; existing revisions preserve their supported geography rather than relocating saved campaigns |
| Road | Infrastructure improving an existing route for defenders and invaders |
| Occupancy | Physical army presence at a node |
| Local control | Faction currently holding a site; can be contested |
| Political ownership | Regional claim or administration based on strategic control requirements |
| Strategic anchor | Important node or access condition used to determine regional control |
| Occupation | Newly imposed local rule that may still have unrest or resistance |
| Capital | Faction's designated seat of government, stored separately from settlement size and headquarters |
| Headquarters | Core military support and command role, initially in starting territory |
| Development focus | One chosen settlement priority influencing development and output |
| Development pressure | Accumulated favorable or unfavorable conditions shaping a place over time |
| Supply connection | Secure friendly route connection to headquarters, used by recovery and other requirements; it can be interrupted by hostile or contested control |
| Outpost | Persistent established site providing presence and potential supply, defense, and settlement growth |
| Siege | Continuing conflict around a fortified site, with persistent progress and participants |
| Relief | Friendly outside force acting to lift a siege |
| Faction turn | One faction's action phase within the current season |
| Round | All active factions act, followed by one shared seasonal resolution |
| Season | Time unit advanced at the end of a complete round; four per year |
| Campaign | Depending on context, the whole saved game or a connected military operation within a longer war |
| Era | Campaign-history label derived from founding, active war, armistice, peace or a kingdom's fall; missing earlier dates are identified explicitly |
| Event | Dated occurrence; narrative details may expire while compact participation facts remain available for progression |
| Knowledge | What a faction has legitimately learned, including last-known observations |
| K01–K18 | Completed foundation work packages; K18 closed under amended testing scope, which does not establish settled balance or full platform acceptance |
| B01–B07 | Completed battle-system work packages; later review findings and limitations remain recorded separately |
| V1 / first functional version | Historical scope term for the initial operational kingdom game; not the current implementation queue |

Where a source uses a term differently, [the decision register](13-decisions-and-open-questions.md) preserves the distinction.
