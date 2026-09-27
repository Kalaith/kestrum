# K15 — Households, service entry and succession

K15 adds saved households and family relationships, sparse deterministic births,
adoption, trainee/service transitions, local apprenticeships and qualified legacy
designations. NPCs use the same validated commands. K14 saves migrate with empty
relationship records and without consuming the people random stream.

## Behavioral coverage

The nine focused succession tests cover four-season partnership qualification
and atomic rejection; parent/child and shared-guardian exclusion; ward population
cost and fresh Recruit records; trainee and service age gates; apprentice cost,
population and yearly limit; deterministic Spring births and save/resume; adoptive
household succession and local command succession; K14 migration; and rival
household formation through the shared validated command path.

## Validation

From the actual Kestrum checkout:

```powershell
cargo fmt -- --check
..\rust_management\cargo.ps1 clippy --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test --locked
.\publish.ps1
```

Formatting, strict Clippy, all 160 locked tests, and the Rust source-size gate
passed. Every Rust source file is at most 800 lines. The no-argument
publisher built and packaged Windows and WebGL, deployed to Preview, and recorded
the `rust_kestrum` publish with Project Roost.

## Native screen review

The shared hidden-window capture wrapper wrote directly to `docs/verification/`
and its launched game exited after each capture batch. Household and succession
screens were reviewed at 1920×1080 and the 1280×720 minimum native size:

- [Households, 1920×1080](ui_households.png) · [1280×720](ui_households_minimum.png)
- [Succession, 1920×1080](ui_succession.png) · [1280×720](ui_succession_minimum.png)

The captures show named household roles, a dependent ward, the legacy categories
and link choices, and a valid local adopted-successor selection. Text and controls
remain within the panel at both sizes. Button targets are at least 48 logical
pixels high. The browser interaction, WebGL minimum-size review and physical-touch
acceptance remain K18 checks; these captures do not claim them.
