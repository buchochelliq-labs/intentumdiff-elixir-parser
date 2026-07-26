# intentdiff-elixir-parser

The **elixir parser plugin** for IntentDiff — a Wasm component (WASI p2, Component Model)
implementing the `intentdiff:plugin` parser interface: it emits a deterministic
`SemanticNode` tree for elixir sources.

## Build

```bash
cargo build --release --target wasm32-wasip2
```

Toolchain: Rust 1.93.0 (pinned in CI). Tests: `cargo test`.

CI is a thin caller of the reusable `parser-ci.yml` owned by
[intentdiff-plugin-sdk](https://github.com/buchochelliq-labs/intentdiff-plugin-sdk) —
parser-CI fixes happen once there, not per parser repo. The `.claude/skills/` copies are
stamped from the SDK master; edits belong there.

## Provenance

Migrated files-only (no history) from the IntentDiff monorepo
(`buchochelliq-labs/intentdiff`), which remains the archive of record.

License: MIT.
