# M03 regional geography

## Compatibility boundary

Campaign worlds now record `topology_revision` independently of presentation
`layout_revision` and campaign rules. Missing topology fields default to the
original graph (revision 1). `assets/data/world_layout.json` remains unchanged
as its durable compatibility source. Unknown topology and mixed presentation
revisions fail validation through the existing recoverable load path.

The atlas integrity checks validate supported historical layouts 1-4, missing
field deserialization with full world equality, and malformed revision/geometry
rejection. These are content-integrity fixtures, not a claim that the user's
on-disk catalogue or every historical battle has been played through.

Formatting, strict all-target/all-feature Clippy and the full remaining test
suite pass. The broader suite is justified by the save/generation boundary;
it includes source-size and asset-registry gates. No visual behavior changes
in this slice; no captures or publication are required for it.

The full suite passes 38 tests with one release profiling case ignored. A final
unsupported-version fixture adjustment passes the focused four-case atlas
integrity target and another strict Clippy run.

Commands: `cargo fmt --all -- --check`,
`..\rust_management\cargo.ps1 clippy -p kestrum --all-targets --all-features '--' -D warnings`,
and `..\rust_management\cargo.ps1 test -p kestrum`.
