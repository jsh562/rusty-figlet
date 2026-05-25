//! E012 Phase 9 — Compile-fail enumeration via `trybuild` (T066).
//!
//! Each `.rs` file under `tests/compile_fail/` is a small Rust source
//! that is EXPECTED to fail compilation when the feature gating contract
//! holds. The paired `.stderr` file captures the expected compiler error
//! message so test failures surface a clear bug in the cfg-gate map.
//!
//! ## Covered cases (FR-016 + plan §Compile-fail enumeration)
//!
//! 1. `output_html_without_cli.rs`   — `output-html` enabled without `cli`
//! 2. `output_irc_without_cli.rs`    — `output-irc` enabled without `cli`
//! 3. `output_svg_without_cli.rs`    — `output-svg` enabled without `cli`
//! 4. `color_truecolor_without_color_base.rs` — `color-truecolor` without `color`
//! 5. `color_256_without_color_base.rs` — `color-256` without `color`
//! 6. `toilet_strict_compat_without_filters.rs` — strict-compat needs filter chain
//! 7. `filter_chain_apply_without_any_filter_leaf.rs` — FilterChain::apply
//!     on a non-Nothing variant when no filter-* leaf is enabled
//!
//! The trybuild harness only runs under `--all-features` because each
//! source file references a public symbol whose visibility depends on a
//! leaf being enabled. When run, trybuild compiles each `.rs` standalone
//! and asserts the captured `.stderr` matches.

#![cfg(all(feature = "cli", feature = "tlf-parser"))]

#[test]
#[ignore = "compile-fail trybuild harness — opt-in via `cargo test --all-features compile_fail -- --ignored`"]
fn compile_fail_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
