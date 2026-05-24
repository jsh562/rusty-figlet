//! clap-derive `Cli` struct + `Completions` subcommand.
//!
//! Default-mode argv parsing flows through this module; Strict-mode
//! parsing bypasses clap and uses [`crate::strict::parse_argv`] for
//! byte-equal upstream diagnostics.

use std::path::PathBuf;

use clap::{Parser, Subcommand as ClapSubcommand, ValueEnum};

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

    /// Tri-state color flag.
    #[arg(long = "color", value_name = "WHEN", value_enum, default_value_t = ColorChoice::Auto)]
    pub color: ColorChoice,

    /// Emit a per-column rainbow gradient.
    #[arg(long = "rainbow")]
    pub rainbow: bool,

    /// Force Strict mode (byte-equal upstream `figlet 2.2.5` behavior).
    #[arg(long = "strict", conflicts_with = "no_strict")]
    pub strict: bool,
    /// Force Default mode (overrides env + argv[0]).
    #[arg(long = "no-strict")]
    pub no_strict: bool,

    /// Positional message text (concatenated with a single space per FR-002).
    #[arg(value_name = "MESSAGE", trailing_var_arg = true)]
    pub message: Vec<String>,

    /// Subcommand (e.g. `completions <shell>`).
    #[command(subcommand)]
    pub subcommand: Option<Subcommand>,
}

/// Tri-state `--color` value.
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

/// Subcommand surface.
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
