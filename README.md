# Kestrum

Kestrum is a generational strategy game about connected territories, persistent armies, people shaped by service, and places changed by decades of war and peace.

**Current status:** complete founding design documentation with the supplied WebHatchery Macroquad template. Kestrum gameplay has not been implemented.

## Game documentation

Start with the [documentation index](docs/README.md). It links the complete design across world and time, kingdoms, economy, armies, logistics, battles, sieges, character development, living settlements, generations, history, UI, data, and implementation planning.

- [Vision and player experience](docs/01-vision-and-experience.md)
- [System decisions and open questions](docs/13-decisions-and-open-questions.md)
- [Delivery plan and template handoff](docs/12-delivery-and-validation.md)
- [Source coverage and preservation](docs/source-coverage.md)
- [Documentation verification](docs/verification/documentation.md)

The three founding drafts are preserved in full under `docs/reference/`. Their root-level copies can be deleted later without losing content from the documentation set. They remain in the initial commit.

## Template and implementation status

The initial commit includes all supplied template source, assets, tests, configuration, publishing scripts, thumbnail, and existing verification images. The original starter README is preserved in [template reference](docs/reference/template_readme.md).

The starter still uses `game_template`, its original toolkit dependency path, demo content, and capture identity. Kestrum is not yet registered in the shared Cargo workspace. Formatting, tests, and Clippy are blocked by that membership issue; the incorrect copied toolkit path is another onboarding requirement. See the [validation record](docs/verification/documentation.md) for exact results. Existing captures show the template, not Kestrum.

Implementation should first complete legitimate onboarding and recompose the map screen using the [Kestrum UI plan](docs/10-interface-and-accessibility.md). No game was published during the documentation milestone.

## Project guidance

Follow [AGENTS.md](AGENTS.md), [CODE_STANDARDS.md](CODE_STANDARDS.md), and [UI_STYLE.md](UI_STYLE.md). Toolkit and setup guidance lives in [MACROQUAD_TOOLKIT.md](MACROQUAD_TOOLKIT.md) and [GAME_DEVELOPMENT_GUIDE.md](GAME_DEVELOPMENT_GUIDE.md). Shared guidance is maintained in `../rust_management/docs/`; Kestrum-specific design belongs in `docs/`.

Use the shared build launcher for implementation checks, preserve Macroquad `=0.4.16`, and work on `master` unless a branch is explicitly requested. After meaningful game changes, complete the required native/browser, touch, and publishing validation from this actual checkout.
