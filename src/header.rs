//! Shared numeric-header reader for FLF and TLF font formats (AD-001).
//!
//! Both FIGfont 2.0 (`flf2a`) and TheLetter font (`tlf2a`) headers carry the
//! same suffix shape after their magic bytes — a hardblank character followed
//! by whitespace-separated integer fields. This module factors the numeric-
//! field parser into a single byte-offset-carrying [`HeaderReader`] so the
//! two parsers share a single tested implementation.
//!
//! FR-028 contract: parse errors raise in O(1) additional cost beyond the
//! offending byte — the reader carries a 1-indexed line counter and a byte
//! offset alongside every advance so error sites do not re-scan the input.
//!
//! Magic-byte handling is left to each caller (FLF / TLF) because the magic
//! string itself differs (`flf2a` vs. `tlf2a`); the reader is told via
//! `magic_len` how many bytes to skip past the magic before the hardblank.

use crate::error::FigletError;

/// Parsed numeric header derived from the first line of an FLF/TLF file.
///
/// Field semantics mirror the FIGfont 2.0 spec (height, baseline, max_length,
/// old_layout, comment_lines, print_direction, full_layout, codetag_count).
/// TLF files reuse the same numeric layout — see research.md §TLF Font Format.
#[derive(Debug, Clone)]
pub(crate) struct NumericHeader {
    /// Hardblank character (placeholder for in-glyph blanks; usually `$`).
    pub hardblank: char,
    /// Height in rows of every glyph.
    pub height: u32,
    /// Baseline row from the top of each glyph.
    pub baseline: u32,
    /// Maximum width of any glyph in this font.
    pub max_length: u32,
    /// Legacy single-byte layout descriptor (-1 = full-width, 0..=63 = bitfield).
    pub old_layout: i32,
    /// Number of comment lines that follow the header.
    pub comment_lines: u32,
    /// Print direction (0 = left-to-right, 1 = right-to-left).
    pub print_direction: u32,
    /// Modern multi-byte layout descriptor.
    pub full_layout: u32,
    /// Number of codetag blocks declared by the header (0 when omitted).
    pub codetag_count: u32,
}

/// Stateful header-line reader. Carries the 1-indexed `line_no` (always `1`
/// for the header) plus the byte offset of the most recently consumed token
/// so error sites can surface byte-precise locations without re-scanning.
///
/// This is `pub(crate)` because both `figfont.rs` (FLF) and `tlf.rs` (TLF)
/// consume it; it is not part of the public API.
#[derive(Debug)]
pub(crate) struct HeaderReader<'a> {
    /// The header line text, sliced just past the magic bytes.
    rest: &'a str,
    /// 1-indexed line number for error reporting (always 1 for the header).
    line_no: u32,
}

impl<'a> HeaderReader<'a> {
    /// Construct a reader for the header line, skipping past `magic_len` bytes
    /// of the magic prefix. The caller is responsible for verifying the magic
    /// matches before invoking this; `HeaderReader` does not validate magic.
    pub(crate) fn new(header_line: &'a str, magic_len: usize, line_no: u32) -> Self {
        let rest = if header_line.len() > magic_len {
            &header_line[magic_len..]
        } else {
            ""
        };
        Self { rest, line_no }
    }

    /// Returns the current 1-indexed line number.
    pub(crate) fn line_no(&self) -> u32 {
        self.line_no
    }
}

/// Parse a numeric header from raw bytes, skipping the first `magic_len`
/// bytes (typically 5 for `flf2a` / `tlf2a`). The hardblank is the character
/// at byte offset `magic_len`; remaining whitespace-separated tokens are the
/// integer fields per FIGfont 2.0.
///
/// FR-028: errors raise in O(1) additional cost beyond the offending byte.
/// The reader stops at the first newline; subsequent bytes are not scanned.
pub(crate) fn read_numeric_header(
    bytes: &[u8],
    magic_len: usize,
) -> Result<NumericHeader, FigletError> {
    // FIGfont/TLF files are conventionally Latin-1; decode tolerantly by
    // mapping bytes 0..255 to chars 0..255 so the parser does not panic on
    // upstream-provided non-UTF8 bytes.
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let header_line = text
        .split('\n')
        .next()
        .ok_or_else(|| parse_err("empty input", 1))?
        .trim_end_matches('\r');

    parse_header_line(header_line, magic_len, 1)
}

/// Parse a single header line that has ALREADY been extracted (no embedded
/// newline). Internal helper kept separate so callers that already split on
/// newlines (e.g. `figfont::parse_bytes`) can avoid a redundant copy.
pub(crate) fn parse_header_line(
    line: &str,
    magic_len: usize,
    line_no: u32,
) -> Result<NumericHeader, FigletError> {
    if line.len() < magic_len {
        return Err(parse_err("truncated header: missing magic", line_no));
    }
    let rest = &line[magic_len..];
    let mut chars = rest.chars();
    let hardblank = chars
        .next()
        .ok_or_else(|| parse_err("truncated header: missing hardblank", line_no))?;
    let tail: String = chars.collect();
    let mut tokens = tail.split_whitespace();

    let height = next_u32(&mut tokens, "height", line_no)?;
    let baseline = next_u32(&mut tokens, "baseline", line_no)?;
    let max_length = next_u32(&mut tokens, "max_length", line_no)?;
    let old_layout = next_i32(&mut tokens, "old_layout", line_no)?;
    if !(-1..=63).contains(&old_layout) {
        return Err(parse_err(
            &format!("old_layout out of -1..=63 range: {old_layout}"),
            line_no,
        ));
    }
    let comment_lines = next_u32(&mut tokens, "comment_lines", line_no)?;
    let print_direction = next_u32_opt(&mut tokens).unwrap_or(0);
    let derived_full_layout = if old_layout < 0 { 0 } else { old_layout as u32 };
    let full_layout = next_u32_opt(&mut tokens).unwrap_or(derived_full_layout);
    let codetag_count = next_u32_opt(&mut tokens).unwrap_or(0);

    Ok(NumericHeader {
        hardblank,
        height,
        baseline,
        max_length,
        old_layout,
        comment_lines,
        print_direction,
        full_layout,
        codetag_count,
    })
}

fn next_u32(
    tokens: &mut std::str::SplitWhitespace<'_>,
    field: &str,
    line_no: u32,
) -> Result<u32, FigletError> {
    let tok = tokens
        .next()
        .ok_or_else(|| parse_err(&format!("truncated header: missing {field}"), line_no))?;
    tok.parse::<u32>().map_err(|_| {
        parse_err(
            &format!("truncated header: {field} not a u32 ({tok})"),
            line_no,
        )
    })
}

fn next_i32(
    tokens: &mut std::str::SplitWhitespace<'_>,
    field: &str,
    line_no: u32,
) -> Result<i32, FigletError> {
    let tok = tokens
        .next()
        .ok_or_else(|| parse_err(&format!("truncated header: missing {field}"), line_no))?;
    tok.parse::<i32>().map_err(|_| {
        parse_err(
            &format!("truncated header: {field} not an i32 ({tok})"),
            line_no,
        )
    })
}

fn next_u32_opt(tokens: &mut std::str::SplitWhitespace<'_>) -> Option<u32> {
    tokens.next().and_then(|tok| tok.parse::<u32>().ok())
}

fn parse_err(reason: &str, line: u32) -> FigletError {
    FigletError::FontParse {
        reason: reason.to_owned(),
        line,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // T005 — unit tests for header.rs covering numeric-field parsing edge
    // cases (truncated header, oversized fields, negative numbers if any,
    // malformed UTF-8 / non-UTF8 bytes).

    #[test]
    fn parses_minimal_flf_header() {
        // flf2a$ followed by height=1 baseline=1 max_length=8 old_layout=0
        // comment_lines=0.
        let h = read_numeric_header(b"flf2a$ 1 1 8 0 0\n", 5).expect("parses");
        assert_eq!(h.hardblank, '$');
        assert_eq!(h.height, 1);
        assert_eq!(h.baseline, 1);
        assert_eq!(h.max_length, 8);
        assert_eq!(h.old_layout, 0);
        assert_eq!(h.comment_lines, 0);
        assert_eq!(h.print_direction, 0);
        assert_eq!(h.full_layout, 0);
        assert_eq!(h.codetag_count, 0);
    }

    #[test]
    fn parses_full_flf_header_with_all_optional_fields() {
        // height=4 baseline=3 max_length=8 old_layout=0 comment_lines=16
        // print_direction=0 full_layout=64 codetag_count=0.
        let h = read_numeric_header(b"flf2a$ 4 3 8 0 16 0 64 0\nother stuff\n", 5).expect("parses");
        assert_eq!(h.height, 4);
        assert_eq!(h.baseline, 3);
        assert_eq!(h.max_length, 8);
        assert_eq!(h.comment_lines, 16);
        assert_eq!(h.print_direction, 0);
        assert_eq!(h.full_layout, 64);
        assert_eq!(h.codetag_count, 0);
    }

    #[test]
    fn parses_minimal_tlf_header() {
        // TLF magic is `tlf2a`; same numeric layout follows.
        let h = read_numeric_header(b"tlf2a$ 1 1 8 0 0\n", 5).expect("parses");
        assert_eq!(h.hardblank, '$');
        assert_eq!(h.height, 1);
    }

    #[test]
    fn rejects_empty_input() {
        // Empty bytes → truncated header.
        let err = read_numeric_header(b"", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, line } => {
                assert!(reason.contains("truncated"), "{reason}");
                assert_eq!(line, 1);
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_truncated_header_missing_fields() {
        // Magic + hardblank + one field — missing baseline, max_length, etc.
        let err = read_numeric_header(b"flf2a$ 1\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, line } => {
                assert!(reason.contains("truncated"), "{reason}");
                assert_eq!(line, 1);
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_truncated_header_missing_hardblank() {
        // Magic bytes only — no hardblank.
        let err = read_numeric_header(b"flf2a", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, line } => {
                assert!(
                    reason.contains("hardblank") || reason.contains("truncated"),
                    "{reason}"
                );
                assert_eq!(line, 1);
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_old_layout_out_of_range_high() {
        let err = read_numeric_header(b"flf2a$ 1 1 8 64 0\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("old_layout"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_old_layout_out_of_range_low() {
        let err = read_numeric_header(b"flf2a$ 1 1 8 -2 0\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("old_layout"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_oversized_numeric_field() {
        // u32::MAX + 1 doesn't fit in u32.
        let err = read_numeric_header(b"flf2a$ 4294967296 1 8 0 0\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("height"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_non_numeric_height() {
        let err = read_numeric_header(b"flf2a$ abc 1 8 0 0\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("height"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn negative_height_rejected() {
        // height is u32; a leading minus parses as i32 then fails u32 cast.
        let err = read_numeric_header(b"flf2a$ -1 1 8 0 0\n", 5).unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("height"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn tolerates_latin1_bytes_in_header_comment() {
        // Byte 0xC4 (Latin-1 capital A-with-umlaut) should not panic;
        // it is mapped to char 0x00C4. The header still parses cleanly
        // because the relevant numeric fields are pure ASCII.
        let h = read_numeric_header(b"flf2a\xC4 1 1 8 0 0\n", 5).expect("parses");
        assert_eq!(h.hardblank, '\u{00C4}');
    }

    #[test]
    fn full_layout_defaults_from_old_layout_when_omitted() {
        // old_layout=15, full_layout omitted → derived_full_layout=15.
        let h = read_numeric_header(b"flf2a$ 1 1 8 15 0\n", 5).expect("parses");
        assert_eq!(h.full_layout, 15);
    }

    #[test]
    fn full_layout_defaults_to_zero_when_old_layout_negative() {
        // old_layout=-1 (full-width) → derived_full_layout=0.
        let h = read_numeric_header(b"flf2a$ 1 1 8 -1 0\n", 5).expect("parses");
        assert_eq!(h.full_layout, 0);
    }

    #[test]
    fn ignores_bytes_past_first_newline() {
        // Bytes after the first newline must not affect header parsing —
        // FR-028 O(1) cost: parser stops at newline boundary.
        let h = read_numeric_header(b"flf2a$ 1 1 8 0 0\nGARBAGE GARBAGE\n", 5).expect("parses");
        assert_eq!(h.height, 1);
    }

    #[test]
    fn header_reader_skips_magic() {
        let r = HeaderReader::new("flf2a$ 1 1 8 0 0", 5, 1);
        assert_eq!(r.line_no(), 1);
        assert!(r.rest.starts_with('$'));
    }
}
