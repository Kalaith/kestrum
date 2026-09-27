# K08 — Genuine service and limited knowledge

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `52f1659` (K07), with a clean project checkout.

## Delivered behavior

Accepted battle and movement receipts identify the actual people and formations
present. Seasonal consumption records compact participation counters, encountered
troop types and distinct travelled routes. Later transfers cannot change who was
present or copy another formation's deeds. Only surviving formations retain XP.
The first meaningful encounter at a site against an opposing faction each round
earns credit; a trivial earlier contact does not block it. Formation XP is capped
at four per round. Seasoned at eight XP and Veteran at twenty multiply attack and
resistance by 110% and 120%; supplied recovery preserves those earned tiers.

Treatment requires real casualties or actual supplied recovery. A fit participating
Medic or a surviving fit person serving under a Medics formation can establish
treatment evidence. A wounded passenger, dead person or remote observer cannot.
Starting fitness is recorded separately from final battle condition; old reports
without that observation cannot establish medical eligibility. Retreat evidence
uses each living person's actual destination, including refuges after a formation
is wiped out. I07 documents these delegated interpretations.

Narrative history is separate from gameplay evidence. Details and their battle
payloads share a 40-round/10,000-event budget. Notable summaries retain twelve per
extant subject for eighty rounds; recent formation service retains eight rounds.
Enemy observations retain dated labels for eighty rounds, capped at 10,000 across
the campaign. Full dead-person records expire after eighty rounds or the 2,000
record budget, preserving pending participants until consumption. Historical labels
remain valid after their subjects or detailed reports expire. Later lifecycle and
family references remain K14/K15 work.

Precombat observation reveals only hostile presence at an own army's site or an
adjacent site. Actual encounters reveal the participants' recorded names, roles,
troops and conditions. Later unseen movement, promotion, injury, death or rename
does not change those snapshots. Current own records and last encountered enemy
records use separate query results. Search and linked history apply the same
observer restrictions. Old K07 saves seed only retained genuine observations and
narrative, grant no already-consumed XP, and preserve eligible pending facts.

## Interface and native visual review

Menu > Records opens People, Places and Own Armies, with Battle Reports and a
touch keyboard for known-person search. Selected places, armies, formations and
people have contextual history/service controls. Overview, Events and Filters
show five rows per screen; event queries fetch fifty records and explicit paging
crosses the batch boundary. Filters select kind and date range. Back restores the
previous view, and expired report links explain why detail is unavailable.

Normal 1920 × 1080 fullscreen and minimum 1280 × 720 captures use the established
hidden wrapper and stable files under `docs/verification/`. The service fixtures
earn eight and twenty XP through real combat and seasonal recovery, with no
injected XP or report. Dense history executes real recruitment/disbanding orders
and displays the second fifty-record batch. Expired history advances real rounds.
The reviewed service screen clearly shows earned XP, its combat effect and the
loss of veteran identity on destruction.

The application caches observer projections by campaign identity and accepted
sequence, invalidating on new/load/capture. Army forecasts refresh after state or
UI actions. Retained reports and transactional previews are therefore not cloned
on every idle frame. K18 still owns measured long-campaign performance acceptance.

## Validation status

- `cargo fmt -p kestrum -- --check`: passed.
- Strict locked all-target/all-feature Clippy: passed.
- Locked all-feature regression suite: 68 tests passed, including the source-size
  gate, five evidence cases and five knowledge cases.
- `git -c core.safecrlf=false diff --check`: passed.
- All 42 stable captures (21 scenes at both sizes) reviewed: records, person,
  known enemy, events, filters, search, empty, pruned, dense, hostile presence,
  Seasoned, Veteran, two Help pages, menu, battle forces/people, army, people,
  recovery and headquarters. Final capture PIDs 23196 and 24676 exited; earlier
  capture PIDs 25152 and 22936 also exited. No Kestrum process remained. Four
  diagnostic logs from fixed failed capture attempts were verified and removed.
- No-argument `publish.ps1`: exit 0. Windows/WebGL release builds, packages,
  Preview deployment and catalogue update succeeded. Project Roost tracking at
  `http://127.0.0.1/project_roost/api/v1` failed because localhost port 80 refused
  the connection. This external tracking warning did not prevent deployment.

Actual in-app browser review used the published Preview served on loopback,
with measured canvas/backing size 1936 × 1048. The K07 Autumn/Round 3 checkpoint
loaded with no reconstructed personal progression and a dated Summer encounter
for Mara Hawthorn. Her link opened that historical report and Back restored the
record. Visible keyboard taps searched `ma` and returned only the known Mara.

Two Rose armies then moved from East Gate into a real Autumn battle at Hawthorn
Headquarters. Rose Ward retreated after six exchanges, from 96 to 36 troops.
Ending the round saved Winter/Round 4, applied supplied recovery and granted the
surviving Spearmen exactly 2 XP with one meaningful encounter. Battle-only filters
showed both genuine retained battles. Reload and Continue restored Winter/Round 4
and the same 2 XP/counters. Browser warning/error collection returned no entries.
The page description and controls were updated and republished for Records.

K08 is complete. K09 construction is next; K09–K18 remain required release work.

Physical touch/pinch remains unverified. The inherited forced minimum-browser
display problem remains open for K18; minimum native captures do not replace the
required minimum WebGL check.
