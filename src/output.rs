//! CLI banner-writer per AD-011.
//!
//! Drives the [`crate::Banner`] lazy line iterator and emits each
//! rendered line to a [`termcolor::WriteColor`] sink. When a rainbow
//! palette is supplied, per-character coloring is delegated to
//! [`crate::color::write_rainbow_line`].

use std::io;

use termcolor::WriteColor;

use crate::Banner;

/// Optional per-banner color configuration.
pub struct ColorConfig {
    /// Pre-computed rainbow palette covering the widest banner line.
    pub rainbow_palette: Option<Vec<anstyle::Color>>,
}

/// Stream `banner` to `writer`, optionally painted per `color`.
///
/// The banner iterator is driven lazily — no whole-banner buffering.
pub fn write_banner<W: WriteColor>(
    banner: &Banner,
    color: Option<&ColorConfig>,
    writer: &mut W,
) -> io::Result<()> {
    for line in banner.lines() {
        match color {
            Some(cfg) if cfg.rainbow_palette.is_some() => {
                let palette = cfg
                    .rainbow_palette
                    .as_deref()
                    .expect("just-checked is_some");
                crate::color::write_rainbow_line(&line, palette, writer)?;
            }
            _ => {
                writeln!(writer, "{line}")?;
            }
        }
    }
    Ok(())
}
