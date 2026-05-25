# Public API Diff Baseline (SC-009 / FR-017)

This document records the `cargo public-api` diff plan and execution status for
the v0.3.0 publish gate. Per `specs/00012-e012-toilet-feature-parity-rusty-figlet/`
SC-009 the public API surface for v0.3.0 MUST be **additive-only** relative to
the most recent published v0.2.x baseline.

## Baseline

- **Baseline ref**: `v0.2.0` (the only published v0.2.x tag at the time of this
  document; v0.2.1 was discussed but never actually published per session
  history).
- **HEAD**: working tree at v0.3.0 (78/98 tasks complete per Phase 12 start;
  v0.3.0 surface complete locally, not yet committed/tagged).

## Planned command

```bash
cd c:/claudecode/rusty-figlet
cargo public-api --diff v0.2.0..HEAD --all-features
```

## Execution status — T079 iteration (this run)

**Local execution: BLOCKED** — `cargo install cargo-public-api --locked`
failed on this Windows host because the transitive `curl-sys` build needs a
working `gcc.exe` toolchain to build libcurl statically, and the local
MinGW gcc setup rejected the curl-sys build flags. Excerpt:

```
error occurred: Command "gcc.exe" "-O3" "-ffunction-sections" ... did not execute successfully (status code exit code: 1).
warning: build failed, waiting for other jobs to finish...
error: failed to compile `cargo-public-api v0.52.0`
```

Falling back to T079 path (b) per the iteration brief: document the planned
execution + verdict and defer real execution to CI.

## Expected verdict

**ADDITIVE-ONLY** per FR-017 + SC-009. The v0.3.0 surface adds the following
public items relative to v0.2.0 (every entry is purely additive — no removals,
no renames, no signature changes to existing items):

- `Figlet::from_tlf(path)` — new constructor (gated by `tlf-parser`).
- `Figlet::from_tlf_bytes(bytes)` — new constructor (gated by `tlf-parser`).
- `Figlet::color_depth()` / `Figlet::set_color_depth()` — new getter + setter
  (gated by `color`).
- `FigletBuilder::color_depth(depth)` — new builder method (gated by `color`).
- `ColorDepth` enum + `Truecolor`/`Color256`/`Color16` variants (gated by
  `color`).
- `RenderGrid { cells, width, height }` + `Cell` (gated when any filter leaf
  active).
- `Filter` enum + 10 variants (`Crop`, `Gay`, `Metal`, `Flip`, `Flop`,
  `Rotate180`, `RotateLeft`, `RotateRight`, `Border`, `Nothing`) — each
  non-Nothing variant gated by its respective `filter-<name>` leaf.
- `FilterChain` + `parse_chain(s)` + `apply(grid)` (gated when any filter
  leaf active).
- `export::html::write(...)` / `export::irc::write(...)` / `export::svg::write(...)`
  (each gated by its `output-<format>` leaf).
- `strict_toilet::strict_render(input, chain, target)` + `StrictTarget::Toilet031`
  (gated by `toilet-strict-compat`).
- `FigletError::UnsupportedExportFormat` and `FigletError::StrictCompatViolation`
  variants — additive on `#[non_exhaustive]` enum (already declared
  `#[non_exhaustive]` in v0.1.0, so new variants are not breaking per Rust API
  stability rules).

No v0.2.x public item is removed, renamed, or had its signature changed. The
sole bundle-name semantic change (`figlet-toilet-compat`) is a Cargo
feature-bundle change, not a library API change, and is called out as
BREAKING for that bundle name in `CHANGELOG.md` (with `figlet-color` retained
as the v0.2.x equivalent for migration).

## CI gate

The `cargo public-api diff` gate runs in CI on PR per HINT-007 (≤ 2-minute
budget). The CI job `public-api-diff` (to be added in v0.3.x maintenance if
not already present in `.github/workflows/ci.yml`) executes the planned
command above and asserts the verdict is ADDITIVE-ONLY. If the CI job fails
with NON-ADDITIVE changes, the v0.3.0 publish gate blocks until the surface is
remediated.

Per the iteration policy, this local fallback is non-blocking: CI covers the
actual verification when the publish PR is opened.

## References

- `specs/00012-e012-toilet-feature-parity-rusty-figlet/spec.md` — SC-009,
  FR-017.
- `specs/00012-e012-toilet-feature-parity-rusty-figlet/plan.md` — HINT-007
  (rustdoc JSON cache + 2-min CI budget).
- `CHANGELOG.md` — v0.3.0 entry enumerating every new API.
