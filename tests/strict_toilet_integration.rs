//! Corpus-driven byte-equality tests for the `toilet-strict-compat` leaf
//! (E012 US6 — T057 + T058, SC-006).
//!
//! ## Harness
//!
//! For each fixture under `tests/fixtures/toilet-corpus/<id>/`:
//!
//! - `input.txt`   — the text passed to `strict_render`.
//! - `filter.txt`  — the `-F` chain spec (one segment per line OR empty).
//! - `expected.bin`— the bytes the renderer is expected to produce.
//!
//! The test reads all three, invokes
//! [`rusty_figlet::strict_toilet::strict_render`], and asserts the produced
//! bytes equal the fixture bytes via `pretty_assertions::assert_eq!` so
//! mismatches surface as a hex-dump-style diff.
//!
//! ## Strict-mode color downgrade (T058 / US6 AS#2)
//!
//! A separate test exercises the [`Filter::Gay`] truecolor downgrade path:
//! the `gay` filter assigns 24-bit `Color::Rgb` foregrounds, but strict
//! mode must downgrade them to the 16-color floor. The test verifies that
//! the resulting bytes contain only SGR codes in the 30..=37 / 90..=97
//! range — never `\x1b[38;2;` (truecolor) or `\x1b[38;5;` (256-color)
//! escapes.
//!
//! ## Fixture format
//!
//! `filter.txt` is parsed as a single `-F`-chain spec, identical to how
//! the toilet CLI accepts `-F crop:border`. An empty `filter.txt` (zero
//! bytes OR a single trailing newline) parses to an empty chain — the
//! "no filter" / identity case.
//!
//! Per the corpus capture policy (see `MANIFEST.md`), v0.3.0 ships with
//! 3 SYNTHETIC fixtures derived from the `rusty-figlet` engine's own
//! output. Real `toilet 0.3-1`-captured fixtures replace these via the
//! `.github/workflows/capture-strict-compat-corpus.yml` PR loop.

#![cfg(all(
    feature = "toilet-strict-compat",
    feature = "filter-crop",
    feature = "filter-gay"
))]
#![allow(non_snake_case)]
// Test names use the `strict_compat__<id>` double-underscore convention to
// preserve the `[strict-compat]` prefix tag from the plan in cargo test
// output without breaking snake_case lints across the rest of the codebase.

use std::fs;
use std::path::PathBuf;

use pretty_assertions::assert_eq;
use rusty_figlet::StrictTarget;
use rusty_figlet::filter::{Filter, FilterChain};
use rusty_figlet::strict_toilet::strict_render;

/// Path to the corpus root, anchored at `$CARGO_MANIFEST_DIR`.
fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/toilet-corpus")
}

/// Read a fixture's `(input, filter_spec, expected_bytes)` triple.
fn read_fixture(name: &str) -> (String, String, Vec<u8>) {
    let dir = corpus_root().join(name);
    let input = fs::read_to_string(dir.join("input.txt"))
        .unwrap_or_else(|e| panic!("read input.txt for {name}: {e}"));
    let filter_raw = fs::read_to_string(dir.join("filter.txt"))
        .unwrap_or_else(|e| panic!("read filter.txt for {name}: {e}"));
    let expected = fs::read(dir.join("expected.bin"))
        .unwrap_or_else(|e| panic!("read expected.bin for {name}: {e}"));
    // Trim trailing newlines from filter spec — `cat`-style fixture files
    // often append a trailing `\n` and we don't want a literal empty
    // segment to surface as `UnknownFilter`.
    let filter_spec = filter_raw.trim_end_matches(['\n', '\r']).to_owned();
    (input, filter_spec, expected)
}

/// Generic per-fixture assertion used by every corpus-driven test below.
/// Test names are prefixed `strict_compat__` per plan
/// §`[strict-compat]` test-name prefix (Cargo test names cannot contain
/// `-`, so we use double-underscore to mirror the prefix tag).
fn assert_fixture_matches(name: &str) {
    let (input, filter_spec, expected) = read_fixture(name);
    let chain = FilterChain::parse(&filter_spec)
        .unwrap_or_else(|e| panic!("parse filter spec {filter_spec:?} for {name}: {e}"));
    let actual = strict_render(&input, &chain, StrictTarget::Toilet031)
        .unwrap_or_else(|e| panic!("strict_render for {name}: {e}"));
    assert_eq!(
        actual, expected,
        "byte-equality failed for fixture {name} (filter_spec={filter_spec:?})"
    );
}

#[test]
fn strict_compat__nothing_hi() {
    assert_fixture_matches("nothing_hi");
}

#[test]
fn strict_compat__crop_hi() {
    assert_fixture_matches("crop_hi");
}

#[test]
fn strict_compat__gay_hi() {
    assert_fixture_matches("gay_hi");
}

/// T058 — Strict-mode downgrades truecolor `Filter::Gay` output to the
/// 16-color floor (US6 AS#2). Exercises `strict_render` directly via the
/// library API — the CLI `--strict` flag is wired in Phase 9 (T060).
#[test]
fn strict_mode_downgrades_truecolor_to_16() {
    let chain = FilterChain::new().push(Filter::Gay);
    let bytes = strict_render("hi", &chain, StrictTarget::Toilet031)
        .expect("strict_render with gay filter must succeed");

    // Must contain at least one SGR escape (gay filter assigns color).
    assert!(
        bytes.windows(2).any(|w| w == [0x1b, b'[']),
        "expected at least one SGR escape; got {:?}",
        String::from_utf8_lossy(&bytes)
    );

    // Must NOT contain any truecolor SGR introducer (`\x1b[38;2;` or
    // `\x1b[48;2;`).
    assert!(
        !bytes.windows(7).any(|w| w == b"\x1b[38;2;"),
        "16-color floor MUST NOT emit truecolor (\x1b[38;2;) escapes; bytes = {:?}",
        String::from_utf8_lossy(&bytes)
    );
    assert!(
        !bytes.windows(7).any(|w| w == b"\x1b[48;2;"),
        "16-color floor MUST NOT emit truecolor (\x1b[48;2;) escapes; bytes = {:?}",
        String::from_utf8_lossy(&bytes)
    );

    // Must NOT contain any 256-color SGR introducer (`\x1b[38;5;` or
    // `\x1b[48;5;`).
    assert!(
        !bytes.windows(7).any(|w| w == b"\x1b[38;5;"),
        "16-color floor MUST NOT emit 256-color (\x1b[38;5;) escapes; bytes = {:?}",
        String::from_utf8_lossy(&bytes)
    );
    assert!(
        !bytes.windows(7).any(|w| w == b"\x1b[48;5;"),
        "16-color floor MUST NOT emit 256-color (\x1b[48;5;) escapes; bytes = {:?}",
        String::from_utf8_lossy(&bytes)
    );
}

/// Sanity smoke: empty chain + empty input must succeed (no panic, no
/// stray SGR codes). Guards against regressions in the empty-input path.
#[test]
fn strict_compat__empty_input_no_panic() {
    let chain = FilterChain::new();
    let bytes =
        strict_render("", &chain, StrictTarget::Toilet031).expect("empty input must succeed");
    assert!(
        !bytes.windows(2).any(|w| w == [0x1b, b'[']),
        "empty-input output should contain no SGR codes; got {bytes:?}"
    );
}
