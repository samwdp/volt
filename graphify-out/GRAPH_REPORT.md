# Graph Report - volt  (2026-08-17)

## Corpus Check
- 234 files · ~589,601 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9437 nodes · 38530 edges · 325 communities (294 shown, 31 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3200 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `735e144f`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Self
- Path
- shell/tests.rs
- .new
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- .id
- shell/browser.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Option
- PickerSession
- .spawn
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- DbService
- state_with_user_library
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- FontSet
- String
- TextBuffer
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- EditorRuntime
- Self
- ShellBuffer
- shell_buffer
- Result
- AbiContextHelpSpec
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- client.rs
- render_workspace_dock
- clipboard.rs
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- LanguageServerSpec
- Path
- shell/git.rs
- ShellError
- .len
- Section
- picker_items
- active_runtime_popup
- editor-lsp/src/lib.rs
- String
- .new
- .char_to_point
- PluginCommand
- UserLibraryModule
- SyntaxRegistry
- AcpClient
- shell/acp.rs
- DebugConfiguration
- capture_mappings
- String
- DbEngine
- .send
- .new_with_secret_store
- editor-path/src/lib.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- volt/src/main.rs
- .new
- TextPoint
- Option
- ShellUiState
- AbiPaneConfig
- draw_diagnostic_underlines_for_segment
- .new
- .oil_directory_sections
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- WorkspaceConfigurationValue
- resolve_picker_extra
- GitEditorState
- modeline.rs
- LspSessionHandle
- state.rs
- .new
- Self
- Vec
- .null
- main
- RVec
- active_git_status_command_context
- PickerOverlay
- PickerItem
- .default
- refresh_pending_syntax
- editor-picker/src/lib.rs
- PluginKeyBinding
- .get
- String
- editor-db/src/lib.rs
- String
- process_supervisor.rs
- StatuslineContext
- browser_host.rs
- From
- Vec
- GitSummaryState
- LiveTerminalSession
- DynamicUserLibrary
- .path
- JobError
- Option
- user/config.rs
- AbiSectionTree
- key_sequence.rs
- editor-icons/src/lib.rs
- String
- common.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- .drain_events
- GhostTextContext
- load_user_library
- String
- CommandLineOverlay
- BufferId
- String
- JobSpec
- .line
- volt/build.rs
- ShellConfig
- Vec
- String
- cargo
- headerline_lines
- oil.rs
- db.rs
- lsp.rs
- graphql.rs
- treesittercontext_ghosttext.rs
- AbiGitFeatureSpec
- load
- Copilot instructions for `volt`
- browser_sync_plan
- flatten_config_select_options
- syntax_language
- theme.rs
- ServiceRegistry
- Option
- String
- user/terminal.rs
- corpus_inventory.rs
- kotlin.rs
- latex.rs
- lua.rs
- keybindings
- JobResult
- perl.rs
- user/browser.rs
- php.rs
- treesittercontext_shared.rs
- `user`
- LspLocation
- predicate_capture_text
- r.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- worktree_remove_from_one_shot
- ruby.rs
- ancestor_contexts_for_cursor
- .oil_directory_sections
- normalize_inline_text
- solidity.rs
- Database Explorer PRD
- aligned_indent_column
- editor-buffer/src/lib.rs
- LspServerCommand
- BTreeMap
- AcpManager
- build_job_command
- OilDefaultsSection
- markdown.rs
- bash.rs
- clojure.rs
- elixir.rs
- syntax_language
- hcl.rs
- java.rs
- .db_feature_spec
- 0004-markdown-pretty-pipeline.md
- nix.rs
- proto.rs
- scala.rs
- .keymap_config
- swift.rs
- lang/vim.rs
- xml.rs
- Language
- .pdf_open_mode
- Domain Docs
- Issue tracker: GitHub
- .acp_client_by_id
- .git_command_for_chord
- .autocomplete_providers
- .browser_feature_spec
- .terminal_feature_spec
- .workspace_roots
- rainbow_paren.rs
- AbiPickerTruncateStrategy
- .context_help_specs
- .debug_adapters
- text_document_content_change
- .git_feature_spec
- .hover_providers
- user/workspace_dock.rs
- package
- .ligature_config
- LspLogEntry
- .oil_feature_spec
- main
- .oil_keybindings
- keymap.rs
- .picker_layout
- shell/mod.rs
- .picker_truncate_strategy
- .terminal_config
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- setup_standalone_user_repository
- syntax_language
- Agent skills
- rainbow_parens.rs
- debug_adapters
- syntax_languages
- query_capture_property_value
- panic_payload_message
- package
- ligatures.rs
- client_capabilities
- .fmt
- syntax_language
- package
- package
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 766 edges
2. `ShellBuffer` - 375 edges
3. `shell_ui_mut()` - 342 edges
4. `register_shell_hooks()` - 262 edges
5. `shell_ui()` - 230 edges
6. `ShellError` - 192 edges
7. `shell_buffer()` - 181 edges
8. `shell_buffer_mut()` - 180 edges
9. `ShellUiState` - 174 edges
10. `TextBuffer` - 170 edges

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

## Communities (325 total, 31 thin omitted)

### Community 0 - "Self"
Cohesion: 0.04
Nodes (41): build_keydown_chord(), ChordModifiers, EmojiFont, ErrorSeverity, FontSet<'ttf>, FontSetInit, IconFont, keycode_name_token() (+33 more)

### Community 1 - "Path"
Cohesion: 0.09
Nodes (22): inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, parse_definition_response(), parse_text_edit_response() (+14 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (63): load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage() (+55 more)

### Community 3 - ".new"
Cohesion: 0.11
Nodes (76): line_ranges_and_char_searches_resolve_expected_points(), move_word_forward_advances_to_the_next_word(), word_motions_treat_punctuation_runs_as_words(), vim_search_entries_trim_whitespace_from_labels(), autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text() (+68 more)

### Community 4 - "ShellState"
Cohesion: 0.04
Nodes (42): clear_key_sequence(), active_lsp_workspace_loaded(), active_runtime_surface(), ActiveTypingFrameProfile, alt_mod(), average_duration(), browser_devtools_shortcut_requested(), build_shell_summary() (+34 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (97): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_acp_picker_items(), exported_autocomplete_providers() (+89 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.10
Nodes (70): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+62 more)

### Community 7 - ".id"
Cohesion: 0.06
Nodes (34): BufferKind, default_vim_target(), absolute_path_hint(), buffer_interaction(), buffer_is_browser(), buffer_is_command_output(), buffer_is_db_browser(), buffer_is_db_connect() (+26 more)

### Community 8 - "shell/browser.rs"
Cohesion: 0.11
Nodes (38): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_state_for_kind(), browser_url_candidates(), browser_url_prefix_len() (+30 more)

### Community 9 - "render.rs"
Cohesion: 0.04
Nodes (125): acp_chat_bubble_cols(), acp_rendered_text_segments(), acp_rendered_text_wrap_cols(), multicursor_selection_offsets(), acp_bubble_remaining_rows(), acp_chat_bubble_width_px(), acp_chat_origin_x(), acp_prefix_columns() (+117 more)

### Community 10 - "AcpEvent"
Cohesion: 0.10
Nodes (27): AcpCommand, AcpEvent, AcpRuntime, build_acp_input_hint(), choose_permission_outcome(), format_acp_mode_label(), format_acp_model_label(), format_permission_option_kind() (+19 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (41): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+33 more)

### Community 12 - "Self"
Cohesion: 0.04
Nodes (32): browser_item(), browser_items(), default_action(), exported_db_browser_items(), hook_command(), Option, AcpActionSpec, AcpPickerContext (+24 more)

### Community 13 - "Option"
Cohesion: 0.09
Nodes (51): build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), fetch_git_prune(), find_paren_number_range(), git_branch_list(), git_branch_merge(), git_branch_push_remote() (+43 more)

### Community 14 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 15 - ".spawn"
Cohesion: 0.08
Nodes (24): Keycode, Mod, terminal_key_for_event(), live_terminal_session_spawns_and_terminates(), LiveTerminalError, must(), Display, E (+16 more)

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
Nodes (16): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig, MarkdownPrettyConfig (+8 more)

### Community 20 - "HookBus"
Cohesion: 0.07
Nodes (23): HookBus, HookDefinition, HookError, HookEvent, HookSubscription, BTreeMap, BufferId, Default (+15 more)

### Community 21 - "EditorModel"
Cohesion: 0.07
Nodes (26): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+18 more)

### Community 22 - "KeymapScope"
Cohesion: 0.10
Nodes (32): autocomplete_overrides_workspace_while_active(), BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeyBinding, KeymapError (+24 more)

### Community 23 - "calculator.rs"
Cohesion: 0.07
Nodes (40): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+32 more)

### Community 24 - "DbService"
Cohesion: 0.14
Nodes (17): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbService, DbSession (+9 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.08
Nodes (72): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk(), buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus(), buffer_save_still_writes_when_format_on_save_fails() (+64 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.10
Nodes (45): centered_rect(), default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names() (+37 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "FontSet"
Cohesion: 0.07
Nodes (54): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, alpha_bitmap_surface() (+46 more)

### Community 32 - "String"
Cohesion: 0.07
Nodes (102): run_command(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer(), create_git_worktree(), create_git_worktree_from_query() (+94 more)

### Community 33 - "TextBuffer"
Cohesion: 0.10
Nodes (8): BufferStats, delimiter_partner(), Default, Into, Option, PathBuf, TextBuffer, TextBufferProvider

### Community 34 - "shell/pdf.rs"
Cohesion: 0.13
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

### Community 39 - "EditorRuntime"
Cohesion: 0.03
Nodes (299): EditorRuntime, Default, focus_active_browser_popup(), Cow, write_system_clipboard(), yank_to_clipboard_text(), ActiveLspBufferContext, WorkspaceId (+291 more)

### Community 40 - "Self"
Cohesion: 0.13
Nodes (18): ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), default_rainbow_parens_enabled(), default_workspace_dock_docked() (+10 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.02
Nodes (82): acp_output_header_title(), acp_tool_call_from_partial_update(), AcpBufferState, AcpPane, advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_line_indent(), apply_lsp_text_edits() (+74 more)

### Community 42 - "shell_buffer"
Cohesion: 0.07
Nodes (82): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range(), acp_input_field_o_and_o_open_new_lines() (+74 more)

### Community 43 - "Result"
Cohesion: 0.04
Nodes (129): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_multiline_text_lines_strip_carriage_returns(), acp_wrapped_text_uses_full_width_on_continuation_rows(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow() (+121 more)

### Community 44 - "AbiContextHelpSpec"
Cohesion: 0.06
Nodes (31): exported_browser_feature_spec(), exported_browser_url_placeholder(), exported_browser_url_prompt(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_oil_feature_spec(), exported_terminal_feature_spec() (+23 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (66): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+58 more)

### Community 47 - "client.rs"
Cohesion: 0.05
Nodes (90): BufRead, active_parameter_label(), char_to_byte_offset(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_documentation(), completion_level_for_message(), completion_parser_handles_lists_and_docs() (+82 more)

### Community 48 - "render_workspace_dock"
Cohesion: 0.12
Nodes (24): refresh_workspace_dock_branches(), render_workspace_dock(), Arc, HashMap, Instant, Mutex, Option, Path (+16 more)

### Community 49 - "clipboard.rs"
Cohesion: 0.19
Nodes (13): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+5 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.22
Nodes (18): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color(), resolve_terminal_named_color() (+10 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (33): AtomicUsize, active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc (+25 more)

### Community 52 - "LanguageServerSpec"
Cohesion: 0.10
Nodes (10): LanguageServerSpec, normalize_unique_entries(), Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String (+2 more)

### Community 53 - "Path"
Cohesion: 0.08
Nodes (22): append_query_source(), default_install_root(), ensure_cloned_grammar_dir_exists(), GrammarSource, InstallCommandSpec, io_error(), LanguageInstallPlan, maybe_read_bundled_query_source() (+14 more)

### Community 54 - "shell/git.rs"
Cohesion: 0.10
Nodes (38): apply_git_fringe_hunk(), apply_git_status_snapshot(), begin_oil_worktree_request(), git_status_checkout_file_command(), git_status_command_name(), git_status_diff_paths_command(), git_status_diff_range_command(), git_status_log_other_command() (+30 more)

### Community 55 - "ShellError"
Cohesion: 0.10
Nodes (116): Display, Error, From, ShellError, render_browser_buffer_body(), Color, adjust_color(), blend_color() (+108 more)

### Community 56 - ".len"
Cohesion: 0.05
Nodes (18): apply_input_operator_motion(), byte_index_for_char_column(), format_undo_snapshot_diff(), input_charwise_motion_range(), InputField, LineCharMap, resolve_block_insert_text(), resolved_tab_width() (+10 more)

### Community 57 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 58 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.10
Nodes (59): active_runtime_popup(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean() (+51 more)

### Community 60 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 61 - "String"
Cohesion: 0.06
Nodes (80): ctrl_mod(), shell_ui(), split_runtime_pane(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_visual_yank_copies_selected_text(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker() (+72 more)

### Community 62 - ".new"
Cohesion: 0.12
Nodes (26): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, default_db_browser_items(), execute_postgres(), execute_sql_server() (+18 more)

### Community 63 - ".char_to_point"
Cohesion: 0.09
Nodes (11): advance_point_by_text(), is_inline_whitespace(), is_object_separator(), is_punctuation_char(), is_sentence_closer(), is_word_char(), matches_word_kind(), Fn (+3 more)

### Community 64 - "PluginCommand"
Cohesion: 0.09
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 65 - "UserLibraryModule"
Cohesion: 0.09
Nodes (20): exported_icon_symbols(), exported_oil_keybindings(), exported_pdf_open_mode(), IconFontSymbol, OilKeybindings, PdfOpenMode, AbiIconFontCategory, AbiIconFontSymbol (+12 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.09
Nodes (31): compile_query_source(), create_parser(), DeferredQuery, desired_indent_for_loaded_language(), highlight_inline_language_per_line(), highlight_loaded_language(), highlight_loaded_language_with_tree(), HighlightWindow (+23 more)

### Community 67 - "AcpClient"
Cohesion: 0.07
Nodes (19): AsyncRead, AcpClient, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, KillTerminalRequest, KillTerminalResponse, ReadTextFileRequest (+11 more)

### Community 68 - "shell/acp.rs"
Cohesion: 0.10
Nodes (58): acp_slash_completion_query(), active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates() (+50 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (27): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+19 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "String"
Cohesion: 0.09
Nodes (18): asset_path_from_parts(), buffer_text_for_byte_range(), CaptureThemeMapping, LanguageConfiguration, LanguageLoader, load_language(), normalize_extension(), normalize_unique_entries() (+10 more)

### Community 72 - "DbEngine"
Cohesion: 0.15
Nodes (12): DbAutocompleteCandidate, DbEngine, DbHistoryEntry, DbQueryBufferMeta, DbSnippet, default_volt_state_dir(), PersistedDbState, QualifiedName (+4 more)

### Community 73 - ".send"
Cohesion: 0.19
Nodes (31): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client(), disconnect_acp_session() (+23 more)

### Community 74 - ".new_with_secret_store"
Cohesion: 0.27
Nodes (7): load_persisted_state(), Arc, Path, Self, Send, Sync, SecretStore

### Community 75 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 76 - "directory.rs"
Cohesion: 0.12
Nodes (46): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+38 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (37): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+29 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (47): LspWorkspaceDiagnostic, PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+39 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.15
Nodes (35): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+27 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.06
Nodes (66): workspace_picker_item(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+58 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 83 - ".new"
Cohesion: 0.08
Nodes (44): apply_git_view(), diff_git_commit_at_point(), diff_git_dwim(), diff_git_stash_at_point(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_status_cherry_open_command() (+36 more)

### Community 84 - "TextPoint"
Cohesion: 0.09
Nodes (11): Self, Selection, TextPoint, TextSnapshot, char_immediately_before(), chars_immediately_before(), InlineCompletionState, normalize_completion_replacement() (+3 more)

### Community 85 - "Option"
Cohesion: 0.10
Nodes (19): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, normalize_optional_string() (+11 more)

### Community 86 - "ShellUiState"
Cohesion: 0.04
Nodes (40): active_buffer_revision_key(), active_runtime_buffer(), active_shell_workspace_id(), apply_lsp_notifications(), BufferViewState, command_builds_user_library(), DirectoryPrefixState, DismissedPopupState (+32 more)

### Community 87 - "AbiPaneConfig"
Cohesion: 0.09
Nodes (14): AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig, AbiPickerLayout, AbiWorkspaceDockSide, fraction_to_hundredths(), MarkdownPrettyConfig, PickerLayout (+6 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.15
Nodes (23): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+15 more)

### Community 89 - ".new"
Cohesion: 0.19
Nodes (15): browser_host_event_for_ipc(), BrowserHostEvent, BrowserHostService, BrowserLocationUpdate, DesktopBrowserHostService, BTreeMap, BufferId, Receiver (+7 more)

### Community 90 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "editor-core/src/lib.rs"
Cohesion: 0.16
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 94 - "registered_queries.rs"
Cohesion: 0.15
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "WorkspaceConfigurationValue"
Cohesion: 0.13
Nodes (16): sanitize_transport_message(), transport_key_is_sensitive(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, From, I, Number, T (+8 more)

### Community 97 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "LspSessionHandle"
Cohesion: 0.08
Nodes (40): ChildStdin, file_uri_to_path(), language_server_session_in_workspace_scope(), live_sessions_for_workspace_includes_root_scoped_and_buffer_served(), LspClientState, LspSessionHandle, normalize_path_for_compare(), parse_publish_diagnostics() (+32 more)

### Community 101 - "state.rs"
Cohesion: 0.12
Nodes (23): BlockInsertState, DirectoryYankEntry, FormatterRegistry, FormatterSpec, LastFind, LastSearch, BTreeMap, BufferId (+15 more)

### Community 102 - ".new"
Cohesion: 0.07
Nodes (38): CodeActionParams, close_buffer_keeps_session_alive_for_next_file(), code_action_params(), code_action_params_use_flattened_lsp_shape(), file_uri_roundtrip_handles_windows_paths(), full_document_range(), live_session_picker_label_includes_server_and_root(), log_message_error_from_ols_does_not_become_ui_notification() (+30 more)

### Community 103 - "Self"
Cohesion: 0.05
Nodes (64): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), normalize_window_blur(), GitStashEntry, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiAcpClient (+56 more)

### Community 104 - "Vec"
Cohesion: 0.12
Nodes (9): DapState, EventLog, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, HoverProvider, Vec (+1 more)

### Community 105 - ".null"
Cohesion: 0.11
Nodes (26): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), explicit_windows_env_value(), Command, Error, spawn_lsp_command() (+18 more)

### Community 106 - "main"
Cohesion: 0.17
Nodes (12): bootstrap(), HostBootstrap, command_palette_items(), main(), print_shell_summary(), DebugAdapterSpec, Error, LanguageConfiguration (+4 more)

### Community 107 - "RVec"
Cohesion: 0.11
Nodes (16): exported_debug_adapters(), exported_terminal_config(), TerminalConfig, AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, DebugAdapterSpec (+8 more)

### Community 108 - "active_git_status_command_context"
Cohesion: 0.12
Nodes (33): active_git_status_command_context(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), git_action_detail(), git_commit_at_point(), git_sequence_in_progress(), git_status_apply_commit_command(), git_status_cherry_pick_apply_command() (+25 more)

### Community 109 - "PickerOverlay"
Cohesion: 0.09
Nodes (39): PickerOverlay, ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec() (+31 more)

### Community 110 - "PickerItem"
Cohesion: 0.19
Nodes (8): match_item(), PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 111 - ".default"
Cohesion: 0.10
Nodes (50): Self, Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids() (+42 more)

### Community 112 - "refresh_pending_syntax"
Cohesion: 0.08
Nodes (20): find_workspace_file_buffer(), index_syntax_lines_with_rainbow_parens(), panic_payload_message(), picker_preview_syntax_lines(), process_syntax_refresh_request(), refresh_buffer_syntax(), refresh_pending_file_reloads(), refresh_pending_syntax() (+12 more)

### Community 113 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.13
Nodes (22): plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding(), normal_binding_commands() (+14 more)

### Community 115 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - "String"
Cohesion: 0.13
Nodes (12): append_lines(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self, String (+4 more)

### Community 117 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (33): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn, DbIndex, DbSchemaCache, DbTable, default_db_browser_line() (+25 more)

### Community 118 - "String"
Cohesion: 0.28
Nodes (19): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+11 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "StatuslineContext"
Cohesion: 0.15
Nodes (9): exported_statusline_render(), statusline_context_from_abi(), user_modeline_context(), AbiLspDiagnosticsInfo, AbiStatuslineContext, LspDiagnosticsInfo, LspDiagnosticsInfo, LspDiagnosticsInfo (+1 more)

### Community 121 - "browser_host.rs"
Cohesion: 0.13
Nodes (13): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+5 more)

### Community 122 - "From"
Cohesion: 0.07
Nodes (28): AbiColor, AbiContextHelpEntry, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiKeymapConfig, AbiLanguageServerRootStrategy, AbiLigatureConfig, AbiOilKeyAction (+20 more)

### Community 123 - "Vec"
Cohesion: 0.21
Nodes (27): oil_directory_line_spans(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token(), git_status_entry_token_from_icon(), git_status_head_spans() (+19 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.11
Nodes (15): git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, refresh_git_fringe(), refresh_pending_git_summary() (+7 more)

### Community 125 - "LiveTerminalSession"
Cohesion: 0.12
Nodes (13): AlacrittyEvent, Self, terminal_scroll_for_motion(), LiveTerminalSession, QueuedEventListener, Arc, Drop, Receiver (+5 more)

### Community 127 - ".path"
Cohesion: 0.21
Nodes (11): db_query_buffer_receives_sql_highlighting_without_blocking(), db_table_preview_buffer_exposes_hidden_sqls_path_without_file_open_hooks(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test() (+3 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.09
Nodes (14): push_snapshot_line(), push_terminal_render_run(), Option, Vec, terminal_render_snapshot(), terminal_render_snapshot_preserves_wide_character_widths(), terminal_render_snapshot_tracks_visible_cursor(), terminal_snapshot_lines() (+6 more)

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "AbiSectionTree"
Cohesion: 0.09
Nodes (18): exported_git_status_sections(), exported_oil_defaults(), exported_oil_directory_sections(), DirectoryEntry, GitStatusSnapshot, OilDefaults, OilSortMode, Path (+10 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 134 - "String"
Cohesion: 0.14
Nodes (17): db_browser_action_from_spec(), DisabledSecretStore, initialize_native_keyring(), InMemorySecretStore, load_postgres_schema(), OsSecretStore, qualified_name_from_spec(), redact_error() (+9 more)

### Community 135 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - ".drain_events"
Cohesion: 0.22
Nodes (8): coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), Receiver, VecDeque, split_acp_events_for_render(), split_acp_events_for_render_defers_later_plan_transitions()

### Community 138 - "GhostTextContext"
Cohesion: 0.11
Nodes (13): GhostTextLine, GhostTextLine, exported_ghost_text_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AutocompleteProvider (+5 more)

### Community 139 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 140 - "String"
Cohesion: 0.08
Nodes (56): acp_complete_slash(), acp_connected(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny(), acp_permission_picker_closed(), acp_permission_picker_submitted() (+48 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "BufferId"
Cohesion: 0.15
Nodes (19): ActiveBufferEventContext, fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_line_is_untracked(), git_snapshot_for_buffer(), git_status_action_targets(), git_status_delete_target_for_line(), git_status_delete_targets() (+11 more)

### Community 143 - "String"
Cohesion: 0.13
Nodes (8): CommandPaletteState, CompilationState, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, String, TerminalState

### Community 144 - "JobSpec"
Cohesion: 0.21
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - ".line"
Cohesion: 0.18
Nodes (9): find_matching_close_tag(), is_tag_name_char(), parse_tag_token(), String, Vec, TagToken, trimmed_line(), visible_line_len() (+1 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "ShellConfig"
Cohesion: 0.16
Nodes (12): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+4 more)

### Community 148 - "Vec"
Cohesion: 0.09
Nodes (28): apply_text_edits_to_span(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport (+20 more)

### Community 149 - "String"
Cohesion: 0.05
Nodes (36): TextRange, diagnostic_matches_request_range(), documentation_lines(), hover_marked_string(), hover_marked_string_markdown_text(), hover_text(), hover_text_lines(), launch_summary() (+28 more)

### Community 150 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 151 - "headerline_lines"
Cohesion: 0.31
Nodes (6): build_headerline_lines(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 152 - "oil.rs"
Cohesion: 0.14
Nodes (23): seti_directory_icon(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), help_entry(), is_oil_icon(), oil_directory_icon(), oil_entry_icon() (+15 more)

### Community 153 - "db.rs"
Cohesion: 0.18
Nodes (14): browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord(), query_buffer_lines() (+6 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 156 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 157 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "browser_sync_plan"
Cohesion: 0.17
Nodes (15): browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Instant (+7 more)

### Community 161 - "flatten_config_select_options"
Cohesion: 0.31
Nodes (9): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+1 more)

### Community 162 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 163 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "Option"
Cohesion: 0.21
Nodes (13): browser_navigation_retry_required(), BrowserBufferPlan, BrowserInstance, BrowserSurfacePlan, BrowserSyncPlan, BrowserViewportRect, optional_non_empty_text(), Duration (+5 more)

### Community 166 - "String"
Cohesion: 0.29
Nodes (6): call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "user/terminal.rs"
Cohesion: 0.24
Nodes (10): default_terminal_args(), default_terminal_program(), default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package(), package_exports_terminal_commands_and_binding() (+2 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.10
Nodes (36): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+28 more)

### Community 169 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 170 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 171 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 172 - "keybindings"
Cohesion: 0.16
Nodes (15): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), feature_spec(), help_lines(), help_lines_reflect_current_keybindings(), keybindings(), keydown_action() (+7 more)

### Community 173 - "JobResult"
Cohesion: 0.18
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 177 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "LspLocation"
Cohesion: 0.15
Nodes (8): definition_parser_preserves_uri_backed_locations(), location_from_link(), location_from_lsp(), location_sorting_deduplicates_reference_results(), LspLocation, parse_reference_response(), Location, LocationLink

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

### Community 181 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.18
Nodes (10): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+2 more)

### Community 185 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 186 - "worktree_remove_from_one_shot"
Cohesion: 0.24
Nodes (11): git_worktree_dashboard_picker_overlay(), git_worktree_list(), git_worktree_list_parser_normalizes_windows_drive_paths(), GitWorktreeListEntry, parse_git_worktree_list(), worktree_remove_from_one_shot(), worktree_remove_git_args(), worktree_remove_git_invocation_for_entries() (+3 more)

### Community 187 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 188 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 189 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 190 - "normalize_inline_text"
Cohesion: 0.20
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 191 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "aligned_indent_column"
Cohesion: 0.21
Nodes (12): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column(), query_property_is_set() (+4 more)

### Community 194 - "editor-buffer/src/lib.rs"
Cohesion: 0.08
Nodes (35): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), EditRecord, edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings() (+27 more)

### Community 195 - "LspServerCommand"
Cohesion: 0.28
Nodes (3): CopilotDeviceCodePrompt, execute_command_params(), LspServerCommand

### Community 196 - "BTreeMap"
Cohesion: 0.31
Nodes (5): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), BTreeMap, workspace_configuration_value_round_trips_through_json()

### Community 197 - "AcpManager"
Cohesion: 0.12
Nodes (26): AcpClientConfig, AvailableCommand, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_set_mode(), acp_set_model() (+18 more)

### Community 198 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 199 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 202 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 203 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 204 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 205 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 206 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 209 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 210 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 211 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 213 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 214 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 215 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 216 - "Language"
Cohesion: 0.25
Nodes (7): External commands, Issues, Language, Language servers, Markdown presentation, Volt, Workspace

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 227 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 230 - "text_document_content_change"
Cohesion: 0.29
Nodes (7): full_sync_uses_null_range_change(), incremental_sync_uses_full_document_replacement_range(), text_document_content_change(), text_document_sync_kind(), TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind

### Community 233 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 234 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 236 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 242 - "shell/mod.rs"
Cohesion: 0.01
Nodes (331): acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter(), acp_multiline_text_lines() (+323 more)

### Community 247 - "setup_standalone_user_repository"
Cohesion: 0.33
Nodes (6): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 248 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 250 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 251 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 252 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 253 - "query_capture_property_value"
Cohesion: 0.50
Nodes (4): query_capture_property_value(), query_capture_property_value_returns_set_property(), query_compiler_accepts_vim_case_insensitive_regex_prefix(), rust_language()

### Community 254 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 255 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 257 - "client_capabilities"
Cohesion: 0.67
Nodes (3): ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document()

## Knowledge Gaps
- **141 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+136 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **31 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `ShellState`, `.id`, `shell/browser.rs`, `.drain_events`, `AcpEvent`, `String`, `Option`, `BufferId`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `command_stream.rs`, `String`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `shell_buffer`, `Result`, `shell/git.rs`, `.len`, `worktree_remove_from_one_shot`, `active_runtime_popup`, `String`, `shell/acp.rs`, `AcpManager`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `.new`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `main`, `active_git_status_command_context`, `PickerOverlay`, `refresh_pending_syntax`, `shell/mod.rs`, `String`, `GitSummaryState`, `.path`?**
  _High betweenness centrality (0.098) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `Self`, `Option`, `ShellState`, `.id`, `shell/browser.rs`, `render.rs`, `BufferId`, `browser_sync_plan`, `TextBuffer`, `shell/pdf.rs`, `EditorRuntime`, `shell_buffer`, `Result`, `ShellError`, `.len`, `shell/acp.rs`, `directory.rs`, `shell/terminal.rs`, `.new`, `TextPoint`, `ShellUiState`, `draw_diagnostic_underlines_for_segment`, `state.rs`, `PickerOverlay`, `refresh_pending_syntax`, `shell/mod.rs`, `Vec`, `GitSummaryState`?**
  _High betweenness centrality (0.073) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `ShellError` to `Self`, `shell/tests.rs`, `ShellState`, `user/lib.rs`, `.id`, `shell/browser.rs`, `load_user_library`, `ShellConfig`, `DynamicUserLibrary`, `editor-render/src/lib.rs`, `HoverOverlay`, `browser_sync_plan`, `ShellBuffer`, `shell_buffer`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `render_workspace_dock`, `HeaderlineTestUserLibrary`, `shell/git.rs`, `directory.rs`, `volt/src/main.rs`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `PickerOverlay`, `shell/mod.rs`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _141 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Self` be split into smaller, more focused modules?**
  _Cohesion score 0.044372612400822804 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.09450171821305842 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.02893772893772894 - nodes in this community are weakly interconnected._