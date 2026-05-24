# Success-Criterion Coverage Map

Per-portfolio convention this file tracks which integration test covers
which success criterion + functional requirement. Each row maps a
single SC / FR ID to one or more tests. Rows are appended as phases
complete; rows MUST NOT be removed when later phases retire a test
(annotate with the replacement test instead).

## Phase 3 — US1 Default-Font Render (T041..T055)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-001 | `tests/compat_default.rs::default_font_renders_hello` | 3 | T047. Asserts exit 0 + non-empty banner + line count >= font height. Cross-OS byte-equal verification across all 5 DDR-003 targets is CI's job. |
| SC-002 | `tests/compat_default.rs::stdin_pipe_renders_each_line_as_banner` | 3 | T048. Soft 5 s integration cap; tighter 50 ms / 1 KiB SC-002 target enforced by future micro-benchmark task. |
| FR-001 | `tests/compat_default.rs::default_font_renders_hello` | 3 | T047. `standard.flf` default-font render path. |
| FR-002 | `tests/compat_default.rs::positional_args_concatenated_with_space` | 3 | T049. Multiple positional argv joined with a single space. |
| FR-003 | `tests/compat_default.rs::stdin_lines_separated_by_blank_banner_gap` | 3 | T053. One blank line between per-stdin-line banners. |
| FR-003 | `tests/compat_default.rs::positional_arg_ignores_stdin` | 3 | T050. Positional argv overrides stdin (precedence). |
| FR-004 | `tests/compat_default.rs::stdin_cap_one_time_warning_per_process` | 3 | T052. 2 MiB stdin → exactly one cap warning. |
| FR-005 | `tests/compat_default.rs::utf8_missing_glyph_one_time_warning` | 3 | T054. CJK codepoint → one-time missing-glyph warning. |
| FR-006 | `tests/compat_default.rs::empty_input_exits_zero_no_output` | 3 | T051. Empty argv + empty stdin → exit 0, no stdout. |

## Phase 4 — US2 Font Selection (T056..T067)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-003 | `tests/compat_default.rs::all_twelve_bundled_fonts_resolve_via_dash_f` | 4 | T060. All 12 bundled fonts render via `-f <name>`. |
| SC-003 | `tests/figfont_parser.rs::all_twelve_bundled_fonts_parse_clean` | 4 | T065. All 12 bundled fonts parse cleanly via the library `FigletBuilder` path. |
| SC-004 | `tests/compat_default.rs::external_flf_loads_from_disk_via_dash_f_path` | 4 | T061. External `.flf` from disk renders byte-identical to the bundled version of the same font. |
| FR-010 | `tests/compat_default.rs::all_twelve_bundled_fonts_resolve_via_dash_f` | 4 | T060. Bundled-font lookup via `-f`. |
| FR-010 | `tests/compat_default.rs::external_flf_loads_from_disk_via_dash_f_path` | 4 | T061. External-path lookup via `-f <path>`. |
| FR-010 | `tests/compat_default.rs::exact_path_beats_dash_d_lookup` | 4 | T062. Exact-path precedence over `-d` directory search. |
| FR-010 | `tests/compat_default.rs::font_dir_flag_resolves_external_flf` | 4 | T063. `-d <dir>` resolves bare-name external font. |
| FR-010 | `tests/compat_default.rs::dash_f_with_or_without_flf_suffix` | 4 | T064. `-f <name>` and `-f <name>.flf` are equivalent. |
| FR-011 | `tests/figfont_parser.rs::all_twelve_bundled_fonts_parse_clean` | 4 | T065. 12 bundled `.flf` assets parse with sane header fields. |
| FR-012 | `tests/compat_default.rs::font_not_found_emits_clear_error_listing_searched_paths` | 4 | T062 companion. Nonexistent font surfaces `FontNotFound` with searched-paths list. |
| FR-013 | `tests/figfont_parser.rs::malformed_bad_signature_is_rejected` | 4 | T066 (1). HINT-001 case 1 — bad signature. |
| FR-013 | `tests/figfont_parser.rs::malformed_truncated_header_is_rejected` | 4 | T066 (2). HINT-001 case 2 — truncated header. |
| FR-013 | `tests/figfont_parser.rs::malformed_comment_lines_mismatch_is_rejected` | 4 | T066 (3). HINT-001 case 3 — `comment_lines` mismatch. |
| FR-013 | `tests/figfont_parser.rs::malformed_short_glyph_block_is_rejected` | 4 | T066 (4). HINT-001 case 4 — short glyph block. |
| FR-013 | `tests/figfont_parser.rs::malformed_missing_endmark_is_rejected` | 4 | T066 (5). HINT-001 case 5 — missing endmark. |
| FR-013 | `tests/figfont_parser.rs::malformed_codetag_count_divergence_is_rejected` | 4 | T066 (6). HINT-001 case 6 — `codetag_count` divergence. |

## Phase 5 — US3 Strict-Compat Drop-In (T068..T090)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-005 | (DEFERRED) `tests/compat_strict.rs::strict_byte_equal_60_base_snapshots` | 5 | T086 DEFERRED pending upstream snapshot capture (T085) on Linux host. |
| SC-005 | (DEFERRED) `tests/compat_strict.rs::strict_byte_equal_20_layout_perm_snapshots` | 5 | T087 DEFERRED pending T085. |
| SC-006 | (DEFERRED) `tests/compat_strict.rs::strict_excluded_flag_byte_equal_stderr` | 5 | T089 DEFERRED pending excluded-flag stderr snapshots (T088). |
| SC-006 | `tests/compat_strict.rs::strict_rejects_short_L` | 5 | T077. Format-equivalence verified via in-binary `figlet:` → `rusty-figlet:` substitution; byte-equal upstream snapshot deferred to T088 + T089. |
| SC-007 | `tests/compat_strict.rs::strict_activates_via_flag` | 5 | T075a. `--strict` flag activation. |
| SC-007 | `tests/compat_strict.rs::strict_activates_via_env_var` | 5 | T075b. `RUSTY_FIGLET_STRICT=1` env-var activation. |
| SC-007 | `tests/compat_strict.rs::strict_activates_via_argv0_then_rejects_excluded_flag` | 5 | T075c. argv[0] auto-detect exercised via env-var path (argv[0]=`figlet` unit-tested in `src/mode.rs::tests::argv0_basename_figlet_returns_strict`). |
| SC-007 | `tests/compat_strict.rs::no_strict_overrides_env` | 5 | T076a. `--no-strict` overrides env. |
| SC-007 | `tests/compat_strict.rs::last_wins_strict_then_no_strict_yields_default` | 5 | T076c. `--strict --no-strict` → Default (Clarifications Q8). |
| SC-007 | `tests/compat_strict.rs::last_wins_no_strict_then_strict_yields_strict` | 5 | T076d. `--no-strict --strict` → Strict (Clarifications Q8). |
| SC-014 | `tests/compat_strict.rs::strict_rejects_color_long_flag` | 5 | T081a. `--color=always` rejected under Strict. |
| SC-014 | `tests/compat_strict.rs::strict_rejects_rainbow_long_flag` | 5 | T081b. `--rainbow` rejected under Strict. |
| SC-014 | `tests/compat_strict.rs::strict_rejects_completions_subcommand` | 5 | T084. `completions <shell>` rejected under Strict (FR-063 + US7 AS3). |
| FR-040 | `tests/compat_strict.rs::strict_activates_via_flag` | 5 | T075a. Flag activation. |
| FR-040 | `tests/compat_strict.rs::strict_activates_via_env_var` | 5 | T075b. Env-var activation. |
| FR-040 | `tests/compat_strict.rs::no_strict_overrides_env` | 5 | T076a. `--no-strict` precedence over env. |
| FR-040 | `tests/compat_strict.rs::last_wins_strict_then_no_strict_yields_default` | 5 | T076c. Clarifications Q8 last-wins. |
| FR-040 | `tests/compat_strict.rs::last_wins_no_strict_then_strict_yields_strict` | 5 | T076d. Clarifications Q8 last-wins. |
| FR-041 | (DEFERRED) `tests/compat_strict.rs::strict_byte_equal_60_base_snapshots` | 5 | T086 DEFERRED. Format-equivalence covered by behavioural tests below. |
| FR-042 | `tests/compat_strict.rs::strict_rejects_short_L` | 5 | T077. Byte-equal `invalid option -- 'L'` (modulo `rusty-figlet:` substitution). |
| FR-042 | `tests/compat_strict.rs::strict_rejects_short_R` | 5 | T078a. |
| FR-042 | `tests/compat_strict.rs::strict_rejects_short_I` | 5 | T078b. |
| FR-042 | `tests/compat_strict.rs::strict_rejects_short_N` | 5 | T078c. |
| FR-042 | `tests/compat_strict.rs::strict_rejects_short_C` | 5 | T079. `-C` excluded in Strict only (Default warns per FR-046). |
| FR-043 | `tests/compat_strict.rs::strict_rejects_long_info_dump` | 5 | T080. Byte-equal `unrecognized option '--info-dump'`. |
| FR-043 | `tests/compat_strict.rs::strict_rejects_long_no_controlfile` | 5 | T080 companion. `--no-controlfile` rejected. |
| FR-044 | `tests/compat_strict.rs::strict_latin1_clamp_passes_low_bytes_through` | 5 | T082a. Low ASCII bytes round-trip cleanly. |
| FR-044 | `tests/compat_strict.rs::strict_latin1_clamp_replaces_multibyte_with_placeholder` | 5 | T082b. CJK input → `?` placeholder under Strict. |
| FR-044 | `tests/compat_strict.rs::lib_clamp_input_latin1_round_trip` | 5 | T082c. Library API `clamp_input_latin1` unit test. |
| FR-045 | `tests/compat_strict.rs::strict_rejects_color_long_flag` | 5 | T081a. |
| FR-045 | `tests/compat_strict.rs::strict_rejects_rainbow_long_flag` | 5 | T081b. |
| FR-046 | `tests/compat_strict.rs::strict_rejects_short_C` | 5 | T079. `-C` excluded in Strict; Default warn-and-ignore covered by T074 main.rs `warn_control_file_ignored`. |

## Phase 6 — US4 Programmatic Library API (T091..T102)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-008 | `tests/library_api.rs::default_features_off_excludes_cli_deps` | 6 | T094. Shells `cargo tree --no-default-features --prefix none --edges normal` and asserts absence of `clap`/`clap_complete`/`anstyle`/`termcolor`/`terminal_size` per HINT-007. |
| SC-009 | `tests/library_api.rs` (compile-time `assert_impl_all!`) | 6 | T095. `FigletBuilder`/`Figlet`/`Banner` are `Send + Sync`; `FigletError` is also `'static` per Clarifications 2026-05-23 Q2 — verified by the dedicated `figlet_error_is_static` test. |
| SC-010 | `src/lib.rs` doctests (per public type) | 6 | T093. Doctests on `FigletBuilder`, `Figlet`, `Banner`, `Font`, `FigletError`, `CompatibilityMode`, `Justify`. Closure verified by `tests/missing_docs.rs::cargo_test_doc_all_doctests_pass` (T138, Phase 10). |
| SC-017 | `src/lib.rs` (`#![deny(missing_docs)]` at crate root) | 6 | T024. Compile-fail gate for any undocumented public item. Polish-phase verification by `tests/missing_docs.rs::cargo_doc_no_deps_succeeds_with_deny_missing_docs` (T138). |
| SC-018 | `tests/library_api.rs::font_bytes_renders_with_zero_fs_calls` | 6 | T097. `FigletBuilder::font_bytes(include_bytes!(...))` path renders end-to-end without any `std::fs` calls. |
| SC-019 | (DEFERRED) `cargo binstall rusty-figlet` post-publish smoke install | Release | T156 DEFERRED — requires post-tag environment. Verified via `release.yml` post-publish smoke job once the tag lands. |
| FR-050 | `tests/library_api.rs::figlet_builder_fluent_chain_returns_self` | 6 | T101. `FigletBuilder::new()` sole construction entry; chain methods return `Self`; terminal `build()` returns `Result<Figlet, FigletError>`. |
| FR-051 | `tests/library_api.rs::default_features_off_excludes_cli_deps` | 6 | T094. Two-config feature matrix (cli-on default vs library-only). |
| FR-052 | `tests/library_api.rs::font_bytes_renders_with_zero_fs_calls` | 6 | T092 + T097. In-memory `.flf` byte slice path consumed by `parse_bytes` at `build()` time with zero `std::fs` calls. |
| FR-053 | `tests/library_api.rs::figlet_builder_render_returns_lazy_banner` | 6 | T100. `Banner::lines()` lazy iterator advances one row at a time. |
| FR-053 | `tests/library_api.rs::banner_display_matches_lines_iterator_loop` | 6 | T101. `impl Display for Banner` drives the same lazy iterator. |
| FR-056 | `tests/library_api.rs::font_bytes_matches_bundled_lookup` | 6 | T097 (companion). In-memory parser path produces byte-equal output to the bundled-lookup path. |

## Phase 7 — US5 Layout / Width / Smushing (T103..T119)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-011 | `tests/compat_default.rs::width_60_center_lines_le_60_visually_centered` | 7 | T110. `-w 60 -c "X"` produces lines ≤ 60 cols with leading-whitespace centering. |
| SC-012 | `tests/smush_rules.rs::rule1_equal_matching_pair_merges_to_single_char` | 7 | T111. Rule 1 (equal) per-rule coverage. Other rule coverage rows below. |
| SC-012 | `tests/smush_rules.rs::rule2_underscore_replaced_by_visible_neighbor` | 7 | T111. Rule 2 (underscore). |
| SC-012 | `tests/smush_rules.rs::rule3_hierarchy_higher_class_wins` | 7 | T111. Rule 3 (hierarchy). |
| SC-012 | `tests/smush_rules.rs::rule4_opposite_pair_yields_pipe` | 7 | T111. Rule 4 (opposite-pair). |
| SC-012 | `tests/smush_rules.rs::rule5_bigx_slash_backslash_yields_pipe` | 7 | T111. Rule 5 (big-X). |
| SC-012 | `tests/smush_rules.rs::rule6_hardblank_pair_merges` | 7 | T111. Rule 6 (hardblank). |
| SC-012 | `tests/smush_rules.rs::cross_rules_eq_over_underscore` | 7 | T111. Cross-rule precedence 1 > 2. |
| SC-012 | `tests/smush_rules.rs::cross_rules_underscore_over_hierarchy` | 7 | T111. Cross-rule precedence 2 > 3. |
| SC-012 | `tests/smush_rules.rs::cross_rules_hierarchy_over_opposite` | 7 | T111. Cross-rule precedence 3 > 4. |
| SC-012 | `tests/smush_rules.rs::cross_rules_opposite_over_bigx` | 7 | T111. Cross-rule precedence 4 > 5. |
| SC-012 | `tests/smush_rules.rs::cross_rules_bigx_over_hardblank` | 7 | T111. Cross-rule precedence 5 > 6. |
| SC-012 | `tests/smush_rules.rs::cross_rules_all_six_letter_pair_yields_equal` | 7 | T111. Full-rule activation (`aa` → rule 1 wins). |
| FR-020 | `tests/compat_default.rs::width_60_center_lines_le_60_visually_centered` | 7 | T110. `-w` budget enforcement. |
| FR-020 | `tests/width_precedence.rs::ad010_precedence_ladder_explicit_w_wins` | 7 | T112. Explicit `-w` precedence ladder root. |
| FR-020 | `tests/width_precedence.rs::ad010_precedence_ladder_default_fallback_is_80` | 7 | T112. Default 80-col fallback. |
| FR-020 | `tests/width_precedence.rs::ad010_precedence_explicit_w_overrides_t` | 7 | T112. `-w` beats `-t` regardless of order. |
| FR-021 | `tests/width_precedence.rs::ad010_precedence_ladder_columns_env_when_t_no_terminal` | 7 | T112. `-t` consults `COLUMNS` when `terminal_size_of(stdout)` returns None (Windows stdout-piped path). |
| FR-021 | `tests/width_precedence.rs::default_vs_strict_t_auto_apply` | 7 | T113. Default vs Strict auto-`-t` policy difference per HINT-005 (verified via piped-stdout substitute path; tty path lives in `src/width.rs::tests::strict_does_not_auto_apply_t`). |
| FR-022 | `tests/compat_default.rs::justify_flags_last_wins` | 7 | T115. `-c -l -r` → right wins; `-r -c` → center. |
| FR-023 | `tests/compat_default.rs::layout_class_flags_last_wins` | 7 | T114. `-k`/`-W`/`-S`/`-s`/`-o`/`-m` last-wins across 6 combos. |
| FR-023 | `tests/compat_default.rs::dash_m_explicit_layout_bitfield` | 7 | T118. `-m 0` / `-m 24` / `-m 63` explicit bitfield. |
| FR-024 | `tests/smush_rules.rs::*` | 7 | T105 + T111. All 24 smush_rules tests cover the 6 rules + universal + cross-rule precedence. |
| FR-025 | `tests/compat_default.rs::over_width_word_warns_once_per_process` | 7 | T116. Single over-width word → one stderr warning + full-glyph render. |
| FR-026 | `tests/compat_default.rs::paragraph_mode_concatenates_consecutive_lines` | 7 | T117. `-p` joins consecutive non-empty lines; `-n` keeps each separate. |

## Phase 8 — US6 Color and Rainbow Output (T120..T130)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-013 | `tests/compat_default.rs::rainbow_emits_24bit_ansi_when_color_always` | 8 | T124. `--rainbow --color=always` emits `\x1b[38;2;R;G;Bm`; `--color=never` is byte-identical to plain rendering. |
| SC-013 | `tests/compat_default.rs::no_color_env_suppresses_regardless_of_flag` | 8 | T125. `NO_COLOR=1` + `--color=always --rainbow` → no escapes; bytes match `--color=never`. |
| FR-030 | `tests/compat_default.rs::color_auto_no_escapes_on_non_tty` | 8 | T126. `--color=auto` over piped stdout suppresses escapes (non-TTY auto path). |
| FR-030 | `tests/compat_default.rs::color_always_overrides_tty_detection` | 8 | T127. `--color=always` emits escapes regardless of TTY; `--color=never` suppresses. |
| FR-030 | `tests/compat_default.rs::default_mode_accepts_color_and_rainbow_flags` | 8 | T129 (default-mode permissibility companion). Confirms FR-045 default-vs-strict dichotomy (Strict rejects per T081). |
| FR-031 | `tests/compat_default.rs::rainbow_emits_24bit_ansi_when_color_always` | 8 | T124. Per-column 24-bit ANSI gradient. |
| FR-031 | `tests/compat_default.rs::rainbow_gradient_spans_banner_width_not_w_budget` | 8 | T128. Hue cycles across actual banner width (HINT-006), not `-w 200` budget. |
| FR-031 | `src/color.rs::tests::rainbow_palette_length_matches_width` | 8 | T121. Palette length contract (unit). |
| FR-032 | `tests/compat_default.rs::no_color_env_suppresses_regardless_of_flag` | 8 | T125. NO_COLOR precedence over `--color=always`. |
| FR-032 | `src/color.rs::tests::no_color_suppresses_always` | 8 | T120. `should_color(Always, NO_COLOR=true, _) == false` (unit). |
| Plan-color-test | `tests/color_isolation.rs::no_color_test_isolation_raii_contract` | 8 | T129. RAII env scope guard contract — sequential guards do not bleed values; drop restores prior value. |

## Phase 9 — US7 Shell Completions with Drift Gate (T131..T137)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| SC-014 | `tests/completions_drift.rs::strict_mode_rejects_completions_subcommand` | 9 | T136. `rusty-figlet --strict completions bash` exits 2 with upstream-format rejection; no completion-script bytes on stdout. Cross-refs T084. |
| SC-015 | `tests/completions_drift.rs::drift_bash` | 9 | T134. Byte-equal drift gate against `completions/rusty-figlet.bash`. |
| SC-015 | `tests/completions_drift.rs::drift_zsh` | 9 | T134. Byte-equal drift gate against `completions/_rusty-figlet`. |
| SC-015 | `tests/completions_drift.rs::drift_fish` | 9 | T134. Byte-equal drift gate against `completions/rusty-figlet.fish`. |
| SC-015 | `tests/completions_drift.rs::drift_powershell` | 9 | T134. Byte-equal drift gate against `completions/rusty-figlet.ps1`. |
| SC-016 | `completions/rusty-figlet.bash` (committed) | 9 | T132. Pre-generated bash completion script ships in release tarballs (T133 / release.yml `generate-completions` job). |
| SC-016 | `completions/_rusty-figlet` (committed) | 9 | T132. Pre-generated zsh completion script. |
| SC-016 | `completions/rusty-figlet.fish` (committed) | 9 | T132. Pre-generated fish completion script. |
| SC-016 | `completions/rusty-figlet.ps1` (committed) | 9 | T132. Pre-generated PowerShell completion script. |
| SC-016 | `.github/workflows/release.yml::generate-completions` | 9 | T133. Release tarball includes the four completion files alongside the binary per FR-060 + SC-016. |
| FR-060 | `tests/completions_drift.rs::drift_bash` | 9 | T131 + T134. `completions <shell>` subcommand emits a bash script via `clap_complete::generate`. |
| FR-060 | `tests/completions_drift.rs::drift_zsh` | 9 | T131 + T134. |
| FR-060 | `tests/completions_drift.rs::drift_fish` | 9 | T131 + T134. |
| FR-060 | `tests/completions_drift.rs::drift_powershell` | 9 | T131 + T134. |
| FR-060 | `tests/completions_drift.rs::bash_completion_is_structurally_complete` | 9 | T135. Structural completeness — `complete -F _rusty-figlet` registration, `--font`/`-f`/`--fontdir`/`-d`/`completions` + four shells listed. (`--font` is `Option<String>`, so clap_complete emits `compgen -f` rather than the 12 bundled-font names verbatim — assertion is structural per the developer note in task T135.) |

## Phase 10 — Polish (T138..T149)

| ID | Covered By | Phase | Notes |
|----|------------|-------|-------|
| FR-055 | `tests/missing_docs.rs::cargo_doc_no_deps_succeeds_with_deny_missing_docs` | 10 | T138. `cargo doc --no-deps` runs cleanly under both `--no-default-features` and `--all-features`; `#![deny(missing_docs)]` at crate root gates undocumented public items at compile time. |
| SC-010 | `tests/missing_docs.rs::cargo_test_doc_all_doctests_pass` | 10 | T138. `cargo test --doc --all-features` succeeds — all doctests pass (≥1 per public type: `FigletBuilder`, `Figlet`, `Banner`, `Font`, `FigletError`, `CompatibilityMode`, `Justify`). |
| SC-017 | `tests/missing_docs.rs::cargo_doc_no_deps_succeeds_with_deny_missing_docs` | 10 | T138. Compile-time enforcement of `#![deny(missing_docs)]` (per SC-017). |

## Test-Isolation Policy

Per `docs/DESIGN.md` §Test Isolation, every integration test:

- Owns a freshly-constructed `tempfile::TempDir` via the `sandbox()` helper from `tests/common/mod.rs`.
- MUST NOT write to relative paths.
- MUST NOT write under `$HOME`.
- MUST NOT share a global mutable temp directory across tests.
- Wraps `NO_COLOR` / `COLUMNS` / `RUSTY_FIGLET_STRICT` / argv0 mutations in a per-test RAII `env_guard` so values do NOT leak across tests.
- Supports concurrent `cargo test -- --test-threads=N` for any N. The CI matrix exercises the full DDR-003 cross-compile target set at the default thread count.

