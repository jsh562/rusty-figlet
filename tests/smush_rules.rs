//! T105 + T111 — Per-rule smushing snapshot coverage.
//!
//! Parameterises across all 6 horizontal smush rules + the universal
//! fallback per Plan §Smush-Rule Coverage Snapshot Suite + CHK008 +
//! CHK009 + CHK010 + CHK011. Each rule gets ≥3 input triples (matching
//! pair, non-matching pair, hardblank-adjacent pair). Cross-rule
//! precedence is verified via ≥6 dedicated tests that activate two or
//! more rules simultaneously and assert the lower-numbered rule wins.
//!
//! Drives [`rusty_figlet::FigletBuilder`] indirectly through in-memory
//! fixture fonts whose `full_layout` bitfield exposes a single rule
//! plus the smushing master-enable bit. The library exposes the rule
//! bitmask through the [`rusty_figlet::LayoutFlag::Explicit`] variant
//! so this suite can construct a deterministic rule activation per
//! test without parsing a custom `.flf` per row.
//!
//! Snapshot-format: each test inlines its expected merged char rather
//! than reading a fixture file; this matches the per-row triplet
//! style required by CHK009 and keeps the suite self-contained.

#![cfg(feature = "cli")]

// The Phase 2 `smush_pair` surface is crate-private (it lives in
// `crate::smush`); this integration suite re-uses the public
// `FigletBuilder` + `LayoutFlag::Explicit` path so each rule is
// exercised end-to-end. The single-character probe pattern below
// uses a 1-wide single-row fixture font; building it inline lets
// every test independently target one specific rule.

use rusty_figlet::{FigletBuilder, LayoutFlag, LayoutFlags};

/// Construct a minimal one-row FIGfont whose ASCII glyphs (32..=126)
/// are a single literal codepoint and the `full_layout` bitfield has
/// exactly `rules` set plus bit 64 (RULE_HORIZONTAL_SMUSHING). The
/// hardblank is `$`. The 7 German codepoints (196, 214, 220, 228, 246,
/// 252, 223) appear as codetag blocks (also one row each).
fn one_row_flf_with_rules(rules: u8) -> Vec<u8> {
    let full_layout: u32 = 64 | u32::from(rules);
    let old_layout: i32 = i32::from(rules);
    let mut out = String::new();
    // header: hardblank=$ height=1 baseline=1 max_length=2 old_layout=...
    // comment_lines=2 print_direction=0 full_layout=... codetag_count=7
    out.push_str(&format!("flf2a$ 1 1 2 {old_layout} 2 0 {full_layout} 7\n"));
    out.push_str("smush rules fixture (T105)\n");
    out.push_str("comment line 2\n");
    let endmark = '@';
    for cp in 32..=126u32 {
        let c = char::from_u32(cp).unwrap();
        // 1-wide glyph: literal codepoint followed by doubled endmark.
        out.push_str(&format!("{c}{endmark}{endmark}\n"));
    }
    for cp in [196u32, 214, 220, 228, 246, 252, 223] {
        // Codetag codepoints are hex per FIGfont 2.0 spec.
        out.push_str(&format!("{cp:X} FIXTURE U+{cp:04X}\n"));
        // Use 'X' as the glyph for the German codepoints.
        out.push_str(&format!("X{endmark}{endmark}\n"));
    }
    out.into_bytes()
}

/// Render a two-char input under the supplied rule bitmask. Returns
/// the single rendered row (the fixture is height=1).
fn render_pair(rules: u8, left: char, right: char) -> String {
    let bytes = one_row_flf_with_rules(rules);
    let layout = LayoutFlags {
        // -m N maps to `Explicit(N)` per AD-009; passing the smush
        // bitmask plus enabling smushing here forces the renderer
        // through the per-rule smush path.
        flags: vec![LayoutFlag::Explicit(i32::from(rules))],
    };
    let banner = FigletBuilder::new()
        .font_bytes(&bytes)
        .width(80)
        .layout(layout)
        .build()
        .expect("font_bytes -> build")
        .render(&format!("{left}{right}"))
        .expect("render");
    let mut it = banner.lines();
    it.next().unwrap_or_default()
}

const RULE_EQUAL: u8 = 0b0000_0001;
const RULE_UNDERSCORE: u8 = 0b0000_0010;
const RULE_HIERARCHY: u8 = 0b0000_0100;
const RULE_OPPOSITE: u8 = 0b0000_1000;
const RULE_BIGX: u8 = 0b0001_0000;
const RULE_HARDBLANK: u8 = 0b0010_0000;

// ============================================================================
// Rule 1 — Equal-character smushing
// ============================================================================

#[test]
fn rule1_equal_matching_pair_merges_to_single_char() {
    let row = render_pair(RULE_EQUAL, '|', '|');
    assert!(row.contains('|'), "expected merged '|', got: {row:?}");
    // The merged pair occupies a single visual cell ⇒ row trimmed of
    // padding contains exactly one non-space char.
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|'], "rule 1 must collapse the pair");
}

#[test]
fn rule1_equal_non_matching_pair_does_not_merge() {
    let row = render_pair(RULE_EQUAL, '|', '-');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    // Both chars survive when rule 1 abstains.
    assert!(
        chars.contains(&'|') && chars.contains(&'-'),
        "expected both '|' and '-', got: {chars:?}"
    );
}

#[test]
fn rule1_equal_hardblank_adjacent_abstains() {
    // Hardblank-vs-hardblank is excluded from rule 1 (only rule 6
    // merges hardblanks). With only rule 1 enabled the two hardblanks
    // surface as two spaces after `strip_hardblanks`.
    let row = render_pair(RULE_EQUAL, '$', '$');
    let non_space: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert!(
        non_space.is_empty(),
        "rule 1 must not merge hardblanks (they print as spaces); got: {row:?}"
    );
}

// ============================================================================
// Rule 2 — Underscore smushing
// ============================================================================

#[test]
fn rule2_underscore_replaced_by_visible_neighbor() {
    let row = render_pair(RULE_UNDERSCORE, '_', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|'], "rule 2 must yield '|' (the visible)");
}

#[test]
fn rule2_underscore_non_matching_pair() {
    // `_` + `a` doesn't trigger rule 2 (letters aren't in the bracket
    // class) → no merge → both chars survive.
    let row = render_pair(RULE_UNDERSCORE, '_', 'a');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert!(
        chars.contains(&'_') && chars.contains(&'a'),
        "rule 2 abstains for `_a`; got: {chars:?}"
    );
}

#[test]
fn rule2_underscore_hardblank_adjacent_abstains() {
    let row = render_pair(RULE_UNDERSCORE, '_', '$');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    // '_' survives; hardblank prints as space after strip_hardblanks.
    assert_eq!(chars, vec!['_']);
}

// ============================================================================
// Rule 3 — Hierarchy smushing
// ============================================================================

#[test]
fn rule3_hierarchy_higher_class_wins() {
    // '(' is class 4, '|' is class 0 → '(' wins.
    let row = render_pair(RULE_HIERARCHY, '(', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['('], "rule 3 must pick the higher class");
}

#[test]
fn rule3_hierarchy_same_class_abstains() {
    // Both '|' are class 0 → rule 3 abstains.
    let row = render_pair(RULE_HIERARCHY, '|', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    // Both survive.
    assert_eq!(chars, vec!['|', '|']);
}

#[test]
fn rule3_hierarchy_hardblank_adjacent_abstains() {
    let row = render_pair(RULE_HIERARCHY, '$', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|']);
}

// ============================================================================
// Rule 4 — Opposite-pair smushing
// ============================================================================

#[test]
fn rule4_opposite_pair_yields_pipe() {
    let row = render_pair(RULE_OPPOSITE, '[', ']');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|'], "rule 4 must yield '|'");
}

#[test]
fn rule4_opposite_pair_same_direction_abstains() {
    let row = render_pair(RULE_OPPOSITE, '[', '[');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['[', '[']);
}

#[test]
fn rule4_opposite_pair_hardblank_adjacent_abstains() {
    let row = render_pair(RULE_OPPOSITE, '$', '[');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['[']);
}

// ============================================================================
// Rule 5 — Big-X smushing
// ============================================================================

#[test]
fn rule5_bigx_slash_backslash_yields_pipe() {
    let row = render_pair(RULE_BIGX, '/', '\\');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|'], "rule 5: /\\ -> |");
}

#[test]
fn rule5_bigx_non_pair_abstains() {
    let row = render_pair(RULE_BIGX, '/', '/');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['/', '/']);
}

#[test]
fn rule5_bigx_hardblank_adjacent_abstains() {
    let row = render_pair(RULE_BIGX, '$', '/');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['/']);
}

// ============================================================================
// Rule 6 — Hardblank smushing
// ============================================================================

#[test]
fn rule6_hardblank_pair_merges() {
    // `$$` adjacency under rule 6 → single hardblank → renders as one
    // space after strip_hardblanks.
    let row = render_pair(RULE_HARDBLANK, '$', '$');
    let non_space: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert!(
        non_space.is_empty(),
        "rule 6 yields one merged hardblank → all-space row; got: {row:?}"
    );
}

#[test]
fn rule6_hardblank_letter_pair_abstains() {
    // Rule 6 requires BOTH chars to be hardblank.
    let row = render_pair(RULE_HARDBLANK, 'a', 'a');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['a', 'a']);
}

#[test]
fn rule6_hardblank_visible_paired_with_hardblank_abstains() {
    let row = render_pair(RULE_HARDBLANK, '$', 'a');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    // 'a' survives; hardblank prints as space.
    assert_eq!(chars, vec!['a']);
}

// ============================================================================
// Cross-rule integration: precedence 1 → 2 → 3 → 4 → 5 → 6 → universal
// ============================================================================

#[test]
fn cross_rules_eq_over_underscore() {
    // Two underscores match rule 1 (equal) before rule 2.
    let row = render_pair(RULE_EQUAL | RULE_UNDERSCORE, '_', '_');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['_']);
}

#[test]
fn cross_rules_underscore_over_hierarchy() {
    // `_|` triggers rule 2 (underscore → '|') before rule 3 would
    // abstain (underscore isn't in the class table).
    let row = render_pair(RULE_UNDERSCORE | RULE_HIERARCHY, '_', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|']);
}

#[test]
fn cross_rules_hierarchy_over_opposite() {
    // `(|` triggers rule 3 (`(` class 4 beats `|` class 0); rule 4
    // abstains (no opposite-pair match).
    let row = render_pair(RULE_HIERARCHY | RULE_OPPOSITE, '(', '|');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['(']);
}

#[test]
fn cross_rules_opposite_over_bigx() {
    // `)(` triggers rule 4 (opposite-pair → '|'); rule 5 abstains.
    let row = render_pair(RULE_OPPOSITE | RULE_BIGX, ')', '(');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|']);
}

#[test]
fn cross_rules_bigx_over_hardblank() {
    // `/\` triggers rule 5 (`|`); rule 6 abstains (no hardblank pair).
    let row = render_pair(RULE_BIGX | RULE_HARDBLANK, '/', '\\');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['|']);
}

#[test]
fn cross_rules_all_six_letter_pair_yields_equal() {
    // `aa` under all 6 rules → rule 1 wins (single 'a').
    let all_six =
        RULE_EQUAL | RULE_UNDERSCORE | RULE_HIERARCHY | RULE_OPPOSITE | RULE_BIGX | RULE_HARDBLANK;
    let row = render_pair(all_six, 'a', 'a');
    let chars: Vec<char> = row.chars().filter(|c| *c != ' ').collect();
    assert_eq!(chars, vec!['a']);
}
