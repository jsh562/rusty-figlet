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
