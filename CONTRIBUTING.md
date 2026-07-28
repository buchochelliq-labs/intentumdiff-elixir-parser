# Contributing to intentdiff-elixir-parser

- Follow the
  [plugin guide](https://github.com/buchochelliq-labs/intentdiff-plugin-sdk/blob/main/docs/PLUGIN_GUIDE.md):
  deterministic trees, 0-based positions, stable labels, structural hashes, no source text in
  facts, in-band error envelopes (never a panic across the boundary).
- Build + test per [BUILDING.md](BUILDING.md); CI is a thin caller of the SDK's reusable
  `parser-ci.yml` — CI fixes belong in the SDK repo.
- The `.claude/skills/` copies here are STAMPED from the SDK master — never edit them in this
  repo; change the masters and let the fan-out re-stamp.
- Released artifacts are pinned (commit SHA + SHA-256 checksums) in
  [intentdiff-registry](https://github.com/buchochelliq-labs/intentdiff-registry) — an
  artifact-affecting change ends with a registry PR updating the entry.
