# K07 — Automatic battles, retreat and recorded consequences

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `4131f58` (K06), with a clean project checkout.

## Delivered behavior

Selected moving armies fight all defending armies of one hostile faction at an
unfortified destination. Eight bounded simultaneous exchanges use exact rational
integer arithmetic, living leadership, troop counters and defensive terrain.
Stable army/slot targeting preserves each identity. Routing, the final percentage
margin, stalemate and mutual destruction produce one committed result. Participants
are exhausted, including defending survivors. Legal adjacent retreats preserve
survivors; no escape destroys the trapped formations. Field damage and hostile
capture occupation persist without being applied again by a report.

Formation wipes and eligible commander losses produce deterministic combat-RNG
death/wound outcomes. Escaping people use actual surviving friendly formations or
legal inhabited refuges. A fit adult in the same army can assume vacant command.
Wounds heal after two supplied boundaries. Existing saves gain absent historical
collections without invented combat or progression. I06 records the delegated
terrain, neutral-control, leadership and person-consequence interpretations.

Battle Reports opens after an encounter and from Menu. Outcome, Forces, People and
Factors use saved participant observations, with pages for long rosters and older
encounters. They never query later enemy state or reapply effects. Reports retain
each person's starting formation and final consequence for genuine participation
evidence in K08. Fortified encounters remain explicitly unavailable until K10.

## Native visual review

The established hidden capture wrapper wrote normal 1920 × 1080 fullscreen and
minimum 1280 × 720 evidence directly to `docs/verification/`. Scenes cover an empty
report list, stalemate, victory, defeat, destruction, force losses, terrain/counters,
people, a six-slot multi-army encounter with long names, commander wounds and
succession. Menu and battle Help were also reviewed. Files are `ui_battle*.png`,
`ui_help_battle*.png` and `ui_menu*.png`.

Fixtures issue real recruitment, movement and combat commands. The wound fixture
adds a local fit successor before a real encounter and uses combat seed 4; it does
not inject a report or award. Capture processes 17544 and 30732 exited. All reviewed
screens keep their main result readable with visible tab, paging and Back controls.

## Automated validation

The actual checkout passed `cargo fmt -p kestrum -- --check`, locked strict
all-target/all-feature Clippy and all **58 tests**, including the source-size gate.
Five combat cases cover exact arithmetic and simultaneous destruction; counters,
terrain and living leadership; selected group/all-defender participation; rout,
retreat priority/blockers, encirclement, neutral control and lasting damage; and
deterministic replay, indexed/envelope migration, malformed-report rejection and
immutable observer-filtered reports. Five wound cases cover exact RNG thresholds
and ordering, legal refuges, succession, repeated injury, supplied healing,
migration and nonrevival. Existing movement expectations now exercise actual
unfortified hostile capture while still rejecting unavailable fortified entry.

## Publication

The no-argument publisher passed Windows and WebGL release builds, packaging,
Preview deployment, catalogue refresh and Project Roost tracking.

## Published browser interaction

The published WebGL build ran on a measured 1936 × 1048 fullscreen canvas. Pointer
controls loaded the K06 checkpoint, selected both armies at Bridge and reviewed
Bridge -> Rosemarch City -> East Gate -> Hawthorn Headquarters at total cost 6.
The route disclosed combat/exhaustion/retreat risk without enemy strength.
Confirmation captured the unopposed intermediate city and resolved a real battle
at headquarters. Hawthorn won at the rout threshold after eight exchanges.

Outcome paging showed Rose Ward 180 -> 60 and Army 5 100 -> 52, both withdrawing
to East Gate; Hawthorn Host 280 -> 152 remained at headquarters. Forces showed
Spearmen 100 -> 52, Archers 80 -> 8 and Warriors 100 -> 52. People showed Aveline
Rose and Mara Hawthorn fit after this encounter. Factors showed both appointed
Officers, the separate leaderless army's 50% factor, ordinary terrain and the
Archers-versus-Spearmen counter. Back, Menu and Battle Reports reopened correctly.

Completing the round, reloading and using Continue restored Autumn / Round 3.
The report retained its original Summer casualties and enemy observation. Current
Rose survivors at East Gate had recovered to 72 Spearmen, 24 Archers and 72
Warriors, with six movement ready and 592 Gold / 265 Wood / 189 Stone. The dated
economy receipt showed income 103/38/23 and paid upkeep 30/30. Browser warning/error
logs were empty. The verification tab was closed.

The inherited forced-minimum browser display/input mismatch remains a K18 defect;
these native minimum captures do not close that WebGL criterion. Physical touch
and pinch remain unverified. Wounds/succession received actual command-driven
native captures and automated coverage; this browser encounter produced no wound.

Commit subject: `Kestrum remembers the price of battle (K07 combat and retreat)`.
Next: K08 participation, veterancy and limited enemy knowledge. K08–K18 remain
required before the full release is complete.
