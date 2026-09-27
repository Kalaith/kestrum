# K16 — Heirlooms, chronicles and eras

K16 completes the documented heirloom and historical-context work in P23. Each
fresh founding Officer carries a stable named Muster Sword with no combat bonus.
Local transfers retain item identity; a physically present designated Item heir
receives the same item on death, with death-site estate fallback and local
collection. Retirement leaves custody in place.

Each custody change creates one dated, faction-private deed linked from the item,
participants and place. The inspector distinguishes the known creation season
from later custody deeds. The history view includes item and remembrance filters,
bounded reminders, current era labels, and known or explicitly unknown site and
war dates. K15 saves receive starting possessions dated at their current season,
without fabricated old deeds or foundations. Departed-person pruning also removes
stale succession and completed-mentor links while preserving earned service
counters.

## Behavioral coverage

Ten focused cases in `tests/legacy.rs` cover same-item local transfers and private
cross-subject deeds; same-site inheritance and estate fallback; retirement custody;
one-per-boundary, non-repeating reminders; age and count retention; pruning of
departed successors and mentors without losing the heir's service; K15 migration;
repeated-war links; and honest dates for earlier active wars. The conquest regression
also verifies defeated wars close instead of remaining the current era. Older
knowledge and siege fixtures now initialize K16 state consistently.

## Validation

From the actual Kestrum checkout:

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 check -p kestrum --locked --all-targets --all-features
..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --locked --all-targets --all-features
.\publish.ps1
```

Formatting, check, strict Clippy, the 800-line source gate and all 170 tests pass.
The no-argument publisher built and packaged Windows and WebGL, deployed both to
Preview, and Project Roost recorded `rust_kestrum` in one tracker.

## Native screen review

The shared hidden-window capture wrapper wrote directly to `docs/verification/`;
both game processes exited after their batches. Inventory, the selected-item
overview and local transfer, a linked deed, and history filters were reviewed at
1920×1080 and 1280×720:

- [Item inventory, 1920×1080](ui_history_items.png) · [1280×720](ui_history_items_minimum.png)
- [Item overview and transfer, 1920×1080](ui_history_item.png) · [1280×720](ui_history_item_minimum.png)
- [Person chronicle with deed, 1920×1080](ui_history_item_deed.png) · [1280×720](ui_history_item_deed_minimum.png)
- [History filters, 1920×1080](ui_history_filters.png) · [1280×720](ui_history_filters_minimum.png)

The transfer and Inspect buttons remain visible, filter/apply controls fit, and
labels stay readable at the minimum size. The real local custody command is
exercised through the library tests. Browser input, minimum-WebGL campaign scaling
and physical-touch interaction remain K18 acceptance checks; these native captures
do not claim them.
