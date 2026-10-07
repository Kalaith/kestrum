# Observer household abandonment — 2026-10-07

## Report and diagnosis

The supplied screenshot shows Winter / Year 24, Ashen Wren acting, and
`campaign.households.home: active household must retain its friendly inhabited seat`.
The matching local audit is
`%LOCALAPPDATA%\kestrum\observer_logs\observer-260926-1791364636863.jsonl`.
Its last record reports this error at completed round 95. The last accepted
action, sequence 1781, recruited Warriors into Army 15 at site 48.

The final snapshot records Rose with 55 sites, Ashen Lark with 30, and Ashen
Wren with 50. Bera Lark has zero sites and was already eliminated. No new
pending defeat is recorded. This was not a fresh kingdom elimination.
The audit snapshots omit households and settlement habitation, so those
snapshots alone cannot identify the invalid household or prove the failed
season's settlement transition.

The source allows seasonal development to downgrade a Camp to Unsettled
without changing its controller. Household cleanup previously detected only
controller changes, leaving an active household at an uninhabited home and
causing final campaign validation to reject the whole action.

## Scope

End an affected household with the distinct `SiteAbandoned` reason when its
home becomes Unsettled; stop childraising and preserve family history.
Capture and kingdom defeat retain their separate reasons and existing rules.
The campaign invariant remains enforced. The AI recovery work and its later
lint/formatting cleanup are recorded in a separate commit and
[verification record](ai-recovery.md).

## Validation

The initial focused regression reproduced the exact `campaign.households.home`
error before the fix, through ordinary End Turn commands from a valid state.
Both tests in `tests/household_capture.rs` now pass. They check unchanged site
ownership during abandonment, household closure, disabled childraising, an
unaffected household remaining active, history text, persistence through
save/load, and ordinary hostile capture retaining its separate end reason.

The existing headless observer example passed with seed 260926, four factions,
and a 160-round cap, continuing past the reported round-95 failure without an
error. It validates the final campaign and left it still running at the cap.
This is a new seeded campaign using the current checkout, not a loaded copy of
the historical audit. The run took 2m07s wall time, including a reported 1m06s
release compilation and 59.74s simulation time.

```powershell
& ..\rust_management\cargo.ps1 test --test household_capture
& ..\rust_management\cargo.ps1 run -p kestrum --locked --release --example observe_campaign '--' 260926 160 4
```

On 2026-10-08 the user requested as little testing as possible. The completed
focused tests and replay are reused; the planned full-suite run and additional
simulations are deliberately unrun under that instruction. The historical
seed-88 round-240 failure was not rechecked.

The first project-wide formatting and strict Clippy checks found existing AI
recovery formatting differences and a `clippy::question_mark` warning. The
household files passed a scoped formatting check. The user then authorized
fixing the AI failures and committing both completed changes. The AI cleanup
uses rustfmt and replaces the equivalent `let Some(...) else { return None; }`
with `?`; the final project-wide check results are recorded below.

```powershell
& ..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
```

Final project-wide `cargo fmt -p kestrum -- --check` and strict Clippy both
pass after the authorized AI cleanup. Clippy took 5.51s. These final checks
compile all targets, including the repaired AI test fixtures; no additional
behavioral tests or simulation runs were added.

The direct source-size scan passed: 477 Rust files, maximum 798 physical lines,
and zero files above the 800-line limit. The native release game build passed
in 1m36.89s wall time (Cargo reported 1m36s):

```powershell
& ..\rust_management\cargo.ps1 build -p kestrum --locked --release --bin kestrum
```

External publication is deliberately unrun under `PROJECT_AGENTS.md`. No
rendering or control layout changes are part of this fix; no visual acceptance
is claimed.
