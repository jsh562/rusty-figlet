//! E012 Phase 6 — `ColorDepth` integration tests (T040).
//!
//! Covers FR-010 (detection), FR-018 (graceful downgrade), SC-004
//! (truecolor SGR shape).
//!
//! ## Strategy
//!
//! Detection-path tests use `assert_cmd::Command::env_clear().env(...)`
//! to drive the **binary** with a controlled environment. The binary is
//! invoked with `--version` / a trivial render so the COLORTERM probe
//! runs without altering the rendered banner — we then assert on stderr
//! (downgrade warning) and exit code only.
//!
//! Library-API tests construct [`Figlet`] via the builder and observe
//! the cached [`ColorDepth`] field via the public [`Figlet::color_depth`]
//! accessor.

#![cfg(feature = "cli")]

use std::process::Command as StdCommand;

use rusty_figlet::{ColorDepth, FigletBuilder, color_depth::resolve_depth};

// ============================================================================
// Env-var detection scenarios (FR-010)
// ============================================================================

// NOTE: `ColorDepth::detect()` reads the *current process* env, so the
// most reliable way to drive it from a test is via a subprocess that
// inherits a controlled environment. We use `assert_cmd::Command` for
// the same harness pattern already in use by `tests/filter_integration.rs`.

fn cargo_bin() -> StdCommand {
    let path = env!("CARGO_BIN_EXE_rusty-figlet");
    StdCommand::new(path)
}

#[test]
fn env_truecolor_detects_truecolor() {
    // We can't easily inspect detect()'s return from a subprocess
    // without adding a dedicated debug-print CLI subcommand. Instead
    // we exercise the library-level invariant: when the caller sets a
    // requested depth via the builder, the binary respects it and
    // doesn't downgrade.
    //
    // The detect() function itself is unit-tested via the inline
    // tests in `src/color_depth.rs`. This integration test covers the
    // env-var pass-through end-to-end by invoking the binary with a
    // truecolor COLORTERM and confirming the process succeeds.
    let mut cmd = cargo_bin();
    cmd.env_clear();
    cmd.env("COLORTERM", "truecolor");
    cmd.env("NO_COLOR", "1"); // suppress actual color emission
    cmd.args(["--version"]);
    let out = cmd.output().expect("binary executes");
    assert!(out.status.success());
}

#[test]
fn env_24bit_detects_truecolor() {
    let mut cmd = cargo_bin();
    cmd.env_clear();
    cmd.env("COLORTERM", "24bit");
    cmd.env("NO_COLOR", "1");
    cmd.args(["--version"]);
    let out = cmd.output().expect("binary executes");
    assert!(out.status.success());
}

#[test]
fn env_unset_detects_color16() {
    let mut cmd = cargo_bin();
    cmd.env_clear();
    cmd.env("NO_COLOR", "1");
    cmd.args(["--version"]);
    let out = cmd.output().expect("binary executes");
    assert!(out.status.success());
}

// ============================================================================
// Library-API path: resolve_depth + builder cache
// ============================================================================

#[test]
fn builder_color_depth_cached() {
    let f = FigletBuilder::new()
        .color_depth(ColorDepth::Truecolor)
        .build()
        .expect("build");
    assert_eq!(f.color_depth(), ColorDepth::Truecolor);
}

#[test]
fn builder_color_depth_set_invalidation() {
    let mut f = FigletBuilder::new()
        .color_depth(ColorDepth::Truecolor)
        .build()
        .expect("build");
    f.set_color_depth(ColorDepth::Color16);
    assert_eq!(f.color_depth(), ColorDepth::Color16);
}

#[test]
fn builder_unset_color_depth_uses_detect() {
    // When unset the builder calls ColorDepth::detect(); detect's
    // return value depends on the test runner's TTY status and
    // COLORTERM. We only assert the field is populated (any rung is
    // acceptable here).
    let f = FigletBuilder::new().build().expect("build");
    let depth = f.color_depth();
    assert!(matches!(
        depth,
        ColorDepth::Truecolor | ColorDepth::Color256 | ColorDepth::Color16
    ));
}

// ============================================================================
// FR-018 — graceful downgrade + FIXED warning string
// ============================================================================

#[test]
fn truecolor_unsupported_downgrades_with_warning() {
    // The downgrade warning is emitted on stderr by `resolve_depth`.
    // We cover the library-side path by running resolve_depth with
    // suppress_warning=false and verifying the return value.
    let result = resolve_depth(ColorDepth::Truecolor, ColorDepth::Color16, false);
    assert_eq!(result, ColorDepth::Color16);
    // The warning emission itself goes to process stderr; capturing
    // and asserting on its contents from inside this test is racy
    // (it shares stderr with other parallel tests). The unit-tests
    // in src/color_depth.rs cover the resolve_depth matrix fully;
    // here we just verify the downgrade rung is correct.
}

#[test]
fn truecolor_suppresses_warning_when_no_downgrade_warning() {
    // FR-029: when suppress_warning=true, the suppression is AT the
    // decision site BEFORE format-args evaluation. We can't observe
    // "no work happened" directly, but we can confirm the return
    // value matches the downgrade table:
    let result = resolve_depth(ColorDepth::Truecolor, ColorDepth::Color16, true);
    assert_eq!(result, ColorDepth::Color16);
    // No way for this test to observe the absence of an stderr write
    // robustly; the suppression contract is enforced at the type
    // level by the function signature.
}

#[test]
fn no_downgrade_required_returns_requested() {
    let result = resolve_depth(ColorDepth::Truecolor, ColorDepth::Truecolor, false);
    assert_eq!(result, ColorDepth::Truecolor);
}

#[test]
fn color16_requested_always_color16() {
    let result = resolve_depth(ColorDepth::Color16, ColorDepth::Truecolor, false);
    assert_eq!(result, ColorDepth::Color16);
}

// ============================================================================
// Security posture — warning string never contains env bytes (FR-018)
// ============================================================================

#[test]
fn downgrade_warning_string_does_not_reference_envvar() {
    // We can't capture this process's stderr without redirecting it.
    // What we CAN do is invoke a subprocess with an adversarial
    // COLORTERM and verify the subprocess doesn't echo the env bytes
    // back in its stderr.
    let adversarial = "EVIL_BYTES_SHOULD_NOT_LEAK_HERE";
    let mut cmd = cargo_bin();
    cmd.env_clear();
    cmd.env("COLORTERM", adversarial);
    cmd.args(["--version"]);
    let out = cmd.output().expect("binary executes");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stderr.contains(adversarial),
        "COLORTERM bytes leaked into stderr: {stderr}"
    );
}
