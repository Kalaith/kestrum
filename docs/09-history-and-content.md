# 09 — History and content

[Documentation index](README.md) · [Characters](06-character-development.md) · [Generations](08-generations-and-succession.md)

## Purpose of campaign memory

History should help the player understand why a person or place matters. Biographies and chronicles arise from actual gameplay events. Contextual summaries keep that history present without flooding ordinary play with logs.

People age, places evolve, and wars leave history behind. The world should never return completely to zero after a cycle of conquest and rebuilding.

## Proposed event record

Each significant occurrence should have a stable event ID, date, kind, participants, place, causal references where available, outcome facts, and knowledge visibility. This is a design contract, not an implemented schema.

Useful event families include:

| Domain | Events to retain |
| --- | --- |
| Military | Recruitment, formation specialization, battle, retreat, siege, relief, army or formation destruction |
| Character | Emergence, training, recognition, trait evidence, class change, wound, command, retirement, death |
| Places | Founding, development, fortification, occupation, sack, ruination, reclamation, renaming, capital relocation |
| Continuity | Mentorship, household ties, entry into service, inheritance, item transfer, institutional legacy |
| Kingdoms | Founding, war, peace, territorial transfer, elimination; vassalage when implemented |

Record facts once and derive different summaries from them. A battle can appear in an army record, a character biography, and a place chronology without awarding its experience three times. Under confirmed decision D05, a formation's own history is gone when it is destroyed. The world can still retain battle facts involving it; this event model does not require a separate destroyed-formation service archive.

## Historical integrity

**Proposal — narrative rules:**

- Events cannot predate a person's birth or relevant service unless the statement concerns inherited context.
- Direct participation needs evidence of presence or a valid retrospective grounding rule.
- A person cannot acquire achievements after death; a posthumous title must say that it is posthumous.
- Name changes preserve stable links and dated aliases.
- A destroyed formation loses its own history and veteran identity. Retained world battle facts neither recreate its service archive nor grant bonuses to a replacement.
- A generated family connection cannot contradict existing chronology or established relationships.
- Unknown enemy facts stay unknown until legitimately revealed.
- Save/load and report viewing cannot add duplicate events or reroll established outcomes.

Retrospective background fills plausible gaps. It is not permission to invent impossible victories or change recorded history.

## Character biography example

| Year | Tomas's record |
| --- | --- |
| 2 | Joined Briar Company as a squire |
| 3 | Fought at Hawthorn Gate |
| 3 | Assumed command after Captain Serai was wounded |
| 4 | Promoted to Knight |
| 5 | Led the defense of Rosemarch |
| 6 | Became known as Tomas of Hawthorn |

These dates belong to an illustrative short campaign. They are not required to match the longer examples below.

## Place chronology example

| Year | Hawthorn Gate |
| --- | --- |
| 6 | Outpost founded by Captain Serai |
| 11 | Expanded into a permanent fortress |
| 17 | First Hawthorn Siege |
| 18 | Tomas assumed command after Serai was wounded |
| 32 | Settlement expanded around the fort |
| 41 | Sacked by the Eastern Host |
| 47 | Reclaimed by the Crown |

The founding source uses different dates in different examples. Preserve them as independent scenarios rather than merging them into a contradictory single canonical timeline.

## Rosemarch campaign example

The player enters Rosemarch through the western entrance and Briar Company captures Milltown. The enemy reinforces Hawthorn Fort. Captain Serai advances on the fort; Tomas and inexperienced soldiers defend Milltown; Mira tends wounded troops behind the front.

An enemy counterattack reaches Milltown. Tomas holds the node after his commander is wounded. Mira treats soldiers affected by contaminated weapons. The enemy commander escapes Hawthorn's fall.

Several campaigns later, Tomas is a recognized knight, Mira is developing toward Plague Cleric, and the escaped officer is a major rival. Rosemarch can remain partly contested. Those outcomes demonstrate the interaction of map, opportunities, medical experience, survival, and recognition.

The small company in this original example predates the six-formation army model. A modern prototype can preserve the same event chain with people attached to formations and forces assigned across multiple armies. It must not silently turn six troop slots back into six individual soldiers.

## Wars across generations

The First Rose War can start in Year 3, enter a temporary truce in Year 11, resume as border conflict in Year 19, outlive the prime of its original commanders by Year 28, be led by a second generation in Year 34, and end in Year 41.

The frontline can likewise shift: River A in Year 5; Hawthorn captured in Year 12; northern forts retaken by the enemy in Year 18; Redplain as the stable frontier in Year 24; a second eastern campaign in Year 31. These separate illustrations show long conflicts made of distinct campaigns.

## Example fifty-year campaign

| Years | World and military change | People and continuity |
| --- | --- | --- |
| 1–5 | One settlement, one small fort, sparse surroundings; Red Plain becomes an outpost | Three named founders and six junior followers |
| 6–15 | First major war; Hawthorn Gate fortified; disease affects a settlement | Squires become recognized; first Plague Cleric emerges; important families have children |
| 16–25 | Frontier expands; Red Plain becomes Redplain Town | Founders become veterans; younger people enter service; a siege kills an original commander whose squire inherits command |
| 26–35 | Hawthorn is sacked; refugees strengthen Redplain; original capital loses strategic importance | Second-generation officers lead units |
| 36–45 | Redplain becomes a major city and government relocates | An enemy first seen as a young squire decades earlier becomes the central rival |
| 46–50 | Map visibly bears fifty years of war and development | Founders are mostly retired or dead; descendants, students, and political heirs lead |

This is an experience target rather than a scripted campaign. The initial numbers of founders and juniors are examples, not fixed setup requirements.

## Eras and reminders

Informal era names can emerge from major events: Founding Years, First Eastern War, Long Peace, Plague Years, War of Three Blooms, or Second Expansion. They organize memory without imposing authored story chapters.

Occasional source examples of contextual reminders:

- “Hawthorn has now stood for 25 years.”
- “Mira enters her 30th year of service.”
- “The children born during the First Siege are now reaching military age.”
- “Few living soldiers remember when Redplain was only an outpost.”

**Proposal:** generate reminders at meaningful anniversaries and suppress repetition. “Few living soldiers” requires actual supporting information; otherwise use a statement the simulation can substantiate. Era naming should preserve uncertainty when a conflict is still unfolding.

## Contextual presentation

A selected place can show founding year, significant sieges, and last capture. A selected person can show important service, number of pupils, and institutional ties. A visible History action opens a fuller chronology; searching and filtering become useful in long campaigns.

Notifications emphasize changed circumstances: a leader's retirement, a new class opportunity, or a key route lost. Persistent state remains inspectable after a toast ends. The UI should not repeat the same commander in every banner and panel.

## Content authoring guidance — proposal

Author reusable conditions and consequences rather than fixed protagonists. Each event definition should specify trigger context, eligible participants, chronological requirements, state changes, evidence tags, possible text, and who can know it. Separate a template's prose from the factual record.

Useful first content groups are basic troop types, junior roles, a small trait vocabulary, mentor disciplines, experience tags, terrain and route types, fortification stages, development focuses, and several local threats. Keep Dragon Knight and Plague Cleric as reference paths; they do not require the prototype to ship every fantasy class.

Names from the sources—Serai, Edrin, Tomas, Mira, Dratanus, Teresa, Elian, Elara, Thomas of Frostmarch, Rosemarch, Hawthorn, Redplain, Ashford, and Frostmarch—are examples. Thomas and Tomas must not be silently assumed to be the same person. Likewise High Fort, Hawthorn Fort, and Hawthorn Gate appear in separate map examples; content authors must assign explicit IDs instead of assuming those labels are interchangeable.

## Tone

Use concrete deeds and consequences. A concise “Held Milltown after Serai was wounded” is stronger than generic hero praise. Botanical heraldry can identify a faction; biographies should remain readable accounts of what occurred. Loss, rebuilding, survival, mentorship, and changed roles should all receive narrative attention, so significance is not confined to kills and conquest.
