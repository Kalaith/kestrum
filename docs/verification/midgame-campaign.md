# Developed midgame campaign — 2026-10-01

The earlier review generator cleared one local threat, returned to headquarters
and mostly passed turns. Its elapsed time did not establish an explored realm,
multiple player settlements or an experienced named roster.

The revised seed-88 generator uses ordinary recruitment, movement, occupation,
threat battles, annual apprentice invitations, construction, patrol service and
career courses. Rivals continue through their production NPC policy. A home
company trains the apprentices while two companies and the founding host expand
connected territory. At the saved player phase it declares war on eligible
bordering rivals through the normal diplomacy rules. Treasury, discovery,
ownership, evidence, ages and classes are earned rather than patched into a save.

The installed catalogue entry is **#25, Briarhold - Midgame Battle Review**:
Year 16, 60 completed rounds, 32/80 revealed world markers, 59/152 physical sites,
26 player settlements, four player armies and three retained battle receipts.
Ten living named adults comprise Fenn Frost (Officer), Iven Stone (Recruit), and
Ashen Cairn, Ullen Hale, Jora Dale, Aldren Pike, Hesta Dale, Vessa Grey,
Ysolde North and Catrin Mere (Scouts). The Scouts have genuine six-route patrol
service and completed courses. Iven emerged from recorded battle service.
Orren Frostmarch and Bera Pike border the realm; an active war remains available.
There is no pending battle or campaign ending. The catalogue payload reloads
identically; earlier entries remain intact and Continue selects the new save.

Loaded campaigns frame the observer-filtered discovered world. One-region saves
retain home framing. Recenter and new-game framing keep their local behavior.
Camera fitting cannot use hidden markers, and it neither adds discoveries nor
changes the campaign state.

## Validation

- Formatting and `git diff --check` pass in the actual checkout.
- Strict all-target, all-feature Clippy passes through the shared build pool.
- The focused midgame, map exploration, navigation, world navigation,
  persistence and source-size suites pass: **24 tests**.
- Five midgame cases cover broad map coverage and framing, supplied settlements
  and active rival borders, ten distinct adult names with service and trained
  careers, a live nonterminal player phase, and immutable save roundtrip/history.
- Every Rust file remains within the 800-physical-line limit.
- No-parameter `publish.ps1` passes Windows and WebGL release builds, packaging,
  Preview deployment, Project Roost recording and catalogue synchronization.

The full suite was not rerun for this fixture change. The unchanged 240-round
production-victory balance failure remains documented in README and
`battle-review.md`; focused results do not claim that blocker is resolved.

## Visual and interaction review

The shared capture wrapper loads this same generated campaign through the
ordinary application load path. Durable evidence covers the resume atlas,
selected frontier, dense named-person records and a trained character biography
at 1920×1080 and 1280×720. These scenes use production UI actions.

| State | Normal | Minimum |
| --- | --- | --- |
| Resumed realm | [Atlas](ui_midgame_map.png) | [Atlas](ui_midgame_map_minimum.png) |
| Rival frontier selection | [Frontier](ui_midgame_frontier.png) | [Frontier](ui_midgame_frontier_minimum.png) |
| Named roster | [People](ui_midgame_heroes.png) | [People](ui_midgame_heroes_minimum.png) |
| Character biography | [Scout](ui_midgame_hero.png) | [Scout](ui_midgame_hero_minimum.png) |

Both size sets were inspected. The atlas and frontier are the dominant play
areas, with region labels and faction seals identifying mixed holdings. The
selected frontier's inspector explains ownership, supply and connections beside
its controls. Records deliberately presents the ten names in two readable pages;
career/condition details remain in individual biographies. Primary map, Close,
Back, Inspect and pagination controls are legible and unclipped at both sizes.
The capture wrapper completed both runs and its launched games exited.

The pooled native debug run restored the installed #25 save directly at Spring,
Year 16 / Round 61 with the wider atlas. At its 1280×720 canvas, click-only checks
opened the headquarters Army, selected Westmere Bridge and displayed its legal
cost-4 preview with Confirm Move. Close dismissed the preview, World Map returned
from the region, and Menu > Records exposed both five-person pages. Inspecting
Catrin Mere showed a fit 24-year-old Scout and her Year 9 apprenticeship / Year 13
completed course on the second biography page. The verification game exited
with code 0; no game process remains and its native writer is released.

The published WebGL build at `http://127.0.0.1/games/kestrum/` also passed a browser
smoke check. Continue restored the browser's existing early Riverfold campaign;
Play full screen produced a 1920×1080 canvas. Clicking Army and Riverfold Bridge
showed six available movement, a cost-3 route and its encounter warning beside
Confirm Move. This verifies the release's visible controls and picking, while
the developed #25 entry belongs to native storage. Browser midgame import and
minimum-size browser interaction were not performed. Physical-touch testing
remains deferred under the project's established scope. The verification tab
was closed after the check.
