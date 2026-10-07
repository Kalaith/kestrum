# Mid-game AI recovery

## Observed failure

The local audit `observer-260926-1791349388141.jsonl` was inspected through
2,852 observer steps, completed round 516, with seed 260926 and four factions.
It records legal passes rather than a simulation error.

- Ashen Wren (yellow, faction 4) raised its second army in round 0. Both armies
  had six formations by round 22, with no subsequent recruitment.
- At round 33, line 633, Rose captured Eastfold Gate (site 55). Both yellow
  armies lost their route to headquarters. The seven-node headquarters
  component had no army; the seventeen-node enclave held both armies.
- Every foreign exit from that enclave belonged to Rose, with whom yellow
  later had peace. Yellow remained at war with Ashen Lark elsewhere.
- Outpost order 57 at site 1080 retained Army 8 as its builder while paused
  for lost supply at 2/3 progress. Its last progress was round 32.
- At round 515, line 2849, both armies still had six movement and 580 troops,
  but no reachable neutral or wartime targets. Yellow had 60,965 gold before
  the pass, 24 owned sites, and no upkeep deficit.
- City candidates 1076, 1078 and 1079 failed the ordinary supply requirement.
  These failures recur in 416 records from rounds 100 through 515. Yellow's
  three city investments were all in its headquarters component.

The old planner normally targeted two armies. It required two supplied armies
before declaring war, so fully equipped isolated armies could neither reopen
the corridor nor trigger recruitment in their supplied homeland. Open paused
construction also excluded a builder from ordinary military movement forever.
Blue had a similar split between its headquarters and a forty-node enclave.

## Recovery scope

The shared NPC planner handles recovery in both ordinary and Observer play.
It releases builders from persistently supply-blocked work and evaluates known
routes that reconnect isolated forces to supplied territory. Peaceful foreign
territory still requires a legal war declaration before entry; truces, observed
enemy strength, ordinary recruitment costs and supply checks remain authoritative.

Army demand can grow with developed holdings, within authored bounds and
income/reserve checks. An isolated understrength army cannot prevent a legal
relief force from forming in the homeland. City investments retain their paid,
safe, supplied requirements and the intervening-node spacing rule.

## Reproduction and validation

The source audit contains state summaries rather than a loadable campaign. A
fresh replay with the same seed can diverge before the historical cutoff after
the planner changes; focused regressions separately recreate the recovery gates.

Run the permanent headless observer tool through the shared pool:

```powershell
& ..\rust_management\cargo.ps1 run -p kestrum --locked --release --example observe_campaign '--' 260926 160 4
```

It prints activity and faction summaries without opening a game window, writing
saves, or altering the user's observer logs. Arguments are seed, round cap and
faction count. A quiet faction alone is not a failed simulation: its strength,
supply, legal targets and continuing economic actions need to be considered.

The existing AI recovery implementation was reviewed alongside the
household abandonment fix. On 2026-10-08 the user
authorized resolving the AI lint/formatting failures and committing the work.
The cleanup applies rustfmt and replaces a `let Some(...) else { return None; }`
in route search with the equivalent `?` expression; it changes no planner rule.
The all-target check also exposed compile errors in the recovery test fixture.
That fixture now owns its small setup/turn helpers instead of importing a
different suite's support module, removes an unused import, and uses the owned
`SiteId` returned by its iterator without dereferencing it.

The completed observer replay reached round 160 in 2,975 steps, with no error
and a valid final campaign. At that cap, Rose had seven armies, Ashen Wren six,
and Ashen Lark two (peak four); Bera Lark was eliminated. All three surviving
factions had taken a non-pass action in rounds 158 or 159. These observations
show continued activity and growth in this seeded run, not general balance
acceptance. The replay included the household fix and cannot establish the
behavior of the AI-only commit in isolation.

The native release game build also passed before the mechanical cleanup.
Final project-wide formatting and strict all-target/all-feature Clippy pass.
The source-size scan passes for all 477 Rust files, with a maximum of 798 lines
and none above 800. The final commands were:

```powershell
cargo fmt -p kestrum -- --check
& ..\rust_management\cargo.ps1 clippy -p kestrum --locked --all-targets --all-features '--' -D warnings
```

The AI-specific behavioral targets were not run; the user requested
as little testing as possible, so the existing replay/build evidence is reused
and the full project suite remains unrun. Tests are compiled by all-target
Clippy; compilation is not a claim that they ran.

External publication is deliberately excluded by `PROJECT_AGENTS.md`.
