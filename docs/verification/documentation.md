# Documentation milestone verification

Date: 2026-09-26

[Documentation index](../README.md) · [Source coverage](../source-coverage.md) · [Delivery plan](../12-delivery-and-validation.md)

## Scope

The initial Kestrum milestone adds a consolidated design set, source preservation, a project README, and a newline-preservation rule for archived Markdown. The supplied template source, assets, configuration, scripts, tests, thumbnail, and existing screenshots are retained. No game behavior or game screen was changed.

## Documentation checks

- Compared all three archived design files byte for byte with the supplied root drafts and recorded SHA-256 hashes. All match.
- Mapped all 200 original headings to primary consolidated chapters: 54 concept, 57 kingdom/army/war, and 89 living-world/generational headings. The ledger retains separate entries for repeated headings.
- Checked contiguous source line coverage and the exact archived files, including original diagrams, examples, and unresolved directions.
- Checked 302 local links and heading fragments across 18 authored Markdown files against their actual destinations. Historical reference files are immutable; the original template README's relative links remain in its original template context.
- Checked that authored documentation links do not depend on the three root-level drafts. Deleting those root copies later will not remove content from `docs/`.
- Reviewed army scale, faction-turn timing, victory/vassal scope, formation destruction/history, graph stability, active roster targets, capital status, prototype wounds, and independent example timelines against the decision register.
- Compared the original 29 non-Markdown template files outside `docs/` against their pre-edit SHA-256 hashes; they remain unchanged.
- Counted all ten project Rust files directly against the 800-physical-line limit; the largest is `src/ui.rs` at 563 lines. This is a source inspection, not a passing Cargo test-suite claim.
- Verified that all four reference documents retain their exact bytes in the staged Git objects, not only in the working files.
- Staged whitespace checks passed for the authored documentation and `.gitattributes`. A full initial-import check allowing CRLF line endings reports existing Markdown trailing-space notices: 53 in `CODE_STANDARDS.md`, 11 in `GAME_DEVELOPMENT_GUIDE.md`, and four each in the root and archived living-world draft. Those supplied files retain their original content; the archive is intentionally exact.

The commit hash is reported in the completion message and Git history. The verification document does not embed its own commit hash.

## Checkout validation results

| Check | Result |
| --- | --- |
| Documentation preservation and coverage | Passed |
| Authored documentation links and independence | Passed |
| Template implementation preservation | Passed |
| Direct Rust physical-line inspection | Passed |
| `cargo fmt -- --check` | Blocked before formatting: package believes it is in the parent workspace but is not a member |
| Shared launcher `test --locked` | Blocked at Cargo workspace discovery with the same membership error |
| Shared launcher `clippy --locked --all-targets --all-features '--' -D warnings` | Blocked at Cargo workspace discovery with the same membership error |
| Kestrum UI visual/touch review | Not applicable to this documentation-only change; no Kestrum screen exists yet |
| Unparameterized publisher | Not run: no meaningful game change; existing metadata still describes the template |

The test launcher first encountered sandbox denial for its normal shared-pool lease path. Rerunning with permission for that shared tool reached Cargo and confirmed the real workspace membership blocker. No build-pool configuration was changed.

## Exact onboarding blocker

Cargo reports that `D:\WebHatchery\RustGames\kestrum\Cargo.toml` believes it belongs to `D:\WebHatchery\RustGames\Cargo.toml`, but Kestrum is not registered there. Read-only inspection also found no Kestrum entry in the canonical `rust_management/workspace/Cargo.toml`.

The copied manifest still names the package `game_template` and points at `../../macroquad-toolkit`, which resolves to `D:\WebHatchery\macroquad-toolkit`. That toolkit manifest does not exist. The actual sibling toolkit is `D:\WebHatchery\RustGames\macroquad-toolkit`. This path issue is established by file inspection; Cargo did not reach dependency resolution during these checks.

Template onboarding must deliberately correct identity and the copied path, register the real project through the canonical shared configuration, and rerun checks. This task did not modify workspace membership, create a placeholder crate, change another project's files, use an alternate manifest, or validate an isolated copy.

## Evidence boundaries

The existing `ui_gameplay.png`, `ui_paused.png`, `ui_scrolled.png`, and `ui_zoomed.png` are pre-existing template captures. The root thumbnail is also supplied template artwork. They were preserved, not generated or reviewed as Kestrum evidence in this task. Future UI work must replace equivalent captures directly here and complete the [planned visual review](../10-interface-and-accessibility.md).

Documentation completion and an initial commit do not imply that the Kestrum game is implemented, builds, or has been published.
