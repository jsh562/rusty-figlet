//! mIRC `^C` color-code export backend (E012 US2 — FR-006, FR-015, FR-027).
//!
//! ## Format
//!
//! mIRC encodes colors as a `^C` (0x03) prefix followed by 1-2 decimal
//! digits selecting the foreground palette index from the 16-color
//! standard table (0..=15). Optionally a `,BB` suffix selects the
//! background. Reset is `^O` (0x0F).
//!
//! ## Non-printable stripping (FR-015)
//!
//! ASCII C0 control bytes (`0x00..=0x1F` except `0x09` tab) and the
//! C1 range (`0x7F..=0x9F`) are stripped from cell text per FR-015.
//! UTF-8 multibyte continuation bytes (`0x80..=0xBF`) are PRESERVED
//! when they're part of a valid UTF-8 sequence — we operate on `&str`,
//! not `&[u8]`, so the standard library has already validated UTF-8
//! boundaries before we see the input. The C1 range only intersects
//! the continuation range, but bytes that appear as the *start* of
//! a multi-byte sequence won't be in 0x80..=0x9F (UTF-8 leaders are
//! 0xC2..=0xF4).
//!
//! In practical terms: iterating `.chars()` gives us validated
//! codepoints, and we strip the BMP code-point if `(c as u32) < 0x20
//! && c != '\t'` OR `c == '\x7F'` OR `(c as u32 >= 0x80 && c as u32 <= 0x9F)`.
//!
//! ## Pre-sized writer (FR-027)
//!
//! Single-pass: we iterate cells once and emit both color codes and
//! text bytes in the same loop. `Vec<u8>::with_capacity(w * h * 6)`
//! covers `^C99,99X` (the longest per-cell encoding).
//!
//! ## Warn-on-strip
//!
//! The CLI flag `--warn-irc-strip` is wired in Phase 9 (T060). This
//! module accepts a boolean `warn_on_strip` parameter; when true the
//! function emits a single deduplicated stderr warning if any byte
//! was stripped during this call.

use crate::filter::{Cell, Color, NamedColor, RenderGrid};

/// mIRC color reset byte (`^O`).
const IRC_RESET: u8 = 0x0F;
/// mIRC color prefix byte (`^C`).
const IRC_COLOR: u8 = 0x03;

/// Encode `grid` as a sequence of mIRC `^C` color-coded bytes per
/// FR-006 + FR-015 + FR-027.
///
/// `warn_on_strip = true` emits a single stderr warning if any byte
/// was stripped per the non-printable filter. The strip is silent by
/// default; the CLI exposes this knob via `--warn-irc-strip` in Phase 9.
#[must_use]
pub fn write_irc(grid: &RenderGrid, warn_on_strip: bool) -> Vec<u8> {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let capacity = w.saturating_mul(h).saturating_mul(6).saturating_add(64);
    let mut out: Vec<u8> = Vec::with_capacity(capacity);
    let mut stripped_any = false;
    let mut prev_color: Option<Color> = None;

    for row in &grid.cells {
        for cell in row {
            // Emit color change only when the color differs from the
            // previous cell — keeps output compact and avoids ^C ^C ^C
            // noise from spans of same-color cells.
            if prev_color != Some(cell.fg) {
                out.push(IRC_COLOR);
                let palette_idx = color_to_mirc_index(cell.fg);
                push_decimal(&mut out, palette_idx);
                prev_color = Some(cell.fg);
            }
            // Strip + emit the cell glyph.
            if char_is_irc_safe(cell.ch) {
                push_utf8(&mut out, cell.ch);
            } else {
                stripped_any = true;
            }
        }
        // End of row: reset and emit newline.
        out.push(IRC_RESET);
        out.push(b'\n');
        prev_color = None;
    }

    if stripped_any && warn_on_strip {
        emit_strip_warning();
    }

    out
}

/// Predicate: is the codepoint emit-safe for mIRC per FR-015?
///
/// Returns `false` for ASCII C0 (`0x00..=0x1F` except `0x09` tab),
/// for DEL (`0x7F`), and for the C1 range (`0x80..=0x9F`).
/// Multi-byte UTF-8 codepoints above U+009F pass through (CJK, emoji,
/// RTL scripts — all preserved per spec Edge Cases).
fn char_is_irc_safe(c: char) -> bool {
    let cp = c as u32;
    if c == '\t' {
        return true;
    }
    if cp < 0x20 {
        return false;
    }
    if cp == 0x7F {
        return false;
    }
    if (0x80..=0x9F).contains(&cp) {
        return false;
    }
    true
}

/// Map a typed [`Color`] to an mIRC 0..=15 palette index.
///
/// mIRC has only 16 colors; truecolor/256-color inputs are mapped down
/// to the closest named color slot. The mapping mirrors the standard
/// mIRC palette table (white=0, black=1, blue=2, green=3, red=4, ...).
fn color_to_mirc_index(c: Color) -> u8 {
    match c {
        Color::Named(n) => named_to_mirc(n),
        Color::Index(idx) => {
            // 256-color → coarse downsample: use the first 16 entries
            // verbatim, else fall back to white (0).
            if idx < 16 {
                named_to_mirc(palette16_to_named(idx))
            } else {
                0
            }
        }
        Color::Rgb(_, _, _) => {
            // Truecolor → mIRC has no 24-bit support; mIRC sees the
            // color as default white. A full downsample would require
            // Lab distance — out of scope for v0.3.0.
            0
        }
    }
}

/// Standard mIRC palette positions per the v1.0 mIRC color spec.
fn named_to_mirc(n: NamedColor) -> u8 {
    match n {
        NamedColor::White => 0,
        NamedColor::Black => 1,
        NamedColor::Blue => 2,
        NamedColor::Green => 3,
        NamedColor::Red => 4,
        NamedColor::BrightRed => 4,
        NamedColor::Magenta => 6,
        NamedColor::Yellow => 8,
        NamedColor::BrightYellow => 8,
        NamedColor::BrightGreen => 9,
        NamedColor::Cyan => 10,
        NamedColor::BrightCyan => 11,
        NamedColor::BrightBlue => 12,
        NamedColor::BrightMagenta => 13,
        NamedColor::BrightBlack => 14,
        NamedColor::BrightWhite => 15,
    }
}

fn palette16_to_named(idx: u8) -> NamedColor {
    match idx {
        0 => NamedColor::Black,
        1 => NamedColor::Red,
        2 => NamedColor::Green,
        3 => NamedColor::Yellow,
        4 => NamedColor::Blue,
        5 => NamedColor::Magenta,
        6 => NamedColor::Cyan,
        7 => NamedColor::White,
        8 => NamedColor::BrightBlack,
        9 => NamedColor::BrightRed,
        10 => NamedColor::BrightGreen,
        11 => NamedColor::BrightYellow,
        12 => NamedColor::BrightBlue,
        13 => NamedColor::BrightMagenta,
        14 => NamedColor::BrightCyan,
        _ => NamedColor::BrightWhite,
    }
}

fn push_decimal(out: &mut Vec<u8>, n: u8) {
    if n >= 10 {
        out.push((n / 10) + b'0');
        out.push((n % 10) + b'0');
    } else {
        // mIRC permits both 1- and 2-digit codes; we use 2 digits for
        // 10..=15 only.
        out.push(n + b'0');
    }
}

fn push_utf8(out: &mut Vec<u8>, c: char) {
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    out.extend_from_slice(s.as_bytes());
}

#[cold]
#[inline(never)]
fn emit_strip_warning() {
    eprintln!("rusty-figlet: IRC export stripped non-printable bytes");
}

#[allow(dead_code)]
fn _suppress_unused(_: Cell) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{Cell, Color, NamedColor, RenderGrid};

    #[test]
    fn empty_grid_returns_empty_bytes() {
        let grid = RenderGrid::empty();
        let out = write_irc(&grid, false);
        assert!(out.is_empty());
    }

    #[test]
    fn plain_ascii_row_has_color_prefix_and_reset() {
        let grid = RenderGrid::from_text_rows(&[String::from("Hi")]);
        let out = write_irc(&grid, false);
        assert!(out.contains(&IRC_COLOR));
        assert!(out.contains(&IRC_RESET));
        assert!(out.contains(&b'H'));
        assert!(out.contains(&b'i'));
    }

    #[test]
    fn strips_c0_controls_except_tab() {
        let row = vec![
            Cell::new('\x00'),
            Cell::new('\x01'),
            Cell::new('\t'),
            Cell::new('A'),
        ];
        let grid = RenderGrid::from_rows(vec![row]);
        let out = write_irc(&grid, false);
        // Output should contain 'A' and '\t' but not 0x00 or 0x01.
        assert!(out.contains(&b'A'));
        assert!(out.contains(&b'\t'));
        assert!(!out.contains(&0x00));
        assert!(!out.contains(&0x01));
    }

    #[test]
    fn strips_del_byte() {
        let grid = RenderGrid::from_rows(vec![vec![Cell::new('\x7F'), Cell::new('B')]]);
        let out = write_irc(&grid, false);
        assert!(!out.contains(&0x7F));
        assert!(out.contains(&b'B'));
    }

    #[test]
    fn strips_c1_range() {
        // U+0085 is in C1.
        let grid = RenderGrid::from_rows(vec![vec![Cell::new('\u{0085}'), Cell::new('C')]]);
        let out = write_irc(&grid, false);
        assert!(out.contains(&b'C'));
        // Output should not contain the C1 byte.
        assert!(!out.contains(&0x85));
    }

    #[test]
    fn preserves_utf8_multibyte_cjk() {
        let grid = RenderGrid::from_text_rows(&[String::from("中")]);
        let out = write_irc(&grid, false);
        // U+4E2D = E4 B8 AD in UTF-8 — all three bytes should be present.
        assert!(out.contains(&0xE4));
        assert!(out.contains(&0xB8));
        assert!(out.contains(&0xAD));
    }

    #[test]
    fn preserves_utf8_emoji() {
        let grid = RenderGrid::from_text_rows(&[String::from("🦀")]);
        let out = write_irc(&grid, false);
        // U+1F980 = F0 9F A6 80 in UTF-8.
        assert!(out.contains(&0xF0));
        assert!(out.contains(&0x9F));
    }

    #[test]
    fn named_color_maps_to_palette_idx() {
        let cell = Cell {
            ch: 'X',
            fg: Color::Named(NamedColor::Red),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let out = write_irc(&grid, false);
        // After ^C, the palette index should be the digit '4' (Red=4).
        let prefix_pos = out.iter().position(|&b| b == IRC_COLOR).expect("has ^C");
        assert_eq!(out[prefix_pos + 1], b'4');
    }

    #[test]
    fn truecolor_falls_back_to_white_index() {
        let cell = Cell {
            ch: 'Y',
            fg: Color::Rgb(123, 45, 67),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let out = write_irc(&grid, false);
        let prefix_pos = out.iter().position(|&b| b == IRC_COLOR).expect("has ^C");
        assert_eq!(out[prefix_pos + 1], b'0');
    }

    #[test]
    fn row_ends_with_reset_and_newline() {
        let grid = RenderGrid::from_text_rows(&[String::from("A")]);
        let out = write_irc(&grid, false);
        assert_eq!(out[out.len() - 2], IRC_RESET);
        assert_eq!(out[out.len() - 1], b'\n');
    }
}
