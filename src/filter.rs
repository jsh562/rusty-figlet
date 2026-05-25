//! `RenderGrid` + `FilterChain` (E012 US1/US5 — FR-002, FR-003, FR-004).
//!
//! This module hosts the typed grid that filters operate on, the
//! [`Filter`] enum enumerating the 10 supported transformations, the
//! [`FilterChain`] orchestrator, and the per-filter pure-function
//! implementations. Each individual filter is gated behind its leaf
//! feature (`filter-crop`, `filter-gay`, `filter-metal`, `filter-flip`,
//! `filter-flop`, `filter-rotate`, `filter-border`) per ADR-0006 +
//! plan §Cargo Feature Surface; the [`Filter::Nothing`] identity has no
//! leaf and is always available.
//!
//! ## Design constraints
//!
//! - **Immutability (AD-002)** — every filter takes an owned [`RenderGrid`]
//!   and returns a new owned grid. No interior mutability, no shared
//!   borrows, no in-place transforms.
//! - **Bounded cell footprint (AD-011)** — [`Cell`] is ~16 bytes; the
//!   grid memory is `O(w·h)` and a chain of `n` filters costs `O(n·w·h)`
//!   (HINT-006 + FR-030).
//! - **No upstream-source consultation** — implementations derived from
//!   the toilet(1) manpage and observed outputs; recorded under
//!   `docs/tlf-derivation.md`.

use crate::error::FigletError;

/// Maximum filter-name length accepted by [`FilterChain::parse`] (spec Edge Cases).
///
/// Names longer than this byte count are rejected with
/// [`FigletError::UnknownFilter`] regardless of contents — guards against
/// adversarial `-F` chains and keeps the error path O(1).
const MAX_FILTER_NAME_BYTES: usize = 64;

/// Canonical list of valid filter names (declaration order).
///
/// Used by [`FilterChain::parse`] for lookup and surfaced in
/// [`FigletError::UnknownFilter::available`] so the CLI can enumerate
/// the supported set in a diagnostic.
const FILTER_NAMES: &[&str] = &[
    "crop",
    "gay",
    "metal",
    "flip",
    "flop",
    "rotate180",
    "rotateleft",
    "rotateright",
    "border",
    "nothing",
];

/// Color carried by a [`Cell`] — bounded footprint per AD-011.
///
/// Three representations cover the SGR surfaces that v0.3.0 emits:
///
/// - [`Color::Named`] for the 16-color palette (`\x1b[30m`..`\x1b[37m`,
///   `\x1b[90m`..`\x1b[97m`) — the toilet 0.3-1 floor;
/// - [`Color::Index`] for 256-color (`\x1b[38;5;Nm`) — Phase 6;
/// - [`Color::Rgb`] for 24-bit truecolor (`\x1b[38;2;R;G;Bm`) — Phase 6.
///
/// The default is [`Color::default`] (white-on-default-bg) so a freshly
/// constructed [`Cell`] needs no explicit color call.
///
/// The enum is `#[non_exhaustive]` so adding e.g. a fully-typed
/// 88-color palette (rare but exists in retro terminals) remains
/// non-breaking under SemVer.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Color {
    /// One of the 16 ANSI named colors (0..=15).
    Named(NamedColor),
    /// One of the 256 indexed palette colors (0..=255).
    Index(u8),
    /// 24-bit truecolor triple.
    Rgb(u8, u8, u8),
}

impl Default for Color {
    fn default() -> Self {
        // White-on-default-bg matches the SGR default for a plain TTY.
        Self::Named(NamedColor::White)
    }
}

/// One of the 16 ANSI named colors.
///
/// Stored as a single-byte enum to keep [`Cell`]'s footprint within the
/// AD-011 ~16-byte budget regardless of `Color` variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(missing_docs)]
pub enum NamedColor {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
}

/// A single rendered cell: a character plus optional color attributes.
///
/// Footprint is bounded by AD-011 to ~16 bytes on 64-bit targets:
/// - `ch: char` — 4 bytes
/// - `fg: Color` — 4 bytes (variant tag + 3-byte payload for `Rgb`)
/// - `bg: Option<Color>` — 5 bytes (tag + 4-byte `Color`)
/// - `attrs: u8` — 1 byte (bold / underline / reverse bitfield)
///
/// The remaining ~2 bytes are alignment padding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Cell {
    /// Glyph character at this cell.
    pub ch: char,
    /// Foreground color.
    pub fg: Color,
    /// Optional background color (`None` = terminal default bg).
    pub bg: Option<Color>,
    /// SGR attribute bitfield: bit 0 = bold, bit 1 = underline, bit 2 = reverse.
    pub attrs: u8,
}

impl Cell {
    /// Construct a cell with the given glyph; default foreground (white),
    /// no background, no attributes.
    #[must_use]
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            fg: Color::default(),
            bg: None,
            attrs: 0,
        }
    }

    /// Blank cell — a space with default colors.
    #[must_use]
    pub fn blank() -> Self {
        Self::new(' ')
    }

    /// `true` if the cell is visually blank (space character).
    ///
    /// Used by [`apply_crop`] to detect all-blank rows/columns. Color
    /// and attribute bits are ignored — a colored space is still blank.
    #[must_use]
    pub fn is_blank(&self) -> bool {
        self.ch == ' '
    }
}

/// A 2D grid of [`Cell`]s with explicit `width` × `height` dimensions.
///
/// Filters operate on owned `RenderGrid`s and return new owned grids per
/// AD-002 (immutable transformations). Construction normalizes ragged
/// row vectors to a rectangular shape padded with [`Cell::blank`] so
/// downstream filters can assume `cells[y].len() == width` for every
/// row.
///
/// Allocated as `Vec<Vec<Cell>>` rather than a single flat `Vec<Cell>`
/// because the filter implementations (transpose, rotate, flip) work
/// row-major and benefit from being able to `.swap()`, `.reverse()`, and
/// `.collect()` per-row without index arithmetic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderGrid {
    /// Cells in row-major order. `cells[y][x]` is the cell at column `x`,
    /// row `y`. Every row is exactly `width` cells long after construction.
    pub cells: Vec<Vec<Cell>>,
    /// Number of columns.
    pub width: u32,
    /// Number of rows.
    pub height: u32,
}

impl RenderGrid {
    /// Construct an empty grid (0×0).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            cells: Vec::new(),
            width: 0,
            height: 0,
        }
    }

    /// Construct a rectangular grid sized `width` × `height`, filled
    /// with blank cells.
    #[must_use]
    pub fn blank(width: u32, height: u32) -> Self {
        let w = width as usize;
        let h = height as usize;
        let cells = (0..h).map(|_| vec![Cell::blank(); w]).collect();
        Self {
            cells,
            width,
            height,
        }
    }

    /// Build a grid from a vector of rows; ragged rows are padded with
    /// [`Cell::blank`] up to the longest row's length.
    #[must_use]
    pub fn from_rows(mut rows: Vec<Vec<Cell>>) -> Self {
        let width = rows.iter().map(Vec::len).max().unwrap_or(0);
        for row in rows.iter_mut() {
            if row.len() < width {
                row.resize(width, Cell::blank());
            }
        }
        let height = rows.len();
        Self {
            cells: rows,
            width: width as u32,
            height: height as u32,
        }
    }

    /// Construct a grid from a multi-line `&str`. Each line is one row;
    /// each character is one cell with default color. Convenient for
    /// tests and the FLF/TLF render pipeline's `Vec<String>` row output.
    #[must_use]
    pub fn from_text_rows(rows: &[String]) -> Self {
        let cells: Vec<Vec<Cell>> = rows
            .iter()
            .map(|line| line.chars().map(Cell::new).collect())
            .collect();
        Self::from_rows(cells)
    }
}

/// The 10 supported toilet-compatible filters (FR-003).
///
/// Each variant maps to a pure function on owned [`RenderGrid`]s
/// (AD-002). Names match toilet(1)'s `-F` chain vocabulary 1:1 (`crop`,
/// `gay`, `metal`, `flip`, `flop`, `rotate180`, `rotateleft`,
/// `rotateright`, `border`, plus the always-available `nothing`
/// identity).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Filter {
    /// Trim surrounding all-blank rows and columns.
    Crop,
    /// Per-column rainbow color sweep (the toilet `--gay` aesthetic).
    Gay,
    /// Blue / gray metallic gradient.
    Metal,
    /// Horizontal mirror (reverse each row's columns).
    Flip,
    /// Vertical mirror (reverse the order of rows).
    Flop,
    /// Rotate 180° (combination of flip + flop).
    Rotate180,
    /// Rotate 90° counter-clockwise (transpose + flip).
    RotateLeft,
    /// Rotate 90° clockwise (transpose + flop).
    RotateRight,
    /// Draw a Unicode box-drawing border around the grid.
    Border,
    /// No-op identity. Always available (no leaf gate).
    Nothing,
}

impl Filter {
    /// Returns the canonical lowercase name parsed from a `-F` chain.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Filter::Crop => "crop",
            Filter::Gay => "gay",
            Filter::Metal => "metal",
            Filter::Flip => "flip",
            Filter::Flop => "flop",
            Filter::Rotate180 => "rotate180",
            Filter::RotateLeft => "rotateleft",
            Filter::RotateRight => "rotateright",
            Filter::Border => "border",
            Filter::Nothing => "nothing",
        }
    }

    /// Map a parsed segment name to a [`Filter`] variant.
    ///
    /// Case-sensitive lowercase match — toilet(1) is documented as
    /// lowercase-only and we preserve that semantic. Returns `None` for
    /// unknown names; the caller turns this into a
    /// [`FigletError::UnknownFilter`].
    fn from_name(name: &str) -> Option<Filter> {
        Some(match name {
            "crop" => Filter::Crop,
            "gay" => Filter::Gay,
            "metal" => Filter::Metal,
            "flip" => Filter::Flip,
            "flop" => Filter::Flop,
            "rotate180" => Filter::Rotate180,
            "rotateleft" => Filter::RotateLeft,
            "rotateright" => Filter::RotateRight,
            "border" => Filter::Border,
            "nothing" => Filter::Nothing,
            _ => return None,
        })
    }
}

/// Ordered list of [`Filter`]s applied left-to-right in
/// [`FilterChain::apply`] (FR-004).
///
/// ## Cost bound
///
/// A chain of `n` filters applied to a `w × h` grid runs in
/// `O(n · w · h)` time and allocates at most `O(w · h)` per step
/// (per AD-002 + AD-007 + HINT-006 + FR-022 + FR-030). The grid is
/// owned and cloned-on-write between steps, so memory peaks at one
/// extra grid above the input.
///
/// SC-012 records the wall-clock linear-scaling guarantee
/// (`tests/filter_scaling.rs` asserts N=20 ≤ 2.5× N=10). HINT-002
/// surfaces this bound to library consumers via this rustdoc.
///
/// ## Construction
///
/// Build programmatically via [`FilterChain::new`] +
/// [`FilterChain::push`] (per US5), or parse from a `-F` flag string
/// via [`FilterChain::parse`] (per FR-002). Parsing handles the
/// `filter1:filter2:...` syntax shared with toilet(1); the CLI
/// concatenates multiple `-F` flags with `:` before invoking parse.
///
/// ```rust
/// use rusty_figlet::filter::{Filter, FilterChain, RenderGrid};
///
/// // Programmatic composition (US5).
/// let chain = FilterChain::new()
///     .push(Filter::Crop)
///     .push(Filter::Border);
///
/// // Parsed from a `-F` flag.
/// let parsed = FilterChain::parse("crop:border").expect("parse");
/// assert_eq!(chain, parsed);
///
/// let grid = RenderGrid::blank(4, 2);
/// let _ = chain.apply(grid).expect("apply");
/// ```
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FilterChain {
    filters: Vec<Filter>,
}

impl FilterChain {
    /// Construct an empty chain.
    ///
    /// An empty chain applied to a grid returns the input unchanged.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append `filter` to this chain and return the updated chain
    /// (consuming-self builder per US5 ergonomics).
    #[must_use]
    pub fn push(mut self, filter: Filter) -> Self {
        self.filters.push(filter);
        self
    }

    /// Parse a `-F <chain>` specification per FR-002.
    ///
    /// Syntax: `filter1:filter2:...` — colon-separated lowercase names.
    /// Multiple `-F` CLI flags are concatenated with `:` by the caller
    /// before invoking parse.
    ///
    /// Empty segments (`crop::flip`), names longer than 64 bytes, and
    /// names not in the canonical list (case-sensitive) are all rejected
    /// with [`FigletError::UnknownFilter`] whose `available` field
    /// enumerates the 10 valid names in declaration order (per FR-016
    /// and spec Edge Cases). An entirely empty `spec` parses to an
    /// empty chain (the no-`-F`-flag case).
    pub fn parse(spec: &str) -> Result<FilterChain, FigletError> {
        let mut filters = Vec::new();
        if spec.is_empty() {
            return Ok(Self { filters });
        }
        for segment in spec.split(':') {
            if segment.is_empty() || segment.len() > MAX_FILTER_NAME_BYTES {
                return Err(FigletError::UnknownFilter {
                    name: segment.to_owned(),
                    available: FILTER_NAMES.iter().map(|&s| s.to_owned()).collect(),
                });
            }
            match Filter::from_name(segment) {
                Some(f) => filters.push(f),
                None => {
                    return Err(FigletError::UnknownFilter {
                        name: segment.to_owned(),
                        available: FILTER_NAMES.iter().map(|&s| s.to_owned()).collect(),
                    });
                }
            }
        }
        Ok(Self { filters })
    }

    /// Number of filters currently in this chain.
    #[must_use]
    pub fn len(&self) -> usize {
        self.filters.len()
    }

    /// `true` when this chain has no filters; `apply` would return its
    /// input unchanged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.filters.is_empty()
    }

    /// Borrow the chain's filters in application order.
    #[must_use]
    pub fn filters(&self) -> &[Filter] {
        &self.filters
    }

    /// Apply each filter in order to `grid` and return the resulting
    /// grid (FR-004).
    ///
    /// ## Cost bound (FR-030 + HINT-006 + AD-007)
    ///
    /// Runs in `O(n · w · h)` where `n = self.filters().len()`,
    /// `w = grid.width`, `h = grid.height`. SC-012 enforces a
    /// wall-clock linear-scaling test on the library so callers can
    /// rely on this bound when composing long chains programmatically
    /// (US5).
    ///
    /// Empty chains are well-defined — they return the input grid
    /// unchanged. Filters whose leaf feature is disabled at
    /// compile-time return a [`FigletError::UnknownFilter`] at
    /// `apply` time rather than at construction time so existing
    /// `FilterChain` values keep working when reused across builds
    /// with different feature surfaces.
    pub fn apply(&self, grid: RenderGrid) -> Result<RenderGrid, FigletError> {
        let mut current = grid;
        for filter in &self.filters {
            current = dispatch(*filter, current)?;
        }
        Ok(current)
    }
}

/// Dispatch a single [`Filter`] onto `grid`. Unknown / leaf-disabled
/// filters return [`FigletError::UnknownFilter`] so `apply` can short-
/// circuit the chain.
fn dispatch(filter: Filter, grid: RenderGrid) -> Result<RenderGrid, FigletError> {
    match filter {
        Filter::Nothing => Ok(apply_nothing(grid)),
        #[cfg(feature = "filter-crop")]
        Filter::Crop => Ok(apply_crop(grid)),
        #[cfg(not(feature = "filter-crop"))]
        Filter::Crop => Err(filter_disabled("crop")),
        #[cfg(feature = "filter-gay")]
        Filter::Gay => Ok(apply_gay(grid)),
        #[cfg(not(feature = "filter-gay"))]
        Filter::Gay => Err(filter_disabled("gay")),
        #[cfg(feature = "filter-metal")]
        Filter::Metal => Ok(apply_metal(grid)),
        #[cfg(not(feature = "filter-metal"))]
        Filter::Metal => Err(filter_disabled("metal")),
        #[cfg(feature = "filter-flip")]
        Filter::Flip => Ok(apply_flip(grid)),
        #[cfg(not(feature = "filter-flip"))]
        Filter::Flip => Err(filter_disabled("flip")),
        #[cfg(feature = "filter-flop")]
        Filter::Flop => Ok(apply_flop(grid)),
        #[cfg(not(feature = "filter-flop"))]
        Filter::Flop => Err(filter_disabled("flop")),
        #[cfg(feature = "filter-rotate")]
        Filter::Rotate180 => Ok(apply_rotate180(grid)),
        #[cfg(not(feature = "filter-rotate"))]
        Filter::Rotate180 => Err(filter_disabled("rotate180")),
        #[cfg(feature = "filter-rotate")]
        Filter::RotateLeft => Ok(apply_rotate_left(grid)),
        #[cfg(not(feature = "filter-rotate"))]
        Filter::RotateLeft => Err(filter_disabled("rotateleft")),
        #[cfg(feature = "filter-rotate")]
        Filter::RotateRight => Ok(apply_rotate_right(grid)),
        #[cfg(not(feature = "filter-rotate"))]
        Filter::RotateRight => Err(filter_disabled("rotateright")),
        #[cfg(feature = "filter-border")]
        Filter::Border => Ok(apply_border(grid)),
        #[cfg(not(feature = "filter-border"))]
        Filter::Border => Err(filter_disabled("border")),
    }
}

#[allow(dead_code)]
fn filter_disabled(name: &str) -> FigletError {
    FigletError::UnknownFilter {
        name: name.to_owned(),
        available: FILTER_NAMES.iter().map(|&s| s.to_owned()).collect(),
    }
}

// ---------------------------------------------------------------------------
// Filter implementations (AD-002 — pure functions on owned RenderGrids).
// ---------------------------------------------------------------------------

/// Identity transform — the [`Filter::Nothing`] dispatch target.
///
/// Always available (no leaf gate). Returns the input unchanged.
fn apply_nothing(grid: RenderGrid) -> RenderGrid {
    grid
}

/// Trim surrounding all-blank rows and columns (T019, FR-003).
///
/// Pure function on the owned grid: clones the retained cells into a
/// fresh [`RenderGrid`] per AD-002. Returns an empty (0×0) grid when
/// the input is entirely blank.
#[cfg(feature = "filter-crop")]
fn apply_crop(grid: RenderGrid) -> RenderGrid {
    let h = grid.cells.len();
    if h == 0 || grid.cells[0].is_empty() {
        return RenderGrid::empty();
    }
    let w = grid.cells[0].len();

    // Locate the bounding box of non-blank cells. None of the four
    // indices change when every cell is blank — in that case we return
    // an empty grid.
    let mut top = h;
    let mut bottom = 0usize;
    let mut left = w;
    let mut right = 0usize;

    for (y, row) in grid.cells.iter().enumerate() {
        for (x, cell) in row.iter().enumerate() {
            if !cell.is_blank() {
                if y < top {
                    top = y;
                }
                if y > bottom {
                    bottom = y;
                }
                if x < left {
                    left = x;
                }
                if x > right {
                    right = x;
                }
            }
        }
    }

    if top == h {
        return RenderGrid::empty();
    }

    let new_h = bottom - top + 1;
    let new_w = right - left + 1;
    let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(new_h);
    for row in grid.cells.iter().skip(top).take(new_h) {
        cells.push(row[left..=right].to_vec());
    }
    RenderGrid {
        cells,
        width: new_w as u32,
        height: new_h as u32,
    }
}

/// Per-column rainbow color sweep (T020, FR-003).
///
/// Replaces each cell's foreground color with an HSV-rainbow index
/// keyed by column. Note: this is the same visual gradient produced by
/// the v0.2.x `--rainbow` CLI flag (which lives in `src/color.rs`).
/// Both paths can coexist; `--rainbow` writes SGR escapes directly to
/// stdout, while `Filter::Gay` rewrites the typed [`Cell::fg`] so
/// downstream exporters (HTML, SVG, IRC) see the same palette.
#[cfg(feature = "filter-gay")]
fn apply_gay(grid: RenderGrid) -> RenderGrid {
    let w = grid.width.max(1);
    let mut cells = grid.cells;
    for row in cells.iter_mut() {
        for (x, cell) in row.iter_mut().enumerate() {
            let hue = 360.0_f32 * (x as f32 / w as f32);
            let (r, g, b) = hsv_to_rgb(hue, 1.0, 1.0);
            cell.fg = Color::Rgb(r, g, b);
        }
    }
    RenderGrid {
        cells,
        width: grid.width,
        height: grid.height,
    }
}

/// Blue/gray metallic gradient (T021, FR-003).
///
/// Cycles a 4-step Cyan → Blue → BrightCyan → BrightBlue palette down
/// the rows so the output reads as a vertical metallic sheen. Toilet's
/// `--metal` filter uses a similar gradient; we derive the row-major
/// palette from the manpage description without consulting upstream
/// source.
#[cfg(feature = "filter-metal")]
fn apply_metal(grid: RenderGrid) -> RenderGrid {
    const PALETTE: [NamedColor; 4] = [
        NamedColor::Cyan,
        NamedColor::Blue,
        NamedColor::BrightCyan,
        NamedColor::BrightBlue,
    ];
    let mut cells = grid.cells;
    for (y, row) in cells.iter_mut().enumerate() {
        let c = PALETTE[y % PALETTE.len()];
        for cell in row.iter_mut() {
            cell.fg = Color::Named(c);
        }
    }
    RenderGrid {
        cells,
        width: grid.width,
        height: grid.height,
    }
}

/// Horizontal mirror — reverse each row (T022, FR-003).
#[cfg(feature = "filter-flip")]
fn apply_flip(grid: RenderGrid) -> RenderGrid {
    let mut cells = grid.cells;
    for row in cells.iter_mut() {
        row.reverse();
    }
    RenderGrid {
        cells,
        width: grid.width,
        height: grid.height,
    }
}

/// Vertical mirror — reverse row order (T023, FR-003).
#[cfg(feature = "filter-flop")]
fn apply_flop(grid: RenderGrid) -> RenderGrid {
    let mut cells = grid.cells;
    cells.reverse();
    RenderGrid {
        cells,
        width: grid.width,
        height: grid.height,
    }
}

/// Rotate 180° — flip + flop (T024, FR-003).
#[cfg(feature = "filter-rotate")]
fn apply_rotate180(grid: RenderGrid) -> RenderGrid {
    let mut cells = grid.cells;
    cells.reverse();
    for row in cells.iter_mut() {
        row.reverse();
    }
    RenderGrid {
        cells,
        width: grid.width,
        height: grid.height,
    }
}

/// Rotate 90° counter-clockwise — transpose then flop (T025, FR-003).
///
/// Output dimensions: `new_width = old_height`, `new_height = old_width`.
#[cfg(feature = "filter-rotate")]
fn apply_rotate_left(grid: RenderGrid) -> RenderGrid {
    let w = grid.width as usize;
    let h = grid.height as usize;
    if w == 0 || h == 0 {
        return RenderGrid::empty();
    }
    let mut new_cells: Vec<Vec<Cell>> = (0..w).map(|_| Vec::with_capacity(h)).collect();
    // For CCW rotation: new[x][h - 1 - y] = old[y][x] → row index of
    // output is `w - 1 - x_old`, column index is `y_old`. Equivalently,
    // iterate columns right-to-left, and for each column collect the
    // entire input column top-to-bottom as the new row.
    for x in (0..w).rev() {
        let row: Vec<Cell> = (0..h).map(|y| grid.cells[y][x]).collect();
        new_cells[w - 1 - x] = row;
    }
    RenderGrid {
        cells: new_cells,
        width: h as u32,
        height: w as u32,
    }
}

/// Rotate 90° clockwise — transpose then flip (T026, FR-003).
///
/// Output dimensions: `new_width = old_height`, `new_height = old_width`.
#[cfg(feature = "filter-rotate")]
fn apply_rotate_right(grid: RenderGrid) -> RenderGrid {
    let w = grid.width as usize;
    let h = grid.height as usize;
    if w == 0 || h == 0 {
        return RenderGrid::empty();
    }
    let mut new_cells: Vec<Vec<Cell>> = (0..w).map(|_| Vec::with_capacity(h)).collect();
    // For CW rotation: new[x][y] is the input column `x` read bottom-to-top.
    for (x, row_out) in new_cells.iter_mut().enumerate().take(w) {
        *row_out = (0..h).rev().map(|y| grid.cells[y][x]).collect();
    }
    RenderGrid {
        cells: new_cells,
        width: h as u32,
        height: w as u32,
    }
}

/// Draw a Unicode box-drawing border around the grid (T027, SC-001).
///
/// Adds one row/column of padding on each side, then writes
/// `┌─...─┐` / `│...│` / `└─...─┘` using single-line Unicode
/// box-drawing characters (U+2500..U+2518).
#[cfg(feature = "filter-border")]
fn apply_border(grid: RenderGrid) -> RenderGrid {
    let w = grid.width as usize;
    let h = grid.height as usize;
    let new_w = w + 2;
    let new_h = h + 2;
    let mut cells: Vec<Vec<Cell>> = Vec::with_capacity(new_h);

    // Top border row.
    let mut top = Vec::with_capacity(new_w);
    top.push(Cell::new('┌'));
    for _ in 0..w {
        top.push(Cell::new('─'));
    }
    top.push(Cell::new('┐'));
    cells.push(top);

    // Interior rows: left │, original cells, right │.
    for row in grid.cells {
        let mut new_row = Vec::with_capacity(new_w);
        new_row.push(Cell::new('│'));
        new_row.extend(row);
        new_row.push(Cell::new('│'));
        cells.push(new_row);
    }

    // Bottom border row.
    let mut bottom = Vec::with_capacity(new_w);
    bottom.push(Cell::new('└'));
    for _ in 0..w {
        bottom.push(Cell::new('─'));
    }
    bottom.push(Cell::new('┘'));
    cells.push(bottom);

    RenderGrid {
        cells,
        width: new_w as u32,
        height: new_h as u32,
    }
}

/// HSV→RGB conversion shared by [`apply_gay`]. Hue in degrees [0,360),
/// saturation and value in [0,1]. Mirrors the helper in `src/color.rs`
/// so the `Filter::Gay` path produces the same palette as the v0.2.x
/// `--rainbow` flag (see T020 rustdoc).
#[cfg(feature = "filter-gay")]
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

#[cfg(test)]
mod tests {
    use super::*;

    /// AD-011: a Cell's footprint must stay bounded. Exact size depends
    /// on the target ABI's enum layout; ≤24 bytes is comfortably within
    /// the ~16-byte design budget for 64-bit targets and absorbs any
    /// alignment padding the compiler chooses.
    #[test]
    fn cell_footprint_is_bounded() {
        assert!(
            std::mem::size_of::<Cell>() <= 24,
            "Cell size {} exceeds AD-011 budget",
            std::mem::size_of::<Cell>()
        );
    }

    #[test]
    fn parse_empty_chain_is_ok() {
        let c = FilterChain::parse("").unwrap();
        assert!(c.is_empty());
    }

    #[test]
    fn parse_single_filter() {
        let c = FilterChain::parse("crop").unwrap();
        assert_eq!(c.filters(), &[Filter::Crop]);
    }

    #[test]
    fn parse_multi_filter_chain() {
        let c = FilterChain::parse("crop:flip:border").unwrap();
        assert_eq!(c.filters(), &[Filter::Crop, Filter::Flip, Filter::Border]);
    }

    #[test]
    fn parse_empty_segment_is_unknown_filter() {
        let err = FilterChain::parse("crop::flip").unwrap_err();
        match err {
            FigletError::UnknownFilter { name, available } => {
                assert_eq!(name, "");
                assert_eq!(available.len(), 10);
            }
            other => panic!("expected UnknownFilter, got {other:?}"),
        }
    }

    #[test]
    fn parse_unknown_name_lists_available() {
        let err = FilterChain::parse("nosuchfilter").unwrap_err();
        match err {
            FigletError::UnknownFilter { name, available } => {
                assert_eq!(name, "nosuchfilter");
                assert!(available.contains(&"crop".to_string()));
                assert!(available.contains(&"nothing".to_string()));
            }
            other => panic!("expected UnknownFilter, got {other:?}"),
        }
    }

    #[test]
    fn parse_oversize_name_rejected() {
        let big = "a".repeat(MAX_FILTER_NAME_BYTES + 1);
        let err = FilterChain::parse(&big).unwrap_err();
        assert!(matches!(err, FigletError::UnknownFilter { .. }));
    }

    #[test]
    fn programmatic_push_matches_parse() {
        let manual = FilterChain::new().push(Filter::Crop).push(Filter::Flip);
        let parsed = FilterChain::parse("crop:flip").unwrap();
        assert_eq!(manual, parsed);
    }

    #[test]
    fn empty_chain_apply_is_identity() {
        let g = RenderGrid::blank(3, 2);
        let chain = FilterChain::new();
        let out = chain.apply(g.clone()).unwrap();
        assert_eq!(out, g);
    }

    #[test]
    fn nothing_filter_is_identity() {
        let g = RenderGrid::blank(3, 2);
        let chain = FilterChain::new().push(Filter::Nothing);
        let out = chain.apply(g.clone()).unwrap();
        assert_eq!(out, g);
    }

    #[cfg(feature = "filter-crop")]
    #[test]
    fn crop_trims_blank_border() {
        let mut rows = vec![vec![Cell::blank(); 4]; 4];
        rows[1][1] = Cell::new('X');
        rows[1][2] = Cell::new('Y');
        rows[2][1] = Cell::new('Z');
        let grid = RenderGrid::from_rows(rows);
        let chain = FilterChain::new().push(Filter::Crop);
        let out = chain.apply(grid).unwrap();
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 2);
        assert_eq!(out.cells[0][0].ch, 'X');
        assert_eq!(out.cells[1][0].ch, 'Z');
    }

    #[cfg(feature = "filter-crop")]
    #[test]
    fn crop_all_blank_returns_empty() {
        let grid = RenderGrid::blank(4, 4);
        let out = FilterChain::new().push(Filter::Crop).apply(grid).unwrap();
        assert_eq!(out.width, 0);
        assert_eq!(out.height, 0);
    }

    #[cfg(feature = "filter-flip")]
    #[test]
    fn flip_reverses_each_row() {
        let grid = RenderGrid::from_text_rows(&[String::from("ABCD"), String::from("1234")]);
        let out = FilterChain::new().push(Filter::Flip).apply(grid).unwrap();
        assert_eq!(out.cells[0][0].ch, 'D');
        assert_eq!(out.cells[0][3].ch, 'A');
        assert_eq!(out.cells[1][0].ch, '4');
    }

    #[cfg(feature = "filter-flop")]
    #[test]
    fn flop_reverses_row_order() {
        let grid = RenderGrid::from_text_rows(&[String::from("AAA"), String::from("BBB")]);
        let out = FilterChain::new().push(Filter::Flop).apply(grid).unwrap();
        assert_eq!(out.cells[0][0].ch, 'B');
        assert_eq!(out.cells[1][0].ch, 'A');
    }

    #[cfg(feature = "filter-rotate")]
    #[test]
    fn rotate180_inverts() {
        let grid = RenderGrid::from_text_rows(&[String::from("AB"), String::from("CD")]);
        let out = FilterChain::new()
            .push(Filter::Rotate180)
            .apply(grid)
            .unwrap();
        assert_eq!(out.cells[0][0].ch, 'D');
        assert_eq!(out.cells[0][1].ch, 'C');
        assert_eq!(out.cells[1][0].ch, 'B');
        assert_eq!(out.cells[1][1].ch, 'A');
    }

    #[cfg(feature = "filter-rotate")]
    #[test]
    fn rotate_left_swaps_dimensions() {
        let grid = RenderGrid::from_text_rows(&[String::from("ABC"), String::from("DEF")]);
        let out = FilterChain::new()
            .push(Filter::RotateLeft)
            .apply(grid)
            .unwrap();
        // Old 3x2 → new 2x3. CCW: top row of output is rightmost column of input,
        // top-to-bottom: 'C','F'.
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 3);
        assert_eq!(out.cells[0][0].ch, 'C');
        assert_eq!(out.cells[0][1].ch, 'F');
        assert_eq!(out.cells[2][0].ch, 'A');
        assert_eq!(out.cells[2][1].ch, 'D');
    }

    #[cfg(feature = "filter-rotate")]
    #[test]
    fn rotate_right_swaps_dimensions() {
        let grid = RenderGrid::from_text_rows(&[String::from("ABC"), String::from("DEF")]);
        let out = FilterChain::new()
            .push(Filter::RotateRight)
            .apply(grid)
            .unwrap();
        // Old 3x2 → new 2x3. CW: top row of output is leftmost column of input,
        // bottom-to-top: 'D','A'.
        assert_eq!(out.width, 2);
        assert_eq!(out.height, 3);
        assert_eq!(out.cells[0][0].ch, 'D');
        assert_eq!(out.cells[0][1].ch, 'A');
        assert_eq!(out.cells[2][0].ch, 'F');
        assert_eq!(out.cells[2][1].ch, 'C');
    }

    #[cfg(feature = "filter-border")]
    #[test]
    fn border_adds_one_cell_of_padding() {
        let grid = RenderGrid::from_text_rows(&[String::from("XX")]);
        let out = FilterChain::new().push(Filter::Border).apply(grid).unwrap();
        assert_eq!(out.width, 4);
        assert_eq!(out.height, 3);
        assert_eq!(out.cells[0][0].ch, '┌');
        assert_eq!(out.cells[0][3].ch, '┐');
        assert_eq!(out.cells[2][0].ch, '└');
        assert_eq!(out.cells[2][3].ch, '┘');
        assert_eq!(out.cells[1][1].ch, 'X');
    }

    #[cfg(feature = "filter-gay")]
    #[test]
    fn gay_assigns_rgb_per_column() {
        let grid = RenderGrid::from_text_rows(&[String::from("ABCD")]);
        let out = FilterChain::new().push(Filter::Gay).apply(grid).unwrap();
        // Every cell should now carry an Rgb color. The hue differs per
        // column so adjacent cells have distinct RGB triples.
        let c0 = out.cells[0][0].fg;
        let c1 = out.cells[0][1].fg;
        assert!(matches!(c0, Color::Rgb(..)));
        assert!(matches!(c1, Color::Rgb(..)));
        assert_ne!(c0, c1);
    }

    #[cfg(feature = "filter-metal")]
    #[test]
    fn metal_cycles_palette_per_row() {
        let grid = RenderGrid::from_text_rows(&[
            String::from("A"),
            String::from("B"),
            String::from("C"),
            String::from("D"),
            String::from("E"),
        ]);
        let out = FilterChain::new().push(Filter::Metal).apply(grid).unwrap();
        // Row 0 and row 4 cycle back to the same palette entry.
        assert_eq!(out.cells[0][0].fg, out.cells[4][0].fg);
        // Row 0 and row 1 differ.
        assert_ne!(out.cells[0][0].fg, out.cells[1][0].fg);
    }

    #[cfg(all(feature = "filter-flip", feature = "filter-gay"))]
    #[test]
    fn chain_order_observable_gay_then_flip() {
        // AD-009 — filter ordering is observable. gay→flip differs from
        // flip→gay because the per-column hue is computed BEFORE the
        // horizontal mirror in the first ordering.
        let grid = RenderGrid::from_text_rows(&[String::from("ABCD")]);
        let a = FilterChain::new()
            .push(Filter::Gay)
            .push(Filter::Flip)
            .apply(grid.clone())
            .unwrap();
        let b = FilterChain::new()
            .push(Filter::Flip)
            .push(Filter::Gay)
            .apply(grid)
            .unwrap();
        assert_ne!(a.cells[0][0].fg, b.cells[0][0].fg);
    }
}
