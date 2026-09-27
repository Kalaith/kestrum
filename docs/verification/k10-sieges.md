# K10 — Persistent sieges and relief

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `75fa500` (K09), with a clean project checkout.

## Implemented behavior

K10 adds inside/outside participants at one physical site, permanent fort damage,
once-seasonal weakening, visible siege decisions, incoming relief, and explicit
survivor destinations. Establishing a siege is not combat and grants no battle XP.
Enemy rosters remain hidden until an actual encounter records them.

The five core tests pass: establishment/unoccupied capture, seasonal progress and
reload, assaults/engines/damage, sortie/escape/no-route rejection, and joint
relief/third parties/withdrawal/peace cleanup. Five save/content tests pass for
schema compatibility, illegal partitions, receipt integrity and observer privacy.
Five history/evidence tests pass for private immutable receipts, 41-season
retention/reload, actual assault evidence, incoming-only relief route service,
and invalid narrative rejection. A fresh authored campaign reaches Hawthorn's
defended Fort through actual movement and establishes a siege without casualties.

I09 records delegated resolutions of the Escape stalemate contradiction, relief
participation, and context-specific survivor destinations. Failed assaults retain
their surviving camp; a successful escape leaves for the selected exit. Every
outcome preserves actual casualties and exhausted movement.

Integration checks found and fixed military supply relaying through a second
siege, civilian settlers using a besieger's supply exception, and a denied later
siege entry rolling back earlier legal travel. Inactive factions cannot keep live
sieges: an incomplete elimination/submission is rejected until its military
cleanup occurs in the same transaction. No survivors or population are invented.

## Focused interface review

The existing shared wrapper captured 23 core states at 1920×1080 fullscreen and
1280×720. Review covered decisions, disabled exits/exhaustion, dense participants,
relief, persistent damage and relevant Help/history. Seven normal captures checked
outcome wording, wall factors and the field-battle fixture after Hawthorn became
fortified. These 53 durable files were already produced when the user requested
less UI work. No further matrix expansion was performed; subsequent packages
prioritize gameplay and use a small set of representative interaction checks.

The review corrected misleading generic withdrawal text for siege stalemates,
distinguished survivors inside the fort from the outside camp, and fixed the
no-exit fixture. All wrapper-launched game processes exited (19956, 22516, 27672,
2476, 29028, 4376); no capture logs or game process remained.

## Final validation and publication

Formatting, whitespace, strict Clippy and the 800-line source gate pass. All 96
tests pass through the real shared Cargo wrapper with locked dependencies and all
features. The full regression run found two obsolete pre-K10 assertions that
fortified entry must fail. They now check occupied siege and unoccupied capture
outcomes, preserving the other rejection and travel tests.

The no-argument publisher passed: Windows release 45.97 seconds, WebGL release
36.04 seconds, packaging, Preview deployment, catalogue and Project Roost updates.
Deployment is `\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`.

A focused smoke test on the existing `localhost:8765/kestrum/` browser origin
fetched `kestrum.wasm?v=1790471325404` with HTTP 200 and restored the prior K09
Autumn Year 1, Round 3 campaign through Continue. Fullscreen rendered normally;
canvas and viewport measured 1936×1048. Browser warning/error logs were empty,
and the agent-created tab was closed. Detailed siege outcomes were exercised by
the engine tests and native review; this smoke test did not repeat that matrix.

K11–K18 remain required. Native minimum captures do not prove minimum WebGL or
physical touch/pinch acceptance. The inherited browser viewport issue remains for
K18. Development publication now uses the publisher's existing `date-now` WASM
cache option, verified on the previously cached browser origin. This downloads a
fresh WASM on each development page load. K10 is complete; K11 is next.
