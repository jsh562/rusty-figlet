//! Helpers shared between the HTML and SVG export backends.
//!
//! Compiled whenever `output-html` OR `output-svg` is enabled. Lives in
//! a sibling module so neither backend forces the other's leaf on.
//!
//! ## Escape table (AD-004 + HINT-004)
//!
//! Only 4 metacharacters are escaped: `<` `>` `&` `"`. Single quotes are
//! NOT escaped because every attribute we emit is double-quoted. See
//! [`super::html`] module docs for the full rationale.
//!
//! ## Color conversion
//!
//! [`color_to_hex`] maps any [`crate::filter::Color`] variant to an
//! `#RRGGBB` string. Named-color RGB values are the VGA-text-mode
//! palette; 256-color indices follow the standard xterm cube + grayscale
//! ramp.

use crate::filter::{Color, NamedColor};

/// Hand-rolled 4-char XSS escape lookup (per AD-004).
#[inline]
const fn escape_lookup(c: char) -> Option<&'static str> {
    match c {
        '<' => Some("&lt;"),
        '>' => Some("&gt;"),
        '&' => Some("&amp;"),
        '"' => Some("&quot;"),
        _ => None,
    }
}

/// Push `s` into `out` with the 4-char HTML/XML escape applied.
///
/// Single-pass writer — every char is examined once and either pushed
/// verbatim or replaced with its escape. UTF-8 multibyte chars pass
/// through unchanged because none of the four escape targets is a
/// multibyte code unit.
pub(crate) fn escape_into(out: &mut String, s: &str) {
    for c in s.chars() {
        match escape_lookup(c) {
            Some(esc) => out.push_str(esc),
            None => out.push(c),
        }
    }
}

/// Convert a typed [`Color`] to a `#RRGGBB` hex string.
pub(crate) fn color_to_hex(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{r:02X}{g:02X}{b:02X}"),
        Color::Index(n) => format!("#{:06X}", index_to_rgb(n)),
        Color::Named(n) => format!("#{:06X}", named_to_rgb(n)),
    }
}

/// Map a 256-color palette index to an `RRGGBB` integer.
///
/// - Indices 0..16: ANSI named-color palette via [`named_to_rgb`].
/// - Indices 16..232: 6×6×6 color cube using component levels
///   {0, 95, 135, 175, 215, 255}.
/// - Indices 232..256: grayscale ramp `8 + 10*step`.
pub(crate) fn index_to_rgb(n: u8) -> u32 {
    if n < 16 {
        let named = match n {
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
        };
        named_to_rgb(named)
    } else if n < 232 {
        let c = n - 16;
        let r = c / 36;
        let g = (c / 6) % 6;
        let b = c % 6;
        let cube = |i: u8| -> u32 {
            match i {
                0 => 0,
                1 => 95,
                2 => 135,
                3 => 175,
                4 => 215,
                _ => 255,
            }
        };
        (cube(r) << 16) | (cube(g) << 8) | cube(b)
    } else {
        let step = (n - 232) as u32;
        let g = 8 + 10 * step;
        (g << 16) | (g << 8) | g
    }
}

/// Map an ANSI named color to an `RRGGBB` integer. Mirrors the common
/// VGA-text-mode palette that most terminal emulators use.
pub(crate) fn named_to_rgb(n: NamedColor) -> u32 {
    match n {
        NamedColor::Black => 0x000000,
        NamedColor::Red => 0x800000,
        NamedColor::Green => 0x008000,
        NamedColor::Yellow => 0x808000,
        NamedColor::Blue => 0x000080,
        NamedColor::Magenta => 0x800080,
        NamedColor::Cyan => 0x008080,
        NamedColor::White => 0xC0C0C0,
        NamedColor::BrightBlack => 0x808080,
        NamedColor::BrightRed => 0xFF0000,
        NamedColor::BrightGreen => 0x00FF00,
        NamedColor::BrightYellow => 0xFFFF00,
        NamedColor::BrightBlue => 0x0000FF,
        NamedColor::BrightMagenta => 0xFF00FF,
        NamedColor::BrightCyan => 0x00FFFF,
        NamedColor::BrightWhite => 0xFFFFFF,
    }
}
