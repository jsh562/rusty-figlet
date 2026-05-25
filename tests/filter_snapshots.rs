//! E012 Phase 5 — per-filter snapshot tests (T027, T028).
//!
//! One test per filter on a fixed input grid, plus chained-filter cases
//! that exercise ordering observability (AD-009) and the empty / long-
//! chain edges (AD-007). The snapshots are inline `assert_eq!` strings
//! rather than insta files because the inputs are intentionally tiny
//! (3×3 to 4×3) so failures are easy to diff in test output without
//! pulling another dev-dep into the crate.
//!
//! Gated by `--all-features` so every filter leaf is active.

#![cfg(all(
    feature = "filter-crop",
    feature = "filter-gay",
    feature = "filter-metal",
    feature = "filter-flip",
    feature = "filter-flop",
    feature = "filter-rotate",
    feature = "filter-border",
))]

use rusty_figlet::filter::{Cell, Color, Filter, FilterChain, NamedColor, RenderGrid};

fn render_chars(grid: &RenderGrid) -> String {
    grid.cells
        .iter()
        .map(|row| row.iter().map(|c| c.ch).collect::<String>())
        .collect::<Vec<_>>()
        .join("\n")
}

fn fixed_input() -> RenderGrid {
    RenderGrid::from_text_rows(&[
        String::from("ABCD"),
        String::from("EFGH"),
        String::from("IJKL"),
    ])
}

#[test]
fn nothing_is_identity() {
    let g = fixed_input();
    let out = FilterChain::new()
        .push(Filter::Nothing)
        .apply(g.clone())
        .expect("apply");
    assert_eq!(render_chars(&out), render_chars(&g));
}

#[test]
fn crop_trims_blank_border() {
    let mut rows = vec![vec![Cell::blank(); 5]; 5];
    rows[1][1] = Cell::new('X');
    rows[1][2] = Cell::new('Y');
    rows[2][1] = Cell::new('Z');
    rows[2][2] = Cell::new('W');
    let grid = RenderGrid::from_rows(rows);
    let out = FilterChain::new()
        .push(Filter::Crop)
        .apply(grid)
        .expect("apply");
    assert_eq!(render_chars(&out), "XY\nZW");
}

#[test]
fn flip_mirrors_horizontal() {
    let out = FilterChain::new()
        .push(Filter::Flip)
        .apply(fixed_input())
        .expect("apply");
    assert_eq!(render_chars(&out), "DCBA\nHGFE\nLKJI");
}

#[test]
fn flop_mirrors_vertical() {
    let out = FilterChain::new()
        .push(Filter::Flop)
        .apply(fixed_input())
        .expect("apply");
    assert_eq!(render_chars(&out), "IJKL\nEFGH\nABCD");
}

#[test]
fn rotate_180_inverts() {
    let out = FilterChain::new()
        .push(Filter::Rotate180)
        .apply(fixed_input())
        .expect("apply");
    assert_eq!(render_chars(&out), "LKJI\nHGFE\nDCBA");
}

#[test]
fn rotate_left_90() {
    let out = FilterChain::new()
        .push(Filter::RotateLeft)
        .apply(fixed_input())
        .expect("apply");
    // 4x3 → 3x4. CCW: top row = original rightmost column top-down → "DHL".
    assert_eq!(render_chars(&out), "DHL\nCGK\nBFJ\nAEI");
}

#[test]
fn rotate_right_90() {
    let out = FilterChain::new()
        .push(Filter::RotateRight)
        .apply(fixed_input())
        .expect("apply");
    // 4x3 → 3x4. CW: top row = original leftmost column bottom-up → "IEA".
    assert_eq!(render_chars(&out), "IEA\nJFB\nKGC\nLHD");
}

#[test]
fn border_wraps_box() {
    let small = RenderGrid::from_text_rows(&[String::from("HI")]);
    let out = FilterChain::new()
        .push(Filter::Border)
        .apply(small)
        .expect("apply");
    assert_eq!(render_chars(&out), "┌──┐\n│HI│\n└──┘");
}

#[test]
fn gay_applies_rainbow_per_column() {
    let out = FilterChain::new()
        .push(Filter::Gay)
        .apply(fixed_input())
        .expect("apply");
    // Glyphs unchanged.
    assert_eq!(render_chars(&out), "ABCD\nEFGH\nIJKL");
    // Each cell now carries an Rgb foreground; column 0 differs from column 1.
    let c0 = out.cells[0][0].fg;
    let c1 = out.cells[0][1].fg;
    assert!(matches!(c0, Color::Rgb(..)));
    assert!(matches!(c1, Color::Rgb(..)));
    assert_ne!(c0, c1);
    // Same column across rows shares the same hue.
    assert_eq!(out.cells[0][0].fg, out.cells[1][0].fg);
}

#[test]
fn metal_cycles_palette_per_row() {
    let out = FilterChain::new()
        .push(Filter::Metal)
        .apply(fixed_input())
        .expect("apply");
    assert_eq!(render_chars(&out), "ABCD\nEFGH\nIJKL");
    // Row 0 = Cyan, Row 1 = Blue, Row 2 = BrightCyan (4-step palette).
    assert_eq!(out.cells[0][0].fg, Color::Named(NamedColor::Cyan));
    assert_eq!(out.cells[1][0].fg, Color::Named(NamedColor::Blue));
    assert_eq!(out.cells[2][0].fg, Color::Named(NamedColor::BrightCyan));
}

// ---- Chained-filter ordering observability per AD-009 -----------------------

#[test]
fn gay_then_flip_differs_from_flip_then_gay() {
    let input = fixed_input();
    let a = FilterChain::new()
        .push(Filter::Gay)
        .push(Filter::Flip)
        .apply(input.clone())
        .expect("apply");
    let b = FilterChain::new()
        .push(Filter::Flip)
        .push(Filter::Gay)
        .apply(input)
        .expect("apply");
    // Both produce the same glyph grid (Flip + Gay are commutative on
    // characters), but the per-column rainbow assignment yields different
    // foreground colors at column 0 because Gay sees the post-flip column
    // index in path A and the pre-flip column index in path B.
    assert_eq!(render_chars(&a), render_chars(&b));
    assert_ne!(a.cells[0][0].fg, b.cells[0][0].fg);
}

#[test]
fn crop_metal_border_chain() {
    let mut rows = vec![vec![Cell::blank(); 5]; 5];
    rows[2][2] = Cell::new('X');
    let grid = RenderGrid::from_rows(rows);
    let out = FilterChain::new()
        .push(Filter::Crop)
        .push(Filter::Metal)
        .push(Filter::Border)
        .apply(grid)
        .expect("apply");
    // Crop trims to a 1x1, Metal recolors, Border wraps to 3x3.
    assert_eq!(out.width, 3);
    assert_eq!(out.height, 3);
    assert_eq!(render_chars(&out), "┌─┐\n│X│\n└─┘");
}

#[test]
fn empty_input_well_defined() {
    let g = RenderGrid::empty();
    let chain = FilterChain::new()
        .push(Filter::Flip)
        .push(Filter::Flop)
        .push(Filter::Rotate180);
    let out = chain.apply(g).expect("apply");
    assert_eq!(out.width, 0);
    assert_eq!(out.height, 0);
}

#[test]
fn long_chain_no_artificial_cap() {
    // AD-007: no upper bound on chain length. A 20-step chain of
    // alternating identity/flip filters applies cleanly.
    let mut chain = FilterChain::new();
    for i in 0..20 {
        chain = chain.push(if i % 2 == 0 {
            Filter::Nothing
        } else {
            Filter::Flip
        });
    }
    let out = chain.apply(fixed_input()).expect("apply");
    // 10 flips = identity; nothings don't change anything either.
    assert_eq!(render_chars(&out), "ABCD\nEFGH\nIJKL");
}
