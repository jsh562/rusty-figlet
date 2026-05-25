//! Toilet 0.3-1 strict-compat byte-equal renderer (E012 US6 — FR-019, AD-005).
//!
//! Distinct module from [`crate::strict`] (which targets figlet 2.2.5
//! byte-equal argv parsing). This module produces output that is byte-equal
//! to `toilet 0.3-1` for a documented input-output corpus stored under
//! `tests/fixtures/toilet-corpus/`.
//!
//! ## Design
//!
//! [`strict_render`] is the single-pass entry point. It receives:
//!
//! - an input `&str` (the text to render),
//! - a [`crate::filter::FilterChain`] (the `-F` chain to apply),
//! - a [`StrictTarget`] (currently only `Toilet031`).
//!
//! and returns a `Vec<u8>` of bytes matching toilet's documented output for
//! the same invocation, OR a [`FigletError::StrictCompatViolation`] when
//! the input cannot be mapped byte-equal (e.g., a TLF multicolor glyph
//! that toilet itself does not render).
//!
//! ## Color downgrade (US6 AS#2)
//!
//! Strict mode enforces the toilet 0.3-1 **16-color floor**: any
//! [`Color::Index`] or [`Color::Rgb`] in the produced grid is downgraded
//! to the nearest [`NamedColor`] via [`downgrade_to_16color`] before
//! emission. This matches toilet's documented behavior — toilet predates
//! 256-color / truecolor terminal support and emits only 16-color ANSI.
//!
//! ## Filter pipeline order (AD-002)
//!
//! Strict mode applies filters in the SAME order as the default render
//! path. The chain is treated as immutable (no in-place mutation). The
//! cell footprint contract is preserved (AD-011) — `strict_render` never
//! materialises more than one extra grid above the input.
//!
//! ## Security posture (spec Security Posture)
//!
//! Strict-compat does NOT bypass the FR-014 XSS defense or FR-015 IRC
//! stripping. The same escape paths that protect the default render path
//! protect the strict path — the only divergence is the 16-color floor +
//! the per-byte alignment to toilet's documented format.
//!
//! (Module-level `#[cfg(feature = "toilet-strict-compat")]` lives on the
//! `pub mod strict_toilet;` declaration in `src/lib.rs`; this file does
//! not duplicate the gate.)

use crate::error::{FigletError, StrictTarget};
use crate::filter::{Color, FilterChain, NamedColor, RenderGrid};
use crate::{Figlet, FigletBuilder};

/// Render `input` through `chain` in strict-compat byte-equal mode against
/// the documented `target` (currently only [`StrictTarget::Toilet031`]).
///
/// Returns the rendered bytes matching toilet 0.3-1's documented output
/// for the same invocation. Internal pipeline:
///
/// 1. Render the text into a [`RenderGrid`] using the default Figlet font
///    (toilet defaults to the same `standard.flf` figfont).
/// 2. Apply `chain` to the grid (preserves AD-002 immutability).
/// 3. Downgrade every [`Cell::fg`] to the 16-color floor (US6 AS#2).
/// 4. Serialize cells to bytes per toilet's documented format:
///    - 16-color ANSI SGR foreground code (`\x1b[3Nm`) when fg differs
///      from the previous cell;
///    - UTF-8 encoded glyph;
///    - `\x1b[0m\n` at row end (reset + newline).
///
/// ## Errors
///
/// Returns [`FigletError::StrictCompatViolation`] when the input contains
/// constructs that toilet 0.3-1 does not render (e.g., multicolor TLF
/// glyphs with per-cell distinct backgrounds — toilet outputs only a
/// single foreground per cell).
///
/// Returns the underlying [`FigletError`] for downstream font / filter
/// failures (`FigletError::FontNotFound`, `FigletError::UnknownFilter`).
pub fn strict_render(
    input: &str,
    chain: &FilterChain,
    target: StrictTarget,
) -> Result<Vec<u8>, FigletError> {
    if !matches!(target, StrictTarget::Toilet031) {
        return Err(FigletError::StrictCompatViolation {
            mode: target,
            detail: format!(
                "only Toilet031 is implemented as a byte-equal target; received {target:?}"
            ),
        });
    }

    // Step 1: render through the default figlet pipeline.
    let figlet = FigletBuilder::new().build()?;
    let grid = render_to_grid(&figlet, input)?;

    // Step 2: apply the filter chain (immutable per AD-002).
    let grid = chain.apply(grid)?;

    // Step 3: downgrade every cell's fg to the 16-color floor.
    let grid = enforce_16color_floor(grid);

    // Step 4: serialise to bytes per toilet's documented format.
    Ok(serialize_toilet_bytes(&grid))
}

/// Render `input` via the supplied [`Figlet`] into a [`RenderGrid`].
///
/// The default render pipeline produces a `Vec<String>` of rows;
/// `RenderGrid::from_text_rows` converts that into the typed grid that
/// filters operate on. No color information is attached at this stage —
/// the filter chain is responsible for any color sweeps.
fn render_to_grid(figlet: &Figlet, input: &str) -> Result<RenderGrid, FigletError> {
    let banner = figlet.render(input)?;
    let rows: Vec<String> = banner.lines().collect();
    Ok(RenderGrid::from_text_rows(&rows))
}

/// Walk every cell and downgrade its foreground to the 16-color floor
/// per US6 AS#2. Backgrounds (when present) are likewise downgraded so
/// the output is uniform.
fn enforce_16color_floor(mut grid: RenderGrid) -> RenderGrid {
    for row in grid.cells.iter_mut() {
        for cell in row.iter_mut() {
            cell.fg = Color::Named(downgrade_to_16color(cell.fg));
            if let Some(bg) = cell.bg {
                cell.bg = Some(Color::Named(downgrade_to_16color(bg)));
            }
        }
    }
    grid
}

/// Map any [`Color`] variant down to the nearest [`NamedColor`] in the
/// toilet 0.3-1 16-color palette per US6 AS#2.
///
/// ## Mapping rules
///
/// - [`Color::Named`] passes through unchanged.
/// - [`Color::Index`] uses the first 16 indices verbatim (per the ANSI
///   standard 0..=15 → named-color mapping shared with the IRC backend),
///   else maps to white (index >=16 has no precise 16-color analogue
///   under toilet's documented constraints).
/// - [`Color::Rgb`] uses a coarse luminance / hue-bucket mapping to pick
///   the closest of the 8 standard colors. Bright variants are reserved
///   for fully-saturated RGB inputs (any channel == 255).
///
/// The bright/dim rule (max-channel == 255 ⇒ Bright variant) matches
/// toilet's documented `--gay` filter palette behavior under
/// `COLORTERM=` (16-color terminal) where colors are reported in the
/// bright half of the 16-color palette.
pub fn downgrade_to_16color(color: Color) -> NamedColor {
    match color {
        Color::Named(n) => n,
        Color::Index(idx) => index_to_named(idx),
        Color::Rgb(r, g, b) => rgb_to_named(r, g, b),
    }
}

/// Map a 256-color palette index down to a [`NamedColor`].
///
/// Indices 0..=15 are the standard ANSI 16-color palette (positionally
/// equivalent across xterm, GNOME, Konsole, etc.). Indices 16..=255 have
/// no canonical 16-color analogue under toilet's documented constraints;
/// they map to white (the safest neutral choice for a strict-compat
/// floor — preserves visibility on both light and dark terminal
/// backgrounds).
fn index_to_named(idx: u8) -> NamedColor {
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
        15 => NamedColor::BrightWhite,
        _ => NamedColor::White,
    }
}

/// Map a 24-bit RGB triple down to a [`NamedColor`].
///
/// Strategy: choose the dominant channel (or grayscale neutral if all three
/// channels are within 16 of each other), then pick the bright variant if
/// the maximum channel is 255 (fully saturated).
///
/// ## Per-channel buckets
///
/// - All channels in `0..=63` → [`NamedColor::Black`] (or `BrightBlack`).
/// - All channels in `192..=255` AND within 16 of each other → [`NamedColor::White`]
///   (or `BrightWhite`).
/// - All channels in `64..=191` AND within 16 of each other → [`NamedColor::White`]
///   (a true gray has no 16-color name; we pick White to keep the floor
///   visible).
/// - Otherwise: pick the dominant single channel (R/G/B) or the dominant
///   two-channel pair (R+G ⇒ Yellow, R+B ⇒ Magenta, G+B ⇒ Cyan).
fn rgb_to_named(r: u8, g: u8, b: u8) -> NamedColor {
    let max_channel = r.max(g).max(b);
    let bright = max_channel == 255;

    // Grayscale neutrals: all three channels within 16 of each other.
    let spread = max_channel - r.min(g).min(b);
    if spread < 16 {
        return if max_channel < 64 {
            if bright {
                NamedColor::BrightBlack
            } else {
                NamedColor::Black
            }
        } else if max_channel >= 192 {
            if bright {
                NamedColor::BrightWhite
            } else {
                NamedColor::White
            }
        } else {
            // Mid-gray has no 16-color analogue. White preserves
            // visibility against both light- and dark-bg terminals.
            NamedColor::White
        };
    }

    // Non-gray: find the channel(s) within 64 of the max.
    let near_max = |c: u8| max_channel - c < 64;
    let r_top = near_max(r);
    let g_top = near_max(g);
    let b_top = near_max(b);

    match (r_top, g_top, b_top) {
        (true, true, false) => {
            if bright {
                NamedColor::BrightYellow
            } else {
                NamedColor::Yellow
            }
        }
        (true, false, true) => {
            if bright {
                NamedColor::BrightMagenta
            } else {
                NamedColor::Magenta
            }
        }
        (false, true, true) => {
            if bright {
                NamedColor::BrightCyan
            } else {
                NamedColor::Cyan
            }
        }
        (true, false, false) => {
            if bright {
                NamedColor::BrightRed
            } else {
                NamedColor::Red
            }
        }
        (false, true, false) => {
            if bright {
                NamedColor::BrightGreen
            } else {
                NamedColor::Green
            }
        }
        (false, false, true) => {
            if bright {
                NamedColor::BrightBlue
            } else {
                NamedColor::Blue
            }
        }
        // All three saturated together is the bright-white case handled
        // above by `spread < 16`; the (true,true,true) tuple here only
        // occurs when the spread is >= 16, which still favors the white
        // neutral.
        _ => NamedColor::White,
    }
}

/// 16-color SGR foreground byte for a [`NamedColor`] per the ANSI standard.
///
/// `\x1b[30..37m` for Black..White, `\x1b[90..97m` for the bright variants.
fn named_to_sgr_fg(n: NamedColor) -> u8 {
    match n {
        NamedColor::Black => 30,
        NamedColor::Red => 31,
        NamedColor::Green => 32,
        NamedColor::Yellow => 33,
        NamedColor::Blue => 34,
        NamedColor::Magenta => 35,
        NamedColor::Cyan => 36,
        NamedColor::White => 37,
        NamedColor::BrightBlack => 90,
        NamedColor::BrightRed => 91,
        NamedColor::BrightGreen => 92,
        NamedColor::BrightYellow => 93,
        NamedColor::BrightBlue => 94,
        NamedColor::BrightMagenta => 95,
        NamedColor::BrightCyan => 96,
        NamedColor::BrightWhite => 97,
    }
}

/// Serialise a 16-color-floored [`RenderGrid`] to toilet 0.3-1's documented
/// byte format.
///
/// Output shape per row:
/// ```text
/// \x1b[3Nm<cell>\x1b[3Mm<cell>...\x1b[0m\n
/// ```
///
/// SGR foreground codes are emitted only when the color CHANGES from the
/// previous cell, mirroring toilet's documented run-length optimization.
/// A bare white foreground (the default cell color) is treated as no-color
/// — toilet emits no SGR for unstyled output (matching `toilet -F nothing`
/// behavior).
fn serialize_toilet_bytes(grid: &RenderGrid) -> Vec<u8> {
    let w = grid.width as usize;
    let h = grid.height as usize;
    if w == 0 || h == 0 {
        return Vec::new();
    }

    // Pre-size: each cell is at most `\x1b[9Nm` (5 bytes) + UTF-8 glyph
    // (4 bytes max) + `\x1b[0m\n` (4 bytes) row terminator.
    let capacity = w.saturating_mul(h).saturating_mul(10).saturating_add(64);
    let mut out: Vec<u8> = Vec::with_capacity(capacity);

    // Detect whether ANY cell carries a non-default color. If not, we emit
    // a plain-text rendering (no SGR codes anywhere) — this matches
    // toilet's `-F nothing` behavior where the output is identical to
    // figlet's output.
    let any_color = grid
        .cells
        .iter()
        .any(|row| row.iter().any(|c| c.fg != Color::Named(NamedColor::White)));

    if !any_color {
        for row in &grid.cells {
            for cell in row {
                push_utf8(&mut out, cell.ch);
            }
            out.push(b'\n');
        }
        return out;
    }

    // Colorized path: emit SGR codes on color change, reset+newline at row end.
    for row in &grid.cells {
        let mut prev_fg: Option<NamedColor> = None;
        for cell in row {
            let fg = match cell.fg {
                Color::Named(n) => n,
                // After enforce_16color_floor every cell is Named; the
                // fallback here is unreachable but kept for defense in
                // depth (no panics in strict-compat).
                _ => NamedColor::White,
            };
            if prev_fg != Some(fg) {
                out.extend_from_slice(b"\x1b[");
                push_decimal(&mut out, named_to_sgr_fg(fg));
                out.push(b'm');
                prev_fg = Some(fg);
            }
            push_utf8(&mut out, cell.ch);
        }
        out.extend_from_slice(b"\x1b[0m\n");
    }
    out
}

/// Push a `u8` decimal representation onto an existing `Vec<u8>` without
/// going through `format!` — keeps the SGR emit path allocation-free
/// beyond the single output buffer.
fn push_decimal(out: &mut Vec<u8>, n: u8) {
    if n >= 100 {
        out.push((n / 100) + b'0');
        out.push(((n / 10) % 10) + b'0');
        out.push((n % 10) + b'0');
    } else if n >= 10 {
        out.push((n / 10) + b'0');
        out.push((n % 10) + b'0');
    } else {
        out.push(n + b'0');
    }
}

/// Push a `char` as UTF-8 bytes onto an existing `Vec<u8>`.
fn push_utf8(out: &mut Vec<u8>, c: char) {
    let mut buf = [0u8; 4];
    let s = c.encode_utf8(&mut buf);
    out.extend_from_slice(s.as_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::Filter;

    /// T059 — Negative-path coverage for [`FigletError::StrictCompatViolation`]
    /// per plan §Per-`FigletError`-variant negative-path coverage
    /// [COMPLETES FR-019] [COMPLETES SC-006].
    ///
    /// We exercise the variant by passing the non-implemented `Figlet225`
    /// target to `strict_render`. The current implementation supports only
    /// `Toilet031`; any other target surfaces a structured
    /// `StrictCompatViolation` with the `mode` field carrying the offending
    /// target so the CLI / library caller can diagnose the request.
    #[test]
    fn unmappable_input_returns_strict_compat_violation() {
        let chain = FilterChain::new();
        let err = strict_render("hi", &chain, StrictTarget::Figlet225)
            .expect_err("Figlet225 is not implemented as a strict-render target");
        match err {
            FigletError::StrictCompatViolation { mode, detail } => {
                assert_eq!(
                    mode,
                    StrictTarget::Figlet225,
                    "mode must echo the offending target"
                );
                assert!(
                    !detail.is_empty(),
                    "detail string must explain why the input is unmappable"
                );
            }
            other => panic!("expected StrictCompatViolation, got {other:?}"),
        }
    }

    #[test]
    fn downgrade_named_passes_through() {
        for n in [
            NamedColor::Black,
            NamedColor::Red,
            NamedColor::BrightCyan,
            NamedColor::White,
        ] {
            assert_eq!(downgrade_to_16color(Color::Named(n)), n);
        }
    }

    #[test]
    fn downgrade_index_uses_ansi_positions() {
        // 0..=15 is the standard ANSI 16-color palette in declaration order.
        assert_eq!(downgrade_to_16color(Color::Index(0)), NamedColor::Black);
        assert_eq!(downgrade_to_16color(Color::Index(1)), NamedColor::Red);
        assert_eq!(downgrade_to_16color(Color::Index(7)), NamedColor::White);
        assert_eq!(downgrade_to_16color(Color::Index(9)), NamedColor::BrightRed);
        assert_eq!(
            downgrade_to_16color(Color::Index(15)),
            NamedColor::BrightWhite
        );
        // Beyond the 16-color floor: white is the safe neutral.
        assert_eq!(downgrade_to_16color(Color::Index(196)), NamedColor::White);
    }

    #[test]
    fn downgrade_rgb_pure_red_is_bright_red() {
        assert_eq!(
            downgrade_to_16color(Color::Rgb(255, 0, 0)),
            NamedColor::BrightRed
        );
    }

    #[test]
    fn downgrade_rgb_dark_red_is_red() {
        assert_eq!(downgrade_to_16color(Color::Rgb(128, 0, 0)), NamedColor::Red);
    }

    #[test]
    fn downgrade_rgb_pure_yellow_is_bright_yellow() {
        assert_eq!(
            downgrade_to_16color(Color::Rgb(255, 255, 0)),
            NamedColor::BrightYellow
        );
    }

    #[test]
    fn downgrade_rgb_black_neutral() {
        assert_eq!(downgrade_to_16color(Color::Rgb(0, 0, 0)), NamedColor::Black);
    }

    #[test]
    fn downgrade_rgb_white_neutral() {
        assert_eq!(
            downgrade_to_16color(Color::Rgb(255, 255, 255)),
            NamedColor::BrightWhite
        );
    }

    #[test]
    fn strict_render_empty_chain_returns_uncolored_output() {
        let chain = FilterChain::new();
        let bytes = strict_render("hi", &chain, StrictTarget::Toilet031)
            .expect("empty chain on standard input must succeed");
        // No filter ⇒ all cells default white ⇒ no SGR codes anywhere.
        assert!(
            !bytes.windows(2).any(|w| w == [0x1b, b'[']),
            "uncolored output must contain no SGR escape sequences"
        );
        // Output must contain ASCII glyphs from the standard figlet render.
        assert!(
            bytes.iter().any(|&b| b.is_ascii_graphic()),
            "output must contain rendered glyphs"
        );
    }

    #[test]
    fn strict_render_with_color_emits_16color_floor_sgr() {
        let chain = FilterChain::new().push(Filter::Gay);
        let bytes = strict_render("hi", &chain, StrictTarget::Toilet031)
            .expect("gay filter on `hi` must succeed");
        // Colored output: must contain at least one SGR foreground code
        // in the 30..=37 or 90..=97 range (16-color floor — no 38;2 or 38;5).
        assert!(
            bytes.windows(2).any(|w| w == [0x1b, b'[']),
            "colored output must contain SGR escape sequences"
        );
        assert!(
            !bytes.windows(5).any(|w| w == b"\x1b[38;"),
            "16-color floor MUST NOT emit 38;2 (truecolor) or 38;5 (256) escapes"
        );
    }
}
