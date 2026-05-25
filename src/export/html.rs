//! HTML5 export backend (E012 US2 — FR-005, FR-014, AD-004, HINT-004).
//!
//! ## Safe-to-embed guarantee
//!
//! Output produced by [`write_html`] is safe to embed verbatim inside an
//! HTML5 document. The hand-rolled 4-char escape table (per AD-004)
//! covers every metacharacter that can break out of either text-content
//! position or a **double-quoted** attribute position:
//!
//! | Byte | Escape    | Position(s) protected             |
//! |------|-----------|-----------------------------------|
//! | `<`  | `&lt;`    | text + attribute                  |
//! | `>`  | `&gt;`    | text + attribute                  |
//! | `&`  | `&amp;`   | text + attribute                  |
//! | `"`  | `&quot;`  | attribute (double-quoted) only    |
//!
//! ## Double-quoted-attribute constraint (HINT-004)
//!
//! Every attribute value emitted by this backend uses `"..."` quoting.
//! Single quotes (`'`) are NOT escaped because they are not metacharacters
//! inside `"..."` quoting. The SVG and HTML backends both consume this
//! same 4-char table — any future backend that emits single-quoted
//! attributes MUST add `'` to the escape set (per AD-004).
//!
//! ## XSS posture
//!
//! Library callers can pass arbitrary user-controlled strings into a
//! [`crate::filter::RenderGrid`] via [`crate::filter::Cell::ch`]; the
//! escape table protects against:
//! - `<script>` payload injection (`<` is escaped → `&lt;script&gt;`).
//! - Attribute-injection (`"` is escaped → `"><img onerror=x"` cannot
//!   close a surrounding attribute).
//! - Double-encoding (`&` is escaped → `&amp;` collisions are explicit).
//!
//! Fuzz harness `fuzz/fuzz_targets/html_escape.rs` (T050) enforces the
//! property:
//!   `output contains no unescaped < > "` AND `len(output) ≤ 6 × len(input)`.
//!
//! ## Pre-sized writer (FR-027)
//!
//! [`write_html`] allocates `String::with_capacity(w * h * 32)` up front
//! to amortize realloc cost. The factor 32 covers `<span style="color:#RRGGBB">X</span>`
//! plus newlines + escape expansion in the typical case.

use super::common::{color_to_hex, escape_into};
use crate::filter::{Cell, RenderGrid};

/// Encode `grid` as an HTML5 fragment.
///
/// Output shape:
/// ```html
/// <pre>
/// <span style="color:#RRGGBB">XX</span><span style="color:#RRGGBB">Y</span>
/// ...
/// </pre>
/// ```
///
/// Adjacent cells with identical foreground colors are coalesced into a
/// single `<span>` to keep the output compact. The writer is pre-sized
/// per FR-027 (`String::with_capacity(w * h * 32)`).
#[must_use]
pub fn write_html(grid: &RenderGrid) -> String {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let capacity = w.saturating_mul(h).saturating_mul(32).saturating_add(64);
    let mut out = String::with_capacity(capacity);

    out.push_str("<pre>\n");
    if w == 0 || h == 0 {
        out.push_str("</pre>\n");
        return out;
    }

    for row in &grid.cells {
        // Coalesce adjacent same-color runs into one <span> per AD-004.
        let mut i = 0;
        while i < row.len() {
            let run_color = row[i].fg;
            let mut j = i + 1;
            while j < row.len() && row[j].fg == run_color {
                j += 1;
            }
            // Emit one span for cells[i..j].
            out.push_str("<span style=\"color:");
            // color hex is numeric only — no escape needed but we still
            // emit it into the double-quoted attribute slot per HINT-004.
            let hex = color_to_hex(run_color);
            out.push_str(&hex);
            out.push_str("\">");
            for cell in &row[i..j] {
                escape_into(&mut out, &cell_to_string(cell));
            }
            out.push_str("</span>");
            i = j;
        }
        out.push('\n');
    }
    out.push_str("</pre>\n");
    out
}

/// Convert a single [`Cell`] to its single-char string representation.
fn cell_to_string(cell: &Cell) -> String {
    cell.ch.to_string()
}

#[cfg(test)]
mod tests {
    use super::super::common::{escape_into, index_to_rgb};
    use super::*;
    use crate::filter::{Cell, Color, NamedColor, RenderGrid};

    #[test]
    fn empty_grid_emits_pre_wrapper() {
        let grid = RenderGrid::empty();
        let html = write_html(&grid);
        assert!(html.starts_with("<pre>"));
        assert!(html.contains("</pre>"));
    }

    #[test]
    fn plain_ascii_pass_through() {
        let grid = RenderGrid::from_text_rows(&[String::from("Hello")]);
        let html = write_html(&grid);
        assert!(html.contains("Hello"));
        assert!(html.contains("<pre>"));
    }

    #[test]
    fn escape_lt_gt_amp_quot() {
        let mut s = String::new();
        escape_into(&mut s, "<>&\"");
        assert_eq!(s, "&lt;&gt;&amp;&quot;");
    }

    #[test]
    fn escape_script_tag() {
        let mut s = String::new();
        escape_into(&mut s, "<script>");
        assert_eq!(s, "&lt;script&gt;");
    }

    #[test]
    fn escape_attribute_breakout() {
        // The classic `"><img onerror=...>` payload. After escape, the
        // double quote and angle brackets are all neutralized.
        let mut s = String::new();
        escape_into(&mut s, "\"><img onerror=alert(1)>");
        assert!(!s.contains('"'));
        assert!(!s.contains('<'));
        assert!(!s.contains('>'));
        assert!(s.contains("&quot;"));
        assert!(s.contains("&lt;img"));
    }

    #[test]
    fn escape_ampersand_double_encoding() {
        // Probe for & double-encoding behavior. `&amp;` should become
        // `&amp;amp;` because each & is escaped exactly once.
        let mut s = String::new();
        escape_into(&mut s, "&amp;");
        assert_eq!(s, "&amp;amp;");
    }

    #[test]
    fn escape_passes_through_cjk() {
        let mut s = String::new();
        escape_into(&mut s, "中文");
        assert_eq!(s, "中文");
    }

    #[test]
    fn escape_passes_through_emoji() {
        let mut s = String::new();
        escape_into(&mut s, "🦀");
        assert_eq!(s, "🦀");
    }

    #[test]
    fn escape_single_quote_unchanged() {
        // Single quote is NOT in the escape set because we only emit
        // double-quoted attributes (HINT-004).
        let mut s = String::new();
        escape_into(&mut s, "it's");
        assert_eq!(s, "it's");
    }

    #[test]
    fn write_html_with_rgb_color() {
        let cell = Cell {
            ch: 'X',
            fg: Color::Rgb(255, 128, 0),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let html = write_html(&grid);
        assert!(html.contains("#FF8000"));
        assert!(html.contains(">X</span>"));
    }

    #[test]
    fn write_html_with_named_color() {
        let cell = Cell {
            ch: 'Y',
            fg: Color::Named(NamedColor::BrightRed),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let html = write_html(&grid);
        assert!(html.contains("#FF0000"));
    }

    #[test]
    fn write_html_no_unescaped_metacharacters_in_output_for_xss_input() {
        let cell = Cell {
            ch: '<',
            fg: Color::default(),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let html = write_html(&grid);
        // The structural `<` characters come from `<pre>` and `<span>`,
        // but the cell's `<` must NOT appear unescaped. Verify by
        // checking that we see `&lt;` exactly where the cell content
        // sits — between `>` and `<` of the span tags.
        assert!(html.contains(">&lt;</span>"));
    }

    #[test]
    fn write_html_coalesces_same_color_run() {
        let g = RenderGrid::from_text_rows(&[String::from("ABC")]);
        let html = write_html(&g);
        // All three cells share default color → one <span> per row,
        // not three.
        let span_count = html.matches("<span").count();
        assert_eq!(span_count, 1);
    }

    #[test]
    fn index_to_rgb_grayscale_ramp() {
        // 232 = darkest gray (8,8,8); 255 = lightest (238,238,238).
        assert_eq!(index_to_rgb(232), 0x080808);
        assert_eq!(index_to_rgb(255), 0xEEEEEE);
    }

    #[test]
    fn index_to_rgb_cube_corner() {
        // Index 16 = (0,0,0); 231 = (255,255,255).
        assert_eq!(index_to_rgb(16), 0x000000);
        assert_eq!(index_to_rgb(231), 0xFFFFFF);
    }

    #[test]
    fn write_html_has_pre_wrapper_around_all_content() {
        let grid = RenderGrid::from_text_rows(&[String::from("X")]);
        let html = write_html(&grid);
        let pre_open = html.find("<pre>").expect("opens with <pre>");
        let pre_close = html.find("</pre>").expect("closes with </pre>");
        assert!(pre_open < pre_close);
    }
}
