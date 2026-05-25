//! SVG 1.1 export backend (E012 US2 — FR-007, FR-027).
//!
//! ## Format
//!
//! Each row of the grid becomes one `<text>` element positioned at
//! `y = (row + 1) * line_height` with `fill="#RRGGBB"` for the
//! row's dominant color. Cells within a row are rendered as one
//! `<tspan fill="#RRGGBB">...</tspan>` per color run, mirroring the
//! HTML backend's coalescing strategy.
//!
//! ## Security posture (spec Edge Cases)
//!
//! - **All attribute values are DOUBLE-QUOTED** per HINT-004 so the
//!   same 4-char escape table from [`super::html`] applies verbatim.
//! - **NO `<script>` element emitted**, ever.
//! - **NO `<foreignObject>` element emitted**, ever.
//! - **NO `xlink:href`, `<image href=...>`, `<use xlink:href=...>`**
//!   emitted — no external resource references can leak via this
//!   backend.
//! - **Inline `style="..."` is NOT emitted by this writer** — color
//!   information lives in the `fill` attribute whose value is a typed
//!   `#RRGGBB` numeric hex string, NOT user bytes.
//!
//! Integration tests in `tests/export_integration.rs` enforce these
//! constraints via the `svg_contains_no_external_resource_attrs` test.
//!
//! ## Pre-sized writer (FR-027)
//!
//! `String::with_capacity(w * h * 64)` covers `<text>` + `<tspan>` +
//! escape expansion in the typical case.

use super::common::{color_to_hex, escape_into};
use crate::filter::RenderGrid;

/// Horizontal advance per cell, in SVG user units.
const CELL_WIDTH: u32 = 10;
/// Vertical advance per row, in SVG user units.
const LINE_HEIGHT: u32 = 14;
/// Monospace font family — kept generic to avoid bundling a custom font.
const FONT_FAMILY: &str = "monospace";

/// Encode `grid` as an SVG 1.1 document (string form).
///
/// Output shape:
/// ```svg
/// <?xml version="1.0" encoding="UTF-8"?>
/// <svg xmlns="http://www.w3.org/2000/svg" width="W" height="H" ...>
///   <text x="0" y="14" font-family="monospace" font-size="12" xml:space="preserve">
///     <tspan fill="#RRGGBB">XX</tspan><tspan fill="#RRGGBB">Y</tspan>
///   </text>
///   ...
/// </svg>
/// ```
///
/// `xml:space="preserve"` ensures whitespace cells render with proper
/// width. All attribute values are double-quoted per HINT-004 + AD-004.
#[must_use]
pub fn write_svg(grid: &RenderGrid) -> String {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let capacity = w.saturating_mul(h).saturating_mul(64).saturating_add(256);
    let mut out = String::with_capacity(capacity);

    let total_w = (w as u32) * CELL_WIDTH;
    let total_h = (h as u32) * LINE_HEIGHT;

    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"");
    push_u32(&mut out, total_w);
    out.push_str("\" height=\"");
    push_u32(&mut out, total_h);
    out.push_str("\" version=\"1.1\">\n");

    if w == 0 || h == 0 {
        out.push_str("</svg>\n");
        return out;
    }

    for (y, row) in grid.cells.iter().enumerate() {
        let y_coord = ((y as u32) + 1) * LINE_HEIGHT;
        out.push_str("  <text x=\"0\" y=\"");
        push_u32(&mut out, y_coord);
        out.push_str("\" font-family=\"");
        out.push_str(FONT_FAMILY);
        out.push_str("\" font-size=\"12\" xml:space=\"preserve\">");

        // Coalesce same-color runs into one <tspan>.
        let mut i = 0;
        while i < row.len() {
            let run_color = row[i].fg;
            let mut j = i + 1;
            while j < row.len() && row[j].fg == run_color {
                j += 1;
            }
            out.push_str("<tspan fill=\"");
            let hex = color_to_hex(run_color);
            out.push_str(&hex);
            out.push_str("\">");
            let mut buf = String::new();
            for cell in &row[i..j] {
                buf.push(cell.ch);
            }
            escape_into(&mut out, &buf);
            out.push_str("</tspan>");
            i = j;
        }

        out.push_str("</text>\n");
    }

    out.push_str("</svg>\n");
    out
}

fn push_u32(out: &mut String, mut n: u32) {
    if n == 0 {
        out.push('0');
        return;
    }
    let mut buf = [0u8; 10];
    let mut i = 0;
    while n > 0 {
        buf[i] = (n % 10) as u8 + b'0';
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        out.push(buf[i] as char);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::{Cell, Color, RenderGrid};

    #[test]
    fn empty_grid_emits_svg_wrapper() {
        let grid = RenderGrid::empty();
        let svg = write_svg(&grid);
        assert!(svg.contains("<?xml"));
        assert!(svg.contains("<svg"));
        assert!(svg.contains("</svg>"));
    }

    #[test]
    fn no_external_resource_attrs_emitted() {
        let grid = RenderGrid::from_text_rows(&[String::from("AB")]);
        let svg = write_svg(&grid);
        // None of these MUST appear in the output.
        assert!(!svg.contains("xlink:href"));
        assert!(!svg.contains("<image"));
        assert!(!svg.contains("<use "));
        assert!(!svg.contains("<script"));
        assert!(!svg.contains("<foreignObject"));
        // `style="..."` is not emitted by this writer either.
        assert!(!svg.contains("style=\""));
    }

    #[test]
    fn double_quoted_attributes_only() {
        let grid = RenderGrid::from_text_rows(&[String::from("X")]);
        let svg = write_svg(&grid);
        // No single-quoted attribute values; we always emit `="...".
        assert!(!svg.contains("='"));
    }

    #[test]
    fn escapes_xss_payload_in_cell() {
        let cell = Cell {
            ch: '<',
            fg: Color::default(),
            bg: None,
            attrs: 0,
        };
        let grid = RenderGrid::from_rows(vec![vec![cell]]);
        let svg = write_svg(&grid);
        // The cell `<` becomes `&lt;`. The structural `<` chars in
        // tags are unaffected, but we can grep for the escaped form.
        assert!(svg.contains("&lt;"));
    }

    #[test]
    fn cjk_passes_through() {
        let grid = RenderGrid::from_text_rows(&[String::from("漢字")]);
        let svg = write_svg(&grid);
        assert!(svg.contains("漢"));
        assert!(svg.contains("字"));
    }
}
