# K14 — Aging, recovery, retirement and mentorship

K14 implements the documented P20–P21 life-cycle rules across saved people,
campaign boundaries, combat, movement, transfer, development, player choices and
rival planning. The implementation decision and exact delegated choices are in
[I13](../13-decisions-and-open-questions.md#i13--aging-local-recovery-and-mentorship).

## Behavioral coverage

- Six lifecycle tests cover birthday/service chronology and K13-save defaults;
  supplied, unbesieged wound recovery and local Medic evidence; age-56 command,
  retirement and governor pressure; deterministic birthday mortality and duty
  cleanup; Light Cavalry movement; and nearest-refuge retirement.
- Eight mentorship tests cover qualification, one-learner capacity, cross-class
  career alternatives, physical-site pause/resume/end, injury and qualification
  loss, two/four-season progression, treatment facilities, Horse Access, and
  equivalent validated player/NPC choices.
- A wound regression confirms injured people cannot enter combat wipe rolls or
  consume field movement while still assigned to a formation.
- The full suite also includes the source-size gate and K13 save compatibility.

## Validation

From the actual Kestrum checkout:

```powershell
cargo fmt -p kestrum -- --check
..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
..\rust_management\cargo.ps1 test -p kestrum --locked --all-features
.\publish.ps1
```

All four passed. The suite contains 151 passing tests. The no-argument publisher
built and packaged Windows and WebGL, deployed to Preview, and recorded the
`rust_kestrum` publish with Project Roost.

## Native screen review

The shared hidden-window capture wrapper wrote directly to `docs/verification/`
and its game process exited after each batch. The lifecycle/person sheet,
available mentorship, wounded pause with Resume/End, and both Help pages were
reviewed at 1920 × 1080 and 1280 × 720:

- [Lifecycle, 1920 × 1080](ui_lifecycle.png) · [1280 × 720](ui_lifecycle_minimum.png)
- [Wounded person and local recovery, 1920 × 1080](ui_lifecycle_wounded.png) · [1280 × 720](ui_lifecycle_wounded_minimum.png)
- [Mentor choice, 1920 × 1080](ui_mentorship.png) · [1280 × 720](ui_mentorship_minimum.png)
- [Paused mentorship, 1920 × 1080](ui_mentorship_paused.png) · [1280 × 720](ui_mentorship_paused_minimum.png)
- [Aging Help, 1920 × 1080](ui_help_lifecycle.png) · [1280 × 720](ui_help_lifecycle_minimum.png)
- [Mentorship Help, 1920 × 1080](ui_help_mentorship.png) · [1280 × 720](ui_help_mentorship_minimum.png)

The twelve captures confirm the age/wound status, readable actions and non-overlapping
page controls at both native sizes. They do not verify WebGL scaling or physical
touch use. Browser interaction, the inherited minimum-WebGL campaign scaling
problem, and physical-touch acceptance remain K18 checks.
