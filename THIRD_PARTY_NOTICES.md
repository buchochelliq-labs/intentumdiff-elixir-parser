# Third-party notices

This repository vendors patched third-party crates under `patches/` (build-script
stabilization; see each directory's own license file): the base build chain
(`quote`, `serde_core`, `serde_json`, `zmij`, `wit-bindgen-rust-macro`, `tree-sitter`)
and, where present, this language's grammar crate(s):

- `tree-sitter-elixir`

All other dependencies are consumed unmodified from crates.io under their declared licenses.
