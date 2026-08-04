# intentumdiff-elixir-parser

[![CI](https://github.com/buchochelliq-labs/intentumdiff-elixir-parser/actions/workflows/ci.yml/badge.svg)](https://github.com/buchochelliq-labs/intentumdiff-elixir-parser/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust 1.95](https://img.shields.io/badge/rust-1.95-orange.svg)](https://www.rust-lang.org/)

The **elixir parser plugin** for IntentumDiff — a Wasm component (WASI p2, Component Model)
implementing the `intentumdiff:plugin` parser interface: it emits a deterministic
`SemanticNode` tree for elixir sources.

## Build

```bash
cargo build --release --target wasm32-wasip2
```

Toolchain: Rust 1.93.0 (pinned in CI). Tests: `cargo test`.

CI is a thin caller of the reusable `parser-ci.yml` owned by
[intentumdiff-plugin-sdk](https://github.com/buchochelliq-labs/intentumdiff-plugin-sdk) —
parser-CI fixes happen once there, not per parser repo. The `.claude/skills/` copies are
stamped from the SDK master; edits belong there.

## Provenance

Migrated files-only (no history) from the IntentumDiff monorepo
(`buchochelliq-labs/intentumdiff`), which remains the archive of record.

License: MIT.
