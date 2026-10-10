# M03 regional geography

## Compatibility boundary

Campaign worlds now record `topology_revision` independently of presentation
`layout_revision` and campaign rules. Missing topology fields default to the
original graph (revision 1). `assets/data/world_layout.json` remains unchanged
as its durable compatibility source. Unknown topology and mixed presentation
revisions fail validation through the existing recoverable load path.

The atlas integrity checks validate supported historical layouts 1-4, missing
field deserialization with full world equality, and malformed revision/geometry
rejection. These are content-integrity fixtures, not a claim that the user's
on-disk catalogue or every historical battle has been played through.

Formatting, strict all-target/all-feature Clippy and the full remaining test
suite pass. The broader suite is justified by the save/generation boundary;
it includes source-size and asset-registry gates. No visual behavior changes
in this slice; no captures or publication are required for it.

The full suite passes 38 tests with one release profiling case ignored. A final
unsupported-version fixture adjustment passes the focused four-case atlas
integrity target and another strict Clippy run.

Commands: `cargo fmt --all -- --check`,
`..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`,
and `..\rust_management\cargo.ps1 test -p kestrum`.

## Three-landscape prototype

New production worlds use topology 2 / layout 5. Northwatch has a wooded
fort approach and a longer bypass; Alder Vale a productive basin with lateral
connections; Westmere a river-bank landscape with secondary crossing geometry.
The overlay preserves 152 physical sites and all 191 baseline route identities,
endpoints and costs, adding routes 192-196. All 32 internal prototype roads have
authored bends, with bridge validation at every river crossing.

City access still requires ordinary paid investment. Layout 5 uses a fixed
country radius of 0.06 and affine local scale 8; discovery cannot rearrange the
map. Site positions, road bends, bridges and landforms share that transform.
Terrain is filtered by known anchors and drawn beneath the existing fog.
Historical regional and layout-4 city projections are unchanged.

The landscape capture journeys use deterministic clear-route seeds, ordinary
movement to discover connections, a legal partial move ending before its new
edge, queued next-season continuation, and paid road work where available.
Queued movement and completed road state are serialized and validated again.
Read receipts are dismissed through the existing actions for the geography
image, without removing campaign history. These controlled journeys do not
stand in for autonomous NPC play or every possible opening seed.

Formatting, strict all-target/all-feature Clippy and the source-size gate pass
on the final renderer. The full retained suite passed 38 tests, one ignored,
after the content/save changes; its unchanged results were reused during
capture-helper and rendering corrections. No experimental gameplay tests were
added. An initial capture lacked discovered destinations, a seed preflight
included a guaranteed opening threat, and legacy focus entered its region
twice; those helper failures were corrected before the successful batch.

The shared hidden capture wrapper completed all six scenes with exit 0. After
the final terrain-only refinement, the five affected scenes were recaptured
with exit 0; unchanged legacy evidence was reused. Every image is 1920x1080.

| Scene | Seed / state | Final bytes |
| --- | --- | ---: |
| [Northwatch](ui_m03_northwatch.png) | 260928; queued route 193, completed paid road | 387302 |
| [Alder Vale](ui_m03_alder.png) | 260931; queued route 194, completed paid road | 376238 |
| [Westmere](ui_m03_westmere.png) | 260932; queued route 196, completed paid road | 388673 |
| [Country Observer](ui_m03_country.png) | 260926; 152 sites, 10 terrain features | 4342259 |
| [Early frontier](ui_m03_early_fog.png) | 260928; three known opening sites | 718537 |
| [Actual legacy save](ui_m03_legacy.png) | catalogue payload 36; original layout 3 | 2676355 |

Primary visual review covered the three working city views, country overview,
early discovery and the older regional view. Distant approaches continue
beyond the working viewport; known roads, sites and river bridges align;
unrevealed countryside remains concealed. Overview and return controls stay
visible. The renderer uses joined landform polygons, small deterministic trees,
ridge marks and field parcels over the existing atlas presentation.

The real legacy payload is read-only: version 2 / layout 3, with one queued
movement plan, 45 construction orders, 481 history events and three battles.
Normal campaign parsing, validation and serialization preserve world,
knowledge, history, queued plans, work, battles, sieges and pending-battle fields.
The original file's SHA256 is unchanged:
`FEA3D8D1C8D421F21DBB634292413E40388F7ED18A65C94452FFF5ED91522224`.
It contains no active siege or pending battle, so their historical-load
behavior is not established by this capture. No save catalogue was modified.

The final pooled release WASM build exited 0. Headless Chromium loaded that
artifact and current checkout assets through an in-memory localhost server.
Test-host sizing measured 1920x1080 CSS and drawing-buffer dimensions at DPR 1,
with active WebGL, no console/page errors, failed requests or server misses.
Mouse tutorial actions invested in the capital (500/200/150 resources became
250/100/75), entered its earned region and returned through Country Map.
Overview changed scale; mouse and emulated-touch drags visibly panned the map,
and a touch tap activated Recenter. Browser and server were closed afterward.
This is emulated touch acceptance, not physical-device play or publication.

The autonomous observer replay, seed 260926 with eight factions, exited 0 at
round 40. It recorded 14 actual traversals of new routes: 192 twice, 194 six
times, 195 twice and 196 four times; 30 retained battles and 68 completed road
orders. No siege-history entries occurred in this bounded replay. These counts
establish exercised connections, not long-campaign balance or victory quality.
The six failed-capture logs created by this task were removed after verifying
their contents and successful replacement evidence; older ignored logs remain.
