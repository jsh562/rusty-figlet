//! Color/rainbow output helpers per AD-011 + AD-012 + HINT-006.
//!
//! This module is CLI-feature-gated; the library API surface does not
//! expose colors directly.

use std::io;

use termcolor::WriteColor;

/// Tri-state `--color` flag value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorChoice {
    /// Color when stdout is a terminal AND NO_COLOR is unset.
    Auto,
    /// Color regardless of TTY status (NO_COLOR still suppresses).
    Always,
    /// Never emit color.
    Never,
}

/// Resolve whether to emit color given the user flag, the NO_COLOR
/// environment status, and the current TTY status of stdout.
///
/// NO_COLOR wins over `--color=always` per FR-032.
pub fn should_color(choice: ColorChoice, no_color_env: bool, is_tty: bool) -> bool {
    if no_color_env {
        return false;
    }
    match choice {
        ColorChoice::Auto => is_tty,
        ColorChoice::Always => true,
        ColorChoice::Never => false,
    }
}

/// Compute a per-column rainbow palette of `width` 24-bit colors per
/// HINT-006 (toilet `--gay` aesthetic).
///
/// Hue cycles `360.0 * (i / width)` across the full width; saturation
/// and value are fixed at 1.0 for visibility on dark terminals.
pub fn rainbow_palette(width: u32) -> Vec<anstyle::Color> {
    let w = width.max(1);
    (0..w)
        .map(|i| {
            let hue = 360.0_f32 * (i as f32 / w as f32);
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            anstyle::Color::from(anstyle::RgbColor(r, g, b))
        })
        .collect()
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_p = (h % 360.0) / 60.0;
    let x = c * (1.0 - (h_p % 2.0 - 1.0).abs());
    let m = v - c;
    let (r1, g1, b1) = match h_p as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let to_u8 = |f: f32| ((f + m) * 255.0).round().clamp(0.0, 255.0) as u8;
    (to_u8(r1), to_u8(g1), to_u8(b1))
}

/// Write `line` to `writer` painted per the supplied `palette`.
///
/// Each character is prefixed with its column's `\x1b[38;2;R;G;Bm`
/// sequence; SGR is reset at end of line.
pub fn write_rainbow_line<W: WriteColor>(
    line: &str,
    palette: &[anstyle::Color],
    writer: &mut W,
) -> io::Result<()> {
    for (i, ch) in line.chars().enumerate() {
        let color = palette.get(i).copied();
        let mut spec = termcolor::ColorSpec::new();
        if let Some(color) = color {
            spec.set_fg(Some(anstyle_to_termcolor(color)));
        }
        writer.set_color(&spec)?;
        write!(writer, "{ch}")?;
    }
    writer.reset()?;
    writeln!(writer)
}

fn anstyle_to_termcolor(color: anstyle::Color) -> termcolor::Color {
    match color {
        anstyle::Color::Rgb(rgb) => termcolor::Color::Rgb(rgb.0, rgb.1, rgb.2),
        anstyle::Color::Ansi256(idx) => termcolor::Color::Ansi256(idx.0),
        anstyle::Color::Ansi(_) => termcolor::Color::White,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_color_suppresses_always() {
        assert!(!should_color(ColorChoice::Always, true, true));
    }

    #[test]
    fn auto_off_when_not_tty() {
        assert!(!should_color(ColorChoice::Auto, false, false));
    }

    #[test]
    fn auto_on_when_tty() {
        assert!(should_color(ColorChoice::Auto, false, true));
    }

    #[test]
    fn never_always_off() {
        assert!(!should_color(ColorChoice::Never, false, true));
    }

    #[test]
    fn rainbow_palette_length_matches_width() {
        assert_eq!(rainbow_palette(7).len(), 7);
        // Zero width still yields one entry to avoid div-by-zero crashes.
        assert_eq!(rainbow_palette(0).len(), 1);
    }
}
