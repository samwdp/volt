# Graph Report - volt  (2026-07-28)

## Corpus Check
- 245 files · ~612,371 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8966 nodes · 36323 edges · 311 communities (298 shown, 13 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 2969 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `573e0717`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- String
- LspClientError
- shell/tests.rs
- .new
- ShellError
- user/lib.rs
- .new
- EditorRuntime
- .from
- render.rs
- Result
- PluginPackage
- Vec
- TextBuffer
- LanguageServerSpec
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapError
- calculator.rs
- editor-picker/src/lib.rs
- .new
- window_effects.rs
- treesitter_install.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- treesittercontext_ghosttext.rs
- state.rs
- Section
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- Self
- ShellBuffer
- Self
- AbiOilFeatureSpec
- String
- shell/mod.rs
- ShellUiState
- AbiGitFeatureSpec
- Vec
- AbiContextHelpSpec
- client.rs
- HeaderlineTestUserLibrary
- String
- KeymapScope
- UserLibrary
- render_buffer_with_view_state
- .len
- UserLibraryModule
- Option
- RVec
- shell/git.rs
- Result
- Self
- LspNotification
- AbiKeymapConfig
- state_with_user_library
- SyntaxRegistry
- .default
- String
- DebugConfiguration
- capture_mappings
- String
- theme.rs
- .send
- .new
- LineCharMap
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- sdk/src/lib.rs
- AcpManager
- FontSet
- Diagnostic
- LanguageServerSession
- volt/src/main.rs
- .new
- draw_diagnostic_underlines_for_segment
- .new
- .from_grammar
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- editor-path/src/lib.rs
- Self
- PickerSession
- WorkspaceConfigurationValue
- shell/browser.rs
- main
- TextPoint
- common.rs
- PickerItem
- editor-lsp/src/lib.rs
- editor-buffer/src/lib.rs
- Option
- String
- PickerOverlay
- BufferId
- .refresh_pdf_view
- Option
- resolve_picker_extra
- PluginKeyBinding
- .new
- String
- PluginCommand
- DbService
- process_supervisor.rs
- editor-db/src/lib.rs
- String
- buffer_is_git_status
- Vec
- GitSummaryState
- statusline.rs
- TerminalRenderSnapshot
- Result
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- oil.rs
- key_sequence.rs
- browser_host.rs
- .new_with_secret_store
- editor-icons/src/lib.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- .path
- DbSessionId
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- Option
- JobSpec
- browser_sync_plan
- standalone_user_manifest.rs
- TextRange
- .get
- OilDefaultsSection
- LspCodeAction
- LspLocation
- Vec
- db.rs
- lsp.rs
- .char_count
- LspLogEntry
- AcpPickerItemSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- clipboard.rs
- AbiPickerTruncateStrategy
- setup_standalone_user_repository
- treesittercontext_shared.rs
- ServiceRegistry
- editor-syntax/src/lib.rs
- String
- user/terminal.rs
- build_output.rs
- build_headerline_lines
- AbiSectionTree
- ancestor_contexts_for_cursor
- Self
- JobResult
- markdown.rs
- user/browser.rs
- .new
- .oil_keybindings
- `user`
- shell/acp.rs
- spawn_terminal_reader
- I
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- panic_payload_message
- Workspace Issues Plugin
- Workspace Switching and Marks
- Workspace Dashboard Worktree Remove + Picker Extra Keybinds
- syntax_language
- .oil_directory_sections
- Database Explorer PRD
- .next_token
- bash.rs
- clojure.rs
- elixir.rs
- graphql.rs
- hcl.rs
- java.rs
- kotlin.rs
- latex.rs
- lua.rs
- nix.rs
- perl.rs
- php.rs
- proto.rs
- r.rs
- ruby.rs
- scala.rs
- solidity.rs
- swift.rs
- lang/vim.rs
- xml.rs
- cargo
- Language
- choose_permission_outcome
- Domain Docs
- Issue tracker: Local Markdown
- syntax_language
- syntax_language
- package
- Issue Store and Create
- Status commands
- Capture focused file
- Async Capture on save
- Issue Board
- Place and jump to code
- Open Issue from Code Reference
- Issue Scan
- 01 — Ambiguous Prefix Timeout
- 03 — Cycle Project Workspaces
- 04 — Mark List Management
- 05 — Marked Workspace Slot Jumps
- debug_adapters
- syntax_language
- 02 — Minor Mode Keybinding Precedence
- package
- load
- VimActionContext
- 01 — Picker Extra Keybinds (dispatch + provider API)
- 02 — Migrate QuickFix Ctrl+q onto provider extras
- package
- CLAUDE.md
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- 03-worktree-remove-command.md
- 04-dashboard-ctrl-d.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 738 edges
2. `ShellBuffer` - 362 edges
3. `shell_ui_mut()` - 324 edges
4. `register_shell_hooks()` - 255 edges
5. `shell_ui()` - 209 edges
6. `ShellError` - 181 edges
7. `shell_buffer()` - 177 edges
8. `shell_buffer_mut()` - 174 edges
9. `TextBuffer` - 166 edges
10. `ShellUiState` - 155 edges

## Surprising Connections (you probably didn't know these)
- `discover_projects()` --calls--> `workspace_project_picker_items()`  [INFERRED]
  crates/editor-fs/src/lib.rs → user/workspace.rs
- `discover_projects()` --calls--> `workspace_switch_picker_items()`  [INFERRED]
  crates/editor-fs/src/lib.rs → user/workspace.rs
- `parse_stash_list()` --calls--> `stashes_display_compact_indices()`  [INFERRED]
  crates/editor-git/src/lib.rs → user/git.rs
- `list_repository_files()` --calls--> `workspace_file_picker_items()`  [INFERRED]
  crates/editor-git/src/lib.rs → user/workspace.rs
- `parse_status()` --calls--> `status_entries_and_untracked_items_omit_status_words()`  [INFERRED]
  crates/editor-git/src/lib.rs → user/git.rs

## Import Cycles
- None detected.

## Communities (311 total, 13 thin omitted)

### Community 0 - "String"
Cohesion: 0.05
Nodes (177): Cow, write_system_clipboard(), yank_from_clipboard_text(), yank_to_clipboard_text(), accept_autocomplete(), activate_db_browser_line(), active_directory_root(), active_shell_buffer_has_input() (+169 more)

### Community 1 - "LspClientError"
Cohesion: 0.07
Nodes (35): CodeActionParams, code_action_params(), code_action_params_use_flattened_lsp_shape(), inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager (+27 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.04
Nodes (47): load_font_set(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage(), contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size() (+39 more)

### Community 3 - ".new"
Cohesion: 0.10
Nodes (79): line_ranges_and_char_searches_resolve_expected_points(), move_word_forward_advances_to_the_next_word(), word_motions_treat_punctuation_runs_as_words(), vim_search_entries_trim_whitespace_from_labels(), Self, autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens() (+71 more)

### Community 4 - "ShellError"
Cohesion: 0.04
Nodes (55): Display, Error, From, ShellError, clear_key_sequence(), active_buffer_event_context(), active_runtime_surface(), alt_mod() (+47 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (113): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit(), exported_autocomplete_token_icon() (+105 more)

### Community 6 - ".new"
Cohesion: 0.13
Nodes (57): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+49 more)

### Community 7 - "EditorRuntime"
Cohesion: 0.07
Nodes (90): EditorRuntime, Default, active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), diff_git_commit_at_point(), diff_git_stash_at_point(), ensure_no_rebase_in_progress() (+82 more)

### Community 8 - ".from"
Cohesion: 0.05
Nodes (50): main(), block_comment_toggle_removal_lens(), comment_toggle_removal_len(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GitStashEntry, main(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers() (+42 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (103): advance_point_by_text(), acp_prefix_columns(), acp_spinner_frame(), adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines(), autocomplete_visible_start(), buffer_point_at_screen() (+95 more)

### Community 10 - "Result"
Cohesion: 0.14
Nodes (26): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_submitted(), acp_pick_session() (+18 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (43): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+35 more)

### Community 12 - "Vec"
Cohesion: 0.10
Nodes (22): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, GhostTextLine, GitCommandBinding (+14 more)

### Community 13 - "TextBuffer"
Cohesion: 0.09
Nodes (10): BufferStats, large_buffers_expose_line_windows_without_full_materialization(), Default, String, Vec, TextBuffer, trimmed_line(), visible_line_len() (+2 more)

### Community 14 - "LanguageServerSpec"
Cohesion: 0.16
Nodes (7): LanguageServerSpec, Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (25): AlacrittyEvent, Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc (+17 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.10
Nodes (41): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+33 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.06
Nodes (36): configure_background_command(), detect_in_progress(), git_available(), GitLogEntry, GitStashEntry, GitStatusError, GitStatusSnapshot, list_repository_files() (+28 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.05
Nodes (114): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+106 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.04
Nodes (16): DynamicUserLibrary, AcpClient, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig (+8 more)

### Community 20 - "HookBus"
Cohesion: 0.07
Nodes (24): HookBus, HookDefinition, HookError, HookEvent, HookSubscription, BTreeMap, BufferId, Default (+16 more)

### Community 21 - "EditorModel"
Cohesion: 0.07
Nodes (26): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+18 more)

### Community 22 - "KeymapError"
Cohesion: 0.15
Nodes (22): autocomplete_overrides_workspace_while_active(), ChordModifier, duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeymapError, normalize_chord_token(), normalize_delimited_token() (+14 more)

### Community 23 - "calculator.rs"
Cohesion: 0.08
Nodes (32): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+24 more)

### Community 24 - "editor-picker/src/lib.rs"
Cohesion: 0.13
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), is_match_boundary(), is_match_end_boundary() (+9 more)

### Community 25 - ".new"
Cohesion: 0.16
Nodes (22): browser_host_event_for_ipc(), BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, BrowserSyncPlan, DesktopBrowserHostService (+14 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (45): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+37 more)

### Community 27 - "treesitter_install.rs"
Cohesion: 0.12
Nodes (55): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands() (+47 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.07
Nodes (56): centered_rect(), default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names() (+48 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.08
Nodes (27): RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind, HoverProviderSpec (+19 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 32 - "state.rs"
Cohesion: 0.07
Nodes (37): formatter_for_path(), formatter_registry(), open_vim_search_prompt(), PendingVimSearchRequest, pick_search_selection_index(), reverse_find_kind(), reverse_search_direction(), VimSearchMatch (+29 more)

### Community 33 - "Section"
Cohesion: 0.14
Nodes (14): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+6 more)

### Community 34 - "shell/pdf.rs"
Cohesion: 0.12
Nodes (44): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_delete_page(), pdf_fit_mode_label(), pdf_header_lines() (+36 more)

### Community 35 - "AutocompleteProviderConfig"
Cohesion: 0.22
Nodes (14): AutocompleteProviderConfig, backends(), hook_command(), lsp_kind_icon(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping() (+6 more)

### Community 36 - "compile.rs"
Cohesion: 0.22
Nodes (13): compile_command_emits_run_command_hook(), compile_package_exports_compile_and_recompile_commands(), compile_package_exports_global_keybindings(), default_build_command(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only() (+5 more)

### Community 37 - "HoverProviderConfig"
Cohesion: 0.25
Nodes (12): hook_command(), HoverProviderConfig, package(), package_exports_hover_commands_and_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping(), HoverProviderTopic, Into (+4 more)

### Community 38 - "script.js"
Cohesion: 0.15
Nodes (9): copyButtons, copyText(), initAutoCodeCopy(), navLinks, navToggle, pageSidebar, revealItems, sections (+1 more)

### Community 40 - "Self"
Cohesion: 0.15
Nodes (14): ConfigPickerTruncateStrategy, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), KeymapSection, PaneSection, PickerTruncateStrategy (+6 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.03
Nodes (75): acp_tool_call_from_partial_update(), advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_line_indent(), apply_markdown_table_update(), apply_multicursor_delete(), apply_multicursor_insert_text(), apply_operator_motion() (+67 more)

### Community 42 - "Self"
Cohesion: 0.06
Nodes (44): exported_pdf_open_mode(), PdfOpenMode, AbiCaptureThemeMapping, AbiGrammarSource, AbiIconFontCategory, AbiLanguageConfiguration, AbiLanguageServerRootStrategy, AbiLspDiagnosticsInfo (+36 more)

### Community 43 - "AbiOilFeatureSpec"
Cohesion: 0.19
Nodes (9): AbiOilDefaults, AbiOilFeatureSpec, AbiOilSortMode, OilDefaults, OilFeatureSpec, OilSortMode, OilDefaults, OilFeatureSpec (+1 more)

### Community 44 - "String"
Cohesion: 0.06
Nodes (42): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, documentation_lines(), explicit_windows_env_value(), hover_marked_string() (+34 more)

### Community 45 - "shell/mod.rs"
Cohesion: 0.02
Nodes (259): absolute_path_hint(), acp_decode_image(), active_buffer_revision_key(), active_project_workspace_root(), active_shell_workspace_id(), active_theme_state_path(), ActiveTypingFrameProfile, adjust_tag_child_indent() (+251 more)

### Community 46 - "ShellUiState"
Cohesion: 0.04
Nodes (87): active_lsp_buffer_context(), active_lsp_code_action_range(), active_lsp_workspace_loaded(), active_runtime_buffer(), apply_lsp_text_edits(), apply_pending_lsp_state(), apply_sqls_workspace_settings_for_active_buffer_context(), buffer_is_oil_preview() (+79 more)

### Community 47 - "AbiGitFeatureSpec"
Cohesion: 0.14
Nodes (13): GitCommandBinding, GitPrefixBinding, exported_git_prefix_for_chord(), AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding (+5 more)

### Community 48 - "Vec"
Cohesion: 0.03
Nodes (113): acp_build_output_lines(), acp_build_plan_lines(), acp_icon_segment(), acp_multiline_text_lines(), acp_padding_prefix(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row() (+105 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 50 - "client.rs"
Cohesion: 0.04
Nodes (78): ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+70 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (34): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc, AutocompleteProvider (+26 more)

### Community 52 - "String"
Cohesion: 0.06
Nodes (102): ctrl_mod(), cycle_hover_provider(), cycle_runtime_pane(), shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger() (+94 more)

### Community 53 - "KeymapScope"
Cohesion: 0.17
Nodes (11): BindingKey, KeyBinding, KeymapRegistry, KeymapScope, KeymapVimMode, normalize_chord(), BTreeMap, Display (+3 more)

### Community 54 - "UserLibrary"
Cohesion: 0.08
Nodes (32): BufferKind, browser_state_for_kind(), default_vim_target(), buffer_interaction(), buffer_is_browser(), buffer_is_compilation(), buffer_is_db_connect(), buffer_is_git_commit() (+24 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.09
Nodes (94): browser_buffer_layout(), BrowserBufferLayout, render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot (+86 more)

### Community 56 - ".len"
Cohesion: 0.07
Nodes (27): apply_input_operator_motion(), byte_index_for_char_column(), char_at_index(), exact_match_positions_in_chars(), find_char_forward(), fuzzy_match_end(), fuzzy_match_end_in_chars(), fuzzy_match_positions_in_chars() (+19 more)

### Community 57 - "UserLibraryModule"
Cohesion: 0.11
Nodes (18): AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AbiIconFontSymbol, AbiOilKeybindings, AbiStatuslineContext, AutocompleteProvider, AutocompleteProviderItem (+10 more)

### Community 58 - "Option"
Cohesion: 0.07
Nodes (51): BufRead, active_parameter_label(), char_to_byte_offset(), completion_documentation(), configuration_item_section(), csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params() (+43 more)

### Community 59 - "RVec"
Cohesion: 0.13
Nodes (14): exported_debug_adapters(), AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, DebugAdapterSpec, HoverProvider, HoverProviderTopic (+6 more)

### Community 60 - "shell/git.rs"
Cohesion: 0.06
Nodes (92): parse_log_oneline(), ActiveBufferEventContext, ActiveLspBufferContext, begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), commit_git_buffer() (+84 more)

### Community 61 - "Result"
Cohesion: 0.08
Nodes (86): execute_vim_command_line(), format_current_line_indent(), shell_buffer(), shell_buffer_mut(), submit_vim_command_line(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode() (+78 more)

### Community 62 - "Self"
Cohesion: 0.04
Nodes (28): browser_item(), default_action(), exported_db_browser_items(), hook_command(), Option, AcpActionSpec, AcpPickerOption, ContextHelpEntry (+20 more)

### Community 63 - "LspNotification"
Cohesion: 0.07
Nodes (29): ChildStdin, completion_level_for_message(), diagnostic_matches_request_range(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel (+21 more)

### Community 64 - "AbiKeymapConfig"
Cohesion: 0.10
Nodes (17): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, PaneConfig, config(), PaneConfig (+9 more)

### Community 65 - "state_with_user_library"
Cohesion: 0.07
Nodes (63): active_window_id(), install_mark_list_state_for_test(), open_oil_directory(), open_workspace_file(), open_workspace_from_project(), WindowId, switch_runtime_workspace(), browser_popup_command_focuses_the_popup_surface() (+55 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.09
Nodes (41): buffer_text_for_byte_range(), changed_range_windows(), collect_injection_regions(), create_parser(), desired_indent_for_loaded_language(), highlight_inline_language_per_line(), highlight_loaded_language(), highlight_loaded_language_with_tree() (+33 more)

### Community 67 - ".default"
Cohesion: 0.09
Nodes (51): Self, browser_display_url_prefers_requested_navigation(), Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec() (+43 more)

### Community 68 - "String"
Cohesion: 0.14
Nodes (46): active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+38 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "String"
Cohesion: 0.07
Nodes (26): append_query_source(), CaptureThemeMapping, cmake_configuration(), command_failure_message(), DeferredQuery, GrammarRecompileFailure, GrammarRecompileReport, installable_rust_configuration() (+18 more)

### Community 72 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 73 - ".send"
Cohesion: 0.10
Nodes (44): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+36 more)

### Community 74 - ".new"
Cohesion: 0.10
Nodes (29): buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), humanize_debug_label(), install_acp_test_buffer(), open_permission_request_reorders_queue_for_requested_picker(), pending_slash_completion_trigger_rejects_multiline_input() (+21 more)

### Community 75 - "LineCharMap"
Cohesion: 0.14
Nodes (16): ascii_control_caret_notation(), display_columns_for_character(), is_wide_display_character(), is_zero_width_display_character(), LineCharMap, LineWrapSegment, resolved_tab_width(), wrap_line_segments() (+8 more)

### Community 76 - "directory.rs"
Cohesion: 0.12
Nodes (46): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+38 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), environment_value(), explicit_windows_env_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.08
Nodes (65): LspWorkspaceDiagnostic, PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (41): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+33 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (84): vim_edit_requires_write(), workspace_picker_item(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search() (+76 more)

### Community 82 - "AcpManager"
Cohesion: 0.15
Nodes (10): acp_connected(), acp_open_permission_request(), acp_permission_picker_closed(), AcpManager, AcpPendingPermissionUi, drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), open_permission_picker() (+2 more)

### Community 83 - "FontSet"
Cohesion: 0.06
Nodes (57): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, acp_slice_chars() (+49 more)

### Community 84 - "Diagnostic"
Cohesion: 0.10
Nodes (23): close_buffer_keeps_session_alive_for_next_file(), file_uri_roundtrip_handles_windows_paths(), LspClientState, LspInlineCompletionItem, normalize_session_root(), parse_inline_completion_item(), parse_inline_completion_response(), path_to_file_uri() (+15 more)

### Community 85 - "LanguageServerSession"
Cohesion: 0.12
Nodes (17): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, path_is_solution() (+9 more)

### Community 86 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (23): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+15 more)

### Community 87 - ".new"
Cohesion: 0.05
Nodes (80): default_error_log_path(), buffer_footer_layout(), render_buffer(), acp_multiline_text_lines_strip_carriage_returns(), acp_section_layout_orders_output_input_footer_and_statusline(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow() (+72 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.13
Nodes (23): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+15 more)

### Community 89 - ".new"
Cohesion: 0.12
Nodes (34): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command(), git_status_log_all_command() (+26 more)

### Community 90 - ".from_grammar"
Cohesion: 0.08
Nodes (31): asset_path_from_parts(), bundled_query_resolution_flattens_inherited_queries(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), finalize_language_install_removes_compiler_sidecars(), GrammarSource, install_plan_compile_command_prefers_cpp_scanner() (+23 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 94 - "registered_queries.rs"
Cohesion: 0.15
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (18): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+10 more)

### Community 97 - "Self"
Cohesion: 0.11
Nodes (13): append_error_log(), ErrorEntry, ErrorLog, errors_buffer_lines(), ErrorSeverity, format_error_entry_lines(), keycode_name_token(), keydown_chord_token() (+5 more)

### Community 98 - "PickerSession"
Cohesion: 0.17
Nodes (6): contiguous_substring_beats_split_path_match(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), PickerSession, result_limit_caps_large_match_sets(), Vec

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (15): sanitize_transport_message(), transport_key_is_sensitive(), document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From (+7 more)

### Community 100 - "shell/browser.rs"
Cohesion: 0.12
Nodes (37): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len(), browser_viewport_contains_point() (+29 more)

### Community 101 - "main"
Cohesion: 0.11
Nodes (20): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), print_shell_summary(), Arc, DebugAdapterSpec (+12 more)

### Community 102 - "TextPoint"
Cohesion: 0.09
Nodes (9): Selection, TextPoint, TextSnapshot, char_immediately_before(), chars_immediately_before(), normalize_completion_replacement(), UndoNode, UndoSnapshot (+1 more)

### Community 103 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 104 - "PickerItem"
Cohesion: 0.19
Nodes (8): match_item(), PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 105 - "editor-lsp/src/lib.rs"
Cohesion: 0.20
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 106 - "editor-buffer/src/lib.rs"
Cohesion: 0.13
Nodes (14): around_word_ranges_at_line_end_exclude_newline(), detect_preferred_line_ending(), EditRecord, is_object_separator(), is_punctuation_char(), is_word_char(), LineEnding, matches_word_kind() (+6 more)

### Community 107 - "Option"
Cohesion: 0.14
Nodes (5): normalize_optional_string(), AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 108 - "String"
Cohesion: 0.10
Nodes (69): checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), create_git_worktree(), delete_git_status_targets(), fetch_git_all() (+61 more)

### Community 109 - "PickerOverlay"
Cohesion: 0.08
Nodes (41): PickerKind, PickerOverlay, ShellTestUserLibrary, workspace_delete_picker_overlay(), workspace_switch_picker_overlay(), buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings() (+33 more)

### Community 110 - "BufferId"
Cohesion: 0.11
Nodes (51): active_runtime_popup(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status() (+43 more)

### Community 111 - ".refresh_pdf_view"
Cohesion: 0.18
Nodes (4): ImageBufferMode, ImageBufferState, Error, pdf_zoom_percent_from_scale()

### Community 112 - "Option"
Cohesion: 0.18
Nodes (6): CommandPaletteState, CompilationState, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 113 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - ".new"
Cohesion: 0.10
Nodes (23): big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings(), move_word_backward_and_end_cover_word_navigation(), must(), reload_from_path_requires_a_backing_file(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean() (+15 more)

### Community 116 - "String"
Cohesion: 0.08
Nodes (24): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+16 more)

### Community 117 - "PluginCommand"
Cohesion: 0.10
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 118 - "DbService"
Cohesion: 0.11
Nodes (15): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, DbAutocompleteCandidate, DbService, looks_like_postgres_connection_string(), looks_like_sql_server_connection_string(), parse_key_value(), parse_postgres_keyword() (+7 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "editor-db/src/lib.rs"
Cohesion: 0.08
Nodes (47): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), current_statement(), DbColumn, DbExecutionOutput, DbTable (+39 more)

### Community 121 - "String"
Cohesion: 0.15
Nodes (16): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferState, DbEngine, DbHistoryEntry, DbIndex, DbQueryBufferMeta (+8 more)

### Community 122 - "buffer_is_git_status"
Cohesion: 0.36
Nodes (8): git_status_command_name(), handle_git_status_chord(), handle_git_status_tab(), set_git_prefix(), take_git_prefix(), toggle_git_section(), buffer_is_git_status(), GitPrefix

### Community 123 - "Vec"
Cohesion: 0.19
Nodes (29): SectionRenderLine, oil_directory_line_spans(), find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token() (+21 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.10
Nodes (18): apply_git_fringe_hunk(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitPrefixState, GitSummarySnapshot, GitSummaryState (+10 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "TerminalRenderSnapshot"
Cohesion: 0.13
Nodes (6): Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalCursorShape, TerminalCursorSnapshot, TerminalRenderLine, TerminalRenderSnapshot

### Community 127 - "Result"
Cohesion: 0.20
Nodes (10): DbSchemaCache, DbSession, InMemorySecretStore, load_postgres_schema(), load_sqlite_columns(), load_sqlite_schema(), Result, sqlite_index_table() (+2 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.13
Nodes (24): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+16 more)

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "oil.rs"
Cohesion: 0.10
Nodes (35): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec(), help_entry() (+27 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "browser_host.rs"
Cohesion: 0.11
Nodes (15): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+7 more)

### Community 134 - ".new_with_secret_store"
Cohesion: 0.13
Nodes (14): default_volt_state_dir(), initialize_native_keyring(), load_persisted_state(), OsSecretStore, Arc, Into, Path, PathBuf (+6 more)

### Community 135 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (15): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+7 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - ".path"
Cohesion: 0.23
Nodes (11): db_query_buffer_receives_sql_highlighting_without_blocking(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test() (+3 more)

### Community 138 - "DbSessionId"
Cohesion: 0.26
Nodes (11): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbBrowserBufferKind, DbSessionId, DbSessionSummary, insert_test_session(), remembered_connections_store_metadata_separately_from_secret(), sqls_initialization_options_for_query_buffer_use_attached_session() (+3 more)

### Community 139 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (24): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DebugAdapterSpec, DirectoryEntry (+16 more)

### Community 140 - "AcpEvent"
Cohesion: 0.08
Nodes (34): AvailableCommand, acp_pick_model(), AcpEvent, AcpSessionInfo, build_acp_input_hint(), command_input_hint(), config_option_is_mode(), config_option_is_model() (+26 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.16
Nodes (8): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, BufferId, Option, Self, String, Vec

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "Option"
Cohesion: 0.20
Nodes (8): delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), find_matching_close_tag(), is_tag_name_char(), parse_tag_token(), Option, TagToken

### Community 144 - "JobSpec"
Cohesion: 0.25
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "browser_sync_plan"
Cohesion: 0.31
Nodes (10): BrowserSurfacePlan, BrowserViewportRect, browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), buffer_uses_browser_host_surface(), rects_intersect() (+2 more)

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "TextRange"
Cohesion: 0.21
Nodes (3): advance_point_by_text(), paragraph_ranges_cover_inner_and_around_text_objects(), TextRange

### Community 148 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 149 - "OilDefaultsSection"
Cohesion: 0.28
Nodes (6): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSection, OilSortMode, OilDefaults

### Community 150 - "LspCodeAction"
Cohesion: 0.15
Nodes (4): LspCodeAction, LspDocumentTextEdits, Error, windows_should_retry_spawn_error()

### Community 151 - "LspLocation"
Cohesion: 0.25
Nodes (3): location_from_link(), LspLocation, LocationLink

### Community 152 - "Vec"
Cohesion: 0.08
Nodes (14): GhostTextLine, EventLog, format_micros_as_millis(), LspState, AutocompleteProvider, ContextHelpSpec, GhostTextLine, GitStatusSnapshot (+6 more)

### Community 153 - "db.rs"
Cohesion: 0.18
Nodes (15): browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord() (+7 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".char_count"
Cohesion: 0.22
Nodes (4): is_inline_whitespace(), is_sentence_closer(), Fn, sentence_ranges_cover_inner_and_around_text_objects()

### Community 156 - "LspLogEntry"
Cohesion: 0.17
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 157 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (15): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_reads_referenced_child_files(), load_uses_defaults_when_files_are_missing() (+7 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "clipboard.rs"
Cohesion: 0.19
Nodes (12): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+4 more)

### Community 161 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 162 - "setup_standalone_user_repository"
Cohesion: 0.33
Nodes (6): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 163 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "editor-syntax/src/lib.rs"
Cohesion: 0.06
Nodes (43): aligned_indent_column(), apply_text_edits_to_span(), capture_requires_theme_token(), compile_query_source(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after() (+35 more)

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "build_output.rs"
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "build_headerline_lines"
Cohesion: 0.22
Nodes (8): packages(), LanguageConfiguration, Vec, syntax_languages(), build_headerline_lines(), headerline_lines(), String, Vec

### Community 170 - "AbiSectionTree"
Cohesion: 0.11
Nodes (15): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiDirectoryEntry, AbiDirectoryEntryKind (+7 more)

### Community 171 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 172 - "Self"
Cohesion: 0.11
Nodes (12): normalize_inline_text(), Into, Item, Iterator, PathBuf, Range, Self, TextByteChunks (+4 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "markdown.rs"
Cohesion: 0.48
Nodes (6): inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), LanguageConfiguration, syntax_language(), syntax_languages_register_markdown_grammars()

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "shell/acp.rs"
Cohesion: 0.10
Nodes (29): acp_complete_slash(), acp_permission_approve(), acp_permission_deny(), acp_pick_mode(), acp_picker_entries(), acp_picker_entry(), acp_slash_completion_query(), AcpUiAction (+21 more)

### Community 180 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.20
Nodes (9): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+1 more)

### Community 185 - "Vec"
Cohesion: 0.36
Nodes (7): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), Vec, WorkspaceRootConfig, WorkspaceSection

### Community 186 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 187 - "Workspace Issues Plugin"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Issues Plugin

### Community 188 - "Workspace Switching and Marks"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Switching and Marks

### Community 189 - "Workspace Dashboard Worktree Remove + Picker Extra Keybinds"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Dashboard Worktree Remove + Picker Extra Keybinds

### Community 191 - ".oil_directory_sections"
Cohesion: 0.33
Nodes (4): DirectoryEntry, OilSortMode, Path, SectionTree

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 194 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 195 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 196 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 197 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 198 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 199 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 200 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 201 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 202 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 203 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 204 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 205 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 206 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 207 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 208 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 209 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 210 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 211 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 212 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 213 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 215 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 216 - "Language"
Cohesion: 0.33
Nodes (5): Issues, Language, Language servers, Volt, Workspace

### Community 217 - "choose_permission_outcome"
Cohesion: 0.40
Nodes (6): choose_permission_outcome(), format_permission_option_kind(), PendingPermission, PermissionOption, PermissionOptionKind, RequestPermissionOutcome

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: Local Markdown"
Cohesion: 0.33
Nodes (5): Conventions, Issue tracker: Local Markdown, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 221 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 222 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 224 - "Issue Store and Create"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Store and Create, Parent, What to build

### Community 225 - "Status commands"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Parent, Status commands, What to build

### Community 226 - "Capture focused file"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Capture focused file, Parent, What to build

### Community 227 - "Async Capture on save"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Async Capture on save, Parent, What to build

### Community 228 - "Issue Board"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Board, Parent, What to build

### Community 229 - "Place and jump to code"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Parent, Place and jump to code, What to build

### Community 230 - "Open Issue from Code Reference"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Open Issue from Code Reference, Parent, What to build

### Community 231 - "Issue Scan"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Scan, Parent, What to build

### Community 232 - "01 — Ambiguous Prefix Timeout"
Cohesion: 0.40
Nodes (4): 01 — Ambiguous Prefix Timeout, Acceptance criteria, Answer, Parent

### Community 233 - "03 — Cycle Project Workspaces"
Cohesion: 0.40
Nodes (4): 03 — Cycle Project Workspaces, Acceptance criteria, Answer, Parent

### Community 234 - "04 — Mark List Management"
Cohesion: 0.40
Nodes (4): 04 — Mark List Management, Acceptance criteria, Answer, Parent

### Community 235 - "05 — Marked Workspace Slot Jumps"
Cohesion: 0.40
Nodes (4): 05 — Marked Workspace Slot Jumps, Acceptance criteria, Answer, Parent

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 237 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 239 - "02 — Minor Mode Keybinding Precedence"
Cohesion: 0.50
Nodes (3): 02 — Minor Mode Keybinding Precedence, Acceptance criteria, Parent

### Community 240 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 241 - "load"
Cohesion: 0.24
Nodes (7): load(), load_from_root(), UserConfig, config(), KeymapConfig, config(), LigatureConfig

## Knowledge Gaps
- **195 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+190 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `String`, `ShellError`, `.path`, `Result`, `AcpEvent`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `treesitter_install.rs`, `state.rs`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `shell/mod.rs`, `ShellUiState`, `Vec`, `shell/acp.rs`, `String`, `KeymapScope`, `UserLibrary`, `.len`, `shell/git.rs`, `Result`, `state_with_user_library`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `AcpManager`, `.new`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `shell/browser.rs`, `main`, `String`, `PickerOverlay`, `BufferId`, `buffer_is_git_status`, `GitSummaryState`?**
  _High betweenness centrality (0.132) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `String`, `ShellError`, `render.rs`, `TextBuffer`, `browser_sync_plan`, `state.rs`, `shell/pdf.rs`, `shell/mod.rs`, `ShellUiState`, `Vec`, `shell/acp.rs`, `UserLibrary`, `render_buffer_with_view_state`, `.len`, `shell/git.rs`, `Result`, `LineCharMap`, `directory.rs`, `shell/terminal.rs`, `.new`, `draw_diagnostic_underlines_for_segment`, `.new`, `shell/browser.rs`, `TextPoint`, `PickerOverlay`, `.refresh_pdf_view`, `Vec`, `GitSummaryState`, `TerminalRenderSnapshot`?**
  _High betweenness centrality (0.085) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `oil.rs`, `user/lib.rs`, `Vec`, `calculator.rs`, `db.rs`, `lsp.rs`, `AcpPickerItemSpec`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `user/terminal.rs`, `build_headerline_lines`, `markdown.rs`, `user/browser.rs`, `HeaderlineTestUserLibrary`, `UserLibrary`, `UserLibraryModule`, `Self`, `bash.rs`, `clojure.rs`, `elixir.rs`, `graphql.rs`, `hcl.rs`, `java.rs`, `capture_mappings`, `kotlin.rs`, `latex.rs`, `lua.rs`, `nix.rs`, `perl.rs`, `php.rs`, `proto.rs`, `r.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `xml.rs`, `sdk/src/lib.rs`, `editor-plugin-host/src/lib.rs`, `syntax_language`, `syntax_language`, `package`, `main`, `common.rs`, `debug_adapters`, `package`, `PluginKeyBinding`, `PluginCommand`, `package`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _195 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `String` be split into smaller, more focused modules?**
  _Cohesion score 0.046724869531288106 - nodes in this community are weakly interconnected._
- **Should `LspClientError` be split into smaller, more focused modules?**
  _Cohesion score 0.07067925517964857 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.03600612870275792 - nodes in this community are weakly interconnected._