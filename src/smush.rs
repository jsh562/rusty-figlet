//! Horizontal smushing per the FIGfont 2.0 spec.
//!
//! Implements all six horizontal rules (equal, underscore, hierarchy,
//! opposite-pair, big-X, hardblank) plus the universal fallback per
//! HINT-002 and AD-005. Rule precedence is fixed at 1 → 2 → 3 → 4 → 5
//! → 6 → universal; the first applicable rule wins.

/// Rule 1 — equal character smushing.
pub const RULE_EQUAL: u8 = 0b0000_0001;
/// Rule 2 — underscore smushing.
pub const RULE_UNDERSCORE: u8 = 0b0000_0010;
/// Rule 3 — hierarchy smushing.
pub const RULE_HIERARCHY: u8 = 0b0000_0100;
/// Rule 4 — opposite-pair smushing.
pub const RULE_OPPOSITE: u8 = 0b0000_1000;
/// Rule 5 — big-X smushing.
pub const RULE_BIGX: u8 = 0b0001_0000;
/// Rule 6 — hardblank smushing.
pub const RULE_HARDBLANK: u8 = 0b0010_0000;
/// Bit 64 — horizontal smushing enabled (vs full-width).
pub const RULE_HORIZONTAL_SMUSHING: u8 = 0b0100_0000;
/// Bit 128 — horizontal kerning enabled (NOT a smush rule itself).
pub const RULE_HORIZONTAL_KERNING: u8 = 0b1000_0000;

/// Attempt to smush a pair of horizontally-adjacent glyph characters.
///
/// Returns `Some(merged_char)` when the pair can be merged under the
/// active `rules` bitmask + universal fallback; returns `None` when no
/// rule applies and smushing is disabled (the caller falls back to
/// kerning, i.e. keeps both chars).
///
/// Execution order is fixed at 1 → 2 → 3 → 4 → 5 → 6 → universal per
/// HINT-002. The first applicable rule wins.
pub fn smush_pair(left: char, right: char, rules: u8, hardblank: char) -> Option<char> {
    // A space always yields to a visible neighbor — this is the kerning
    // primitive that runs underneath every layout mode.
    if left == ' ' {
        return Some(right);
    }
    if right == ' ' {
        return Some(left);
    }

    if rules & RULE_EQUAL != 0 {
        if let Some(c) = rule_equal(left, right, hardblank) {
            return Some(c);
        }
    }
    if rules & RULE_UNDERSCORE != 0 {
        if let Some(c) = rule_underscore(left, right) {
            return Some(c);
        }
    }
    if rules & RULE_HIERARCHY != 0 {
        if let Some(c) = rule_hierarchy(left, right) {
            return Some(c);
        }
    }
    if rules & RULE_OPPOSITE != 0 {
        if let Some(c) = rule_opposite_pair(left, right) {
            return Some(c);
        }
    }
    if rules & RULE_BIGX != 0 {
        if let Some(c) = rule_big_x(left, right) {
            return Some(c);
        }
    }
    if rules & RULE_HARDBLANK != 0 {
        if let Some(c) = rule_hardblank(left, right, hardblank) {
            return Some(c);
        }
    }

    if rules & RULE_HORIZONTAL_SMUSHING != 0 {
        Some(universal(left, right, hardblank))
    } else {
        None
    }
}

fn rule_equal(left: char, right: char, hardblank: char) -> Option<char> {
    if left == right && left != hardblank {
        Some(left)
    } else {
        None
    }
}

fn rule_underscore(left: char, right: char) -> Option<char> {
    const REPLACERS: &str = "|/\\[]{}()<>";
    if left == '_' && REPLACERS.contains(right) {
        Some(right)
    } else if right == '_' && REPLACERS.contains(left) {
        Some(left)
    } else {
        None
    }
}

fn rule_hierarchy(left: char, right: char) -> Option<char> {
    // Classes from low to high precedence per the FIGfont 2.0 spec; a
    // character in a higher class replaces an adjacent character in a
    // lower class.
    const CLASSES: &[&str] = &["|", "/\\", "[]", "{}", "()", "<>"];
    let l_rank = CLASSES.iter().position(|c| c.contains(left));
    let r_rank = CLASSES.iter().position(|c| c.contains(right));
    match (l_rank, r_rank) {
        (Some(lr), Some(rr)) if lr != rr => Some(if lr > rr { left } else { right }),
        _ => None,
    }
}

fn rule_opposite_pair(left: char, right: char) -> Option<char> {
    matches!(
        (left, right),
        ('[', ']') | (']', '[') | ('{', '}') | ('}', '{') | ('(', ')') | (')', '(')
    )
    .then_some('|')
}

fn rule_big_x(left: char, right: char) -> Option<char> {
    match (left, right) {
        ('/', '\\') => Some('|'),
        ('\\', '/') => Some('Y'),
        ('>', '<') => Some('X'),
        _ => None,
    }
}

fn rule_hardblank(left: char, right: char, hardblank: char) -> Option<char> {
    if left == hardblank && right == hardblank {
        Some(hardblank)
    } else {
        None
    }
}

fn universal(left: char, right: char, hardblank: char) -> char {
    // Hardblanks dominate visible characters in universal smushing.
    // When neither char is a hardblank, the later (right) char wins.
    if left == hardblank { left } else { right }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL_RULES: u8 = RULE_EQUAL
        | RULE_UNDERSCORE
        | RULE_HIERARCHY
        | RULE_OPPOSITE
        | RULE_BIGX
        | RULE_HARDBLANK
        | RULE_HORIZONTAL_SMUSHING;

    #[test]
    fn space_yields_to_visible() {
        assert_eq!(smush_pair(' ', 'a', 0, '$'), Some('a'));
        assert_eq!(smush_pair('a', ' ', 0, '$'), Some('a'));
    }

    #[test]
    fn rule_equal_merges_same_char() {
        assert_eq!(smush_pair('|', '|', RULE_EQUAL, '$'), Some('|'));
        assert_eq!(smush_pair('|', '|', RULE_EQUAL, '|'), None); // hardblank excluded
    }

    #[test]
    fn rule_underscore_replaces_with_visible() {
        assert_eq!(smush_pair('_', '|', RULE_UNDERSCORE, '$'), Some('|'));
        assert_eq!(smush_pair('[', '_', RULE_UNDERSCORE, '$'), Some('['));
        assert_eq!(smush_pair('_', 'a', RULE_UNDERSCORE, '$'), None);
    }

    #[test]
    fn rule_opposite_pair_yields_pipe() {
        assert_eq!(smush_pair('[', ']', RULE_OPPOSITE, '$'), Some('|'));
        assert_eq!(smush_pair(')', '(', RULE_OPPOSITE, '$'), Some('|'));
    }

    #[test]
    fn rule_big_x_table() {
        assert_eq!(smush_pair('/', '\\', RULE_BIGX, '$'), Some('|'));
        assert_eq!(smush_pair('\\', '/', RULE_BIGX, '$'), Some('Y'));
        assert_eq!(smush_pair('>', '<', RULE_BIGX, '$'), Some('X'));
    }

    #[test]
    fn rule_hardblank_merges_hardblanks() {
        assert_eq!(smush_pair('$', '$', RULE_HARDBLANK, '$'), Some('$'));
    }

    #[test]
    fn no_smush_no_rules_returns_none() {
        // Neither '@' nor '#' can smush under any single-rule activation.
        assert_eq!(smush_pair('@', '#', RULE_EQUAL, '$'), None);
    }

    #[test]
    fn universal_fallback_when_smushing_enabled() {
        // No rule applies but smushing bit is set -> universal: right wins.
        assert_eq!(
            smush_pair('@', '#', RULE_HORIZONTAL_SMUSHING, '$'),
            Some('#')
        );
        // Hardblank dominates visible in universal.
        assert_eq!(
            smush_pair('$', '#', RULE_HORIZONTAL_SMUSHING, '$'),
            Some('$')
        );
    }

    #[test]
    fn precedence_rule1_over_rule3() {
        // Two equal '|' could match rule 1 (equal) OR rule 3 (hierarchy
        // returns None when ranks tie). Rule 1 wins because executed first.
        assert_eq!(smush_pair('|', '|', ALL_RULES, '$'), Some('|'));
    }

    // ====================================================================
    // Per-rule positive / negative / hardblank-adjacent triples (T039).
    // Each rule gets 3 tests: matching pair, non-matching pair, hardblank
    // adjacency. 6 rules × 3 = 18 tests beyond the basics above.
    // ====================================================================

    #[test]
    fn rule_equal_positive_letter() {
        assert_eq!(smush_pair('x', 'x', RULE_EQUAL, '$'), Some('x'));
    }
    #[test]
    fn rule_equal_negative_diff_chars() {
        assert_eq!(smush_pair('a', 'b', RULE_EQUAL, '$'), None);
    }
    #[test]
    fn rule_equal_hardblank_adjacent_abstains() {
        // Hardblank-vs-letter is not "equal".
        assert_eq!(smush_pair('$', 'x', RULE_EQUAL, '$'), None);
    }

    #[test]
    fn rule_underscore_positive_paren() {
        assert_eq!(smush_pair('_', '(', RULE_UNDERSCORE, '$'), Some('('));
    }
    #[test]
    fn rule_underscore_negative_letter() {
        assert_eq!(smush_pair('_', 'a', RULE_UNDERSCORE, '$'), None);
    }
    #[test]
    fn rule_underscore_hardblank_adjacent_abstains() {
        assert_eq!(smush_pair('_', '$', RULE_UNDERSCORE, '$'), None);
    }

    #[test]
    fn rule_hierarchy_positive_higher_class_wins() {
        // `(` is class 4, `|` is class 0; the higher-class char wins.
        assert_eq!(smush_pair('(', '|', RULE_HIERARCHY, '$'), Some('('));
    }
    #[test]
    fn rule_hierarchy_negative_same_class() {
        // Two pipes are both class 0 — rule 3 abstains (rule 1 would
        // handle this if enabled, but here only RULE_HIERARCHY is on).
        assert_eq!(smush_pair('|', '|', RULE_HIERARCHY, '$'), None);
    }
    #[test]
    fn rule_hierarchy_hardblank_adjacent_abstains() {
        assert_eq!(smush_pair('$', '|', RULE_HIERARCHY, '$'), None);
    }

    #[test]
    fn rule_opposite_positive_braces() {
        assert_eq!(smush_pair('{', '}', RULE_OPPOSITE, '$'), Some('|'));
    }
    #[test]
    fn rule_opposite_negative_same_dir() {
        assert_eq!(smush_pair('[', '[', RULE_OPPOSITE, '$'), None);
    }
    #[test]
    fn rule_opposite_hardblank_adjacent_abstains() {
        assert_eq!(smush_pair('$', '[', RULE_OPPOSITE, '$'), None);
    }

    #[test]
    fn rule_bigx_positive_diamond() {
        assert_eq!(smush_pair('/', '\\', RULE_BIGX, '$'), Some('|'));
    }
    #[test]
    fn rule_bigx_negative_non_pair() {
        assert_eq!(smush_pair('/', '/', RULE_BIGX, '$'), None);
    }
    #[test]
    fn rule_bigx_hardblank_adjacent_abstains() {
        assert_eq!(smush_pair('$', '/', RULE_BIGX, '$'), None);
    }

    #[test]
    fn rule_hardblank_positive_pair() {
        assert_eq!(smush_pair('$', '$', RULE_HARDBLANK, '$'), Some('$'));
    }
    #[test]
    fn rule_hardblank_negative_letter() {
        assert_eq!(smush_pair('a', 'a', RULE_HARDBLANK, '$'), None);
    }
    #[test]
    fn rule_hardblank_visible_paired_with_hardblank() {
        // Only hardblank+hardblank merges under rule 6.
        assert_eq!(smush_pair('$', 'a', RULE_HARDBLANK, '$'), None);
    }

    // ====================================================================
    // Cross-rule precedence integration tests (≥6 per T039).
    // ====================================================================

    #[test]
    fn cross_rule_precedence_eq_over_underscore() {
        // '_' and '_' match rule 1 (equal) before rule 2 (underscore
        // requires `_` opposite a visible bracket-class char).
        assert_eq!(
            smush_pair('_', '_', RULE_EQUAL | RULE_UNDERSCORE, '$'),
            Some('_')
        );
    }

    #[test]
    fn cross_rule_precedence_underscore_over_hierarchy() {
        // '_' + '|' eligible under rule 2 (underscore → '|') but rule 3
        // (hierarchy) treats '_' as outside the class table → abstain.
        // Rule 2 wins regardless of order because rule 3 abstains.
        assert_eq!(
            smush_pair('_', '|', RULE_UNDERSCORE | RULE_HIERARCHY, '$'),
            Some('|')
        );
    }

    #[test]
    fn cross_rule_precedence_hierarchy_over_opposite() {
        // '(' and '|' don't form an opposite pair but '(' is class 4 and
        // '|' is class 0 → rule 3 wins.
        assert_eq!(
            smush_pair('(', '|', RULE_HIERARCHY | RULE_OPPOSITE, '$'),
            Some('(')
        );
    }

    #[test]
    fn cross_rule_precedence_opposite_over_bigx() {
        // ')(' is an opposite pair (rule 4) and not a big-X pair (rule 5
        // abstains for parens). Rule 4 wins.
        assert_eq!(
            smush_pair(')', '(', RULE_OPPOSITE | RULE_BIGX, '$'),
            Some('|')
        );
    }

    #[test]
    fn cross_rule_precedence_bigx_over_hardblank() {
        // '/' + '\\' is a big-X (rule 5); rule 6 abstains (no hardblank
        // pair).
        assert_eq!(
            smush_pair('/', '\\', RULE_BIGX | RULE_HARDBLANK, '$'),
            Some('|')
        );
    }

    #[test]
    fn cross_rule_all_rules_letter_pair_yields_equal() {
        // 'a' + 'a' matches rule 1 (equal) only — every other rule
        // abstains. Equal wins per precedence order.
        assert_eq!(smush_pair('a', 'a', ALL_RULES, '$'), Some('a'));
    }
}
