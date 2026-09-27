# K17 — Production world and seeded founding

K17 switches normal New Game to a deterministic production campaign while
preserving Rosemarch as the small regression and capture fixture. The published
world has 80 major markers, 152 physical sites, eight ten-site regions, and 191
fixed routes. The production layout is loaded through the toolkit JSON path and
validated for graph reachability, terrain coverage, entrances, anchors, and
headquarters viability before a campaign can be created.

## Production setup

The setup sheet exposes a trimmed 1–32 character kingdom name, one of eight
botanical emblems, 4–8 total factions, and the seed used for the campaign. The
eight neutral Village candidates have Horse Access, nearby wood and stone, and
at least four physical routes between any pair. The selected candidates are
shuffled deterministically. Rival names and emblems are unique; every founding
faction receives equal resources and starting formations, a named Officer, and
two adjacent neutral sites with one Bandit and one Wildlife threat.

Other neutral settlements receive seeded Unsettled/Camp/Outpost/Hamlet/Village/
Town tiers with weights 40/10/10/18/19/3. Production save validation checks
against the production layout rather than Rosemarch IDs. Older saves without a
scenario kind or stored production threat manifest retain the Rosemarch defaults.

## Behavioral coverage

`tests/generation.rs` adds five focused cases:

1. Exact 80-marker, 152-site counts and fixed connected topology.
2. Same-seed replay and changed seeded contents under a different seed.
3. Connected, separated four- through eight-faction starts with two local threats.
4. Ordinary course-site support and no starting battle credit.
5. Invalid setup rejection and independent four/eight-faction save round-trips.

The existing content regressions also continue to validate the Rosemarch fixture.
The first visual recapture caught a mismatch between moved single-site markers
and their physical records; both positions now come from the same authored
coordinates and the content/generation suites pass against the corrected file.

## Native visual review

The setup controls, name-entry keyboard, eight-faction world, unselected regional
map, and selected-site inspector were captured with the shared hidden-window
wrapper at both supported canvas sizes. All ten images are stored directly in
`docs/verification/`:

| State | 1920 × 1080 | 1280 × 720 |
| --- | --- | --- |
| Setup | [setup](ui_production_setup.png) | [setup minimum](ui_production_setup_minimum.png) |
| Name entry | [keyboard](ui_production_setup_name.png) | [keyboard minimum](ui_production_setup_name_minimum.png) |
| Dense eight-faction world | [world](ui_production_world.png) | [world minimum](ui_production_world_minimum.png) |
| Ten-site region | [region map](ui_production_region_map.png) | [region map minimum](ui_production_region_map_minimum.png) |
| Selected-site inspector | [region inspection](ui_production_region.png) | [region inspection minimum](ui_production_region_minimum.png) |

The review checked marker and faction distinction, map labels, threat and army
badges, visible setup controls, keyboard fit, and regional inspection at each
size. The labels are quiet at world scale and full regional site names appear
after entering a region. This is native pointer/capture evidence; it does not
verify a physical touchscreen or the known minimum-WebGL display-scaling issue.

## Release checks

The following checks passed on the actual Kestrum checkout:

- `cargo fmt -p kestrum -- --check`
- `..\rust_management\cargo.ps1 check -p kestrum --locked --all-targets --all-features`
- `..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings`
- `..\rust_management\cargo.ps1 test -p kestrum --locked --all-targets --all-features` — all 175 tests, including the source-size gate.
- `..\publish.ps1` with no parameters — Windows and WebGL Preview builds deployed; Project Roost recorded one publish tracker.

The shared capture wrapper produced all ten images and exited after each batch.
The earlier failed capture was the paired-coordinate defect described above;
the invalid-data process was stopped after its startup error and the generated
capture logs were removed. The corrected data passes content and generation
tests, and subsequent captures completed normally.

Browser production play, the 200/400-round profiles, integrated continuity and
ending scenarios, measured performance, native fullscreen, actual WebGL 720p,
and physical-touch acceptance remain K18 work.
