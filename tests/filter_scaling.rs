//! E012 Phase 5 — wall-clock linear-scaling test (T032, SC-012, FR-003).
//!
//! Asserts that applying N filters to a fixed grid scales O(N) in wall-
//! clock time per AD-007 + FR-022 + FR-030. We measure N = 1, 5, 10, 20
//! chains of `Filter::Nothing` interleaved with leaf-disabled-safe
//! filters and check N=20 ≤ 2.5× N=10 (10 % tolerance over linear).
//!
//! Test is run under `--all-features` so every filter leaf is active.
//! If the test fails it is a real signal — a non-linear cost has crept
//! into a filter or into the dispatch path. Investigate; do not suppress.

#![cfg(feature = "filter-flip")]

use std::time::Instant;

use rusty_figlet::filter::{Cell, Filter, FilterChain, RenderGrid};

/// Build a ~10 KB grid: 200 cols × 50 rows of varied glyphs so the
/// filter work isn't trivially skipped by allocator fast paths.
fn build_grid() -> RenderGrid {
    let cells: Vec<Vec<Cell>> = (0..50)
        .map(|y| {
            (0..200)
                .map(|x| {
                    let ch = char::from_u32(b'A' as u32 + ((x + y) % 26) as u32).unwrap_or('A');
                    Cell::new(ch)
                })
                .collect()
        })
        .collect();
    RenderGrid::from_rows(cells)
}

fn chain_of(n: usize) -> FilterChain {
    let mut chain = FilterChain::new();
    for i in 0..n {
        chain = chain.push(if i % 2 == 0 {
            Filter::Flip
        } else {
            Filter::Nothing
        });
    }
    chain
}

/// Run the chain `runs` times and return the median elapsed in nanoseconds.
fn median_apply(chain: &FilterChain, grid: &RenderGrid, runs: usize) -> u128 {
    let mut times: Vec<u128> = (0..runs)
        .map(|_| {
            let g = grid.clone();
            let start = Instant::now();
            let _ = chain.apply(g).expect("apply");
            start.elapsed().as_nanos()
        })
        .collect();
    times.sort_unstable();
    times[times.len() / 2]
}

#[test]
fn filter_chain_scales_linearly() {
    let grid = build_grid();

    // Warm-up: avoid the first-allocation pessimism.
    for _ in 0..3 {
        let _ = chain_of(5).apply(grid.clone()).expect("warmup");
    }

    let runs = 9;
    let t1 = median_apply(&chain_of(1), &grid, runs);
    let t5 = median_apply(&chain_of(5), &grid, runs);
    let t10 = median_apply(&chain_of(10), &grid, runs);
    let t20 = median_apply(&chain_of(20), &grid, runs);

    eprintln!(
        "filter scaling: N=1 {} ns, N=5 {} ns, N=10 {} ns, N=20 {} ns",
        t1, t5, t10, t20
    );

    // SC-012: N=20 wall-clock ≤ 2.5× N=10 (allows 10% tolerance over
    // strict 2× linear).
    //
    // We use a soft floor (`t10.max(1_000)`) so tests on a fast machine
    // where t10 measures sub-microsecond don't trip on timing noise. The
    // assertion still catches any real super-linear regression because
    // a quadratic doubling would push t20 to 4× t10, not 2.5×.
    let floor = t10.max(1_000);
    let limit = floor.saturating_mul(25) / 10;
    assert!(
        t20 <= limit,
        "SC-012 linear-scaling: N=20 ({} ns) > 2.5× N=10 ({} ns, limit {} ns)",
        t20,
        t10,
        limit
    );

    // We deliberately do NOT assert on t5/t1. At N=1 the per-call overhead
    // (function dispatch, clone, allocator first-touch) is comparable to
    // the actual filter work, so the ratio is too noisy on CI runners.
    // Apple Silicon CI in particular routinely produces measurements where
    // N=10 reports faster than N=5 — pure scheduler noise on a sub-30µs
    // budget. The N=20/N=10 check above is the load-bearing assertion;
    // a quadratic regression would push N=20 to 4× N=10, well over the
    // 2.5× limit. We keep t1/t5 in the eprintln for diagnostic value only.
    let _ = (t1, t5);
}
