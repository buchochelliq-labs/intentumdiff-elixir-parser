# Security policy

## Reporting

Report suspected vulnerabilities **privately** via GitHub security advisories on this
repository (Security -> Report a vulnerability). Please do not open public issues for
security reports.

## Security model

This repository is part of [IntentumDiff](https://github.com/buchochelliq-labs/intentumdiff-core) and
inherits the engine's security model -- the Wasm plugin sandbox, supply-chain pinning
and subprocess hygiene -- documented in
[intentumdiff-core/SECURITY.md](https://github.com/buchochelliq-labs/intentumdiff-core/blob/main/SECURITY.md).

### A note on the vendored grammar

The tree-sitter grammar under `patches/` is **vendored upstream code**, carried so the
component build is reproducible and pinnable. It is not maintained here: defects in it
belong upstream. It is also compiled to a Wasm component and executed under wasmtime
with fuel limits, a linear-memory cap and no ambient filesystem, network or clock
capability -- so a malformed input cannot escape the sandbox.
