# K06 — Movement, composition and connected recovery

Date: 2026-09-27. Actual checkout: `D:\WebHatchery\RustGames\kestrum`, `master`.
Starting commit: `dc06c50` (K05), with a clean project checkout.

## Delivered behavior

Physical routes use stable cheapest-path selection, actual gate/road costs and
per-member movement. Preview and confirmation share an exact route; confirmation
never silently reroutes. A blocked first edge leaves the campaign unchanged;
later interruption commits only the legal prefix. Groups use their slowest member.
Secure unfortified neutral sites change local control on arrival. Enemy forces
remain hidden; hostile or fortified encounters are explicitly blocked before K07.

Whole formations, their attached people and individual people transfer at a shared
physical site. Splitting creates a fresh army identity. No operation refreshes
spent movement. A transferred commander follows into a vacant command; occupied
commands remain unchanged. Opening Armies pauses rival progression for legal
off-turn composition changes. Orders, People, recipient slots and route review
have visible controls, reasons and Help.

The full round snapshots supplied sites, grants income, pays upkeep, then restores
surviving formations in faction/army/slot order. Recovery respects the 20% capacity
cap, missing troops and exact rounded affordable Gold cost. It changes headcount
and Gold only. Saved dated receipts retain historical troop kind/capacity even
after disbanding. Earlier saves gain no invented recovery receipt. I05 records
the delegated route, command-transfer and recovery decisions.

## Automated checks

The real checkout passed formatting, locked strict all-target/all-feature Clippy,
all **48 tests**, and the 800-line source gate.

Five movement cases cover cheapest routes/gates/ties; legal prefixes and changed
conditions; roads and mixed group allowances; co-location/slots/ownership and
commanders; and split/recruit/off-turn/save behavior without refreshed allowance.
Five recovery cases cover every troop capacity; exact ceil costs and partial
affordability; stable scarce-Gold priority; supply/deficit/inactive/dead exclusions;
and metadata/RNG/receipt/migration/replay integrity. K08 adds the scheduled veteran
metadata to this preservation regression when that state exists.

## Native visual review

The established hidden capture wrapper replaced stable evidence at 1920 × 1080
fullscreen and 1280 × 720. Reviewed group selection, eight co-located armies and
paging, map destination selection, route review, blocked/exhausted routes, roster
Orders, local People, formation/person transfer recipients and slots, recovery
forecast/cutoff/actual receipt, paused rivals, long names and all four Help pages.
Evidence is `ui_move*`, `ui_transfer*`, `ui_army*` and `ui_help*`, with minimum-size
counterparts. The final map details can close without cancelling the order; Help
paragraphs were split across pages after review caught overlap.

Capture processes 3040, 24104, 29992, 14796, 19208 and 17908 exited. Running Cargo
formatting in the same shell after the capture wrapper exposed an empty restored
`CARGO_TARGET_DIR`; a fresh ordinary formatting invocation passed. No alternate
workspace or target setting was used. The wrapper environment cleanup needs
review during integrated tooling acceptance.

## Published browser interaction

The published Preview WebGL build ran at a measured 1936 × 1048 fullscreen canvas.
Pointer-only interaction loaded the K05 founding save, split Warriors with
Aveline Rose into Army 5, selected both local armies, entered Rosemarch and
reviewed Rose Headquarters -> West Gate -> Milltown (cost 2 + 2). Closing and
reopening route details retained the group. Confirmation moved both armies,
preserving two movement points and recording four spent for each member.

A second route to High Fort stopped at Bridge before the unavailable encounter,
charged only the two traversed points and left both armies at zero. Aveline then
transferred to Rose Ward's Spearmen with six spent points intact and restored its
93.3% leadership contribution. Completing the round and reloading restored both
armies at Bridge, the new commander assignment, six ready movement points and
resources 540 Gold / 227 Wood / 166 Stone. The receipt showed income 70/27/16,
upkeep 30/30 and zero shortfall. Browser warning/error logs were empty.

The inherited forced-1280 × 720 browser display/input mismatch remains open;
native minimum captures do not close that browser criterion. Physical touch and
pinch are unverified. Dense people/long production routes receive further coverage
when K13/K17 introduce those ordinary states. These are explicit K18 checks.

## Publication and continuation

The no-argument publisher passed Windows and WebGL release builds, packaging,
Preview deployment, catalogue update and Project Roost tracking.

Commit subject: `Kestrum follows its roads and supply lines (K06 movement and recovery)`.
Next: K07 automatic battles, casualties, retreats and reports. K07–K18 remain
required; K06 is not full-release completion.
