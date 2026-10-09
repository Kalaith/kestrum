# Opening balance — 2026-10-08

Follows the first-100-turn review. It changes the opening campaign's economy,
diplomacy and AI war rules; it does not claim settled campaign balance.

## Changes

| Rule | Data | Effect |
| --- | --- | --- |
| Holdings with no secure path to headquarters pay a share of their income | `economy.json` `unsupplied_income_percent: 50` | Scattered rushes and cut corridors cost money; Manage shows a Supply modifier |
| A war at least `stalemate_rounds` old in which a side has taken no site from its opponent inside the retained loss window is acceptable to end | `diplomacy.json` `stalemate_rounds: 12` | Holding the line can earn peace; rivals offer and accept peace in stalled wars |
| A rival declares war only on a neighbor no stronger than itself (P08 private aggregate) | `ai.json` `war_strength_percent: 100` | A visibly strong kingdom deters attack |
| A rival opens no new war while `maximum_wars` neighbors are already enemies; distant unreachable wars do not count | `ai.json` `maximum_wars: 1` | Fewer dogpiles; one frontier at a time |
| A boxed-in rival outmatched by a neighbor raises one more army at a time, bounded by sustainable income | planner | Removes treasury hoarding behind a stronger neighbor |
| Each stranded (unsupplied) army leaves room for one supplied relief army | planner | Fixes rivals that idled forever with one army cut off |
| A kingdom that lost its headquarters may re-found it at a held Village or larger without an army or Training Ground | engine | Fixes a soft-lock where a seatless, armyless kingdom could never act or be defeated |

A claim-halt rule (claiming a Hamlet+ ends the army's movement) was prototyped
and withdrawn: it broke about ten authored Rosemarch movement, plan and
notification suites whose routes pass neutral settlements, and supply-dependent
income already gives expansion a cost. It is recorded in [todo](../../todo.md).

The midgame review script (`examples/prepare_midgame`) now plays like a
deliberate player: it invests in the capital as the guide teaches, offers peace
when at war, keeps travelling armies near home in war and retakes adjacent lost
ground, reinforces at headquarters only while income covers upkeep, and disbands
its newest unnamed formation in a deficit.

## Measurements

Tools: `observe_campaign [seed] [rounds] [factions] [trace faction]` (the trace
argument is new) and the new `opening_report [rounds] [interval]`, which runs
the midgame review script beside its rivals.

AI-only observer, four factions, sites held by survivors at round 100:

| Seed | Before | After |
| --- | --- | --- |
| 260926 | 93 / 29 / 20, one eliminated by round 37 | 83 / 45 / 11, one eliminated at 53; the seatless kingdom recovers and keeps acting |
| 88 | 49 / 42 / 28 / 24 | 73 / 28 / 25 / 23 |
| 4242 | 83 / 48 / 14 / 6 | 76 / 70 / 6, one eliminated at 49 |
| 7 | 67 / 37 / 34 / 7; Rose hoarded 2,672 Gold | 62 / 41 / 40, one eliminated at 70; no hoard above 1,000 Gold |

Snowballing by the strongest rival remains; the new rules mostly change *who*
is attacked (the weaker neighbor) and remove stalled factions.

Review script (seed 88): before, 25 sites and 5 supplied settlements at round 60,
headquarters lost by round 100. After, 43 sites and 31 supplied settlements at
round 40 in peace; it then opens a war at round 30+ on its weakest neighbor and
holds 37 sites and 12 supplied settlements at round 60 with its headquarters held
through round 90. The founding army falls before round 100.

## Validation

Formatting, strict all-target Clippy and the 800-line gate. Hidden native
captures at 1920×1080 under Xvfb (`KESTRUM_CAPTURE_MANIFEST`, 30 frames) were
inspected: [development overview](ui_development_overview.png) shows the income
line, which now lists only reducing modifiers (damage, occupation, supply) and
the labelled focus so it fits one line; [help turn page](ui_help_turn.png)
(new `help_turn` scene) shows the claiming and supply guidance within the sheet.
Minimum-size and browser captures were not run. New
`tests/opening_rules.rs` covers unsupplied income, stalemate peace, the
strength gate with parity build-up, and headquarters re-founding. Existing
fixtures updated for the supply rule: the K05 captured-headquarters income
expectation now derives the cut-off village's share from data.

Full release suite (`cargo test --release --all-features --no-fail-fast`): every
target passes except `midgame`, including the seed-88 `k18_production_battle`
victory script that failed before this slice. `midgame` passes four of five
tests; `midgame_reveals_a_broad_connected_atlas_and_known_rivals` sees 52 known
sites against its 55-site threshold. The threshold is unchanged; see todo.

## Economy follow-up

City investment is still the strongest return in the opening, so its price now
scales: each City or Major City the investor already holds adds
`cost_increase_percent_per_city` (50%) to the next one (250/100/75, then
375/150/113, then 500/200/150). Riders now recruit for 80 Gold with 12 upkeep
(was 120 and 16); with 40 troopers at 24 attack and 18 resistance they were
priced well above Warriors for similar damage and less endurance.

Cheaper Outposts (50/40/20) were tried and withdrawn: rivals built many more,
and the seed-88 production victory script fell back to Defeat. Outposts keep
their 80/60/40 price.

Generated person names (emergence, households and successors) now step past
any name a living person already holds, keeping the same random draws, after a
replay produced two living namesakes in one kingdom.

AI-only observer at round 100 with these changes, most cities held by one
kingdom: 4, 4, 6 and 2 for seeds 260926, 88, 4242 and 7 (before: 10, 11, 15
and 2). The full release suite passes every target except `midgame`, where two
tests now fall short: 53 of 55 known sites, and one of sixteen adults waits at
an unsupplied holding without a formation. The seed-88 victory script passes.
