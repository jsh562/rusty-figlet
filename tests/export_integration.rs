//! E012 Phase 7 — Export backend integration tests (T047 + T048).
//!
//! Covers FR-005 + FR-006 + FR-007 (HTML/IRC/SVG format conformance),
//! FR-014 + FR-015 (escape + non-printable stripping), FR-016
//! (UnsupportedExportFormat), and SC-002 (XSS hardening + UTF-8 +
//! bidirectional script handling per spec Edge Cases coverage matrix).

use rusty_figlet::export::{ExportFormat, write_export};
use rusty_figlet::filter::{Cell, Color, NamedColor, RenderGrid};

// ============================================================================
// HTML — T047
// ============================================================================

#[cfg(feature = "output-html")]
#[test]
fn html_emits_valid_html5() {
    let grid = RenderGrid::from_text_rows(&[String::from("HELLO")]);
    let bytes = write_export(&grid, ExportFormat::Html).expect("html ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.contains("<pre>"));
    assert!(s.contains("</pre>"));
    assert!(s.contains("HELLO"));
    // Manual structural assertion: opening <pre> precedes closing </pre>.
    let open = s.find("<pre>").expect("opens with <pre>");
    let close = s.find("</pre>").expect("closes with </pre>");
    assert!(open < close);
}

#[cfg(feature = "output-html")]
#[test]
fn html_escapes_script_tags() {
    // Cells contain `<` / `>` chars that, unescaped, would close the
    // surrounding <pre> tag and inject a <script>.
    let cells = vec![
        Cell::new('<'),
        Cell::new('s'),
        Cell::new('c'),
        Cell::new('r'),
        Cell::new('i'),
        Cell::new('p'),
        Cell::new('t'),
        Cell::new('>'),
    ];
    let grid = RenderGrid::from_rows(vec![cells]);
    let bytes = write_export(&grid, ExportFormat::Html).expect("html ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    // The escaped form must appear and the raw cell-content `<script>`
    // must NOT appear (only `<pre>`, `<span>`, etc. — the structural
    // tags — contain literal `<`).
    assert!(s.contains("&lt;script&gt;"));
    assert!(!s.contains("<script>"));
}

#[cfg(feature = "output-html")]
#[test]
fn html_escapes_attribute_injection() {
    // The classic `"><img onerror=...>` payload. The escape MUST
    // neutralize the closing `"`, `>`, and the opening `<` of the
    // injected `<img>`.
    let cells: Vec<Cell> = "\"><img onerror=alert(1)>".chars().map(Cell::new).collect();
    let grid = RenderGrid::from_rows(vec![cells]);
    let bytes = write_export(&grid, ExportFormat::Html).expect("html ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.contains("&quot;"));
    assert!(s.contains("&lt;img"));
    assert!(!s.contains("<img"));
}

// ============================================================================
// IRC — T047
// ============================================================================

#[cfg(feature = "output-irc")]
#[test]
fn irc_emits_mirc_codes() {
    let cell = Cell {
        ch: 'R',
        fg: Color::Named(NamedColor::Red),
        bg: None,
        attrs: 0,
    };
    let grid = RenderGrid::from_rows(vec![vec![cell]]);
    let bytes = write_export(&grid, ExportFormat::Irc).expect("irc ok");
    // Expect ^C (0x03) prefix and ^O (0x0F) reset byte.
    assert!(bytes.contains(&0x03));
    assert!(bytes.contains(&0x0F));
    assert!(bytes.contains(&b'R'));
}

#[cfg(feature = "output-irc")]
#[test]
fn irc_strips_non_printable() {
    // Insert C0 + DEL + C1 bytes; they should not appear in the
    // output. Tab (0x09) IS preserved.
    let row = vec![
        Cell::new('\x00'),
        Cell::new('\x07'),
        Cell::new('\t'),
        Cell::new('A'),
        Cell::new('\x7F'),
        Cell::new('\u{0085}'),
    ];
    let grid = RenderGrid::from_rows(vec![row]);
    let bytes = write_export(&grid, ExportFormat::Irc).expect("irc ok");
    assert!(!bytes.contains(&0x00));
    assert!(!bytes.contains(&0x07));
    assert!(bytes.contains(&b'\t'));
    assert!(bytes.contains(&b'A'));
    assert!(!bytes.contains(&0x7F));
    assert!(!bytes.contains(&0x85));
}

// ============================================================================
// SVG — T047
// ============================================================================

#[cfg(feature = "output-svg")]
#[test]
fn svg_emits_valid_xml() {
    let grid = RenderGrid::from_text_rows(&[String::from("AB")]);
    let bytes = write_export(&grid, ExportFormat::Svg).expect("svg ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.starts_with("<?xml"));
    assert!(s.contains("<svg "));
    assert!(s.contains("</svg>"));
    assert!(s.contains("<text "));
    assert!(s.contains("xmlns=\"http://www.w3.org/2000/svg\""));
    // Double-quoted attribute discipline (HINT-004).
    assert!(!s.contains("='"));
}

#[cfg(feature = "output-svg")]
#[test]
fn svg_contains_no_external_resource_attrs() {
    // Critical correctness note 2 from spec: SVG MUST NOT emit any
    // external-resource attributes or escape-hatch elements that
    // could exfiltrate data or execute scripts.
    let grid = RenderGrid::from_text_rows(&[String::from("test")]);
    let bytes = write_export(&grid, ExportFormat::Svg).expect("svg ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(!s.contains("xlink:href"));
    assert!(!s.contains("<image"));
    assert!(!s.contains("<use "));
    assert!(!s.contains("<script"));
    assert!(!s.contains("<foreignObject"));
    assert!(!s.contains("style=\""));
}

#[cfg(feature = "output-svg")]
#[test]
fn svg_escapes_javascript_uri() {
    // `javascript:` URIs in SVG `href` slots are a known XSS vector.
    // We don't emit `href` at all, but as a defense-in-depth check
    // verify that even when the cell contains `javascript:alert(1)`
    // the angle brackets and ampersands in surrounding HTML can't be
    // exploited.
    let cells: Vec<Cell> = "javascript:alert(1)".chars().map(Cell::new).collect();
    let grid = RenderGrid::from_rows(vec![cells]);
    let bytes = write_export(&grid, ExportFormat::Svg).expect("svg ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    // The literal `javascript:alert(1)` text survives as text content
    // (it's not an executable context inside <tspan>), but no
    // attribute could carry it.
    assert!(!s.contains("href="));
    assert!(!s.contains("xlink:"));
}

// ============================================================================
// UnsupportedExportFormat — T046
// ============================================================================

#[test]
fn leaf_disabled_returns_unsupported_export_format() {
    // When all output-* leaves are enabled (default-features), every
    // documented format should succeed except AnsiTrue / Ansi256 /
    // Ansi16, which Phase 7 leaves intentionally unwired (Phase 9
    // T061 ties them in via the existing color module).
    let grid = RenderGrid::blank(1, 1);
    let err = write_export(&grid, ExportFormat::AnsiTrue).unwrap_err();
    match err {
        rusty_figlet::FigletError::UnsupportedExportFormat {
            requested,
            available: _,
        } => {
            assert_eq!(requested, "ansi-true");
        }
        other => panic!("expected UnsupportedExportFormat, got {other:?}"),
    }
}

// ============================================================================
// UTF-8 + bidirectional script coverage — T048
// ============================================================================

#[cfg(feature = "output-html")]
#[test]
fn html_handles_cjk() {
    let grid = RenderGrid::from_text_rows(&[String::from("中文字符")]);
    let bytes = write_export(&grid, ExportFormat::Html).expect("html ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.contains("中"));
    assert!(s.contains("文"));
    assert!(s.contains("字"));
    assert!(s.contains("符"));
}

#[cfg(feature = "output-html")]
#[test]
fn html_handles_emoji() {
    let grid = RenderGrid::from_text_rows(&[String::from("🦀🚀✨")]);
    let bytes = write_export(&grid, ExportFormat::Html).expect("html ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.contains("🦀"));
    assert!(s.contains("🚀"));
    assert!(s.contains("✨"));
}

#[cfg(feature = "output-svg")]
#[test]
fn svg_handles_arabic_rtl() {
    // Arabic text (RTL script). The export backend does not rearrange
    // characters — it emits codepoints in logical order; the consuming
    // renderer (browser, image viewer) handles bidi shaping.
    let grid = RenderGrid::from_text_rows(&[String::from("مرحبا")]);
    let bytes = write_export(&grid, ExportFormat::Svg).expect("svg ok");
    let s = String::from_utf8(bytes).expect("utf-8");
    assert!(s.contains("مرحبا"));
}

#[cfg(feature = "output-irc")]
#[test]
fn irc_handles_hebrew_rtl() {
    let grid = RenderGrid::from_text_rows(&[String::from("שלום")]);
    let bytes = write_export(&grid, ExportFormat::Irc).expect("irc ok");
    // Convert back to UTF-8 string and verify Hebrew chars survive.
    // (Strip the color/reset bytes manually.)
    let s = String::from_utf8_lossy(&bytes);
    assert!(s.contains("ש"));
    assert!(s.contains("ל"));
    assert!(s.contains("ו"));
    assert!(s.contains("ם"));
}
