# K09 — Persistent work and conserved settlers

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `ddb2c08` (K08), with a clean project checkout.

## Delivered behavior

Construction uses fixed physical sites and routes. Outposts and Forts take three
eligible seasonal boundaries; roads take two and road repairs one. Facility costs,
durations and prerequisites load through the toolkit's validated JSON content.
Training Grounds, Stables, Infirmaries, Workshops and Temples are site facilities.
Recruitment uses the actual completed facilities, habitation, horse access, Fort
layer and damage condition.

Orders pay once when placed. Cancellation refunds the full prepaid cost before
any progress and zero afterward. One open order reserves each target, and one
field builder can serve only one open order. Facilities release their placing
builder immediately. Departure, lost supply, combat or a missing builder pauses
field work without erasing progress. Reassignment supplies another legal builder.
Losing control cancels the previous owner's work without a refund or transfer.

Outpost completion transfers up to fifty people from the nearest eligible supplied
friendly settlement. A donor retains its habitation minimum; an existing Camp
keeps its residents. No available surplus pauses the final step. Donor eligibility,
surplus and route costs are frozen before construction, so one completion cannot
recycle its arrivals into another completion at the same boundary. Stable order
IDs resolve competing claims on that original surplus. Total population and graph
identities are conserved. I08 records these delegated implementation decisions.

Construction uses the round's supply snapshot and runs after income, before
recovery. Newly completed work cannot retroactively change that snapshot or the
income already paid. Focus selection stores one replaceable local priority;
development effects are scheduled for K11 and the current interface says so.

Saves retain prepaid orders, progress, interruptions and terminal results.
Only the latest terminal receipt per target is kept alongside current work;
separately bounded history holds dated, immutable narratives. Road receipts appear
at both endpoints. Work orders, exact population and focus are visible only to
the controlling player. Public geography and built site layers retain their
existing visibility.

## Behavioral verification

- Five construction cases cover exact costs and first/third progress timing;
  builder departure, supply loss, replacement and actual battle/capture;
  refunds, reservations, duplicate rejection and focus; partial settlers and
  competing completion budgets; and roads, repair, facilities and income timing
  for both player and NPC commands. Supported older saves migrate, partial groups
  and invalid receipts fail, and resumed work matches uninterrupted rounds.
- Five content cases cover toolkit loading, duplicate/unknown keys, supported
  prices/durations, complete facility prerequisites and population baselines.
  They also check atomic rejection of a refund overflow and unchanged observer
  projections when hidden foreign population/focus changes.
- Three history integration cases cover simultaneous completion with a one-event
  history budget, private immutable labels at both road endpoints, and forty-one
  rounds of pruning followed by reload without reopening paid work or forging
  its observer.
- Locked all-feature regression suite through `..\rust_management\cargo.ps1`:
  81 tests passed, including the 800-line source gate.
- Formatting and whitespace checks passed. Strict Clippy identified one needless
  borrow in a newly extracted test helper; the corrected rerun passed, as did the
  final strict run after interface and fixture corrections.

## Interface review

Manage is available in the selected owned place's inspector. Overview keeps
population, supply, damage, facilities and local orders together; Build, Roads and
Focus open the relevant decision. Reviews show the prepaid cost, eligible seasonal
steps, current effect and builder requirement beside Confirm Build. Details offers
Replace Builder and an explicit cancellation review with the exact refund.
Choices and orders use visible Previous/Next controls, with four rows per page.

Forty-six native captures were reviewed at 1920×1080 fullscreen and 1280×720.
They cover twenty settlement states and Help at both sizes, plus refreshed
headquarters and history filters. Dense content includes five actual orders,
page two, and a long place name. Reviews cover blocked/off-turn/foreign states,
Outpost and facility costs, builder replacement, interrupted work, both refund
amounts, completed/cancelled details, roads and focus. Terminal details correctly
explain that the builder is released. Stable `ui_settlement_*.png` and
`ui_help_construction*.png` files hold the evidence directly in this directory.
Capture processes 24476, 23184, 25892, 23976 and 26928 exited; no hidden capture
logs remained.

Published browser interaction used a fresh campaign at `localhost:8765/kestrum/`.
A Stable deducted exactly 100 Gold/60 Wood/40 Stone, and cancelling before progress
restored all three. A replacement order reached 1/2 at Summer Year 1, Round 2;
reopening the browser and Continue restored that order, Growth focus, population
250 and resources 420/159/122. The next season completed the Stable at 2/2, added
it to built facilities, retained population/focus, and showed 440/178/134.
The obsolete cancelled receipt was pruned. Visible pointer controls handled these
flows. Final canvas and viewport were both 1936×1048; browser warning/error logs
were empty. The agent-created tab was closed after verification.

## Publication and remaining platform checks

The final no-argument `./publish.ps1` passed: Windows release 43.81 seconds,
WebGL release 36.35 seconds, packaging, Preview deployment to
`\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`, catalogue updates and
Project Roost tracking all succeeded.

K09 is complete with these recorded platform limitations. K10–K18 remain required.
Physical touch/pinch remains unverified. The browser viewport override did not
remain at 1280×720 after reload; actual canvas dimensions reverted to the host
viewport. Native minimum-size captures do not establish minimum-size WebGL
acceptance. K18 must resolve this measurement/layout issue.

The older `127.0.0.1` origin also retained a cached K08 WASM while its HTML already
showed K09. The generated loader uses an unversioned `load("kestrum.wasm")`;
switching to the fresh localhost origin loaded the verified K09 artifact.
K18 must fix publication cache invalidation and check updates on an existing
origin, rather than treating fresh-origin verification as an update-path test.
