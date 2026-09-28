# K18 — Integrated campaigns and release acceptance

**Status: In progress.** Integrated simulation, long-run continuity, native
visual review and a production-AI victory pass. Browser/physical-touch
acceptance remains unverified, and the isolated 8-faction NPC phase p95 exceeds
the provisional 250 ms review target. K18 is not closed.

## Integrated scenarios

The locked all-features suite exercises these K18 cases against the real project
and shared Cargo workspace:

| Scenario | Evidence |
| --- | --- |
| Rosemarch combat and character continuity | [`rosemarch_counterattack_retreat_and_rematch_survive_save_reload_once`](../../tests/k18_integrated_scenarios.rs): actual attacks, retreat and rematch preserve one history/report after save and reload. [`medic_career_witnesses_casualties_then_recovers_a_wounded_commander_once`](../../tests/k18_integrated_people.rs): ordinary recruitment, infirmary, Medic service and combat treatment precede wound recovery and recognition; remote and duplicate credit are rejected. |
| Isolation, construction, siege and relief | [`isolated_outpost_veteran_stays_wounded_through_save_then_recovers_for_the_receipt_cost`](../../tests/k18_integrated_isolation.rs) cuts and restores supply through legal orders, saves the recovery receipt and checks the exact cost. [`outpost_and_fortified_siege_survive_catalogue_reload_then_resolve_relief_once`](../../tests/k18_siege_continuity.rs) persists an Outpost and siege through reload, then resolves relief once. |
| Family and non-blood continuity with places | [`unrelated_apprentice_inherits_an_item_and_survives_the_200_round_checkpoint`](../../tests/k18_scenario_c.rs) keeps an unrelated apprentice's own identity and inheritance through 200 rounds and reload. [`rosemarch_outpost_safe_growth_and_capital_orders_survive_reload`](../../tests/k18_scenario_c_places.rs) covers growth, rename and capital relocation after settlement change. |
| Production-world campaign victory, battle and save continuity | [`four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`](../../tests/k18_production_battle.rs) uses public campaign observations for player decisions and actual production AI turns. Seed `88` reached Victory at round 60 after 406 NPC actions and 131 player orders; player-v-rival combat destroyed a formation at Site 29. The test validates the world/campaign, reloads the exact 445,792-byte terminal save, and proves neither player nor NPC turns replay terminal effects. |
| Production defeat against actual rivals | [`four_faction_production_campaign_reaches_and_keeps_a_real_defeat_ending`](../../tests/k18_production_ending.rs) reaches a terminal defeat through production-map AI turns and verifies terminal save/reload cannot replay effects. Seed `1809281` ended at round 10 after ten actual boundaries and 199 NPC actions. |
| 4/8-faction continuity and retention | [`four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds`](../../tests/campaign_scenarios.rs) replays a 200-round checkpoint, checks uninterrupted/resumed state equality, runs each faction count through round 400, and validates retained entities, event bounds, counters and authored topology. `active_pupil_item_heir_survives_mentor_death_at_a_campaign_boundary` covers the production succession edge. |
| Save discovery and reload | `production_save_library_retains_more_than_five_campaigns` verifies 24 discovered entries. Existing persistence tests cover supported migration, corrupt/unsupported candidates, retry and rejected-load preservation. |

Annexation, submission and their terminal reload behavior also remain covered
by the Rosemarch diplomacy regressions. The production victory test reaches the
ending through the campaign's ordinary orders, diplomacy, siege and AI systems;
it does not call an ending helper or reveal hidden faction state to its policy.

## Isolated campaign measurements

**Hardware:** Windows 11 Pro Insider Preview build 26220, AMD Ryzen 7 5800X,
NVIDIA GeForce RTX 4080 SUPER. The four checkpoints below came from the
486.88-second isolated serial `campaign_scenarios` replay test on the final
checkout, without concurrent Rust test workloads. P50/P95/max are per completed
round, NPC faction phase and atomic NPC action.

| Factions / rounds | Wall time | Round ms P50 / P95 / max | NPC phase ms P50 / P95 / max | NPC action ms P50 / P95 / max | Largest save |
| --- | ---: | ---: | ---: | ---: | ---: |
| 4 / 200 | 73.97 s | 353.2 / 593.3 / 761.9 | 103.6 / 341.8 / 557.9 | 26.8 / 42.5 / 199.9 | 709,554 B |
| 4 / 400 | 174.48 s | 481.4 / 583.3 / 790.4 | 129.9 / 344.1 / 557.9 | 25.2 / 38.9 / 199.9 | 710,068 B |
| 8 / 200 | 150.62 s | 784.4 / 1,013.3 / 1,500.6 | 64.2 / 285.3 / 452.7 | 27.4 / 43.3 / 263.8 | 913,792 B |
| 8 / 400 | 312.37 s | 773.5 / 976.3 / 1,500.6 | 64.1 / 277.5 / 452.7 | 27.9 / 41.1 / 263.8 | 977,537 B |

At 8 factions, phase median is below the provisional 250 ms target, while p95
and maximum exceed it. Single-action p95 is below 60 ms but the maximum is
263.8 ms. The benchmark therefore records the median as meeting the target and
the tail as an open tuning result; it does not claim measured 60 Hz frame
responsiveness. Kestrum still advances one accepted, deterministic NPC action
per call without changing simulation order.

At the 8-faction round-400 peak the campaign had 258 people (208 alive), 258
family rows, 15 armies, 82 formations, eight items and 1,187 detailed history
events. After retention pruning it had 258 people (203 living), 1,091 history
events and no orphaned family rows. The authored population peak was 97,993.
The largest encoded round checkpoint was 977,537 bytes. The 4-faction run
retained 152 people (121 living), 762 events, seven armies and 39 formations
after pruning. Replayed checkpoints matched their uninterrupted campaign state.

### Phase-tail tuning trials

Two local planner alternatives were measured against the isolated 4/8-faction
400-round replay and reverted because neither improved the 8-faction phase
tail. A sorted route-neighbor index measured 299.6 ms p95 and 336.92 seconds
total at 8 factions/400 rounds. A heap-backed Dijkstra queue measured 286.9 ms
p95 and 332.49 seconds. Both p95 results exceeded the 250 ms provisional
target and the committed baseline's 277.5 ms p95 / 312.37-second total. The
committed planner is unchanged; phase-tail tuning remains open.

## Native visual review

The established hidden-window capture wrapper completed ten states at actual
fullscreen (1920 × 1080) and ten at 1280 × 720. `MinFrameMilliseconds=16.67`
paced capture frames; this setting is not an observed gameplay frame-time
measurement. Both batches exited normally. The review found a text/button
overlap in the dense army warning state; the training control now follows the
wrapped summary, and the replacement captures show the warning and control
separately.

| State | Fullscreen | 1280 × 720 |
| --- | --- | --- |
| Eight-faction production world | [image](ui_production_world.png) | [image](ui_production_world_minimum.png) |
| Dense six-slot army and deficit | [image](ui_army_dense.png) | [image](ui_army_dense_minimum.png) |
| Mentorship | [image](ui_mentorship.png) | [image](ui_mentorship_minimum.png) |
| Households | [image](ui_households.png) | [image](ui_households_minimum.png) |
| Succession | [image](ui_succession.png) | [image](ui_succession_minimum.png) |
| Wounded battle report | [image](ui_battle_wounded.png) | [image](ui_battle_wounded_minimum.png) |
| Siege relief route | [image](ui_siege_relief_review.png) | [image](ui_siege_relief_review_minimum.png) |
| Dense history | [image](ui_history_dense.png) | [image](ui_history_dense_minimum.png) |
| Long save catalogue | [image](ui_save_list.png) | [image](ui_save_list_minimum.png) |
| Lifecycle Help | [image](ui_help_lifecycle.png) | [image](ui_help_lifecycle_minimum.png) |

The associated hidden debug capture-process reports are
[`fullscreen`](k18-native-fullscreen-process.json) and [`720p`](k18-native-720-process.json).
Across the ten-scene runs, sampled working set peaked at 804,036,608 bytes at
fullscreen and 962,428,928 bytes at 720p. These are scene-switching capture
process peaks, not steady-state gameplay memory measurements. GPU counters were
available. Further memory profiling remains appropriate before calling these
capture-process peaks representative of a player's session.

## Browser and touch acceptance still open

The no-argument publisher passed: Windows and WebGL release builds packaged,
the Preview deployment was copied to
`\\wsl.localhost\Ubuntu\home\kalai\dev\games\kestrum`, and Project Roost
recorded the Preview publish. No current browser session reached that deployed
page. The existing browser address `http://127.0.0.1:8765/kestrum/` refused the
connection. The configured `Apache2.4` service is stopped and the current user
cannot open it through service control. A second browser attempt at
`http://127.0.0.1/kestrum/` showed the generic WebHatchery landing page, not the
game. An attempt to start a loopback Python static server for the published
directory was rejected by execution policy; no more specific reason was
provided. The title-to-campaign minimum-WebGL scaling regression therefore
remains unverified and unresolved in this release pass.

No touchscreen or physical pinch device was available. Native captures do not
test pointer interaction, keyboard text entry, drag, pinch, resize, Full Screen
or physical touch. Do not treat pointer or automated checks as physical-touch
acceptance. K18 remains In progress until browser acceptance is completed and
these remaining checks are resolved or explicitly accepted in scope.

## Final validation completed

On the actual Kestrum checkout:

- `cargo fmt -p kestrum -- --check`
- `..\rust_management\cargo.ps1 check -p kestrum --locked --all-targets --all-features`
- `..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings`
- `..\rust_management\cargo.ps1 test -p kestrum --locked --all-features` — the full suite and source-size gate passed on the production-victory checkout; the 4/8-faction 400-round replay completed in 552.03 seconds.
- Focused production integration: `..\rust_management\cargo.ps1 test -p kestrum --locked --all-features --test k18_production_battle '--' --nocapture` — seed `88`, round-60 Victory, 60 boundaries, 406 NPC actions, 131 player orders, destroyed formation at Site 29, exact terminal save reload.
- Isolated 4/8-faction replay: `..\rust_management\cargo.ps1 test -p kestrum --locked --test campaign_scenarios four_and_eight_faction_campaigns_retain_and_replay_through_four_hundred_rounds '--' --nocapture --test-threads=1`
- `..\publish.ps1` with no parameters — Windows and WebGL release builds, packaging, Preview deployment and Project Roost publish bookkeeping passed.

All commands ran against the actual Kestrum checkout and shared workspace. An
earlier concurrent suite replay measured 8-faction NPC phase p95 at 278.7 ms;
the isolated 277.5 ms p95 above is the controlled comparison for the
provisional target. The game engine was unchanged since those measurements.
K18 remains open for actual browser/minimum-WebGL verification, physical touch
and pinch review, and phase tail tuning.
