# Border visuals — 2026-10-02

The map now draws clearer edges between known political claims. It reuses the
existing 8 px claim grid and 0.19-alpha territory fill, adding a 6.5 px dark
underlay at 0.94 alpha and opaque 2 px faction-colored rails offset 1.6 px to
each side. Long borders are drawn in 512-quad batches to stay within
Macroquad's mesh capacity. The text extraction in commit `feb1a6f` keeps all
972 original interface strings and the title, subtitle, edition, four season
names and geography labels unchanged in `game_text.json`.

## Visual review

The hidden native capture wrapper completed seven scenes at 1920×1080, the
sole acceptance size recorded in the [project README](../../README.md), and
exited successfully (PID 34836 was verified stopped). Review covered:

- [Urgent overview](ui_overview_urgent.png), including the eight faction colors.
- [Yellow border at working zoom](ui_overview_border_yellow.png): Oak's yellow
  rail stays clear against pale plains and trees.
- [Production world](ui_production_world.png), [observer world](ui_observer_world.png)
  and [observer region](ui_observer_region.png).
- [Title](ui_title.png) and [help](ui_help.png).

The rails remain readable beside roads and location markers. The review found
no new overlap; title and help text remain intact, and the existing fog rules
remain in place. The atlas shoreline mask remains world-scope only. Regional
background and claim-mask limitations are inherited and are not established as
fixed by these captures. The fresh observer's unclaimed regions are not
evidence for faction border colors.

## Browser smoke check

The [published Preview](http://127.0.0.1/games/kestrum/) returned HTTP 200.
Headless Chromium loaded the WASM with no page errors or failed same-origin
responses. An isolated touch-enabled
context used a 1920×1080 viewport at DPR 1. The embedded canvas initially
measured 1200×675. Applying the existing `body.game-playing` layout class and
a resize event only in this test context made the CSS and backing canvas
1920×1080 at (0, 0); no fullscreen API or control was used. Emulated taps completed title →
Found a Kingdom → Create Campaign → a fresh campaign. The [browser capture](ui_border_browser.png)
shows its normal fog. This checks browser loading and campaign creation; the
native eight-faction scene provides the border-color review.

Page widgets remain visible in this layout harness: the Support me widget
overlaps the lower-left camera controls. Browser camera interaction was not
covered by this campaign creation check.

## Validation

Formatting, strict Clippy, the source-size gate and no-parameter publishing
passed. Publishing passed Windows, WASM, Preview and Roost. The full test suite
passed all targets except the inherited
`k18_production_battle::four_faction_production_campaign_reaches_victory_and_roundtrips_terminal_save`,
which still misses victory at round 240. The 400-round continuity target passed
in 116.84 seconds; `ai_performance::production_decision_costs` remains
intentionally ignored. The text extraction stage also passed its 29 focused
tests, formatting, strict Clippy, source-size gate and publishing checks.

Physical-device touch and fullscreen acceptance were not tested.
