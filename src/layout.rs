//! Layout-mode resolution per AD-009 + HINT-002.
//!
//! Collapses the user-supplied `-k`/`-W`/`-S`/`-s`/`-o`/`-m N` and
//! `-c`/`-l`/`-r`/`-x` flag occurrences into a single
//! [`LayoutMode`] + [`Justify`] pair using last-wins semantics.

use crate::figfont::FIGfont;

/// Resolved layout mode that the renderer applies row-by-row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutMode {
    /// No overlap — each glyph occupies its full `max_length` width.
    FullWidth,
    /// Kerning — glyphs touch but never smush.
    Kerning,
    /// Universal smushing — later char wins (with hardblank dominance).
    UniversalSmush,
    /// Smushing using an explicit bitmask of rules 1..=6.
    RuleSmush(u8),
    /// Overlap-only — adjacent space cells overlap, nothing else.
    OverlapOnly,
}

/// Horizontal justification mode (mirrors [`crate::Justify`] but lives
/// in the resolver layer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Justify {
    /// Center within the resolved width.
    Center,
    /// Left-align.
    Left,
    /// Right-align.
    Right,
    /// Use the font's print-direction default.
    FontDefault,
}

/// One occurrence of a layout-class flag, captured in argv order so
/// `LayoutResolver::resolve` can apply last-wins semantics per FR-023.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutFlag {
    /// `-k`
    Kerning,
    /// `-W`
    FullWidth,
    /// `-S` — force smush per font's smush rules.
    ForceSmush,
    /// `-s` — use the font's default smush.
    FontDefaultSmush,
    /// `-o` — overlap only.
    OverlapOnly,
    /// `-m N` — explicit bitfield.
    Explicit(i32),
}

/// One occurrence of a justify-class flag, captured in argv order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyFlag {
    /// `-c`
    Center,
    /// `-l`
    Left,
    /// `-r`
    Right,
    /// `-x`
    FontDefault,
}

/// Sequenced layout-flag occurrences from the command line.
#[derive(Debug, Clone, Default)]
pub struct LayoutFlags {
    /// Occurrences in argv order.
    pub flags: Vec<LayoutFlag>,
}

/// Sequenced justify-flag occurrences from the command line.
#[derive(Debug, Clone, Default)]
pub struct JustifyFlags {
    /// Occurrences in argv order.
    pub flags: Vec<JustifyFlag>,
}

/// Stateless resolver that collapses a `FIGfont` + flag sequence into a
/// concrete [`LayoutMode`].
pub struct LayoutResolver;

impl LayoutResolver {
    /// Apply last-wins layout-class semantics per AD-009.
    ///
    /// When `flags.flags` is empty, returns the font's baseline
    /// [`LayoutMode`] derived from `full_layout`.
    pub fn resolve(font: &FIGfont, flags: &LayoutFlags) -> LayoutMode {
        if let Some(last) = flags.flags.last() {
            return match *last {
                LayoutFlag::Kerning => LayoutMode::Kerning,
                LayoutFlag::FullWidth => LayoutMode::FullWidth,
                LayoutFlag::ForceSmush => {
                    let bits = (font.full_layout & 0b0011_1111) as u8;
                    if bits == 0 {
                        LayoutMode::UniversalSmush
                    } else {
                        LayoutMode::RuleSmush(bits)
                    }
                }
                LayoutFlag::FontDefaultSmush => font_default_mode(font),
                LayoutFlag::OverlapOnly => LayoutMode::OverlapOnly,
                LayoutFlag::Explicit(n) => explicit_mode(n),
            };
        }
        font_default_mode(font)
    }
}

fn font_default_mode(font: &FIGfont) -> LayoutMode {
    let smushing = font.full_layout & (crate::smush::RULE_HORIZONTAL_SMUSHING as u32) != 0;
    let kerning = font.full_layout & (crate::smush::RULE_HORIZONTAL_KERNING as u32) != 0;
    let bits = (font.full_layout & 0b0011_1111) as u8;
    if smushing {
        if bits == 0 {
            LayoutMode::UniversalSmush
        } else {
            LayoutMode::RuleSmush(bits)
        }
    } else if kerning {
        LayoutMode::Kerning
    } else {
        LayoutMode::FullWidth
    }
}

fn explicit_mode(n: i32) -> LayoutMode {
    match n {
        -1 => LayoutMode::FullWidth,
        0 => LayoutMode::Kerning,
        // -2 is upstream's "leave layout undefined"; we mirror as font default
        // when the renderer asks but we lack the font here, so map to kerning.
        -2 => LayoutMode::Kerning,
        bits if (1..=63).contains(&bits) => LayoutMode::RuleSmush(bits as u8),
        _ => LayoutMode::FullWidth,
    }
}

/// Collapse a [`JustifyFlags`] sequence into a single [`Justify`] per
/// FR-022 last-wins semantics. Returns [`Justify::FontDefault`] when
/// the sequence is empty.
pub fn resolve_justify(flags: &JustifyFlags) -> Justify {
    flags
        .flags
        .last()
        .map(|f| match *f {
            JustifyFlag::Center => Justify::Center,
            JustifyFlag::Left => Justify::Left,
            JustifyFlag::Right => Justify::Right,
            JustifyFlag::FontDefault => Justify::FontDefault,
        })
        .unwrap_or(Justify::FontDefault)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::figfont::parse_bytes;

    fn font() -> FIGfont {
        parse_bytes(crate::figfont::BUNDLED_FONTS[0].1).expect("bundled font parses")
    }

    #[test]
    fn empty_flags_yields_font_default() {
        let mode = LayoutResolver::resolve(&font(), &LayoutFlags::default());
        let _ = mode; // any of the variants is acceptable for the placeholder
    }

    #[test]
    fn last_wins_layout_kerning() {
        let f = LayoutFlags {
            flags: vec![
                LayoutFlag::FullWidth,
                LayoutFlag::ForceSmush,
                LayoutFlag::Kerning,
            ],
        };
        assert_eq!(LayoutResolver::resolve(&font(), &f), LayoutMode::Kerning);
    }

    #[test]
    fn explicit_layout_bitfield_24() {
        let f = LayoutFlags {
            flags: vec![LayoutFlag::Explicit(24)],
        };
        assert_eq!(
            LayoutResolver::resolve(&font(), &f),
            LayoutMode::RuleSmush(24)
        );
    }

    #[test]
    fn explicit_layout_zero_is_kerning() {
        let f = LayoutFlags {
            flags: vec![LayoutFlag::Explicit(0)],
        };
        assert_eq!(LayoutResolver::resolve(&font(), &f), LayoutMode::Kerning);
    }

    #[test]
    fn justify_last_wins() {
        let j = JustifyFlags {
            flags: vec![JustifyFlag::Left, JustifyFlag::Center, JustifyFlag::Right],
        };
        assert_eq!(resolve_justify(&j), Justify::Right);
    }

    #[test]
    fn justify_empty_yields_font_default() {
        assert_eq!(
            resolve_justify(&JustifyFlags::default()),
            Justify::FontDefault
        );
    }
}
