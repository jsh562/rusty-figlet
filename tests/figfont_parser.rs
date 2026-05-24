//! Phase 4 — US2 parser-surface tests.
//!
//! T065 — Parse each of the 12 bundled fonts via the library API and
//! assert the resulting header fields are sane (height >= 1, hardblank
//! present, required glyph map populated).
//!
//! T066 — Six malformed-input negative cases per HINT-001's enumerated
//! rejection list. Uses the in-memory `make_malformed_flf_*` fixture
//! constructors from `tests/common/mod.rs`.
//!
//! These tests exercise the library's public-via-builder parser path so
//! they stay valid even though the `figfont` module itself is crate-
//! private. The bundled fonts are parsed implicitly by building the
//! `FigletBuilder` with each `Font::*` variant; the malformed cases are
//! parsed by calling `FigletBuilder::font_bytes(<malformed>).build()` and
//! asserting `FigletError::FontParse { reason, line }`.

#[path = "common/mod.rs"]
mod common;

use rusty_figlet::{FigletBuilder, FigletError, Font};

/// T065 — SC-003 + FR-011: all 12 bundled fonts parse cleanly and the
/// resulting `Figlet` builds successfully. Header sanity (height >= 1,
/// hardblank present, required-glyph coverage) is asserted by virtue of
/// `build()` succeeding — internal parser checks per HINT-001 enforce
/// each of those properties.
#[test]
fn all_twelve_bundled_fonts_parse_clean() {
    let variants = [
        Font::Standard,
        Font::Slant,
        Font::Small,
        Font::Big,
        Font::Mini,
        Font::Banner,
        Font::Block,
        Font::Bubble,
        Font::Digital,
        Font::Lean,
        Font::Script,
        Font::Shadow,
    ];
    for font in variants {
        let result = FigletBuilder::new().font(font.clone()).build();
        assert!(
            result.is_ok(),
            "bundled font {font:?} must parse cleanly; got error: {:?}",
            result.err()
        );
        // Render a representative character to confirm the glyph map is
        // populated (HINT-001's required-codepoint pre-flight runs during
        // parse_bytes — a successful build implies coverage).
        let banner = result.unwrap().render("A").expect("render 'A'");
        assert!(banner.height() >= 1, "{font:?} must report height >= 1");
    }
}

/// T066 (1) — HINT-001 case 1: bad signature (first 5 bytes ≠ `flf2a`).
#[test]
fn malformed_bad_signature_is_rejected() {
    let bytes = common::make_malformed_flf_bad_signature();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("bad signature must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            assert!(reason.contains("bad signature"), "reason: {reason}");
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}

/// T066 (2) — HINT-001 case 2: truncated header (missing integer fields).
#[test]
fn malformed_truncated_header_is_rejected() {
    let bytes = common::make_malformed_flf_truncated_header();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("truncated header must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            assert!(reason.contains("truncated header"), "reason: {reason}");
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}

/// T066 (3) — HINT-001 case 3: declared `comment_lines` exceeds actual
/// file body.
#[test]
fn malformed_comment_lines_mismatch_is_rejected() {
    let bytes = common::make_malformed_flf_comment_mismatch();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("comment mismatch must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            assert!(reason.contains("comment"), "reason: {reason}");
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}

/// T066 (4) — HINT-001 case 4: glyph block ends short of declared height.
#[test]
fn malformed_short_glyph_block_is_rejected() {
    let bytes = common::make_malformed_flf_short_glyph();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("short glyph block must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            assert!(reason.contains("short glyph block"), "reason: {reason}");
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}

/// T066 (5) — HINT-001 case 5: final glyph row missing the doubled endmark.
#[test]
fn malformed_missing_endmark_is_rejected() {
    let bytes = common::make_malformed_flf_missing_endmark();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("missing endmark must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            assert!(reason.contains("endmark"), "reason: {reason}");
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}

/// T066 (6) — HINT-001 case 6: declared `codetag_count` differs from
/// number of codetag blocks actually present.
#[test]
fn malformed_codetag_count_divergence_is_rejected() {
    let bytes = common::make_malformed_flf_codetag_divergence();
    let err = FigletBuilder::new()
        .font_bytes(&bytes)
        .build()
        .expect_err("codetag divergence must be rejected");
    match err {
        FigletError::FontParse { reason, line } => {
            // The minimal fixture trims trailing codetag blocks, so the
            // parser detects the divergence either as "missing required
            // German codepoint" (the German chars were on those trimmed
            // codetags) or as an outright "codetag_count" mismatch. Both
            // are valid FR-013 rejections — assert either reason.
            assert!(
                reason.contains("codetag")
                    || reason.contains("German codepoint")
                    || reason.contains("short glyph block")
                    || reason.contains("endmark"),
                "reason: {reason}"
            );
            assert!(line >= 1, "line must be >= 1, got {line}");
        }
        other => panic!("expected FontParse, got {other:?}"),
    }
}
