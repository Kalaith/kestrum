# K01 — Typed content and a durable strategic scenario

Date: 2026-09-26. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, on `master`.
Commit subject: `Rosemarch gains its sites and founding kingdoms (K01 typed content)`.
Use `git log -1 --format=%H --grep="K01 typed content"` to resolve the containing
implementation commit. K02 has not been started.

## Prerequisites and scope

Read AGENTS, code/UI standards, implementation plan, C01–C07, chapters 02/03/11,
the decision register, P01/P06 and the acceptance packet. The clean starting shell
passed its 13 existing tests through the shared Cargo launcher. Its real workspace
membership, sibling toolkit path, pinned Macroquad `=0.4.16`, root thumbnail,
publisher and capture harness were present. No dependency or workspace edits
were needed. No unresolved mechanics contradiction was found.

`GameData::load()` assembles presentation, economy, campaign rules and Rosemarch
using `macroquad_toolkit::include_json!`, then applies project semantic validation.
It works in library tests without a graphics context and is the actual native/WASM
startup path. The UI receives presentation data. Embedded content stays out of
the runtime asset registry.

All existing economy values are preserved. Only its stale status label changes
to `provisional_defaults`. Every existing field is typed, validated and retained
by serialization. Unsupported enum/policy values and unknown fields are rejected;
economy table deserialization rejects duplicate IDs before a map can replace them.

## Durable scenario contract

| Authored fact | Implementation |
| --- | --- |
| Identity/version | `rosemarch_prototype`, schema 1, content rules version 1, seed 260926 |
| World markers | 1–4: Rose, Oak, Hawthorn and Fern HQs; 5: Rosemarch region |
| Physical sites | 1–4: HQ villages; 5–14: west_gate, milltown, orchard, bridge, high_fort, city, east_gate, quarry, ruined_hold, hill |
| Routes | 1–12: the exact P01 internal edge list; 13–14: western HQs to west_gate; 15–16: eastern HQs to east_gate |
| Entrances | Four route mappings to two distinct gates; both endpoint-parent links and reverse region mappings validated |
| Anchors | All(city, high_fort, Any(supplied west_gate, supplied east_gate)); a supplied gate also requires secure control |
| Local control | Each faction holds its HQ; Hawthorn also holds city/high_fort; all other sites are neutral |
| Founding grants | Each faction references economy starting resources and full Warriors/Spearmen/Archers; one age-24 Officer commander attached to Warriors |
| HQ context | Controlled Village, training ground, local horse access, same initial capital/HQ |
| Diplomacy | Rose/Hawthorn at War, other pairs at Peace; one explicit symmetric relation per pair |
| Geometry | Normalized world and regional coordinates, fixed physical routes, P04 base costs and initially unimproved, undamaged roads |
| Setup | 4–8 total factions, default 4, Normal with no AI income bonus, names up to 32 Unicode scalar characters, eight authored botanical emblem identities |

MarkerId, SiteId, RouteId and FactionId are separate integer newtypes. Numeric IDs
can overlap across families; vector order and display labels never define identity.
Content round-trips preserve those IDs, coordinates, gates, control and grants.
Changing vector order or reversing an undirected route preserves validity.

Geometry is durable prototype authoring. The world positions were checked against
the atlas's central landmass; the authored region contains its river bridge.
Drawing/picking strategic routes and markers remains K04. This named 5-marker,
14-site scenario does not override K17's production count or HQ-spacing rules.
Anchors are checked for valid local references and hypothetical attainability from
every HQ. Current political ownership and live supply are not inferred at load time.

Founding troops/person are validated content grants only. Campaign entities,
monotonic allocation counters, RNG streams and strategic save version 2 belong to
their owning packages. K01 introduces no persisted campaign fields or migration:
the original v1 slot, schema and shell chronology remain unchanged. Local threats,
facility orders, later development layers and production generation remain in
their scheduled packages; no placeholder engines or controls were added.

## Behavioral evidence

Exactly five new tests in `tests/content.rs`, with table-driven variants:

1. Toolkit loading resolves all content; pins P01 counts/internal edges, initial
   holdings, economy grants/capacities; round-trips scenario, rules and economy.
2. Duplicate IDs (including JSON economy keys), missing endpoints, bad foreign
   keys, self/duplicate edges and missing economy definitions fail with source
   and field/ID diagnostics.
3. Gates map both ways and every HQ reaches all 14 sites; missing/wrong reverse
   mappings, disconnected interiors and isolated HQs fail. Reordered storage and
   reversed undirected routes remain valid.
4. Invalid capacities, costs, resources, versions, percentages, policy flags,
   setup bounds, coordinates and unknown fields/enums fail. A 32-character Unicode
   name succeeds and 33 fails. HQ facilities/control and founder grants are checked.
5. Both anchor sites and either securely held, supplied gate satisfy the expression;
   missing control/supply, empty all/any expressions, foreign/missing sites and
   non-gate supply references fail, even inside an otherwise feasible alternative.

The first run exposed an incorrectly authored duplicate-edge test mutation. The
test now actually duplicates an edge; the subsequent full suite passes.

## Validation results

| Command/check | Result |
| --- | --- |
| `cargo fmt -p kestrum -- --check` | Pass |
| `..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings` | Pass |
| `..\rust_management\cargo.ps1 test -p kestrum --locked --all-features` | Pass: 18 tests, zero failures/ignored; five new content tests plus 13 preserved regressions |
| Source-size gate in the test run | Pass, empty exception list; largest Rust file `tests/content.rs`, 594 physical lines |
| `git diff --check` | Pass |
| `scripts\capture_ui.ps1 -Scenes title,gameplay,help -Fullscreen` | Pass, 1920 × 1080; PID 27740 exited |
| `scripts\capture_ui.ps1 -Scenes title_minimum,gameplay_minimum,help_minimum -WindowWidth 1280 -WindowHeight 720` | Captures complete, 1280 × 720; PID 26396 exited |
| `publish.ps1` without parameters | Exit 0; Windows/WASM release builds, packaging, Preview deployment and catalog updates complete |

The minimum capture invocation also ran a trailing `Get-Process` query, whose
no-match result set that shell invocation's exit code to 1. The wrapper completed
all three captures. A separate process-count check confirmed zero remaining
Kestrum processes. This was not a capture or game failure.

Preview output: `\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`.
Project Roost tracking failed because `http://127.0.0.1/project_roost/api/v1`
refused its connection on port 80. Builds and deployment completed; tracking is
not claimed successful. No temporary projects, dummy crates or alternate manifests
were used, and shared build/workspace configuration was preserved.

## Visual evidence and limits

Fresh captures were written directly to the stable filenames and visually read:

| Scene | Normal | Minimum |
| --- | --- | --- |
| Title | [1920 × 1080](ui_title.png) | [1280 × 720](ui_title_minimum.png) |
| Empty atlas | [1920 × 1080](ui_gameplay.png) | [1280 × 720](ui_gameplay_minimum.png) |
| Help | [1920 × 1080](ui_help.png) | [1280 × 720](ui_help_minimum.png) |

All six are byte-identical to the prior tracked screenshots. The title and empty
atlas open after the new validation path. Map focus, readable controls, Help text,
modal spacing and unclipped layout remain intact at both sizes. No player-facing
flow changed; the existing screen brief and Help still describe the actual shell.
No strategic dense-state capture or strategic playtest is claimed.

Browser fullscreen behavior and physical touchscreen/pinch remain the explicit
limitations in [initial-map verification](initial-map.md). They were not re-tested
or resolved by K01. Native capture and passing navigation tests do not establish
new physical touch or browser interaction evidence. Economic balance, faction
rounds and the strategic campaign are not yet playable. Next eligible package:
**K02 — Campaign ownership, commands and seasonal phases**.
