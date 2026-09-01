# rigforge_domain

V1-1 production thin Workflow Domain crate.

This is Product identity / version / provenance / validation code. It is not
a catalog database, GUI, Blender worker, Mapping algorithm, or Preview
generator.

```text
cargo test
```

Public API: validated constructors, read-only getters, `from_json_validated`,
`Validated<T>` store/application boundary, graph publication. Tests use no
network, Blender, FBX assets, or browser.

Rev1/Rev2 closed V1-1-MAJOR-001..004 and V1-1-MINOR-001 in this crate. Rust is
**Accepted** (`ADR-0002`, V1-1 Gate A). V1-1 is `COMPLETE / PASS / BASELINED`.
