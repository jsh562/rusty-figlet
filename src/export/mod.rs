//! Multi-format export backends (E012 US2 — FR-005, Phase 7).
//!
//! ## Backends
//!
//! Each backend is gated behind a distinct `output-<format>` Cargo leaf
//! per ADR-0006:
//!
//! | Format    | Leaf            | Module                      |
//! |-----------|-----------------|-----------------------------|
//! | HTML5     | `output-html`   | [`html`]                    |
//! | mIRC `^C` | `output-irc`    | [`irc`]                     |
//! | SVG 1.1   | `output-svg`    | [`svg`]                     |
//! | ANSI 16   | (always)        | (renders via [`crate::color_depth`]) |
//! | ANSI 256  | `color-256`     | (renders via [`crate::color_depth`]) |
//! | ANSI 24bit| `color-truecolor` | (renders via [`crate::color_depth`]) |
//!
//! ## Dispatch
//!
//! [`write_export`] receives a [`crate::filter::RenderGrid`] and an
//! [`ExportFormat`] and dispatches to the appropriate backend, returning
//! a `Vec<u8>` with the encoded bytes. When the requested format's leaf
//! is disabled at compile time, [`FigletError::UnsupportedExportFormat`]
//! is returned with the full list of available formats so the CLI can
//! produce a useful diagnostic per FR-016.
//!
//! ## Security
//!
//! - HTML/SVG: 4-char escape applied to every text-content and double-
//!   quoted-attribute byte per AD-004 + HINT-004 — see [`html`] module
//!   docs for the exact set + reasoning.
//! - IRC: ASCII C0/C1 non-printable bytes stripped per FR-015; UTF-8
//!   continuation bytes preserved per spec Edge Cases — see [`irc`]
//!   module docs.
//! - SVG: NO `<script>`, `<foreignObject>`, `xlink:href`, `<image href=...>`,
//!   `<use xlink:href=...>` emission. Inline `style="..."` IS emitted
//!   but contains only typed numeric `#RRGGBB` values — no user bytes
//!   ever flow into a `style` value per spec Edge Cases.

use crate::error::FigletError;
use crate::filter::RenderGrid;

#[cfg(feature = "output-html")]
pub mod html;

#[cfg(feature = "output-irc")]
pub mod irc;

#[cfg(feature = "output-svg")]
pub mod svg;

// Helpers shared between the HTML and SVG backends (both consume the
// 4-char escape table and the typed Color → #RRGGBB conversion).
// Compiled whenever either leaf is enabled.
#[cfg(any(feature = "output-html", feature = "output-svg"))]
mod common;

/// Supported export formats (E012 US2 — FR-005).
///
/// The enum is `#[non_exhaustive]` so additive variants (e.g., terminfo,
/// PNG via a future raster crate) remain non-breaking under SemVer.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportFormat {
    /// HTML5 `<pre><span style="color:#RRGGBB">...</span></pre>`.
    Html,
    /// mIRC `^C` color codes embedded inline with the text.
    Irc,
    /// SVG 1.1 `<text>` elements with `fill="#RRGGBB"`.
    Svg,
    /// ANSI 24-bit truecolor SGR (`\x1b[38;2;R;G;Bm`).
    AnsiTrue,
    /// ANSI 256-color SGR (`\x1b[38;5;Nm`).
    Ansi256,
    /// ANSI 16-color named SGR.
    Ansi16,
}

impl ExportFormat {
    /// Canonical lowercase name for CLI parsing / error diagnostics.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            ExportFormat::Html => "html",
            ExportFormat::Irc => "irc",
            ExportFormat::Svg => "svg",
            ExportFormat::AnsiTrue => "ansi-true",
            ExportFormat::Ansi256 => "ansi-256",
            ExportFormat::Ansi16 => "ansi-16",
        }
    }
}

/// Dispatch a [`RenderGrid`] to the requested [`ExportFormat`] backend.
///
/// Returns [`FigletError::UnsupportedExportFormat`] when the requested
/// format's leaf is disabled at compile time; the `available` field
/// enumerates the format names that ARE compiled into this build.
pub fn write_export(grid: &RenderGrid, fmt: ExportFormat) -> Result<Vec<u8>, FigletError> {
    // When all backend leaves are disabled the `grid` binding is unused.
    let _ = grid;
    match fmt {
        #[cfg(feature = "output-html")]
        ExportFormat::Html => Ok(html::write_html(grid).into_bytes()),
        #[cfg(not(feature = "output-html"))]
        ExportFormat::Html => Err(unsupported("html")),

        #[cfg(feature = "output-irc")]
        ExportFormat::Irc => Ok(irc::write_irc(grid, false)),
        #[cfg(not(feature = "output-irc"))]
        ExportFormat::Irc => Err(unsupported("irc")),

        #[cfg(feature = "output-svg")]
        ExportFormat::Svg => Ok(svg::write_svg(grid).into_bytes()),
        #[cfg(not(feature = "output-svg"))]
        ExportFormat::Svg => Err(unsupported("svg")),

        // ANSI dispatches are gated by the existing color depth leaves;
        // implementation lives in `crate::color_depth`. For Phase 7 we
        // expose only HTML/IRC/SVG. ANSI exports are wired via the
        // existing `output` module in Phase 9 (T061).
        ExportFormat::AnsiTrue | ExportFormat::Ansi256 | ExportFormat::Ansi16 => {
            Err(unsupported(fmt.name()))
        }
    }
}

/// Construct a [`FigletError::UnsupportedExportFormat`] populated with
/// the list of formats whose leaves are enabled in this build.
fn unsupported(requested: &str) -> FigletError {
    let available: Vec<String> = available_format_names();
    FigletError::UnsupportedExportFormat {
        requested: requested.to_owned(),
        available,
    }
}

/// Build the list of export format names whose leaves are enabled in this
/// build. Used by `unsupported` to populate the diagnostic. Split into a
/// separate function so the per-leaf cfg branches don't trigger the
/// `vec_init_then_push` lint (different leaves yield different lengths).
fn available_format_names() -> Vec<String> {
    #[allow(unused_mut)]
    let mut v: Vec<String> = Vec::with_capacity(3);
    #[cfg(feature = "output-html")]
    {
        v.push("html".to_owned());
    }
    #[cfg(feature = "output-irc")]
    {
        v.push("irc".to_owned());
    }
    #[cfg(feature = "output-svg")]
    {
        v.push("svg".to_owned());
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_format_names_are_canonical() {
        assert_eq!(ExportFormat::Html.name(), "html");
        assert_eq!(ExportFormat::Irc.name(), "irc");
        assert_eq!(ExportFormat::Svg.name(), "svg");
    }

    #[test]
    fn dispatch_returns_unsupported_for_ansi_in_phase7() {
        let grid = RenderGrid::blank(1, 1);
        let err = write_export(&grid, ExportFormat::AnsiTrue).unwrap_err();
        match err {
            FigletError::UnsupportedExportFormat { requested, .. } => {
                assert_eq!(requested, "ansi-true");
            }
            other => panic!("expected UnsupportedExportFormat, got {other:?}"),
        }
    }
}
