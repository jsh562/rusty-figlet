//! FIGfont 2.0 parser, bundled-font lookup table, and font resolver.
//!
//! This module owns the `flf2a` decoder (header + comments + glyph
//! endmarks + codetag table) per HINT-001 and the 12 bundled-font
//! `include_bytes!` table per AD-008 + AD-016. The resolver follows the
//! search-path ladder enumerated in FR-010.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::FigletError;

/// Required codepoints that every FIGfont MUST define (ASCII 32..=126
/// plus seven German chars).
const REQUIRED_CODEPOINTS_ASCII: std::ops::RangeInclusive<u32> = 32..=126;
const REQUIRED_CODEPOINTS_GERMAN: &[u32] = &[196, 214, 220, 228, 246, 252, 223];

/// Parsed FIGfont 2.0 representation. Owns its glyph data.
#[derive(Debug, Clone)]
pub struct FIGfont {
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
    /// Modern multi-byte layout descriptor (bits per HINT-002).
    pub full_layout: u32,
    /// Print direction (0 = left-to-right, 1 = right-to-left).
    pub print_direction: u32,
    /// Map from codepoint → `height` rows of glyph data (endmarks already stripped).
    pub glyphs: HashMap<u32, Vec<String>>,
    /// Number of codetag blocks declared by the header (0 when omitted).
    pub codetag_count: u32,
}

/// Parse a FIGfont 2.0 byte slice (FR-052; no filesystem access).
///
/// Implements HINT-001 line-by-line per the enumerated rejection cases:
/// (1) bad signature, (2) truncated header, (3) `comment_lines` mismatch,
/// (4) short glyph block, (5) missing endmark, (6) `codetag_count`
/// divergence. Every failure surfaces [`FigletError::FontParse`] with a
/// 1-indexed line number.
pub fn parse_bytes(input: &[u8]) -> Result<FIGfont, FigletError> {
    // FIGfonts are conventionally Latin-1; decode tolerantly by mapping
    // bytes 0..255 to chars 0..255 so the parser does not panic on
    // upstream-provided non-UTF8 bytes.
    let text: String = input.iter().map(|&b| b as char).collect();
    let mut lines = text.split('\n');
    let header_line = lines
        .next()
        .ok_or_else(|| parse_err("empty input", 1))?
        .trim_end_matches('\r');

    let header = parse_header(header_line, 1)?;

    // Skip exactly `comment_lines` next lines.
    let mut current_line: u32 = 1;
    for _ in 0..header.comment_lines {
        current_line += 1;
        if lines.next().is_none() {
            return Err(parse_err(
                "truncated comment block: comment_lines header value exceeds available lines",
                current_line,
            ));
        }
    }

    // Read required glyphs. The endmark is the LAST character of the
    // first glyph's first line (per FIGfont 2.0 spec); detection happens
    // implicitly via `strip_endmarks` below. The required set is ASCII
    // 32..=126 inline, then the 7 German codepoints either inline OR as
    // codetag blocks — both forms appear in upstream / placeholder fonts
    // and the parser accepts whichever shape the file uses.
    let mut glyphs: HashMap<u32, Vec<String>> = HashMap::new();
    let mut endmark: Option<char> = None;

    for cp in REQUIRED_CODEPOINTS_ASCII.clone() {
        let rows = read_glyph(&mut lines, header.height, &mut current_line, &mut endmark)?;
        glyphs.insert(cp, rows);
    }

    // Buffer the next line so we can peek: if it looks like a codetag
    // header (first whitespace-separated token parses as a hex/decimal
    // codepoint), switch to codetag mode for the 7 German chars;
    // otherwise consume them inline.
    let mut buffered: Option<String> = None;

    {
        // Try to fetch the next non-empty line.
        let peek_line_no = current_line + 1;
        let peeked = next_non_empty(&mut lines, &mut current_line);
        if let Some(line) = peeked {
            if looks_like_codetag_header(&line) {
                buffered = Some(line);
                // current_line already advanced to peek_line_no
                let _ = peek_line_no;
            } else {
                // Treat this line as the first German glyph row.
                let mut rows = Vec::with_capacity(header.height as usize);
                let stripped =
                    strip_endmark(&line, header.height == 1, &mut endmark, current_line)?;
                rows.push(stripped);
                for row in 1..header.height {
                    current_line += 1;
                    let raw = lines
                        .next()
                        .ok_or_else(|| {
                            parse_err("short glyph block: hit EOF mid-glyph", current_line)
                        })?
                        .trim_end_matches('\r');
                    let stripped =
                        strip_endmark(raw, row == header.height - 1, &mut endmark, current_line)?;
                    rows.push(stripped);
                }
                glyphs.insert(REQUIRED_CODEPOINTS_GERMAN[0], rows);
                // Read the remaining 6 German glyphs inline.
                for &cp in &REQUIRED_CODEPOINTS_GERMAN[1..] {
                    let rows =
                        read_glyph(&mut lines, header.height, &mut current_line, &mut endmark)?;
                    glyphs.insert(cp, rows);
                }
            }
        }
        // If `peeked` was None we hit EOF immediately after ASCII; the
        // codetag stream may still supply German chars (rare but valid)
        // or the file is truncated — the post-loop check enforces.
    }

    // Codetag blocks: each is `<hexcode> <comment>\n<glyph rows>`.
    let mut actual_codetag = 0u32;
    loop {
        let header_text = if let Some(b) = buffered.take() {
            b
        } else {
            match next_non_empty(&mut lines, &mut current_line) {
                Some(line) => line,
                None => {
                    if header.codetag_count != 0 && actual_codetag != header.codetag_count {
                        return Err(parse_err(
                            &format!(
                                "codetag_count divergence: header declared {}, parsed {}",
                                header.codetag_count, actual_codetag
                            ),
                            current_line,
                        ));
                    }
                    // Verify required German chars are all present.
                    for &cp in REQUIRED_CODEPOINTS_GERMAN {
                        if !glyphs.contains_key(&cp) {
                            return Err(parse_err(
                                &format!("missing required German codepoint U+{cp:04X}"),
                                current_line,
                            ));
                        }
                    }
                    return Ok(FIGfont {
                        hardblank: header.hardblank,
                        height: header.height,
                        baseline: header.baseline,
                        max_length: header.max_length,
                        old_layout: header.old_layout,
                        full_layout: header.full_layout,
                        print_direction: header.print_direction,
                        glyphs,
                        codetag_count: header.codetag_count,
                    });
                }
            }
        };

        let codepoint = parse_codetag_codepoint(&header_text, current_line)?;
        let rows = read_glyph(&mut lines, header.height, &mut current_line, &mut endmark)?;
        glyphs.insert(codepoint, rows);
        actual_codetag += 1;
    }
}

/// Read the next non-empty line, advancing `current_line` for every
/// line skipped (including the consumed non-empty one). Returns `None`
/// at EOF.
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

/// Heuristic: does this line look like a codetag header? Codetag
/// headers begin with `<hex>` or `<decimal>` followed by whitespace
/// and a comment. Real glyph rows of FIGfonts end in an endmark and
/// do NOT have an interior space followed by a comment.
fn looks_like_codetag_header(line: &str) -> bool {
    let mut parts = line.splitn(2, char::is_whitespace);
    let Some(first) = parts.next() else {
        return false;
    };
    let rest = parts.next();
    if rest.is_none() || rest == Some("") {
        return false;
    }
    parse_codetag_codepoint(first, 0).is_ok()
}

/// Header field bag derived from the `flf2a` signature line.
struct Header {
    hardblank: char,
    height: u32,
    baseline: u32,
    max_length: u32,
    old_layout: i32,
    comment_lines: u32,
    print_direction: u32,
    full_layout: u32,
    codetag_count: u32,
}

fn parse_header(line: &str, line_no: u32) -> Result<Header, FigletError> {
    if !line.starts_with("flf2a") {
        return Err(parse_err("bad signature: expected flf2a prefix", line_no));
    }
    let rest = &line["flf2a".len()..];
    // First char after `flf2a` is the hardblank; remaining whitespace-
    // separated tokens are the integer fields.
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
    // Derive default from old_layout per FIGfont 2.0 spec when omitted.
    let derived_full_layout = if old_layout < 0 { 0 } else { old_layout as u32 };
    let full_layout = next_u32_opt(&mut tokens).unwrap_or(derived_full_layout);
    let codetag_count = next_u32_opt(&mut tokens).unwrap_or(0);

    Ok(Header {
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

/// Read exactly `height` glyph lines from `lines`, advancing
/// `current_line`. Strips endmarks per HINT-001: single endmark on rows
/// 0..height-1, doubled endmark on the final row.
///
/// `endmark` is shared across all glyphs in the font; the first call
/// initializes it from the last character of the first glyph's first
/// line, and every subsequent call MUST observe the same character.
fn read_glyph<'a, I>(
    lines: &mut I,
    height: u32,
    current_line: &mut u32,
    endmark: &mut Option<char>,
) -> Result<Vec<String>, FigletError>
where
    I: Iterator<Item = &'a str>,
{
    let mut rows = Vec::with_capacity(height as usize);
    for row in 0..height {
        *current_line += 1;
        let raw = lines
            .next()
            .ok_or_else(|| parse_err("short glyph block: hit EOF mid-glyph", *current_line))?
            .trim_end_matches('\r');
        if raw.is_empty() {
            return Err(parse_err(
                "short glyph block: blank line where glyph row expected",
                *current_line,
            ));
        }
        let stripped = strip_endmark(raw, row == height - 1, endmark, *current_line)?;
        rows.push(stripped);
    }
    Ok(rows)
}

fn strip_endmark(
    raw: &str,
    last_row: bool,
    endmark: &mut Option<char>,
    line_no: u32,
) -> Result<String, FigletError> {
    let chars: Vec<char> = raw.chars().collect();
    if chars.is_empty() {
        return Err(parse_err("missing endmark: glyph row is empty", line_no));
    }
    let candidate = *chars.last().expect("non-empty just checked");

    // The endmark is determined by the LAST character of the first
    // glyph's first line; subsequent lines MUST end with the same char.
    let mark = match *endmark {
        Some(m) => m,
        None => {
            *endmark = Some(candidate);
            candidate
        }
    };

    if candidate != mark {
        return Err(parse_err(
            &format!("missing endmark: row ends with '{candidate}', expected endmark '{mark}'"),
            line_no,
        ));
    }

    // Strip a single endmark; the final row MUST carry a doubled endmark.
    let mut end = chars.len() - 1;
    if last_row {
        if end == 0 || chars[end - 1] != mark {
            return Err(parse_err(
                "missing endmark: final glyph row lacks doubled endmark",
                line_no,
            ));
        }
        end -= 1;
    }
    Ok(chars[..end].iter().collect())
}

/// Parse the first whitespace-separated token of a codetag header line
/// as a hexadecimal integer (HINT-001 rejects decimal interpretation).
fn parse_codetag_codepoint(line: &str, line_no: u32) -> Result<u32, FigletError> {
    let tok = line
        .split_whitespace()
        .next()
        .ok_or_else(|| parse_err("codetag header missing codepoint token", line_no))?;
    // Accept optional `0x` / `0X` prefix or bare hex digits.
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
        parse_err(
            &format!("codetag codepoint not hexadecimal: {tok}"),
            line_no,
        )
    })?;
    if negative {
        // Upstream allows negative codetags as a "delete this codepoint"
        // marker; preserve the magnitude per HINT-001 but warn on a
        // separate, non-failing channel. For now, store as wrapping u32.
        Ok(value.wrapping_neg())
    } else {
        Ok(value)
    }
}

fn parse_err(reason: &str, line: u32) -> FigletError {
    FigletError::FontParse {
        reason: reason.to_owned(),
        line,
    }
}

/// Compile-time-embedded bundled-font assets. Populated by
/// `include_bytes!` so the binary needs zero runtime IO to render with
/// any of the 12 bundled fonts.
pub static BUNDLED_FONTS: &[(&str, &[u8])] = &[
    ("standard", include_bytes!("../assets/fonts/standard.flf")),
    ("slant", include_bytes!("../assets/fonts/slant.flf")),
    ("small", include_bytes!("../assets/fonts/small.flf")),
    ("big", include_bytes!("../assets/fonts/big.flf")),
    ("mini", include_bytes!("../assets/fonts/mini.flf")),
    ("banner", include_bytes!("../assets/fonts/banner.flf")),
    ("block", include_bytes!("../assets/fonts/block.flf")),
    ("bubble", include_bytes!("../assets/fonts/bubble.flf")),
    ("digital", include_bytes!("../assets/fonts/digital.flf")),
    ("lean", include_bytes!("../assets/fonts/lean.flf")),
    ("script", include_bytes!("../assets/fonts/script.flf")),
    ("shadow", include_bytes!("../assets/fonts/shadow.flf")),
];

/// Look up a single codepoint's glyph rows in `font`.
///
/// Returns `Some(rows)` when the codepoint has a dedicated glyph in
/// the font's codetag map; returns `None` for codepoints absent from
/// the font. Callers implementing HINT-009 substitute the font's
/// `codepoint 0` "missing-character" glyph (when present) before
/// emitting the one-time stderr warning.
pub fn lookup_codepoint(font: &FIGfont, cp: u32) -> Option<&Vec<String>> {
    font.glyphs.get(&cp)
}

/// Look up a bundled font by name (case-sensitive). The `.flf` suffix
/// MUST already be stripped per HINT-003.
pub fn resolve_bundled(name: &str) -> Option<&'static [u8]> {
    BUNDLED_FONTS
        .iter()
        .find_map(|(n, bytes)| if *n == name { Some(*bytes) } else { None })
}

/// Resolve a font name (or path) per the FR-010 search ladder and
/// return the raw `.flf` bytes.
///
/// Search order:
/// 1. Exact path (if `name` looks like a `.flf` path that exists on disk).
/// 2. Bundled font table (after stripping `.flf` suffix).
/// 3. Each directory in `extra_dirs` (from repeated `-d` flags).
/// 4. Platform user-data dir (`~/.local/share/figlet/` on Unix,
///    `%APPDATA%\figlet\fonts\` on Windows).
/// 5. `/usr/share/figlet/` on Unix.
///
/// Returns [`FigletError::FontNotFound`] with the list of inspected
/// paths on miss.
pub fn resolve_font(name: &str, extra_dirs: &[PathBuf]) -> Result<Vec<u8>, FigletError> {
    let mut searched: Vec<PathBuf> = Vec::new();

    // (1) exact path.
    let path = Path::new(name);
    if path.extension().is_some_and(|ext| ext == "flf") {
        searched.push(path.to_path_buf());
        if path.is_file() {
            return std::fs::read(path).map_err(FigletError::from);
        }
    }

    // (2) bundled table (strip optional .flf suffix first).
    let bare = name.strip_suffix(".flf").unwrap_or(name);
    if let Some(bytes) = resolve_bundled(bare) {
        return Ok(bytes.to_vec());
    }

    // (3) repeated `-d` dirs.
    for dir in extra_dirs {
        for candidate_name in [name.to_owned(), format!("{bare}.flf")] {
            let p = dir.join(&candidate_name);
            searched.push(p.clone());
            if p.is_file() {
                return std::fs::read(&p).map_err(FigletError::from);
            }
        }
    }

    // (4) per-platform user-data dir.
    if let Some(user_dir) = user_data_dir() {
        let p = user_dir.join(format!("{bare}.flf"));
        searched.push(p.clone());
        if p.is_file() {
            return std::fs::read(&p).map_err(FigletError::from);
        }
    }

    // (5) Unix system dir.
    #[cfg(unix)]
    {
        let p = PathBuf::from("/usr/share/figlet").join(format!("{bare}.flf"));
        searched.push(p.clone());
        if p.is_file() {
            return std::fs::read(&p).map_err(FigletError::from);
        }
    }

    Err(FigletError::FontNotFound {
        name: name.to_owned(),
        searched,
    })
}

fn user_data_dir() -> Option<PathBuf> {
    #[cfg(unix)]
    {
        std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share/figlet"))
    }
    #[cfg(windows)]
    {
        std::env::var_os("APPDATA").map(|a| PathBuf::from(a).join("figlet\\fonts"))
    }
    #[cfg(not(any(unix, windows)))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_table_has_twelve_entries() {
        assert_eq!(BUNDLED_FONTS.len(), 12);
    }

    #[test]
    fn resolve_bundled_finds_standard() {
        assert!(resolve_bundled("standard").is_some());
    }

    #[test]
    fn resolve_bundled_misses_unknown() {
        assert!(resolve_bundled("nonexistent").is_none());
    }

    #[test]
    fn parses_each_bundled_font() {
        for (name, bytes) in BUNDLED_FONTS {
            let font = parse_bytes(bytes).unwrap_or_else(|err| {
                panic!("bundled font {name} failed to parse: {err}");
            });
            assert!(font.height >= 1, "{name} height >= 1");
            // ASCII coverage check.
            for cp in 32..=126u32 {
                assert!(
                    font.glyphs.contains_key(&cp),
                    "{name} missing ASCII codepoint {cp}"
                );
            }
            for &cp in REQUIRED_CODEPOINTS_GERMAN {
                assert!(
                    font.glyphs.contains_key(&cp),
                    "{name} missing German codepoint {cp}"
                );
            }
        }
    }

    #[test]
    fn rejects_bad_signature() {
        let err = parse_bytes(b"NOTflf2a$ 1 1 8 0 0\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, line } => {
                assert!(reason.contains("bad signature"), "{reason}");
                assert_eq!(line, 1);
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_truncated_header() {
        let err = parse_bytes(b"flf2a$ 1 1\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, line } => {
                assert!(reason.contains("truncated header"), "{reason}");
                assert_eq!(line, 1);
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_old_layout_out_of_range() {
        // old_layout=64 is out of -1..=63 range.
        let err = parse_bytes(b"flf2a$ 1 1 8 64 0\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("old_layout"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_old_layout_below_negative_one() {
        let err = parse_bytes(b"flf2a$ 1 1 8 -2 0\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("old_layout"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_comment_lines_mismatch() {
        // Declare 99 comment lines but provide only 1 → EOF before all
        // comments consumed.
        let err = parse_bytes(b"flf2a$ 1 1 8 0 99\nonly one\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("comment"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_short_glyph_block() {
        // height=3 but only 1 row before EOF.
        let err = parse_bytes(b"flf2a$ 3 1 8 0 0\nrow1@@\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("short glyph block"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn rejects_missing_doubled_endmark_on_final_row() {
        // Single `@` on final row when doubled is required.
        let err = parse_bytes(b"flf2a$ 1 1 8 0 0\nsingle@\n").unwrap_err();
        match err {
            FigletError::FontParse { reason, .. } => {
                assert!(reason.contains("endmark"), "{reason}");
            }
            other => panic!("expected FontParse, got {other:?}"),
        }
    }

    #[test]
    fn lookup_codepoint_finds_ascii_and_german() {
        let font = parse_bytes(BUNDLED_FONTS[0].1).expect("standard parses");
        // ASCII 'A' (0x41) MUST resolve.
        assert!(lookup_codepoint(&font, b'A' as u32).is_some());
        // German U+00C4 MUST resolve via codetag.
        assert!(lookup_codepoint(&font, 0x00C4).is_some());
        // Far-out CJK codepoint MUST miss.
        assert!(lookup_codepoint(&font, 0x4E2D).is_none());
    }

    #[test]
    fn parses_codetag_codepoint_as_hex() {
        // "C4" is hex 196; ensure we never decode it as decimal 4*10+12=124.
        let cp = parse_codetag_codepoint("C4 GERMAN AE", 0).unwrap();
        assert_eq!(cp, 0xC4);
        let cp = parse_codetag_codepoint("0x20 SPACE", 0).unwrap();
        assert_eq!(cp, 0x20);
    }
}
