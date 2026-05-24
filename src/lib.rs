//! # rusty-figlet
//!
//! Rust port of cmatsuoka's `figlet(6)` v2.2.5 with an in-house FIGfont 2.0
//! parser, all six horizontal smush rules + universal, 12 bundled `.flf`
//! fonts via `include_bytes!`, terminal-width-aware layout, color/rainbow
//! output, byte-equal Strict-mode upstream compatibility, and a typed
//! library API.
//!
//! ## Library API quick tour
//!
//! ```rust
//! use rusty_figlet::{FigletBuilder, Font};
//!
//! let banner = FigletBuilder::new()
//!     .font(Font::Standard)
//!     .width(80)
//!     .build()
//!     .expect("build")
//!     .render("Hello")
//!     .expect("render");
//!
//! for line in banner.lines() {
//!     println!("{line}");
//! }
//! ```
//!
//! ## Default features
//!
//! `default = ["cli"]` enables the CLI binary surface (clap, clap_complete,
//! anstyle, termcolor, terminal_size). Library consumers should depend on
//! `rusty-figlet` with `default-features = false` to strip every CLI-only
//! dep so only `thiserror` and the in-house FIGfont parser are pulled in
//! (per FR-051 + AD-001).
//!
//! ## Error handling
//!
//! [`FigletError`] is `#[non_exhaustive]`; downstream pattern matches MUST
//! include a wildcard `_` arm (per AD-013).

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

use std::path::PathBuf;
use std::sync::OnceLock;

mod error;
pub use error::FigletError;

// The cross-cutting modules below are foundational scaffolds (Phase 2).
// Each one's public surface is consumed by US1..US7 in later phases;
// until those wires land, individual symbols look unused to clippy.
// Module-level allow(dead_code) keeps the foundation green without
// polluting individual definitions.
#[allow(dead_code)]
mod figfont;
#[allow(dead_code)]
mod layout;
#[allow(dead_code)]
mod mode;
#[allow(dead_code)]
mod smush;

pub use layout::{JustifyFlag, JustifyFlags, LayoutFlag, LayoutFlags};
/// Hand-rolled Strict-mode argv parser (AD-007). Public so the
/// `rusty-figlet` binary can dispatch to its byte-equal upstream
/// diagnostics; the SemVer policy on this module's surface matches the
/// rest of the public library API per FR-050.
#[allow(dead_code)]
pub mod strict;

#[cfg(feature = "cli")]
#[allow(dead_code)]
mod cli;
/// CLI-gated color/rainbow helpers (per AD-011 + AD-012 + HINT-006).
///
/// Exposed publicly for the `rusty-figlet` binary to consume; library
/// callers SHOULD NOT depend on this module directly (it lives under the
/// `cli` feature and is subject to change without a major version bump
/// per FR-051 + AD-001).
#[cfg(feature = "cli")]
#[doc(hidden)]
#[allow(dead_code)]
pub mod color;
/// CLI-gated banner writer (per AD-011).
///
/// Exposed publicly for the `rusty-figlet` binary to consume; library
/// callers SHOULD NOT depend on this module directly.
#[cfg(feature = "cli")]
#[doc(hidden)]
#[allow(dead_code)]
pub mod output;
#[cfg(feature = "cli")]
#[allow(dead_code)]
mod width;

/// Re-export of [`width::resolve_width`] for the rusty-figlet binary's
/// CLI wiring path (T106 + T109). Library consumers that need to
/// resolve a width budget under the same precedence ladder may call
/// this helper directly. CLI-gated because the underlying lookup
/// depends on `terminal_size`.
#[cfg(feature = "cli")]
pub fn resolve_width_for(
    explicit_w: Option<u32>,
    use_t: bool,
    columns_env: Option<u32>,
    is_tty: bool,
    mode: CompatibilityMode,
) -> u32 {
    width::resolve_width(explicit_w, use_t, columns_env, is_tty, mode)
}

/// Re-export of [`layout::resolve_justify`] for the rusty-figlet binary's
/// CLI wiring path (T103 + T109). Translates a sequence of
/// [`JustifyFlag`] occurrences into the resolved [`Justify`] value via
/// last-wins semantics per FR-022.
pub fn resolve_justify_for(flags: &JustifyFlags) -> Justify {
    match layout::resolve_justify(flags) {
        layout::Justify::Center => Justify::Center,
        layout::Justify::Left => Justify::Left,
        layout::Justify::Right => Justify::Right,
        layout::Justify::FontDefault => Justify::FontDefault,
    }
}

/// Compatibility mode that governs argv parsing + rendering rules.
///
/// In `Default` mode the CLI behaves like a modern Rust-native tool
/// (UTF-8 input, color flags accepted, ergonomic clap diagnostics). In
/// `Strict` mode the binary mirrors upstream `figlet 2.2.5` byte-for-byte
/// (Latin-1 clamped input, color flags rejected, hand-rolled getopt-style
/// diagnostics) so existing shell scripts that target upstream `figlet`
/// run unmodified.
///
/// Marked `#[non_exhaustive]` so future modes (e.g. `Toilet`) remain a
/// non-breaking addition.
///
/// ```rust
/// use rusty_figlet::CompatibilityMode;
///
/// let mode = CompatibilityMode::default();
/// assert_eq!(mode, CompatibilityMode::Default);
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CompatibilityMode {
    /// Modern, Rust-native behavior (UTF-8 input, color enabled, ergonomic
    /// diagnostics).
    Default,
    /// Byte-equal upstream `figlet 2.2.5` behavior (Latin-1 input, color
    /// flags rejected, getopt-style diagnostics).
    Strict,
}

impl Default for CompatibilityMode {
    fn default() -> Self {
        Self::Default
    }
}

/// Bundled-font selector and external-file escape hatch.
///
/// The 12 named variants correspond one-to-one to the bundled `.flf`
/// assets shipped under `assets/fonts/` (AD-016 + FR-011). The
/// [`Font::External`] variant covers `-f <path>` and `-d <dir>` resolution
/// paths for user-supplied `.flf` files.
///
/// The enum is intentionally exhaustive: the bundled set is pinned for
/// v0.1.0 SemVer. Adding a 13th bundled font would be a breaking change
/// requiring a major bump.
///
/// ```rust
/// use rusty_figlet::{FigletBuilder, Font};
///
/// // Pick one of the 12 bundled fonts.
/// let _ = FigletBuilder::new().font(Font::Slant);
///
/// // Or load from disk via the External variant.
/// let _ = FigletBuilder::new().font(Font::External("/tmp/my.flf".into()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Font {
    /// `standard.flf` — the default FIGfont, used when no `-f` flag is set.
    Standard,
    /// `slant.flf`
    Slant,
    /// `small.flf`
    Small,
    /// `big.flf`
    Big,
    /// `mini.flf`
    Mini,
    /// `banner.flf`
    Banner,
    /// `block.flf`
    Block,
    /// `bubble.flf`
    Bubble,
    /// `digital.flf`
    Digital,
    /// `lean.flf`
    Lean,
    /// `script.flf`
    Script,
    /// `shadow.flf`
    Shadow,
    /// User-supplied `.flf` file resolved from a filesystem path.
    External(PathBuf),
}

impl Font {
    /// Returns the lowercase, suffix-stripped bundled-font name for the
    /// 12 named variants. Returns `None` for [`Font::External`].
    pub(crate) fn bundled_name(&self) -> Option<&'static str> {
        Some(match self {
            Font::Standard => "standard",
            Font::Slant => "slant",
            Font::Small => "small",
            Font::Big => "big",
            Font::Mini => "mini",
            Font::Banner => "banner",
            Font::Block => "block",
            Font::Bubble => "bubble",
            Font::Digital => "digital",
            Font::Lean => "lean",
            Font::Script => "script",
            Font::Shadow => "shadow",
            Font::External(_) => return None,
        })
    }
}

impl Default for Font {
    fn default() -> Self {
        Self::Standard
    }
}

/// Source of the resolved `.flf` bytes that [`FigletBuilder::build`] will
/// parse. Internal — used to express the "font_bytes wins over font" rule
/// without leaking the enum to callers.
#[derive(Debug, Clone)]
enum FontSource {
    /// One of the 12 bundled-font variants.
    Bundled(Font),
    /// User-supplied path resolved via [`figfont::resolve_font`].
    External(PathBuf),
    /// In-memory bytes supplied via [`FigletBuilder::font_bytes`].
    Bytes(Vec<u8>),
}

/// Fluent builder for [`Figlet`] renderers.
///
/// Construct via [`FigletBuilder::new`] and chain configuration methods
/// (`#[must_use]`); terminate with [`FigletBuilder::build`] to obtain a
/// reusable [`Figlet`], or use [`FigletBuilder::render`] as a one-shot.
///
/// ```rust
/// use rusty_figlet::{FigletBuilder, Font};
///
/// let figlet = FigletBuilder::new()
///     .font(Font::Standard)
///     .width(80)
///     .build()
///     .expect("build");
/// let _banner = figlet.render("X").expect("render");
/// ```
#[derive(Debug, Clone)]
pub struct FigletBuilder {
    source: FontSource,
    width: u32,
    layout_override: Option<LayoutOverride>,
    layout_flags: LayoutFlags,
    justify: Option<Justify>,
    font_dirs: Vec<PathBuf>,
}

/// Layout override carried through the builder. Internal — translated
/// into a concrete `LayoutMode` at `build()` time once the font's default
/// is known. Retained for backward-compatibility with the per-method
/// `kerning()` / `full_width()` / `smush()` builders; the
/// [`FigletBuilder::layout`] path supersedes this for full last-wins
/// semantics across all six layout-class flags.
#[derive(Debug, Clone, Copy)]
enum LayoutOverride {
    Kerning,
    FullWidth,
    ForceSmush,
}

/// Horizontal justification mode.
///
/// ```rust
/// use rusty_figlet::{FigletBuilder, Justify};
///
/// let _ = FigletBuilder::new().justify(Justify::Center);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Justify {
    /// Center the rendered banner within the resolved width.
    Center,
    /// Left-align the rendered banner.
    Left,
    /// Right-align the rendered banner.
    Right,
    /// Use the font's print-direction default (LTR fonts default to Left).
    FontDefault,
}

impl Default for FigletBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl FigletBuilder {
    /// Construct a builder with all defaults:
    ///
    /// - font: [`Font::Standard`] (resolves to `standard.flf`)
    /// - width: 80 columns
    /// - layout: font-default
    /// - justify: font-default
    #[must_use]
    pub fn new() -> Self {
        Self {
            source: FontSource::Bundled(Font::Standard),
            width: 80,
            layout_override: None,
            layout_flags: LayoutFlags::default(),
            justify: None,
            font_dirs: Vec::new(),
        }
    }

    /// Select a font.
    ///
    /// When `font` is one of the 12 bundled variants, [`build`](Self::build)
    /// resolves the embedded `.flf` bytes via `include_bytes!`. When `font`
    /// is [`Font::External`], the supplied path is resolved at `build()`
    /// time. Default: [`Font::Standard`].
    #[must_use]
    pub fn font(mut self, font: Font) -> Self {
        self.source = match font {
            Font::External(path) => FontSource::External(path),
            other => FontSource::Bundled(other),
        };
        self
    }

    /// Supply raw `.flf` bytes directly (no filesystem access; FR-052 +
    /// FR-056). Overrides any prior [`font`](Self::font) call.
    #[must_use]
    pub fn font_bytes(mut self, bytes: &[u8]) -> Self {
        self.source = FontSource::Bytes(bytes.to_vec());
        self
    }

    /// Add an extra directory to search for [`Font::External`] resolutions
    /// (CLI `-d <dir>` counterpart per FR-010). Repeatable; directories are
    /// searched in the order added. Has no effect on bundled or
    /// [`font_bytes`](Self::font_bytes) sources.
    #[must_use]
    pub fn font_dirs(mut self, dirs: Vec<PathBuf>) -> Self {
        self.font_dirs = dirs;
        self
    }

    /// Set the output width budget in columns. Default: 80.
    #[must_use]
    pub fn width(mut self, cols: u32) -> Self {
        self.width = cols;
        self
    }

    /// Force horizontal kerning (`-k` CLI counterpart).
    /// Overrides the font's default layout. Last layout-override wins.
    #[must_use]
    pub fn kerning(mut self) -> Self {
        self.layout_override = Some(LayoutOverride::Kerning);
        self
    }

    /// Force full-width layout (`-W` CLI counterpart).
    /// Overrides the font's default layout. Last layout-override wins.
    #[must_use]
    pub fn full_width(mut self) -> Self {
        self.layout_override = Some(LayoutOverride::FullWidth);
        self
    }

    /// Force smushing per the font's smush bits (`-S` CLI counterpart).
    /// Overrides the font's default layout. Last layout-override wins.
    #[must_use]
    pub fn smush(mut self) -> Self {
        self.layout_override = Some(LayoutOverride::ForceSmush);
        self
    }

    /// Apply a full sequence of layout-class flag occurrences with
    /// last-wins semantics (FR-023). When non-empty, this sequence
    /// supersedes any per-method [`kerning`](Self::kerning) /
    /// [`full_width`](Self::full_width) / [`smush`](Self::smush)
    /// override.
    ///
    /// ```rust
    /// use rusty_figlet::{FigletBuilder, LayoutFlag, LayoutFlags};
    ///
    /// let flags = LayoutFlags {
    ///     flags: vec![LayoutFlag::FullWidth, LayoutFlag::Kerning],
    /// };
    /// let _ = FigletBuilder::new().layout(flags);
    /// ```
    #[must_use]
    pub fn layout(mut self, flags: LayoutFlags) -> Self {
        self.layout_flags = flags;
        self
    }

    /// Set the justification mode. Default: font's print-direction default.
    #[must_use]
    pub fn justify(mut self, j: Justify) -> Self {
        self.justify = Some(j);
        self
    }

    /// Resolve the font, parse the `.flf`, and build a reusable
    /// [`Figlet`] renderer.
    pub fn build(self) -> Result<Figlet, FigletError> {
        let bytes = match self.source {
            FontSource::Bundled(font) => {
                let name = font
                    .bundled_name()
                    .ok_or(FigletError::Internal("Font::External missed bundled match"))?;
                let slice =
                    figfont::resolve_bundled(name).ok_or_else(|| FigletError::FontNotFound {
                        name: name.to_owned(),
                        searched: Vec::new(),
                    })?;
                slice.to_vec()
            }
            FontSource::External(path) => {
                figfont::resolve_font(path.to_string_lossy().as_ref(), &self.font_dirs)?
            }
            FontSource::Bytes(bytes) => bytes,
        };
        let font = figfont::parse_bytes(&bytes)?;
        Ok(Figlet {
            font,
            width: self.width,
            layout_override: self.layout_override,
            layout_flags: self.layout_flags,
            justify: self.justify.unwrap_or(Justify::FontDefault),
        })
    }

    /// Terminal convenience equivalent to `self.build()?.render(text)`.
    pub fn render(self, text: &str) -> Result<Banner, FigletError> {
        self.build()?.render(text)
    }
}

/// A reusable renderer holding a parsed [`Font`] and resolved layout
/// settings.
///
/// Cheap to clone; clone the [`Figlet`] across threads to render many
/// banners concurrently with the same font configuration.
///
/// ```rust
/// use rusty_figlet::{FigletBuilder, Font};
///
/// let figlet = FigletBuilder::new()
///     .font(Font::Standard)
///     .build()
///     .expect("build");
/// let banner = figlet.render("Hi").expect("render");
/// assert!(banner.height() >= 1);
/// ```
#[derive(Debug, Clone)]
pub struct Figlet {
    font: figfont::FIGfont,
    width: u32,
    layout_override: Option<LayoutOverride>,
    layout_flags: LayoutFlags,
    justify: Justify,
}

impl Figlet {
    /// Render `text` into a [`Banner`].
    ///
    /// The returned banner exposes a lazy line iterator (per FR-053): row
    /// buffers are precomputed once during `render()`, and [`Banner::lines`]
    /// yields one row per `next()` without copying the whole banner.
    pub fn render(&self, text: &str) -> Result<Banner, FigletError> {
        let layout = self.resolved_layout();
        let rows = render_to_rows(&self.font, text, layout, self.width);
        let rows = apply_justify(rows, self.justify, self.width, self.font.print_direction);
        let rows = strip_hardblanks(rows, self.font.hardblank);
        Ok(Banner {
            rows,
            height: self.font.height,
        })
    }

    /// Translate the captured `layout_override` and/or `layout_flags`
    /// (CLI `-k`/`-W`/`-S`/`-s`/`-o`/`-m N`) into a concrete
    /// [`layout::LayoutMode`], falling back to the font's `full_layout`
    /// default when no override is set.
    ///
    /// When [`FigletBuilder::layout`] has been used (non-empty
    /// `layout_flags`), its sequence wins over any per-method
    /// `kerning()` / `full_width()` / `smush()` setting; the
    /// `LayoutResolver` then applies last-wins per FR-023.
    fn resolved_layout(&self) -> layout::LayoutMode {
        use layout::{LayoutFlag, LayoutFlags, LayoutResolver};
        if !self.layout_flags.flags.is_empty() {
            return LayoutResolver::resolve(&self.font, &self.layout_flags);
        }
        let mut flags = LayoutFlags::default();
        if let Some(ov) = self.layout_override {
            flags.flags.push(match ov {
                LayoutOverride::Kerning => LayoutFlag::Kerning,
                LayoutOverride::FullWidth => LayoutFlag::FullWidth,
                LayoutOverride::ForceSmush => LayoutFlag::ForceSmush,
            });
        }
        LayoutResolver::resolve(&self.font, &flags)
    }
}

/// Render `text` into `height` row buffers using the resolved layout
/// mode. Implements the per-row glyph accumulator described in T044
/// with horizontal smushing per HINT-002 + AD-005 and word-wrap per
/// HINT-008. Returns a `Vec<String>` of length `font.height`.
fn render_to_rows(
    font: &figfont::FIGfont,
    text: &str,
    layout: layout::LayoutMode,
    width: u32,
) -> Vec<String> {
    let height = font.height.max(1) as usize;
    if text.is_empty() {
        return vec![String::new(); height];
    }

    // Word-wrap per HINT-008: split on ASCII whitespace, accumulate
    // words into output lines whose post-smush width does not exceed
    // `width`. Each line then renders into `height` rows; lines are
    // separated by blank rows.
    let words: Vec<&str> = text.split(' ').collect();
    let target_width = width.max(1) as usize;

    let mut all_rows: Vec<String> = vec![String::new(); height];
    let mut current_rows: Vec<String> = vec![String::new(); height];
    let mut current_visual_width: usize = 0;
    let mut line_started = false;

    for word in &words {
        // Compute the prospective rows after appending this word (with
        // a single space-separator glyph when the current line is
        // already non-empty).
        let mut probe = current_rows.clone();
        let mut probe_width = current_visual_width;
        if line_started {
            append_codepoint(&mut probe, &mut probe_width, font, ' ' as u32, layout);
        }
        append_word(&mut probe, &mut probe_width, font, word, layout);

        if probe_width <= target_width || !line_started {
            // First word OR fits — commit the probe.
            // FR-025 + HINT-008: if this is a single word on a fresh
            // line AND it exceeds the target width, emit a one-time
            // stderr warning per process. The word is still rendered
            // at full glyph width (no mid-word break).
            if !line_started && probe_width > target_width {
                warn_over_width(word, target_width);
            }
            current_rows = probe;
            current_visual_width = probe_width;
            line_started = true;
        } else {
            // Flush current line, start new one with this word.
            for (acc, line) in all_rows.iter_mut().zip(current_rows.iter()) {
                if !acc.is_empty() {
                    acc.push('\n');
                }
                acc.push_str(line);
            }
            current_rows = vec![String::new(); height];
            current_visual_width = 0;
            append_word(
                &mut current_rows,
                &mut current_visual_width,
                font,
                word,
                layout,
            );
            // FR-025: a single word that overflows the budget on its
            // own line also triggers the over-width warning.
            if current_visual_width > target_width {
                warn_over_width(word, target_width);
            }
        }
    }

    if line_started {
        for (acc, line) in all_rows.iter_mut().zip(current_rows.iter()) {
            if !acc.is_empty() {
                acc.push('\n');
            }
            acc.push_str(line);
        }
    }

    // Flatten the all_rows accumulator: each entry may contain N
    // physical lines separated by `\n` (wrapped lines). For US1's
    // single-banner-per-render contract we keep them as separate rows
    // in the resulting Vec<String>: row 0 line 0, row 1 line 0, ...,
    // row 0 line 1, row 1 line 1, ... Splitting by `\n` and
    // interleaving handles the wrap case; for the common no-wrap path
    // there are no `\n` chars and the vector is unchanged.
    interleave_wrapped(all_rows, height)
}

fn append_word(
    rows: &mut [String],
    visual_width: &mut usize,
    font: &figfont::FIGfont,
    word: &str,
    layout: layout::LayoutMode,
) {
    for ch in word.chars() {
        append_codepoint(rows, visual_width, font, ch as u32, layout);
    }
}

fn append_codepoint(
    rows: &mut [String],
    visual_width: &mut usize,
    font: &figfont::FIGfont,
    cp: u32,
    layout: layout::LayoutMode,
) {
    let glyph = match figfont::lookup_codepoint(font, cp) {
        Some(g) => g,
        None => {
            // HINT-009: substitute codepoint-0 missing-character glyph
            // if present; else skip the char and emit a one-time stderr
            // warning. The warning is deduplicated globally via a
            // process-wide OnceLock so library callers don't pollute
            // their stderr when the same CJK input is rendered twice.
            warn_missing_codepoint(cp);
            match figfont::lookup_codepoint(font, 0) {
                Some(g) => g,
                None => return,
            }
        }
    };

    merge_glyph(rows, visual_width, glyph, layout, font.hardblank);
}

fn merge_glyph(
    rows: &mut [String],
    visual_width: &mut usize,
    glyph: &[String],
    layout: layout::LayoutMode,
    hardblank: char,
) {
    use layout::LayoutMode;

    // Determine smush behavior per LayoutMode.
    //
    // FIGfont 2.0 semantics: bit 64 (RULE_HORIZONTAL_SMUSHING) enables
    // smushing. The lower 6 bits select the active rules. When ANY of
    // the lower 6 bits is set, those rules are exhaustive and the
    // universal-fallback (right-wins) is NOT used. Universal-fallback
    // applies only when smushing is enabled AND no specific rule bit
    // is set (the "all six bits clear" case → `UniversalSmush`
    // LayoutMode).
    let (rules, allow_smush, allow_kerning_only) = match layout {
        LayoutMode::FullWidth => (0u8, false, false),
        LayoutMode::Kerning => (0u8, false, true),
        LayoutMode::UniversalSmush => (smush::RULE_HORIZONTAL_SMUSHING, true, true),
        LayoutMode::RuleSmush(bits) => {
            // Mask off any spurious upper bits so callers can't
            // accidentally re-enable universal-fallback via bit 64.
            let only_rule_bits = bits & 0b0011_1111;
            (only_rule_bits, true, true)
        }
        LayoutMode::OverlapOnly => (0u8, false, true),
    };

    let glyph_chars: Vec<Vec<char>> = glyph.iter().map(|s| s.chars().collect()).collect();
    let glyph_width = glyph_chars.iter().map(|r| r.len()).max().unwrap_or(0);

    if glyph_width == 0 {
        return;
    }

    // FullWidth: no overlap, no smushing; just append.
    if !allow_smush && !allow_kerning_only {
        for (i, row) in rows.iter_mut().enumerate() {
            if let Some(gr) = glyph_chars.get(i) {
                for &c in gr {
                    row.push(c);
                }
                // Pad short glyph rows out to glyph_width.
                for _ in gr.len()..glyph_width {
                    row.push(' ');
                }
            } else {
                for _ in 0..glyph_width {
                    row.push(' ');
                }
            }
        }
        *visual_width += glyph_width;
        return;
    }

    // Determine the maximum overlap `k` (number of columns by which
    // the glyph can shift left into the accumulator) such that every
    // row still produces a legal smush/kerning result.
    let row_chars: Vec<Vec<char>> = rows.iter().map(|s| s.chars().collect()).collect();
    let acc_widths: Vec<usize> = row_chars.iter().map(|r| r.len()).collect();
    let acc_min_width = acc_widths.iter().copied().min().unwrap_or(0);

    let max_possible = acc_min_width.min(glyph_width);
    let mut overlap = 0usize;
    // For overlap == 0 we always append directly (legal). For larger
    // overlaps we test each row.
    'outer: for k in 1..=max_possible {
        // Build merged-char arrays for each row at this overlap.
        let mut row_merges: Vec<Vec<char>> = Vec::with_capacity(rows.len());
        for (i, acc_row) in row_chars.iter().enumerate() {
            let glyph_row = glyph_chars.get(i).cloned().unwrap_or_default();
            // Overlapping columns: acc_row[acc.len()-k+j] vs glyph_row[j].
            let mut merged = Vec::with_capacity(k);
            for j in 0..k {
                let l = acc_row.get(acc_row.len() - k + j).copied().unwrap_or(' ');
                let r = glyph_row.get(j).copied().unwrap_or(' ');
                match smush::smush_pair(l, r, rules, hardblank) {
                    Some(c) => merged.push(c),
                    None => {
                        // No smush possible at this column → this overlap
                        // is illegal. Roll back.
                        break 'outer;
                    }
                }
            }
            row_merges.push(merged);
        }
        // All rows produced legal merges at this overlap; record and
        // continue trying larger k.
        overlap = k;
        // Cache the merges by stashing them — we'll recompute on commit.
        let _ = row_merges;
    }

    // Commit the chosen overlap.
    for (i, row) in rows.iter_mut().enumerate() {
        let acc_chars: Vec<char> = row.chars().collect();
        let glyph_row: Vec<char> = glyph_chars.get(i).cloned().unwrap_or_default();
        // Trim `overlap` cols off the accumulator and append merged + tail.
        let keep = acc_chars.len().saturating_sub(overlap);
        let mut new_row: String = acc_chars[..keep].iter().collect();
        for j in 0..overlap {
            let l = acc_chars.get(keep + j).copied().unwrap_or(' ');
            let r = glyph_row.get(j).copied().unwrap_or(' ');
            let merged = smush::smush_pair(l, r, rules, hardblank).unwrap_or(r);
            new_row.push(merged);
        }
        for j in overlap..glyph_width {
            new_row.push(glyph_row.get(j).copied().unwrap_or(' '));
        }
        *row = new_row;
    }
    *visual_width = visual_width
        .saturating_add(glyph_width)
        .saturating_sub(overlap);
}

fn interleave_wrapped(all_rows: Vec<String>, height: usize) -> Vec<String> {
    // Each entry in `all_rows` is a `\n`-joined list of physical lines
    // (one per wrap segment). If no entries contain `\n` the input is
    // returned verbatim. Otherwise we re-interleave: for each wrap
    // segment index, emit `height` rows in order.
    let has_wrap = all_rows.iter().any(|r| r.contains('\n'));
    if !has_wrap {
        return all_rows;
    }
    let per_row: Vec<Vec<&str>> = all_rows.iter().map(|r| r.split('\n').collect()).collect();
    let segments = per_row.first().map(Vec::len).unwrap_or(0);
    let mut out: Vec<String> = Vec::with_capacity(height * segments);
    for seg in 0..segments {
        for row_lines in per_row.iter().take(height) {
            let s = row_lines.get(seg).copied().unwrap_or("");
            out.push(s.to_owned());
        }
        // No blank line between wrap segments — upstream figlet word-
        // wrap concatenates the height-line blocks back-to-back. Banner
        // separators (one blank line between distinct invocations) are
        // inserted by the binary's stdin per-line loop instead.
    }
    out
}

fn apply_justify(
    rows: Vec<String>,
    justify: Justify,
    width: u32,
    print_direction: u32,
) -> Vec<String> {
    let effective = match justify {
        Justify::Center => Justify::Center,
        Justify::Left => Justify::Left,
        Justify::Right => Justify::Right,
        Justify::FontDefault => {
            if print_direction == 1 {
                Justify::Right
            } else {
                Justify::Left
            }
        }
    };
    let target = width as usize;
    rows.into_iter()
        .map(|line| match effective {
            Justify::Left | Justify::FontDefault => line,
            Justify::Center => {
                let w = line.chars().count();
                if w >= target {
                    line
                } else {
                    let pad = (target - w) / 2;
                    let mut out = String::with_capacity(target);
                    for _ in 0..pad {
                        out.push(' ');
                    }
                    out.push_str(&line);
                    out
                }
            }
            Justify::Right => {
                let w = line.chars().count();
                if w >= target {
                    line
                } else {
                    let pad = target - w;
                    let mut out = String::with_capacity(target);
                    for _ in 0..pad {
                        out.push(' ');
                    }
                    out.push_str(&line);
                    out
                }
            }
        })
        .collect()
}

fn strip_hardblanks(rows: Vec<String>, hardblank: char) -> Vec<String> {
    rows.into_iter()
        .map(|line| line.replace(hardblank, " "))
        .collect()
}

/// Clamp UTF-8 input down to Latin-1 (ISO-8859-1) bytes per FR-044.
///
/// In Strict mode the upstream `figlet(6)` binary treats every input
/// byte as a Latin-1 codepoint (bytes 0..=255). This helper mirrors
/// that semantics by mapping every input `char` whose value fits in
/// `u8` (0..=255) to the equivalent single-byte Latin-1 codepoint and
/// substituting multi-byte UTF-8 codepoints with the upstream-
/// compatible `?` (0x3F) placeholder. The returned `Vec<u8>` can be
/// passed verbatim to the figfont codepoint lookup (which already
/// indexes by `u32`, so any byte 0..=255 round-trips cleanly).
///
/// HINT-009 explicitly excludes Strict mode from the UTF-8 missing-
/// glyph fallback path because this clamp precedes lookup. See the
/// BREAKING-CHANGE entry in `CHANGELOG.md` for the Default-mode UTF-8
/// vs. Strict-mode Latin-1 divergence.
pub fn clamp_input_latin1(input: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(input.len());
    for ch in input.chars() {
        let cp = ch as u32;
        if cp <= 0xFF {
            out.push(cp as u8);
        } else {
            // Upstream figlet emits `?` for non-Latin-1 input bytes.
            out.push(b'?');
        }
    }
    out
}

/// Process-wide dedup for the "missing codepoint" stderr warning per
/// FR-005 + Clarifications Q6. The first missing codepoint emits a
/// warning; subsequent missing codepoints are silently substituted.
static MISSING_GLYPH_WARNED: OnceLock<()> = OnceLock::new();

fn warn_missing_codepoint(cp: u32) {
    if MISSING_GLYPH_WARNED.set(()).is_ok() {
        eprintln!(
            "rusty-figlet: codepoint U+{cp:04X} missing from font; substituting fallback glyph"
        );
    }
}

/// Process-wide dedup for the "over-width word" stderr warning per
/// FR-025 + Clarifications Q6 + HINT-008. The first single word wider
/// than the resolved `-w` budget emits a warning; subsequent over-width
/// words are silently rendered at full glyph width.
static OVER_WIDTH_WARNED: OnceLock<()> = OnceLock::new();

fn warn_over_width(word: &str, width: usize) {
    if OVER_WIDTH_WARNED.set(()).is_ok() {
        eprintln!(
            "rusty-figlet: '{word}' too wide for width {width}; emitting at full glyph width"
        );
    }
}

/// A rendered ASCII-art banner.
///
/// `Banner` is a lazy line iterator (per FR-053) from the caller's
/// perspective: row buffers are computed once during
/// [`Figlet::render`], and each call to `next()` on the iterator
/// returned by [`Banner::lines`] yields one row.
///
/// `Banner` also implements [`core::fmt::Display`]; `write!(stdout,
/// "{banner}")` drives the same lazy iterator and emits a trailing `\n`
/// after the final line.
///
/// ```rust
/// use rusty_figlet::{FigletBuilder, Font};
///
/// let banner = FigletBuilder::new()
///     .font(Font::Standard)
///     .build()
///     .expect("build")
///     .render("X")
///     .expect("render");
/// // Iterate lazily; each .next() yields exactly one rendered row.
/// let mut it = banner.lines();
/// let _first = it.next();
/// ```
#[derive(Debug, Clone)]
pub struct Banner {
    rows: Vec<String>,
    height: u32,
}

impl Banner {
    /// Return a lazy iterator yielding one rendered line per `.next()`.
    pub fn lines(&self) -> impl Iterator<Item = String> + '_ {
        self.rows.iter().cloned()
    }

    /// The font's row count (height). Library callers occasionally want
    /// to know how many rows a banner contains without iterating.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// `true` when the banner produced no rendered rows (empty input).
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() || self.rows.iter().all(|r| r.is_empty())
    }
}

impl core::fmt::Display for Banner {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        for line in self.lines() {
            writeln!(f, "{line}")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use static_assertions::assert_impl_all;

    // SC-009: FigletError is Send + Sync + 'static so it crosses async
    // await + thread boundaries. The other public types are Send + Sync
    // but intentionally NOT `'static` because they may borrow from
    // caller-supplied input (`font_bytes(&[u8])`).
    assert_impl_all!(FigletBuilder: Send, Sync);
    assert_impl_all!(Figlet: Send, Sync);
    assert_impl_all!(Banner: Send, Sync);
    assert_impl_all!(FigletError: Send, Sync);

    fn _figlet_error_is_static() {
        fn assert_static<T: 'static>() {}
        assert_static::<FigletError>();
    }

    #[test]
    fn builder_default_font_is_standard() {
        let builder = FigletBuilder::new();
        match builder.source {
            FontSource::Bundled(Font::Standard) => {}
            _ => panic!("default font must be Standard"),
        }
    }
}
