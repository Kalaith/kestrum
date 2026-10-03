# Capital fill and borders — 2026-10-03

Regional territory fill and colored borders now use each revealed site's
physical controller. World-scope regional fill still uses the known political
claim. The local renderer checks `known_sites`, so unknown sites remain hidden;
simulation ownership and political claims are unchanged. The map legend, Help,
README now describe the distinction. The completed capital border item is
removed from `todo.md`; outstanding catalogue verification and the unrelated
batched-capture failure remain there.

## Visual review

- [Fresh home region](ui_production_world.png): the Riverfold capital has pink
  local fill and a colored border against the neutral bridge; the surrounding
  fog remains in place.
- [Developed regional view](ui_overview_region.png): a post-season fixture
  shows the regional fill using local control. Its two visible sites belong to
  the same faction, so they do not form an internal border; the capital is out
  of frame. This is not a saved-campaign Continue capture.

Both scenes passed separately through the hidden shared capture wrapper at
1920×1080, with the game process exiting successfully.

## Validation

- `cargo fmt -p kestrum --check` and `git diff --check`: passed.
- `..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`: passed.
- `..\rust_management\cargo.ps1 test -p kestrum --all-features --test code_standards --test content --test map_overview --test map_fog`: passed; source-size gate 1, content 5, map overview 5 and fog 5 tests.
- `..\rust_management\cargo.ps1 build -p kestrum`: native debug build passed.
- The existing capture harness was used without changes. No capture scene, test, campaign rule, or save data was added or modified.
