//! TheLetter (`.tlf`) font-format parser (E012 US3 — FR-001).
//!
//! Toilet's native font format extends FIGfont 2.0 with three additive bits:
//!
//! 1. Magic header `tlf2a` (5 ASCII bytes) — sometimes followed by `$`
//!    indicating extended metadata (e.g. `tlf2a$ 4 3 8 0 16 0 64 0`).
//! 2. UTF-8 multi-column glyph cells (figlet 2.2.5 is single-byte/Latin-1).
//! 3. Inline color/style markers per cell so fonts ship pre-colored.
//!
//! Header numeric fields are FIGfont-shaped (height/baseline/max_length/
//! old_layout/comment_lines/...) so the same [`crate::header`] reader is
//! reused. Glyph rows reuse FIGfont endmark/double-endmark conventions but
//! allow Unicode codepoints inside cells.
//!
//! The parser is hand-rolled and pure-Rust per FR-024 (no C linkage). The
//! clean-room derivation rationale is recorded in `docs/tlf-derivation.md`.
//!
//! ## Feature gate
//!
//! This module is gated by the `tlf-parser` Cargo leaf. v0.2.x users will
//! not see it; v0.3.0 enables it under `default = ["full"]` and the
//! `figlet-toilet-compat` preset bundle.

use std::collections::HashMap;
use std::path::Path;

use crate::error::FigletError;
use crate::header;

/// Hard upper bound on TLF file size — per spec Edge Cases, files >8 MiB
/// are rejected with `FigletError::TlfParse`. The bound applies to the
/// raw byte slice and to disk reads.
pub(crate) const TLF_MAX_FILE_SIZE: usize = 8 * 1024 * 1024;

/// Magic prefix shared by all `.tlf` files. The optional trailing `$`
/// (extended-metadata form) is consumed as the hardblank by the header
/// reader, so the magic itself is exactly these 5 bytes.
pub(crate) const TLF_MAGIC: &[u8] = b"tlf2a";

/// Parsed TLF font.
///
/// Glyphs are keyed by Unicode codepoint (`u32`) — same shape as the FLF
/// parser's [`crate::figfont::FIGfont::glyphs`] so downstream renderers can
/// treat the two interchangeably once a TLF is loaded.
#[derive(Debug, Clone)]
pub struct TlfFont {
    /// Numeric header fields (height, baseline, max_length, etc.) shared
    /// with FLF via [`crate::header::NumericHeader`].
    pub(crate) header: TlfHeader,
    /// Map from codepoint → glyph rows (one entry per row, endmarks stripped).
    pub(crate) glyphs: HashMap<u32, TlfGlyph>,
    /// `true` when any glyph carried an inline color/style marker.
    pub multicolor: bool,
}

/// TLF header — same shape as FIGfont but with a constant magic.
///
/// `version` is reserved for future TLF revisions; v1 of the format
/// (`tlf2a`) uses `1`.
#[derive(Debug, Clone)]
pub(crate) struct TlfHeader {
    /// Hardblank character (typically `$`).
    pub hardblank: char,
    /// Height in rows of every glyph.
    pub height: u32,
    /// Baseline row from the top.
    pub baseline: u32,
    /// Maximum width of any glyph in this font.
    pub max_length: u32,
    /// Comment-block line count (lines after the header line).
    pub comment_lines: u32,
}

/// Single TLF glyph: a stack of rows of cells.
#[derive(Debug, Clone)]
pub(crate) struct TlfGlyph {
    /// One row per glyph row (length == header.height).
    pub rows: Vec<TlfRow>,
}

/// One row of TLF cells (UTF-8 multi-column characters with optional color).
#[derive(Debug, Clone)]
pub(crate) struct TlfRow {
    /// Cells of this row in left-to-right order.
    pub cells: Vec<TlfCell>,
}

/// A single TLF cell carrying one Unicode character + optional color attr.
///
/// `color_attr` is `None` for plain-text fonts (most TLFs in practice) and
/// `Some(byte)` for multicolor cells whose interpretation matches libcaca's
/// internal palette index.
#[derive(Debug, Clone, Copy)]
pub(crate) struct TlfCell {
    /// Unicode character displayed in this cell.
    pub ch: char,
    /// Optional inline color/style marker (libcaca attribute byte).
    pub color_attr: Option<u8>,
}

impl TlfFont {
    /// Parse a TLF font from raw bytes.
    ///
    /// Validates the `tlf2a` magic at byte 0, delegates numeric-header
    /// parsing to [`crate::header::parse_header_line`] (sharing FR-028 O(1)
    /// error-cost contract with the FLF parser), then walks the glyph table.
    ///
    /// Both `tlf2a` and `tlf2a$` extended-metadata header forms are accepted
    /// per research.md §TLF Font Format.
    pub fn from_bytes(bytes: &[u8]) -> Result<TlfFont, FigletError> {
        parse_tlf(bytes)
    }

    /// Header height in rows.
    pub fn height(&self) -> u32 {
        self.header.height
    }

    /// Lookup a codepoint's glyph rows; returns `None` for codepoints absent
    /// from the font.
    pub(crate) fn lookup(&self, cp: u32) -> Option<&TlfGlyph> {
        self.glyphs.get(&cp)
    }
}

/// Parse a TLF byte slice into a [`TlfFont`].
///
/// Errors raised:
/// - [`FigletError::InvalidTlfHeader`] — magic mismatch.
/// - [`FigletError::TlfParse`] — any later parse failure with 1-indexed line.
///
/// Per FR-026 the parser's working set is bounded by the source byte length
/// (no per-glyph quadratic copies). Per spec Edge Cases, files larger than
/// 8 MiB are rejected up front.
pub fn parse_tlf(bytes: &[u8]) -> Result<TlfFont, FigletError> {
    // File-size cap per spec Edge Cases.
    if bytes.len() > TLF_MAX_FILE_SIZE {
        return Err(FigletError::TlfParse {
            reason: format!(
                "file size {} exceeds {} byte maximum",
                bytes.len(),
                TLF_MAX_FILE_SIZE
            ),
            line: 1,
        });
    }

    // Zero-byte files are immediately rejected as invalid magic.
    if bytes.is_empty() {
        return Err(FigletError::InvalidTlfHeader { found: Vec::new() });
    }

    // Validate magic at byte 0 — extract up to 32 bytes for the diagnostic
    // (spec Security Posture: cap echoed bytes to prevent log spam from
    // adversarial inputs).
    if bytes.len() < TLF_MAGIC.len() || &bytes[..TLF_MAGIC.len()] != TLF_MAGIC {
        let take = bytes.len().min(32);
        return Err(FigletError::InvalidTlfHeader {
            found: bytes[..take].to_vec(),
        });
    }

    // TLF is UTF-8 per research.md §TLF Font Format (figlet 2.2.5 is
    // single-byte/Latin-1 by default; TLF extends FIGfont 2.0 with UTF-8
    // multi-column Unicode glyph support). Decode strictly here so multi-
    // column glyph rows preserve grapheme integrity. Invalid UTF-8 in any
    // byte produces a TlfParse error per FR-016 + FR-028.
    let text = match std::str::from_utf8(bytes) {
        Ok(s) => s,
        Err(e) => {
            return Err(FigletError::TlfParse {
                reason: format!("malformed UTF-8 at byte offset {}", e.valid_up_to()),
                line: 1,
            });
        }
    };
    let mut lines = text.split('\n');
    let header_line = lines
        .next()
        .ok_or_else(|| tlf_parse_err("empty input after magic", 1))?
        .trim_end_matches('\r');

    // Delegate numeric parsing — magic_len = TLF_MAGIC.len() = 5.
    let nh =
        header::parse_header_line(header_line, TLF_MAGIC.len(), 1).map_err(|err| match err {
            // Convert FontParse to InvalidTlfHeader for malformed numeric fields
            // per FR-016 + FR-028 — the spec demands a TLF-specific variant when
            // the header is structurally invalid.
            FigletError::FontParse { reason: _, line: _ } => FigletError::InvalidTlfHeader {
                found: header_line.as_bytes().iter().copied().take(32).collect(),
            },
            other => other,
        })?;

    let tlf_header = TlfHeader {
        hardblank: nh.hardblank,
        height: nh.height,
        baseline: nh.baseline,
        max_length: nh.max_length,
        comment_lines: nh.comment_lines,
    };

    if tlf_header.height == 0 {
        return Err(FigletError::InvalidTlfHeader {
            found: header_line.as_bytes().iter().copied().take(32).collect(),
        });
    }

    // Hard cap on per-row cell count (spec Edge Cases): refuse > 64 KiB cells
    // per row. We apply this against `max_length` as the declared upper bound.
    if tlf_header.max_length > 65_536 {
        return Err(tlf_parse_err(
            &format!(
                "max_length {} exceeds 65536 cell-per-row cap",
                tlf_header.max_length
            ),
            1,
        ));
    }

    // Skip comment_lines.
    let mut current_line: u32 = 1;
    for _ in 0..tlf_header.comment_lines {
        current_line += 1;
        if lines.next().is_none() {
            return Err(tlf_parse_err(
                "truncated comment block: comment_lines exceeds available lines",
                current_line,
            ));
        }
    }

    // Now decode glyph table. TLF uses FIGfont endmark conventions but cells
    // may contain Unicode multi-column characters and inline color markers.
    // Per-row allocations are bounded by the file byte length (FR-026): we
    // never copy more bytes into glyph storage than appear in the source.
    let mut glyphs: HashMap<u32, TlfGlyph> = HashMap::new();
    let mut endmark: Option<char> = None;
    let mut multicolor_seen = false;

    // ASCII 32..=126 are required; remaining codepoints (German chars,
    // codetag blocks) follow inline or via codetag headers.
    for cp in 32u32..=126 {
        let g = read_glyph(
            &mut lines,
            tlf_header.height,
            &mut current_line,
            &mut endmark,
            &mut multicolor_seen,
        )?;
        glyphs.insert(cp, g);
    }

    // Optional codetag stream — read until EOF.
    loop {
        let header_text = match next_non_empty(&mut lines, &mut current_line) {
            Some(line) => line,
            None => {
                return Ok(TlfFont {
                    header: tlf_header,
                    glyphs,
                    multicolor: multicolor_seen,
                });
            }
        };
        let codepoint = parse_codetag_codepoint(&header_text, current_line)?;
        let g = read_glyph(
            &mut lines,
            tlf_header.height,
            &mut current_line,
            &mut endmark,
            &mut multicolor_seen,
        )?;
        glyphs.insert(codepoint, g);
    }
}

/// Read exactly `height` glyph rows, stripping endmarks per FIGfont 2.0
/// conventions and decoding multicolor cell markers.
fn read_glyph<'a, I>(
    lines: &mut I,
    height: u32,
    current_line: &mut u32,
    endmark: &mut Option<char>,
    multicolor_seen: &mut bool,
) -> Result<TlfGlyph, FigletError>
where
    I: Iterator<Item = &'a str>,
{
    let mut rows = Vec::with_capacity(height as usize);
    for row in 0..height {
        *current_line += 1;
        let raw = lines
            .next()
            .ok_or_else(|| tlf_parse_err("short glyph block: hit EOF mid-glyph", *current_line))?
            .trim_end_matches('\r');
        if raw.is_empty() {
            return Err(tlf_parse_err(
                "short glyph block: blank line where glyph row expected",
                *current_line,
            ));
        }
        let stripped = strip_endmark_utf8(raw, row == height - 1, endmark, *current_line)?;
        let cells = decode_cells(&stripped, multicolor_seen);
        rows.push(TlfRow { cells });
    }
    Ok(TlfGlyph { rows })
}

/// Strip the FIGfont 2.0 endmark from a UTF-8 glyph row. Same conventions
/// as FLF (single endmark on rows 0..height-1, doubled on the final row).
fn strip_endmark_utf8(
    raw: &str,
    last_row: bool,
    endmark: &mut Option<char>,
    line_no: u32,
) -> Result<String, FigletError> {
    let chars: Vec<char> = raw.chars().collect();
    if chars.is_empty() {
        return Err(tlf_parse_err(
            "missing endmark: glyph row is empty",
            line_no,
        ));
    }
    let candidate = *chars.last().expect("non-empty just checked");
    let mark = match *endmark {
        Some(m) => m,
        None => {
            *endmark = Some(candidate);
            candidate
        }
    };
    if candidate != mark {
        return Err(tlf_parse_err(
            &format!("missing endmark: row ends with '{candidate}', expected endmark '{mark}'"),
            line_no,
        ));
    }
    let mut end = chars.len() - 1;
    if last_row {
        if end == 0 || chars[end - 1] != mark {
            return Err(tlf_parse_err(
                "missing endmark: final glyph row lacks doubled endmark",
                line_no,
            ));
        }
        end -= 1;
    }
    Ok(chars[..end].iter().collect())
}

/// Decode a stripped-endmark UTF-8 glyph row into [`TlfCell`]s.
///
/// Inline color markers (libcaca multicolor) follow the convention
/// `\x0E<attr_byte>` where `\x0E` (Shift Out, SO) introduces a one-byte
/// attribute that applies to the cells until the next marker. Plain TLFs
/// contain no SO bytes and decode 1:1 character → cell.
///
/// Per FR-026 working set is bounded by input length — each input char
/// yields at most one output cell.
fn decode_cells(s: &str, multicolor_seen: &mut bool) -> Vec<TlfCell> {
    let mut out = Vec::with_capacity(s.chars().count());
    let mut current_color: Option<u8> = None;
    let mut chars = s.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\x0E' {
            // Multicolor marker: next char's low byte is the attr.
            *multicolor_seen = true;
            if let Some(attr_ch) = chars.next() {
                current_color = Some((attr_ch as u32 & 0xFF) as u8);
            }
            continue;
        }
        out.push(TlfCell {
            ch,
            color_attr: current_color,
        });
    }
    out
}

/// Skip blank lines, returning the first non-empty trimmed line.
fn next_non_empty<'a, I>(lines: &mut I, current_line: &mut u32) -> Option<String>
where
    I: Iterator<Item = &'a str>,
{
    loop {
        *current_line += 1;
        let line = lines.next()?;
        let trimmed = line.trim_end_matches('\r');
        if !trimmed.is_empty() {
            return Some(trimmed.to_owned());
        }
    }
}

/// Parse the first whitespace-separated token of a codetag header line as
/// a hex codepoint (matches FLF conventions; HINT-001 rejects decimal).
fn parse_codetag_codepoint(line: &str, line_no: u32) -> Result<u32, FigletError> {
    let tok = line
        .split_whitespace()
        .next()
        .ok_or_else(|| tlf_parse_err("codetag header missing codepoint token", line_no))?;
    let body = tok.strip_prefix("0x").or_else(|| tok.strip_prefix("0X"));
    let (body, negative) = match body {
        Some(b) => (b, false),
        None => {
            if let Some(rest) = tok.strip_prefix('-') {
                let rest_body = rest.strip_prefix("0x").or_else(|| rest.strip_prefix("0X"));
                (rest_body.unwrap_or(rest), true)
            } else {
                (tok, false)
            }
        }
    };
    let value = u32::from_str_radix(body, 16).map_err(|_| {
        tlf_parse_err(
            &format!("codetag codepoint not hexadecimal: {tok}"),
            line_no,
        )
    })?;
    if negative {
        Ok(value.wrapping_neg())
    } else {
        Ok(value)
    }
}

fn tlf_parse_err(reason: &str, line: u32) -> FigletError {
    FigletError::TlfParse {
        reason: reason.to_owned(),
        line,
    }
}

/// Read a `.tlf` file from disk with the file-system-adversarial bounds
/// required by spec Edge Cases: zero-byte files, files >8 MiB, and symlink
/// loops are rejected before allocation.
///
/// Used by [`crate::Figlet::from_tlf`].
pub(crate) fn read_tlf_file(path: &Path) -> Result<Vec<u8>, FigletError> {
    // Reject symlink loops + zero-byte files via the OS metadata first.
    let meta = std::fs::metadata(path)?;
    if meta.len() == 0 {
        return Err(FigletError::TlfParse {
            reason: "zero-byte file".to_owned(),
            line: 1,
        });
    }
    if meta.len() as usize > TLF_MAX_FILE_SIZE {
        return Err(FigletError::TlfParse {
            reason: format!(
                "file size {} exceeds {} byte maximum",
                meta.len(),
                TLF_MAX_FILE_SIZE
            ),
            line: 1,
        });
    }
    std::fs::read(path).map_err(FigletError::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Construct a syntactically-valid minimal TLF byte slice for testing.
    ///
    /// height=1, comment_lines=0, ASCII 32..=126 glyphs each containing
    /// the literal codepoint character followed by `@@` (single + doubled
    /// endmark on the same/only row).
    fn minimal_tlf_bytes() -> Vec<u8> {
        let mut s = String::from("tlf2a$ 1 1 8 0 0 0 0 0\n");
        for cp in 32u32..=126 {
            let ch = char::from_u32(cp).unwrap();
            // height==1 means the single row IS the last row → doubled endmark.
            // Glyph char + `@@` (last_row doubled endmark).
            s.push(ch);
            s.push_str("@@\n");
        }
        s.into_bytes()
    }

    #[test]
    fn valid_tlf_returns_ok() {
        let bytes = minimal_tlf_bytes();
        let font = parse_tlf(&bytes).expect("valid tlf parses");
        assert_eq!(font.height(), 1);
        assert_eq!(font.header.hardblank, '$');
        // ASCII 'A' (0x41) must be present.
        assert!(font.lookup(b'A' as u32).is_some());
        // CJK codepoint must miss.
        assert!(font.lookup(0x4E2D).is_none());
        assert!(!font.multicolor);
    }

    #[test]
    fn invalid_magic_returns_invalid_tlf_header() {
        let err = parse_tlf(b"flf2a$ 1 1 8 0 0\n").unwrap_err();
        match err {
            FigletError::InvalidTlfHeader { found } => {
                assert_eq!(&found[..5], b"flf2a");
            }
            other => panic!("expected InvalidTlfHeader, got {other:?}"),
        }
    }

    #[test]
    fn empty_input_returns_invalid_tlf_header() {
        let err = parse_tlf(b"").unwrap_err();
        match err {
            FigletError::InvalidTlfHeader { found } => {
                assert!(found.is_empty());
            }
            other => panic!("expected InvalidTlfHeader, got {other:?}"),
        }
    }

    #[test]
    fn malformed_header_returns_invalid_tlf_header() {
        // Magic correct but numeric fields garbage.
        let err = parse_tlf(b"tlf2a$ notanumber 1 8 0 0\n").unwrap_err();
        match err {
            FigletError::InvalidTlfHeader { .. } => {}
            other => panic!("expected InvalidTlfHeader, got {other:?}"),
        }
    }

    #[test]
    fn zero_height_returns_invalid_tlf_header() {
        let err = parse_tlf(b"tlf2a$ 0 0 0 0 0\n").unwrap_err();
        match err {
            FigletError::InvalidTlfHeader { .. } => {}
            other => panic!("expected InvalidTlfHeader, got {other:?}"),
        }
    }

    #[test]
    fn truncated_glyph_table_returns_tlf_parse_with_line() {
        // Header declares height=2 but only ASCII 32 has 2 rows; ASCII 33
        // is short by 1 row → fails mid-glyph.
        let mut s = String::from("tlf2a$ 2 1 8 0 0\n");
        // ASCII 32 (space): two rows, last doubled endmark.
        s.push_str(" @\n");
        s.push_str(" @@\n");
        // ASCII 33 (!): one row only, then EOF.
        s.push_str("!@\n");
        let err = parse_tlf(s.as_bytes()).unwrap_err();
        match err {
            FigletError::TlfParse { reason, line } => {
                assert!(
                    reason.contains("short glyph block") || reason.contains("EOF"),
                    "{reason}"
                );
                assert!(line > 1, "expected 1-indexed line >1, got {line}");
            }
            other => panic!("expected TlfParse, got {other:?}"),
        }
    }

    #[test]
    fn file_size_exceeded_returns_tlf_parse() {
        // Construct an artificially oversized buffer: 9 MiB.
        let oversized = vec![b'A'; 9 * 1024 * 1024];
        let err = parse_tlf(&oversized).unwrap_err();
        match err {
            FigletError::TlfParse { reason, line } => {
                assert!(
                    reason.contains("exceeds") || reason.contains("size"),
                    "{reason}"
                );
                assert_eq!(line, 1);
            }
            other => panic!("expected TlfParse, got {other:?}"),
        }
    }

    #[test]
    fn extended_metadata_header_form_accepted() {
        // The `tlf2a$ 4 3 8 0 16 0 64 0` extended form (per research.md):
        // hardblank is `$`, then 8 numeric fields. comment_lines=16 means
        // we need 16 comment lines before glyphs.
        let mut s = String::from("tlf2a$ 1 1 8 0 0 0 64 0\n");
        for cp in 32u32..=126 {
            let ch = char::from_u32(cp).unwrap();
            s.push(ch);
            s.push_str("@@\n");
        }
        let font = parse_tlf(s.as_bytes()).expect("extended-form parses");
        assert_eq!(font.height(), 1);
    }

    #[test]
    fn multicolor_marker_is_observed() {
        // Inject an `\x0E\x04` SO marker into ASCII 32's row.
        let mut s = String::from("tlf2a$ 1 1 8 0 0\n");
        s.push_str("\x0E\x04 @@\n"); // SO + attr 0x04 + space glyph + endmarks
        for cp in 33u32..=126 {
            let ch = char::from_u32(cp).unwrap();
            s.push(ch);
            s.push_str("@@\n");
        }
        let font = parse_tlf(s.as_bytes()).expect("multicolor parses");
        assert!(font.multicolor, "multicolor flag must be set");
        let g = font.lookup(b' ' as u32).unwrap();
        let first_cell = g.rows[0].cells.iter().find(|c| c.ch == ' ').unwrap();
        assert_eq!(first_cell.color_attr, Some(0x04));
    }

    #[test]
    fn rejects_inconsistent_endmark() {
        // First glyph row uses `@`; second glyph row uses `#` → mismatch.
        let mut s = String::from("tlf2a$ 1 1 8 0 0\n");
        s.push_str(" @@\n");
        s.push_str("!##\n");
        for cp in 34u32..=126 {
            let ch = char::from_u32(cp).unwrap();
            s.push(ch);
            s.push_str("@@\n");
        }
        let err = parse_tlf(s.as_bytes()).unwrap_err();
        match err {
            FigletError::TlfParse { reason, .. } => {
                assert!(reason.contains("endmark"), "{reason}");
            }
            other => panic!("expected TlfParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_doubled_endmark_final_row() {
        // height=1 → single row IS final row, needs doubled endmark.
        let err = parse_tlf(b"tlf2a$ 1 1 8 0 0\n single@\n").unwrap_err();
        match err {
            FigletError::TlfParse { reason, .. } => {
                assert!(reason.contains("endmark"), "{reason}");
            }
            other => panic!("expected TlfParse, got {other:?}"),
        }
    }

    #[test]
    fn unicode_glyph_cell_decodes() {
        // Use a CJK char inside a glyph row.
        let mut s = String::from("tlf2a$ 1 1 8 0 0\n");
        // ASCII 32 row uses Chinese char `中` + doubled endmark.
        s.push_str("中@@\n");
        for cp in 33u32..=126 {
            let ch = char::from_u32(cp).unwrap();
            s.push(ch);
            s.push_str("@@\n");
        }
        let font = parse_tlf(s.as_bytes()).expect("unicode parses");
        let g = font.lookup(b' ' as u32).unwrap();
        assert_eq!(g.rows[0].cells[0].ch, '中');
    }

    #[test]
    fn max_length_cap_enforced() {
        // max_length=65537 → above cap.
        let err = parse_tlf(b"tlf2a$ 1 1 65537 0 0\n@@\n").unwrap_err();
        match err {
            FigletError::TlfParse { reason, .. } => {
                assert!(
                    reason.contains("max_length") || reason.contains("cap"),
                    "{reason}"
                );
            }
            other => panic!("expected TlfParse, got {other:?}"),
        }
    }
}
