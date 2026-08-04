# Building intentumdiff-elixir-parser

Toolchain: **Rust 1.93.0**; target `wasm32-wasip2`.

```bash
rustup toolchain install 1.93.0
rustup target add wasm32-wasip2

cargo build --release --target wasm32-wasip2   # the component
cargo test                                      # SDK compliance + Tier-B tests
```

The component lands at `target/wasm32-wasip2/release/intentumdiff_elixir_parser.wasm`.
The SDK dependency is a git dep on
[intentumdiff-plugin-sdk](https://github.com/buchochelliq-labs/intentumdiff-plugin-sdk) pinned by
tag; for a private clone set `CARGO_NET_GIT_FETCH_WITH_CLI=true`. C-backed grammars need a
wasm32 C compiler (clang; CI has it).
