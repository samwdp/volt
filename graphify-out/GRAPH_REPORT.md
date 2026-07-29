# Graph Report - volt  (2026-07-29)

## Corpus Check
- 223 files · ~562,691 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8944 nodes · 36595 edges · 276 communities (271 shown, 5 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3023 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `8343ad30`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- shell/mod.rs
- LspClientError
- shell/tests.rs
- .new
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- String
- .new
- render.rs
- String
- PluginPackage
- sdk/src/lib.rs
- TextBuffer
- LanguageServerSpec
- .spawn
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- String
- .new
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- load_font_set_with_mode
- EditorRuntime
- Path
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- Self
- Option
- UserLibraryModule
- state.rs
- String
- ShellUiState
- draw_primary_ligature_texture_if_available
- AbiGitFeatureSpec
- PathBuf
- AbiContextHelpSpec
- Option
- HeaderlineTestUserLibrary
- String
- Vec
- .id
- render_buffer_with_view_state
- .len
- ROption
- client.rs
- RVec
- Path
- Result
- Self
- spawn_reader_thread
- AbiKeymapConfig
- state_with_user_library
- SyntaxRegistry
- .default
- shell/acp.rs
- DebugConfiguration
- capture_mappings
- LanguageConfiguration
- theme.rs
- .send
- .new
- editor-path/src/lib.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- AcpManager
- FontSet
- .from
- Option
- volt/src/main.rs
- shell_ui_mut
- draw_diagnostic_underlines_for_segment
- .new
- main
- editor-plugin-host/src/lib.rs
- CommandRegistry
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- Option
- PluginBuffer
- LiveTerminalSession
- WorkspaceConfigurationValue
- shell/browser.rs
- cargo
- LspInlineCompletionItem
- common.rs
- String
- editor-lsp/src/lib.rs
- String
- LspServerCommand
- AbiLanguageConfiguration
- shell/picker.rs
- BufferId
- commit_git_buffer
- browser_host.rs
- editor-picker/src/lib.rs
- PluginKeyBinding
- AbiOilDefaults
- String
- PluginCommand
- DbService
- process_supervisor.rs
- cmake.rs
- DbEngine
- AbiSectionTree
- shell/git.rs
- GitSummaryState
- statusline.rs
- Option
- BrowserBufferState
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- oil.rs
- key_sequence.rs
- Self
- test_service
- treesittercontext_ghosttext.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- TerminalCursorSnapshot
- AbiPickerTruncateStrategy
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- I
- JobSpec
- ShellConfig
- standalone_user_manifest.rs
- editor-icons/src/lib.rs
- setup_standalone_user_repository
- TextRange
- Duration
- StartupTrace
- VimActionContext
- db.rs
- lsp.rs
- .oil_directory_sections
- LspLogEntry
- AcpPickerItemSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- AbiPdfOpenMode
- treesittercontext_shared.rs
- ServiceRegistry
- aligned_indent_column
- String
- load
- build_output.rs
- build_headerline_lines
- ancestor_contexts_for_cursor
- JobResult
- user/browser.rs
- browser_sync_plan
- Default
- `user`
- AcpCommand
- spawn_terminal_reader
- markdown.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- Vec
- syntax_language
- Database Explorer PRD
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
- Language
- Domain Docs
- Issue tracker: GitHub
- keymap.rs
- syntax_language
- package
- debug_adapters
- package
- Agent skills
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 741 edges
2. `ShellBuffer` - 362 edges
3. `shell_ui_mut()` - 331 edges
4. `register_shell_hooks()` - 256 edges
5. `shell_ui()` - 217 edges
6. `ShellError` - 182 edges
7. `shell_buffer()` - 178 edges
8. `shell_buffer_mut()` - 173 edges
9. `TextBuffer` - 166 edges
10. `ShellUiState` - 161 edges

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

## Communities (276 total, 5 thin omitted)

### Community 0 - "shell/mod.rs"
Cohesion: 0.03
Nodes (313): Cow, yank_to_clipboard_text(), accept_autocomplete(), acp_decode_image(), activate_db_browser_line(), active_lsp_buffer_context(), active_lsp_code_action_range(), active_project_workspace_root() (+305 more)

### Community 1 - "LspClientError"
Cohesion: 0.06
Nodes (46): BufRead, ClientCapabilities, client_capabilities(), close_buffer_keeps_session_alive_for_next_file(), csharp_metadata_request_params(), execute_command_params(), execute_command_params_from_inline_item(), formatting_parser_maps_text_edits() (+38 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.02
Nodes (102): accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_multiline_text_lines_strip_carriage_returns(), acp_wrapped_text_uses_full_width_on_continuation_rows(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow() (+94 more)

### Community 3 - ".new"
Cohesion: 0.11
Nodes (73): Self, autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean() (+65 more)

### Community 4 - "ShellError"
Cohesion: 0.04
Nodes (46): Display, Error, From, ShellError, clear_key_sequence(), active_buffer_event_context(), active_lsp_workspace_loaded(), active_runtime_surface() (+38 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.04
Nodes (88): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+80 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.08
Nodes (85): line_ranges_and_char_searches_resolve_expected_points(), move_word_forward_advances_to_the_next_word(), word_motions_treat_punctuation_runs_as_words(), index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names(), vim_search_entries_trim_whitespace_from_labels(), additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root() (+77 more)

### Community 7 - "String"
Cohesion: 0.07
Nodes (86): apply_git_status_snapshot(), cancel_git_commit_buffer(), ensure_no_rebase_in_progress(), ensure_rebase_in_progress(), fetch_git_pushremote(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_log_args() (+78 more)

### Community 8 - ".new"
Cohesion: 0.04
Nodes (53): BufferKind, default_vim_target(), absolute_path_hint(), append_error_log(), buffer_interaction(), buffer_is_command_output(), buffer_is_db_connect(), buffer_is_oil_preview() (+45 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (100): acp_rendered_text_wrap_cols(), advance_point_by_text(), multicursor_selection_offsets(), acp_pane_body_visible_rows(), acp_prefix_columns(), acp_slice_chars(), acp_spinner_frame(), adjusted_contextual_ligature_pixel_size() (+92 more)

### Community 10 - "String"
Cohesion: 0.10
Nodes (45): AcpClientConfig, acp_complete_slash(), acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session() (+37 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (44): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), package_with_path_matchers() (+36 more)

### Community 12 - "sdk/src/lib.rs"
Cohesion: 0.05
Nodes (50): vim_edit_requires_write(), AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+42 more)

### Community 13 - "TextBuffer"
Cohesion: 0.03
Nodes (75): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending() (+67 more)

### Community 14 - "LanguageServerSpec"
Cohesion: 0.12
Nodes (10): LanguageServerSpec, normalize_optional_string(), Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String (+2 more)

### Community 15 - ".spawn"
Cohesion: 0.09
Nodes (20): Keycode, Mod, terminal_key_for_event(), live_terminal_session_spawns_and_terminates(), LiveTerminalError, Display, Error, Formatter (+12 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.10
Nodes (41): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+33 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.06
Nodes (35): configure_background_command(), detect_in_progress(), git_available(), GitLogEntry, GitStashEntry, GitStatusError, GitStatusSnapshot, list_repository_files() (+27 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.05
Nodes (114): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+106 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.04
Nodes (17): DynamicUserLibrary, AcpClient, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, GitStatusPrefix, IconFontSymbol, KeymapConfig (+9 more)

### Community 20 - "HookBus"
Cohesion: 0.07
Nodes (24): HookBus, HookDefinition, HookError, HookEvent, HookSubscription, BTreeMap, BufferId, Default (+16 more)

### Community 21 - "EditorModel"
Cohesion: 0.07
Nodes (26): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+18 more)

### Community 22 - "KeymapScope"
Cohesion: 0.10
Nodes (32): autocomplete_overrides_workspace_while_active(), BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeyBinding, KeymapError (+24 more)

### Community 23 - "calculator.rs"
Cohesion: 0.07
Nodes (39): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+31 more)

### Community 24 - "String"
Cohesion: 0.07
Nodes (63): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn (+55 more)

### Community 25 - ".new"
Cohesion: 0.15
Nodes (24): browser_additional_args(), browser_host_event_for_ipc(), BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, BrowserSurfacePlan (+16 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.08
Nodes (69): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+61 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.10
Nodes (44): centered_rect(), default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names() (+36 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (30): AutocompleteProviderKind, RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay, HoverProviderContent (+22 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "load_font_set_with_mode"
Cohesion: 0.08
Nodes (28): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode(), load_icon_font() (+20 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.10
Nodes (62): EditorRuntime, Default, checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), create_git_worktree(), delete_git_status_targets(), fetch_git_all() (+54 more)

### Community 33 - "Path"
Cohesion: 0.09
Nodes (21): asset_path_from_parts(), command_failure_message(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), GrammarSource, InstallCommandSpec, io_error(), LanguageInstallPlan (+13 more)

### Community 34 - "shell/pdf.rs"
Cohesion: 0.13
Nodes (43): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_delete_page(), pdf_fit_mode_label(), pdf_header_lines() (+35 more)

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
Cohesion: 0.13
Nodes (13): ConfigOilSortMode, ConfigPickerTruncateStrategy, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy(), OilDefaultsSection (+5 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (110): ActiveLspBufferContext, WorkspaceId, acp_tool_call_from_partial_update(), AcpBufferState, AcpPane, advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update() (+102 more)

### Community 42 - "UserLibraryModule"
Cohesion: 0.12
Nodes (15): exported_icon_symbols(), exported_oil_keybindings(), IconFontSymbol, OilKeybindings, AbiIconFontCategory, AbiIconFontSymbol, AbiOilKeybindings, IconFontCategory (+7 more)

### Community 43 - "state.rs"
Cohesion: 0.07
Nodes (40): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+32 more)

### Community 44 - "String"
Cohesion: 0.06
Nodes (46): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), documentation_lines(), explicit_windows_env_value(), hover_marked_string() (+38 more)

### Community 45 - "ShellUiState"
Cohesion: 0.04
Nodes (53): active_buffer_revision_key(), active_runtime_buffer(), active_shell_workspace_id(), active_window_id(), apply_lsp_notifications(), apply_pending_lsp_state(), apply_sqls_workspace_settings_for_buffer(), autocomplete_request_for_buffer() (+45 more)

### Community 46 - "draw_primary_ligature_texture_if_available"
Cohesion: 0.12
Nodes (20): Canvas, cached_primary_text_runs(), draw_primary_ligature_texture_if_available(), draw_text_texture_with_cache(), draw_undercurl_canvas(), fill_rounded_rect_canvas(), LigatureShapeCacheEntry, LigatureShapeCacheValue (+12 more)

### Community 47 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 48 - "PathBuf"
Cohesion: 0.03
Nodes (87): AcpDecodedImage, AcpRenderedImageLine, active_directory_root(), active_shell_buffer_path(), active_theme_state_path(), asset_path_from_parts(), built_user_library_path_for_command(), comment_style_for_buffer() (+79 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.06
Nodes (31): exported_browser_feature_spec(), exported_browser_url_placeholder(), exported_browser_url_prompt(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_oil_feature_spec(), exported_terminal_feature_spec() (+23 more)

### Community 50 - "Option"
Cohesion: 0.06
Nodes (36): char_to_byte_offset(), completion_documentation(), completion_level_for_message(), configuration_item_section(), effective_workspace_configuration_settings(), initialization_options_for_server(), is_csharp_server(), log_message_error_from_ols_does_not_become_ui_notification() (+28 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (57): AtomicUsize, load_font_set(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage(), contextual_ligature_raster_size_never_upscales_smaller_substitute_glyphs(), directory_view_state_uses_user_oil_defaults() (+49 more)

### Community 52 - "String"
Cohesion: 0.06
Nodes (75): cycle_runtime_pane(), shell_ui(), split_runtime_pane(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_visual_yank_copies_selected_text(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker() (+67 more)

### Community 53 - "Vec"
Cohesion: 0.10
Nodes (10): EventLog, format_micros_as_millis(), LspState, AutocompleteProvider, ContextHelpSpec, HoverProvider, StatuslineContext, String (+2 more)

### Community 54 - ".id"
Cohesion: 0.11
Nodes (20): buffer_is_quickfix(), ErrorSeverity, find_oil_buffer(), quickfix_clear_marks(), quickfix_entries_from_one_shot(), quickfix_entry_for_cursor(), quickfix_mark_all(), quickfix_open_current_list() (+12 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.11
Nodes (89): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, is_dark_color(), Color, RuntimePopupSnapshot (+81 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (18): apply_input_operator_motion(), ascii_control_caret_notation(), byte_index_for_char_column(), display_columns_for_character(), input_charwise_motion_range(), InputField, is_wide_display_character(), is_zero_width_display_character() (+10 more)

### Community 57 - "ROption"
Cohesion: 0.15
Nodes (12): GhostTextLine, exported_ghost_text_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AutocompleteProvider, AutocompleteProviderItem (+4 more)

### Community 58 - "client.rs"
Cohesion: 0.04
Nodes (80): client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range(), copilot_status_notifications_offer_sign_in_action(), diagnostics_parser_maps_lsp_fields() (+72 more)

### Community 59 - "RVec"
Cohesion: 0.15
Nodes (12): exported_terminal_config(), TerminalConfig, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider (+4 more)

### Community 60 - "Path"
Cohesion: 0.08
Nodes (51): parse_log_oneline(), begin_oil_worktree_request(), build_git_fringe_snapshot(), command_output_transcript(), create_git_worktree_from_query(), fetch_git_prune(), git_branch_list(), git_branch_merge() (+43 more)

### Community 61 - "Result"
Cohesion: 0.08
Nodes (94): default_error_log_path(), format_current_line_indent(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line() (+86 more)

### Community 62 - "Self"
Cohesion: 0.07
Nodes (16): browser_item(), browser_items(), default_action(), exported_db_browser_items(), AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserContext (+8 more)

### Community 63 - "spawn_reader_thread"
Cohesion: 0.13
Nodes (24): ChildStdin, launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), Arc, AtomicBool (+16 more)

### Community 64 - "AbiKeymapConfig"
Cohesion: 0.10
Nodes (17): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, PaneConfig, config(), PaneConfig (+9 more)

### Community 65 - "state_with_user_library"
Cohesion: 0.04
Nodes (122): active_runtime_popup(), ctrl_mod(), install_mark_list_state_for_test(), open_oil_directory(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm() (+114 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.06
Nodes (61): append_query_source(), buffer_text_for_byte_range(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser(), DeferredQuery, desired_indent_for_loaded_language() (+53 more)

### Community 67 - ".default"
Cohesion: 0.08
Nodes (56): Self, DbSchemaCache, load_persisted_state(), load_postgres_schema(), load_sql_server_schema(), load_sqlite_schema(), Path, sqlite_query_execution_and_schema_cache_work() (+48 more)

### Community 68 - "shell/acp.rs"
Cohesion: 0.10
Nodes (58): acp_slash_completion_query(), active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates() (+50 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "LanguageConfiguration"
Cohesion: 0.09
Nodes (13): CaptureThemeMapping, LanguageConfiguration, LanguageLoader, load_language(), normalize_unique_entries(), I, Into, IntoIterator (+5 more)

### Community 72 - "theme.rs"
Cohesion: 0.15
Nodes (30): assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages(), bundled_themes_use_pallet_sections_and_token_references(), list_theme_files() (+22 more)

### Community 73 - ".send"
Cohesion: 0.13
Nodes (36): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+28 more)

### Community 74 - ".new"
Cohesion: 0.08
Nodes (35): AcpRuntime, buffer_lookup_is_scoped_to_workspace(), choose_permission_outcome(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), format_permission_option_kind(), open_permission_request_reorders_queue_for_requested_picker(), PendingPermission (+27 more)

### Community 75 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (62): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+54 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), environment_value(), explicit_windows_env_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (47): LspWorkspaceDiagnostic, PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+39 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (39): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+31 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.06
Nodes (66): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+58 more)

### Community 82 - "AcpManager"
Cohesion: 0.13
Nodes (14): acp_permission_picker_closed(), AcpManager, AcpPendingPermissionUi, AcpUiAction, drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), handle_acp_ui_action(), PendingSlashTrigger (+6 more)

### Community 83 - "FontSet"
Cohesion: 0.08
Nodes (44): DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, alpha_bitmap_surface(), cached_emoji_layout() (+36 more)

### Community 84 - ".from"
Cohesion: 0.06
Nodes (41): main(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), StatuslineContext, main(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers() (+33 more)

### Community 85 - "Option"
Cohesion: 0.10
Nodes (19): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, path_is_solution() (+11 more)

### Community 86 - "volt/src/main.rs"
Cohesion: 0.10
Nodes (33): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), CommandPaletteState, CompilationState, dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, load_user_library() (+25 more)

### Community 87 - "shell_ui_mut"
Cohesion: 0.16
Nodes (36): shell_ui_mut(), buffer_footer_layout(), acp_section_layout_orders_output_input_footer_and_statusline(), browser_input_layout_uses_symmetric_vertical_padding(), install_plugin_sections_test_buffer(), install_plugin_sections_test_buffer_with_update(), install_terminal_test_buffer(), plugin_sections_layout_keeps_output_pane_at_bottom_with_single_row_start() (+28 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.12
Nodes (25): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+17 more)

### Community 89 - ".new"
Cohesion: 0.27
Nodes (16): diff_git_dwim(), git_args_with_no_pager(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), merge_git_preview(), open_git_diff_buffer(), open_git_diff_commit(), open_git_diff_staged() (+8 more)

### Community 90 - "main"
Cohesion: 0.13
Nodes (15): bootstrap(), HostBootstrap, command_palette_items(), main(), panic_payload_message(), Any, Box, DebugAdapterSpec (+7 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandRegistry"
Cohesion: 0.08
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, RegisteredCommand, BTreeMap, Display, Error (+9 more)

### Community 93 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): CommandSource, command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), Into (+12 more)

### Community 94 - ".from_grammar"
Cohesion: 0.11
Nodes (42): default_install_root(), csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting() (+34 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "Option"
Cohesion: 0.11
Nodes (36): active_git_status_command_context(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), git_line_is_untracked() (+28 more)

### Community 97 - "PluginBuffer"
Cohesion: 0.10
Nodes (6): PickerKeybindingContext, PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate, RVec

### Community 98 - "LiveTerminalSession"
Cohesion: 0.12
Nodes (12): AlacrittyEvent, Self, LiveTerminalSession, QueuedEventListener, Arc, Drop, Receiver, Sender (+4 more)

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (14): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, Number (+6 more)

### Community 100 - "shell/browser.rs"
Cohesion: 0.15
Nodes (30): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_url_candidates(), browser_url_prefix_len(), detect_browser_url(), ensure_browser_popup_buffer() (+22 more)

### Community 101 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 102 - "LspInlineCompletionItem"
Cohesion: 0.22
Nodes (8): LspClientState, LspInlineCompletionItem, parse_inline_completion_item(), parse_inline_completion_response(), BTreeSet, PathBuf, SessionKey, TrackedBufferState

### Community 103 - "common.rs"
Cohesion: 0.14
Nodes (18): binding_suffix(), GrammarSourceSpec, GrammarSourceSpec<'a>, CaptureThemeMapping, GrammarSource, LanguageConfiguration, Self, String (+10 more)

### Community 104 - "String"
Cohesion: 0.30
Nodes (21): apply_language_options_table(), apply_options_table(), parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option() (+13 more)

### Community 105 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 106 - "String"
Cohesion: 0.28
Nodes (19): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+11 more)

### Community 107 - "LspServerCommand"
Cohesion: 0.24
Nodes (4): CopilotDeviceCodePrompt, LspServerCommand, parse_copilot_sign_in_response(), parse_lsp_server_command()

### Community 108 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (36): buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay(), picker_overlay_from_spec(), picker_preview_is_opt_in() (+28 more)

### Community 110 - "BufferId"
Cohesion: 0.16
Nodes (28): active_and_secondary_buffer_ids(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items(), git_status_ctrl_v_visual_u_unstages_selected_items() (+20 more)

### Community 111 - "commit_git_buffer"
Cohesion: 0.22
Nodes (11): commit_git_buffer(), git_command_output_owned(), git_commit_message(), git_commit_temp_path(), git_status_action_targets(), git_status_stage_command(), git_status_unstage_command(), mark_git_fringe_snapshots_stale() (+3 more)

### Community 112 - "browser_host.rs"
Cohesion: 0.12
Nodes (14): allow_browser_drag_drop(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests(), browser_navigation_retry_required() (+6 more)

### Community 113 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (47): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+39 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.10
Nodes (26): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, hook_command(), leader_binding() (+18 more)

### Community 115 - "AbiOilDefaults"
Cohesion: 0.21
Nodes (8): exported_oil_defaults(), OilDefaults, AbiOilDefaults, AbiOilSortMode, OilDefaults, OilSortMode, OilDefaults, OilSortMode

### Community 116 - "String"
Cohesion: 0.11
Nodes (13): append_lines(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self, String (+5 more)

### Community 117 - "PluginCommand"
Cohesion: 0.08
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 118 - "DbService"
Cohesion: 0.10
Nodes (22): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbBrowserBufferView, DbService, DbSession (+14 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 121 - "DbEngine"
Cohesion: 0.18
Nodes (9): DbAutocompleteCandidate, DbEngine, DbHistoryEntry, DbIndex, DbQueryBufferMeta, DbSnippet, PersistedDbState, QualifiedName (+1 more)

### Community 122 - "AbiSectionTree"
Cohesion: 0.11
Nodes (16): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree, AbiDirectoryEntry (+8 more)

### Community 123 - "shell/git.rs"
Cohesion: 0.10
Nodes (53): ActiveBufferEventContext, apply_git_view(), find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_delete_target_for_line(), git_status_delete_targets() (+45 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.08
Nodes (25): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_status_command_name(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitPrefixState (+17 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "Option"
Cohesion: 0.13
Nodes (6): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot

### Community 127 - "BrowserBufferState"
Cohesion: 0.29
Nodes (6): browser_display_url_prefers_requested_navigation(), browser_state_for_kind(), BrowserBufferState, BrowserPane, Default, Self

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.15
Nodes (28): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+20 more)

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (21): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+13 more)

### Community 131 - "oil.rs"
Cohesion: 0.09
Nodes (37): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec(), help_entry() (+29 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "Self"
Cohesion: 0.06
Nodes (44): GitStashEntry, exported_language_servers(), AbiContextHelpEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiLanguageServerRootStrategy, AbiLanguageServerSpec (+36 more)

### Community 134 - "test_service"
Cohesion: 0.18
Nodes (15): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_volt_state_dir(), insert_test_session(), Arc, PathBuf, Self, Send (+7 more)

### Community 135 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 138 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 139 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (24): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DebugAdapterSpec, DirectoryEntry (+16 more)

### Community 140 - "AcpEvent"
Cohesion: 0.11
Nodes (23): AvailableCommand, acp_pick_model(), AcpEvent, AcpSessionInfo, config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options() (+15 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 144 - "JobSpec"
Cohesion: 0.25
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "ShellConfig"
Cohesion: 0.15
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (15): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+7 more)

### Community 148 - "setup_standalone_user_repository"
Cohesion: 0.33
Nodes (6): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 149 - "TextRange"
Cohesion: 0.09
Nodes (19): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), diagnostic_matches_request_range(), location_from_link() (+11 more)

### Community 150 - "Duration"
Cohesion: 0.07
Nodes (34): ActiveTypingFrameProfile, average_duration(), current_theme_source_fingerprint(), current_user_config_source_fingerprint(), format_timestamp(), format_typing_frame_profile(), FpsOverlaySnapshot, FpsOverlayState (+26 more)

### Community 151 - "StartupTrace"
Cohesion: 0.40
Nodes (3): Instant, Self, StartupTrace

### Community 153 - "db.rs"
Cohesion: 0.18
Nodes (14): browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord(), query_buffer_lines() (+6 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 156 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 157 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 161 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 163 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "aligned_indent_column"
Cohesion: 0.12
Nodes (23): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column() (+15 more)

### Community 166 - "String"
Cohesion: 0.29
Nodes (6): call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "load"
Cohesion: 0.17
Nodes (13): default_terminal_args(), default_terminal_program(), load(), config(), LigatureConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program() (+5 more)

### Community 168 - "build_output.rs"
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "build_headerline_lines"
Cohesion: 0.22
Nodes (8): packages(), LanguageConfiguration, Vec, syntax_languages(), build_headerline_lines(), headerline_lines(), String, Vec

### Community 171 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 175 - "user/browser.rs"
Cohesion: 0.21
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "browser_sync_plan"
Cohesion: 0.24
Nodes (16): BrowserViewportRect, browser_buffer_layout(), browser_host_viewport_rect(), browser_surface_buffer_at_point(), browser_sync_plan(), browser_viewport_contains_point(), browser_viewport_rect(), browser_viewport_rect_rect() (+8 more)

### Community 177 - "Default"
Cohesion: 0.23
Nodes (10): KeymapSection, OilKeybindingsSection, OilSection, PaneSection, Default, OilKeybindings, PaneConfig, TerminalSection (+2 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "AcpCommand"
Cohesion: 0.17
Nodes (13): acp_permission_approve(), acp_permission_deny(), AcpCommand, build_acp_input_hint(), format_acp_mode_label(), format_acp_model_label(), PermissionDecision, resolve_permission() (+5 more)

### Community 180 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 181 - "markdown.rs"
Cohesion: 0.48
Nodes (6): inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), LanguageConfiguration, syntax_language(), syntax_languages_register_markdown_grammars()

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

### Community 188 - "Vec"
Cohesion: 0.03
Nodes (106): TextSnapshot, acp_build_output_lines(), acp_build_plan_lines(), acp_icon_segment(), acp_multiline_text_lines(), acp_padding_prefix(), acp_pane_content_rows(), acp_pane_cursor_visual_row() (+98 more)

### Community 191 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

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

### Community 216 - "Language"
Cohesion: 0.33
Nodes (5): Issues, Language, Language servers, Volt, Workspace

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 221 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 222 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 240 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **136 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+131 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **5 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `shell/mod.rs`, `shell/tests.rs`, `ShellError`, `String`, `.new`, `String`, `AcpEvent`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `Duration`, `command_stream.rs`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `ShellUiState`, `PathBuf`, `AcpCommand`, `String`, `.id`, `.len`, `Path`, `Vec`, `Result`, `state_with_user_library`, `shell/acp.rs`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `AcpManager`, `shell_ui_mut`, `.new`, `main`, `editor-plugin-host/src/lib.rs`, `CommandRegistry`, `editor-core/src/lib.rs`, `Option`, `shell/browser.rs`, `String`, `shell/picker.rs`, `BufferId`, `commit_git_buffer`, `shell/git.rs`, `GitSummaryState`?**
  _High betweenness centrality (0.129) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `oil.rs`, `user/lib.rs`, `.new`, `sdk/src/lib.rs`, `calculator.rs`, `db.rs`, `lsp.rs`, `AcpPickerItemSpec`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `load`, `build_headerline_lines`, `UserLibraryModule`, `user/browser.rs`, `HeaderlineTestUserLibrary`, `markdown.rs`, `Self`, `syntax_language`, `bash.rs`, `clojure.rs`, `elixir.rs`, `graphql.rs`, `hcl.rs`, `java.rs`, `capture_mappings`, `kotlin.rs`, `latex.rs`, `lua.rs`, `nix.rs`, `perl.rs`, `php.rs`, `proto.rs`, `r.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `xml.rs`, `PickerItemSpec`, `main`, `editor-plugin-host/src/lib.rs`, `syntax_language`, `package`, `PluginBuffer`, `common.rs`, `debug_adapters`, `package`, `PluginKeyBinding`, `PluginCommand`, `cmake.rs`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `shell/mod.rs`, `ShellError`, `.new`, `render.rs`, `TextBuffer`, `shell/pdf.rs`, `state.rs`, `ShellUiState`, `browser_sync_plan`, `PathBuf`, `render_buffer_with_view_state`, `.len`, `Vec`, `Result`, `shell/acp.rs`, `directory.rs`, `shell/terminal.rs`, `shell_ui_mut`, `draw_diagnostic_underlines_for_segment`, `shell/browser.rs`, `shell/picker.rs`, `commit_git_buffer`, `shell/git.rs`, `GitSummaryState`, `Option`, `BrowserBufferState`?**
  _High betweenness centrality (0.069) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _136 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shell/mod.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.029137568648721947 - nodes in this community are weakly interconnected._
- **Should `LspClientError` be split into smaller, more focused modules?**
  _Cohesion score 0.06477838971938106 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.023946360153256706 - nodes in this community are weakly interconnected._