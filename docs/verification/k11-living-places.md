# K11 — Living places and ordinary threats

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `71159d1` (K10), with a clean project checkout.

## Gameplay and integration

Settlement pressure, geographic limits, natural growth, displacement, migration,
occupation decay and repair now resolve at the seasonal boundary. Conditions are
frozen before construction; population is read after conserved settler transfers.
Ruins retain their site and fortification identities and require explicit
reclamation. Capital and HQ relocation have distinct eligibility, costs and roles.
Income records the actual damage, occupation and focus operands from its boundary.

Bandits and Wildlife use the shared combat arithmetic with an independent typed
defender. Their actual survivors, terminal state and one-time rewards persist.
Threats interrupt supply, civilian routes and ordinary military entry. Repeated
ruination has one bounded spawn opportunity per transition. Dated threat reports
and participation receipts remain meaningful after terminal records are pruned.

Five development cases cover growth/caps, ruin/reclamation/repair, occupation and
income, conserved migration, and stable rename/capital/HQ identity. Five threat
cases cover combat/rewards, defeat and mutual destruction, storage/replay,
reclamation, and bounded later spawning without trivial experience farming.
Five integration cases cover genuine old payload migration, atomic invalid-load
protection, dated labels/private migration, real threat service, and 41-round
history pruning without respawn or duplicated rewards.

Integration review fixed active threats relaying supply, future-route forecasts
reading hidden occupants, and remote safety forecasts revealing unseen armies.
Current forecast uncertainty is explicit; actual seasonal simulation still uses
physical conditions. Construction regressions now account for natural population
growth separately from conserved founding transfers.

Delegated choices are recorded in [I10](../13-decisions-and-open-questions.md#i10--conserved-population-and-local-threats).

## Validation

All 111 tests pass through `..\rust_management\cargo.ps1 test -p kestrum
--locked --all-features --quiet`, including the source-size gate. Strict Clippy
with all targets/features and `-D warnings`, formatting and `git diff --check`
also pass. No-argument `publish.ps1` passed: Windows release 48.50 s, WebGL release
41.12 s, both packages, Preview deployment, catalogue refresh and Project Roost.

The published build was exercised in the in-app browser at
`http://127.0.0.1:8765/kestrum/`, with a 1936×1048 canvas. Continue restored an
existing Spring Year 1 campaign. One real round produced Summer Year 1, population
252 from 250, pressure +5 and recorded local income 10 Gold / 4 Wood / 2 Stone.
Reload and Continue retained those exact results and the 520/219/162 balance.
Browser warning/error logs were empty. The verification tab was closed afterward.

Native review passed for four representative changed decisions at each size:
settlement overview, rename entry, resettlement review and actual threat report.
Captures use stable `ui_development_*` filenames through the established wrapper.
All wrapper processes exited (PIDs 29496, 29500 and 28948). Review covered readable
income/pressure, Unicode text entry, the exact resettlement cost/destination and
actual threat outcome/reward. Representative evidence:
[minimum overview](ui_development_overview_minimum.png),
[minimum rename](ui_development_rename_minimum.png),
[minimum resettlement](ui_development_resettle_minimum.png), and
[minimum threat report](ui_development_threat_report_minimum.png).
Minimum browser sizing and physical touch remain separate checks; native captures
do not establish either one.
