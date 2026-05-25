//! E012 Phase 5 — `-F` CLI flag integration tests (T031).
//!
//! Drives the `rusty-figlet` binary with `assert_cmd` to verify:
//!  - Unknown filter names exit non-zero with the canonical filter
//!    list on stderr per FR-016 + spec Edge Cases.
//!  - Valid `-F` chains accept and run cleanly under `--all-features`.
//!  - Multiple `-F` flags concatenate per FR-002.

#![cfg(feature = "cli")]

use assert_cmd::Command;
use predicates::prelude::*;

/// Set `CLICOLOR_FORCE`/`NO_COLOR` so the CLI test harness produces
/// deterministic byte streams regardless of the runner's TTY state.
fn cmd() -> Command {
    let mut c = Command::cargo_bin("rusty-figlet").expect("binary built");
    c.env("NO_COLOR", "1");
    c
}

#[test]
fn unknown_filter_returns_enumerable_error() {
    let assert = cmd()
        .args(["-F", "nosuchfilter", "Hello"])
        .assert()
        .failure();
    let out = assert.get_output();
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("unknown filter"),
        "stderr should mention 'unknown filter', got: {stderr}"
    );
    // Enumerated filter list (FR-016).
    assert!(stderr.contains("crop"));
    assert!(stderr.contains("gay"));
    assert!(stderr.contains("metal"));
    assert!(stderr.contains("flip"));
    assert!(stderr.contains("flop"));
    assert!(stderr.contains("rotate180"));
    assert!(stderr.contains("rotateleft"));
    assert!(stderr.contains("rotateright"));
    assert!(stderr.contains("border"));
    assert!(stderr.contains("nothing"));
    assert!(stderr.contains("nosuchfilter"));
}

#[test]
fn empty_segment_in_chain_rejected() {
    cmd()
        .args(["-F", "crop::flip", "Hello"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("unknown filter"));
}

#[cfg(all(feature = "filter-crop", feature = "filter-border"))]
#[test]
fn valid_filter_chain_accepted() {
    cmd().args(["-F", "crop:border", "Hi"]).assert().success();
}

#[cfg(all(feature = "filter-flip", feature = "filter-flop"))]
#[test]
fn multiple_dash_f_flags_concatenate() {
    // FR-002: two -F flags become "flip:flop" via the join(":") in main.rs.
    cmd()
        .args(["-F", "flip", "-F", "flop", "Hi"])
        .assert()
        .success();
}

#[test]
fn nothing_filter_always_available() {
    // Filter::Nothing has no leaf gate.
    cmd().args(["-F", "nothing", "Hi"]).assert().success();
}
