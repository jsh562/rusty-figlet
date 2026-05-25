//! clap-derive `Cli` struct + `Completions` subcommand.
//!
//! Default-mode argv parsing flows through this module; Strict-mode
//! parsing bypasses clap and uses [`crate::strict::parse_argv`] for
//! byte-equal upstream diagnostics.

use std::path::PathBuf;

use clap::Parser;
#[cfg(feature = "completions")]
use clap::Subcommand as ClapSubcommand;
#[cfg(feature = "color")]
use clap::ValueEnum;

/// Top-level CLI surface for `rusty-figlet`.
#[derive(Debug, Parser)]
#[command(
    name = "rusty-figlet",
    version,
    about = "Render ASCII-art banners from text",
    long_about = None,
)]
pub struct Cli {
    /// Font name (one of the 12 bundled) or path to a `.flf` file. The
    /// `.flf` suffix is optional for bundled-font lookup.
    #[arg(short = 'f', long = "font", value_name = "FONT")]
    pub font: Option<String>,

    /// Additional font directory to search (repeatable).
    #[arg(short = 'd', long = "fontdir", value_name = "DIR")]
    pub font_dirs: Vec<PathBuf>,

    /// Output width in columns.
    #[arg(short = 'w', long = "width", value_name = "INT")]
    pub width: Option<u32>,

    /// Auto-detect terminal width (overrides `-w` precedence per AD-010).
    #[arg(short = 't', long = "terminal-width")]
    pub use_terminal_width: bool,

    /// Center the rendered banner.
    #[arg(short = 'c', long = "center")]
    pub center: bool,
    /// Left-align the rendered banner.
    #[arg(short = 'l', long = "left")]
    pub left: bool,
    /// Right-align the rendered banner.
    #[arg(short = 'r', long = "right")]
    pub right: bool,
    /// Use the font's print-direction default for justification.
    #[arg(short = 'x', long = "font-default-justify")]
    pub justify_default: bool,

    /// Force kerning layout.
    #[arg(short = 'k', long = "kerning")]
    pub kerning: bool,
    /// Force full-width layout.
    #[arg(short = 'W', long = "full-width")]
    pub full_width: bool,
    /// Force smushing per the font's smush bits.
    #[arg(short = 'S', long = "force-smush")]
    pub force_smush: bool,
    /// Use the font's default smush layout.
    #[arg(short = 's', long = "smush")]
    pub default_smush: bool,
    /// Overlap-only layout.
    #[arg(short = 'o', long = "overlap")]
    pub overlap: bool,
    /// Explicit layout bitfield.
    #[arg(
        short = 'm',
        long = "layout-mode",
        value_name = "INT",
        allow_hyphen_values = true
    )]
    pub explicit_layout: Option<i32>,

    /// Paragraph mode (concatenate consecutive non-empty stdin lines).
    #[arg(short = 'p', long = "paragraph")]
    pub paragraph: bool,
    /// Normal newline mode (each stdin line is a separate banner).
    #[arg(short = 'n', long = "normal")]
    pub normal: bool,

    /// Control file (accepted-but-ignored in Default mode per FR-046;
    /// rejected in Strict mode).
    #[arg(short = 'C', long = "control-file", value_name = "FILE")]
    pub control_file: Option<PathBuf>,
    /// Suppress control-file processing (accepted-but-ignored per FR-046).
    #[arg(short = 'N', long = "no-controlfile")]
    pub no_controlfile: bool,

    /// Tri-state color flag. Gated by `color` leaf (v0.2+).
    #[cfg(feature = "color")]
    #[arg(long = "color", value_name = "WHEN", value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Emit a per-column rainbow gradient. Gated by `rainbow` leaf (v0.2+).
    #[cfg(feature = "rainbow")]
    #[arg(long = "rainbow")]
    pub rainbow: bool,

    /// Force Strict mode (byte-equal upstream `figlet 2.2.5` behavior).
    #[arg(long = "strict", conflicts_with = "no_strict")]
    pub strict: bool,
    /// Force Default mode (overrides env + argv[0]).
    #[arg(long = "no-strict")]
    pub no_strict: bool,

    /// Toilet-compatible filter chain (`-F filter1:filter2:...`). E012 US1.
    ///
    /// Multiple `-F` flags are concatenated with `:` per FR-002 before parsing.
    /// Gated by any `filter-*` leaf (in practice: visible whenever the binary
    /// is compiled with at least one filter leaf enabled — see Cargo.toml).
    #[cfg(any(
        feature = "filter-crop",
        feature = "filter-gay",
        feature = "filter-metal",
        feature = "filter-flip",
        feature = "filter-flop",
        feature = "filter-rotate",
        feature = "filter-border",
    ))]
    #[arg(short = 'F', long = "filter", value_name = "CHAIN")]
    pub filter: Vec<String>,

    /// Export the rendered banner as `html`, `irc`, or `svg` (E012 US2 — FR-005).
    ///
    /// Visible when at least one `output-*` leaf is enabled. Unknown values
    /// (or values whose leaf is not compiled in) exit non-zero with the
    /// enumerated available list per FR-016.
    #[cfg(any(
        feature = "output-html",
        feature = "output-irc",
        feature = "output-svg",
    ))]
    #[arg(short = 'E', long = "export", value_name = "FORMAT")]
    pub export_format: Option<String>,

    /// Force 24-bit truecolor SGR emission (E012 US4 — FR-008).
    ///
    /// Gated by the `color-truecolor` leaf. When the terminal does not
    /// advertise truecolor support the request downgrades gracefully unless
    /// `--no-downgrade-warning` is also set.
    #[cfg(feature = "color-truecolor")]
    #[arg(long = "truecolor")]
    pub truecolor: bool,

    /// Force 256-color SGR emission (E012 US4 — FR-009).
    ///
    /// Gated by the `color-256` leaf.
    #[cfg(feature = "color-256")]
    #[arg(long = "ansi256")]
    pub ansi256: bool,

    /// Background color spec — one of the 16 named ANSI colors or `#RRGGBB`
    /// hex (E012 US7 — SC-007). Parsed BEFORE export emit; arbitrary user
    /// bytes never flow into a color slot per spec Edge Cases.
    #[cfg(feature = "color")]
    #[arg(long = "background", value_name = "COLOR")]
    pub background: Option<String>,

    /// Suppress the one-time downgrade-warning stderr line when the
    /// requested color depth is unavailable (E012 US4 — FR-029).
    ///
    /// FR-029 zero-cost: the suppression short-circuits BEFORE the warning's
    /// format-args evaluation per `color_depth::resolve_depth`.
    #[cfg(any(feature = "color-truecolor", feature = "color-256"))]
    #[arg(long = "no-downgrade-warning")]
    pub no_downgrade_warning: bool,

    /// Warn when IRC-format export strips a non-printable byte from the
    /// input (E012 US2 — FR-015 ergonomics knob).
    #[cfg(feature = "output-irc")]
    #[arg(long = "warn-irc-strip")]
    pub warn_irc_strip: bool,

    /// Positional message text (concatenated with a single space per FR-002).
    #[arg(value_name = "MESSAGE", trailing_var_arg = true)]
    pub message: Vec<String>,

    /// Subcommand (e.g. `completions <shell>`). Gated by `completions` leaf.
    #[cfg(feature = "completions")]
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}

/// Tri-state `--color` value. Gated by `color` leaf (v0.2+).
#[cfg(feature = "color")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, ValueEnum)]
#[value(rename_all = "lower")]
pub enum ColorChoice {
    /// Auto-detect from TTY status.
    Auto,
    /// Always emit color (still suppressed by NO_COLOR per FR-032).
    Always,
    /// Never emit color.
    Never,
}

/// Subcommand surface. Gated by `completions` leaf (v0.2+).
#[cfg(feature = "completions")]
#[derive(Debug, ClapSubcommand)]
pub enum Subcommand {
    /// Emit shell-completion scripts.
    Completions {
        /// Shell to generate completions for.
        shell: clap_complete::Shell,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_command_builds() {
        Cli::command().debug_assert();
    }
}
