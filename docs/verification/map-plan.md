# Map plan documentation verification

Recorded 2026-10-01 against the Kestrum checkout on `master`, starting at
`b6978a3`. This change plans the map improvements and reconciles documentation;
it does not implement the map milestones or establish new runtime results.

## Scope

- Added the active M01–M05 map playability plan, including observer-safe map
  information, contextual actions, persistent orders, seasonal outcomes,
  distinct regions, old-world compatibility and contextual onboarding.
- Replaced contradictory project/index/status text and the old prototype
  delivery sequence in active chapters. Updated all thirteen system chapters,
  the decision register, glossary and completed implementation packets.
- Clarified K01–K18 and B01–B07 completion under their recorded scope and retained
  real known limitations. Removed solved questions as implied approval gates.
- Added the evidence index, prioritizing the latest movement-plan record and
  distinguishing its final checks from checks on preceding builds.
- Kept original drafts and source hashes unchanged. Dated evidence retains its
  results; four links to removed decision-history sections now point to their
  current system chapters.

## Checks

Read current map rendering, movement, tutorial, round resolution, production
layout and saved-world validation. The plan accounts for strict authored graph
validation, saved route/site references, rules content-version checks and
serialized tutorial progress. A separate documentation review checked the plan
for contradictions, new approval barriers and unsupported validation claims.

Checked 918 relative Markdown file links and heading anchors across 64 maintained
documents, excluding immutable reference bodies from link maintenance. Checked
all four archived reference SHA-256 values against the
source-coverage manifest and checked the three root draft copies against their
archived counterparts. `git diff --check` passes. Changed files are Markdown
only; no Rust, content JSON, assets, saves or shared workspace configuration changed.

## Limits

No build, tests, capture or publish run was required for this documentation-only
change. Existing failures, deferred browser/performance/balance work and the
physical-touch waiver are linked from the [evidence index](README.md).
The five map milestones remain planned. The playability targets require future
implementation, interaction review and clearly scoped player observation.
