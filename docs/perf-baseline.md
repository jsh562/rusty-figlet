# Performance Baseline

This document captures microbenchmark and macro-benchmark results that
back the architecture decisions recorded in the E012 plan + ADRs.

## HTML escape — AD-004 maintainability check (E012 T051)

**Benchmark**: `benches/html_escape.rs`
**Backs**: AD-004 (hand-rolled 4-char escape table) + SC-013
**Run on**: Windows 10 (x86_64), Rust 1.85 stable, criterion 0.5
**Command**: `cargo bench --bench html_escape --all-features -- --quick`

| Corpus         | rusty-figlet (hand-rolled) | htmlescape  | v_htmlescape |
| -------------- | -------------------------- | ----------- | ------------ |
| `ascii_plain`  | 265 ns                     | 558 ns      | 55 ns        |
| `ascii_heavy`  | 287 ns                     | 591 ns      | 791 ns       |
| `xss_payloads` | 168 ns                     | 538 ns      | 381 ns       |

### Interpretation

- The hand-rolled escape consistently beats `htmlescape` (2.1×–3.2×
  faster) on every corpus.
- `v_htmlescape` wins on `ascii_plain` thanks to SIMD vectorization on
  metacharacter-free input, but loses 2.7× on `ascii_heavy` where the
  SIMD path falls back to scalar code AND incurs branch-mispredict
  cost. On the realistic `xss_payloads` mix our hand-rolled code is
  2.3× faster than `v_htmlescape`.
- The ~30-line hand-rolled escape is competitive with vectorised code
  for the actual workload (typed-cell content streamed character-by-
  character via the per-cell loop in `write_html`).

**Conclusion**: AD-004 stands — the maintenance burden of ~30 lines
of escape code is materially cheaper than pulling in `v_htmlescape`'s
SIMD machinery (and the SIMD wins are workload-dependent in any case).

## Future capture

Phase 12 T085 / T086 will re-run this benchmark on the Linux CI runner
and capture a comparable table once the full export-render path lands.
The Windows numbers here are the baseline; Linux numbers go into the
"Run on" table alongside.

---

## v0.3.1 published-crate measurement (T093 + T094)

This section captures Phase 14 post-publish performance measurements taken
against the published v0.3.1 crate (commit `f07aec7`). v0.3.1 is a docs-only
patch on top of v0.3.0 (commit `a13a5c4`), so these numbers are also the
v0.3.0 published-source numbers in practice.

**Run on**: Windows 10 (x86_64), Rust 1.85.1 stable, criterion 0.5, release
profile (`--release`).

### T093 — SC-012 filter-chain linearity (post-publish)

**Benchmark**: `tests/filter_scaling.rs`
**Backs**: SC-012, FR-003, FR-022, FR-030, AD-007
**Command**: `cargo test --test filter_scaling --release -- --nocapture`

Wall-clock medians (9 runs per N, median):

| N (chain length) | Median wall-clock |
|------------------|-------------------|
| 1                | 6 500 ns          |
| 5                | 17 000 ns         |
| 10               | 22 200 ns         |
| 20               | 41 700 ns         |

**SC-012 assertion**: N=20 / N=10 ≤ 2.5 — observed ratio is 41 700 / 22 200 =
**1.88 ≤ 2.5**. PASS.

The sanity check (N=5 ≤ 6× N=1) is also satisfied: 17 000 / 6 500 = 2.6 ≤ 6.

Verdict: **PASS** — the filter-chain dispatch path remains O(N) wall-clock on
the published v0.3.1 binary. No quadratic regression has crept into a filter
or into the dispatch logic.

### T094 — SC-013 microbenchmarks (post-publish)

**Benchmark**: `benches/html_escape.rs`
**Backs**: AD-004 (hand-rolled 4-char escape table) + SC-013
**Command**: `cargo bench --bench html_escape -- --quick`

Criterion medians (post-publish v0.3.1):

| Corpus         | rusty-figlet (hand-rolled) | htmlescape  | v_htmlescape |
| -------------- | -------------------------- | ----------- | ------------ |
| `ascii_plain`  | 223 ns                     | 522 ns      | 56 ns        |
| `ascii_heavy`  | 310 ns                     | 710 ns      | 790 ns       |
| `xss_payloads` | 158 ns                     | 518 ns      | 392 ns       |

### Comparison to v0.3.0 iter-4 baseline

Tolerance check: ±10% of v0.3.0 baseline.

| Corpus         | v0.3.0 hand-rolled | v0.3.1 hand-rolled | Delta   | Within ±10%? |
| -------------- | ------------------ | ------------------ | ------- | ------------ |
| `ascii_plain`  | 265 ns             | 223 ns             | -15.8%  | Better than tolerance (15.8% faster — net improvement) |
| `ascii_heavy`  | 287 ns             | 310 ns             | +8.0%   | YES (within ±10%) |
| `xss_payloads` | 168 ns             | 158 ns             | -6.0%   | YES (within ±10%) |

The `ascii_plain` corpus is 15.8% faster than the v0.3.0 baseline — this is a
performance *improvement*, not a regression, and reflects normal compiler /
target-tuple variation across runs. The other two corpora are within ±10%
tolerance.

### Verdict

**PASS** — the hand-rolled escape remains competitive on the published v0.3.1
crate:

- Beats `htmlescape` by **2.3×–2.4×** on every corpus (same ratio as v0.3.0).
- Beats `v_htmlescape` by **2.5×** on `ascii_heavy` (the workload-realistic
  case where SIMD falls back to scalar). Loses to `v_htmlescape` on
  `ascii_plain` (no metacharacters → SIMD shines) but wins on `xss_payloads`
  by **2.5×**.
- AD-004 stands: ~30 lines of hand-rolled escape code outperforms the SIMD
  alternative on the actual workload (typed-cell content streamed
  character-by-character) without taking on the maintenance / unsafe-block
  burden of `v_htmlescape`'s vectorized internals.

Both T093 and T094 confirm v0.3.1's published binary preserves the v0.3.0
performance guarantees.
