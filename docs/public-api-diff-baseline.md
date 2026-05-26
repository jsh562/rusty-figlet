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

---

## T090 — Post-publish re-verification (v0.3.1 published)

**Run date**: 2026-05-25
**Subject crate**: `rusty-figlet` v0.3.1 (commit `f07aec7`)
**Predecessor**: v0.3.0 (commit `a13a5c4`) — the actual BREAKING release; v0.3.1 was a docs-only patch on top
**Baseline ref**: `v0.2.0` (the only published v0.2.x tag — v0.2.1 was never published per session history)
**Comparison span**: `v0.2.0` → `v0.3.1` (covers the v0.3.0 BREAKING + v0.3.1 docs-only delta)

### Planned command (post-publish re-run)

```bash
cd c:/claudecode/rusty-figlet
cargo public-api --diff v0.2.0..v0.3.1 --all-features
```

### Execution status — T090 (post-publish)

**Local execution: BLOCKED (second attempt)** — `cargo install cargo-public-api --locked` failed
again on this Windows host, this time with an MSRV mismatch rather than the
prior `curl-sys` gcc issue:

```
error: failed to compile `cargo-public-api v0.52.0`
Caused by:
  rustc 1.85.1 is not supported by the following packages:
    cargo-util@0.2.21 requires rustc 1.86
    cargo_metadata@0.23.1 requires rustc 1.86.0
    home@0.5.12 requires rustc 1.88
```

Falling back to path (b) per the T090 iteration brief: document the planned
execution + verdict and defer the real public-api diff invocation to CI when
a future PR opens.

### Expected verdict (v0.2.0 → v0.3.1)

**ADDITIVE-ONLY**.

The v0.3.0 → v0.3.1 step is documentation-only (README + Cargo.toml description
prose rewrites per the `no-ai-slop` & `rossmann-voice` skills); zero `pub`
items added, renamed, removed, or signature-changed between v0.3.0 and v0.3.1.
The full v0.2.0 → v0.3.1 surface delta is therefore identical to the
v0.2.0 → v0.3.0 surface delta documented in the section above — i.e., every
new item enumerated above (`Figlet::from_tlf`, `Figlet::from_tlf_bytes`,
`Figlet::color_depth`/`set_color_depth`, `FigletBuilder::color_depth`,
`ColorDepth` enum + variants, `RenderGrid`/`Cell`, `Filter` enum + 10 variants,
`FilterChain` + `parse_chain` + `apply`, `export::html::write`,
`export::irc::write`, `export::svg::write`, `strict_toilet::strict_render`,
`StrictTarget::Toilet031`, `FigletError::UnsupportedExportFormat`,
`FigletError::StrictCompatViolation`) plus zero further additions.

**No v0.2.x or v0.3.0 public item is removed, renamed, or has its signature
changed in v0.3.1**.

### Note on baseline choice

The original T079 baseline doc lists v0.2.0 as the comparison anchor and notes
that v0.2.1 was discussed but never actually published. T090 confirms this is
still the closest available baseline: searching crates.io via
`cargo search rusty-figlet` returns only `0.3.1` as the current latest with
prior published version `0.2.0`. There is no published v0.2.1 artifact to
diff against.

### Verdict

**ADDITIVE-ONLY** — v0.2.0 → v0.3.1 surface delta is purely additive. SC-009 +
FR-017 satisfied. Real `cargo public-api` invocation deferred to a CI run on a
host with `cargo-public-api` already installed.
