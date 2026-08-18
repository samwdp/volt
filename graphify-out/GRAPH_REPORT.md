# Graph Report - volt  (2026-08-17)

## Corpus Check
- 235 files · ~591,369 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9491 nodes · 38710 edges · 316 communities (283 shown, 33 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3206 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ba1c1cca`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- load_font_set_with_mode
- Path
- shell/tests.rs
- .new
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- sync_quickfix_popup_buffer
- shell/browser.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Path
- String
- .spawn
- editor-fs/src/lib.rs
- editor-git/src/lib.rs
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- String
- Result
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- FontSet
- String
- .new
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- EditorRuntime
- Self
- Option
- shell_buffer
- .new
- PluginBuffer
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- Option
- render_workspace_dock
- clipboard.rs
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- TextPoint
- .with_install_root
- Result
- .from
- .len
- shell/git.rs
- AcpPickerItemSpec
- BufferId
- build_output.rs
- TextRange
- .new
- TextBuffer
- PluginCommand
- PickerOverlay
- SyntaxRegistry
- AcpClient
- shell/acp.rs
- DebugConfiguration
- capture_mappings
- String
- Option
- .send
- DbSessionId
- state.rs
- Section
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- volt/src/main.rs
- Option
- .next_token
- LanguageServerSpec
- ShellUiState
- AbiPaneConfig
- draw_diagnostic_underlines_for_segment
- show_paren.rs
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
- .start
- .load_from_path
- PathBuf
- Self
- Vec
- Diagnostic
- main
- editor-path/src/lib.rs
- refresh_pending_syntax
- shell/picker.rs
- GitStatusSnapshot
- .default
- editor-picker/src/lib.rs
- PickerSession
- PluginKeyBinding
- String
- String
- editor-db/src/lib.rs
- PickerItem
- process_supervisor.rs
- .move_object_end_forward
- record_runtime_error
- AbiKeymapConfig
- Vec
- GitSummaryState
- DirectoryViewState
- DynamicUserLibrary
- LiveTerminalSession
- JobSpec
- Option
- user/config.rs
- AbiOilFeatureSpec
- key_sequence.rs
- editor-icons/src/lib.rs
- load_sql_server_schema
- cmake.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- .get
- String
- LspLocation
- String
- CommandLineOverlay
- .query
- directory.rs
- update_acp_input_hint
- .path
- volt/build.rs
- ShellConfig
- aligned_indent_column
- String
- cargo
- headerline_lines
- oil.rs
- UserLibraryModule
- lsp.rs
- .new
- treesittercontext_ghosttext.rs
- AbiStatuslineContext
- load
- Copilot instructions for `volt`
- TerminalCursorSnapshot
- flatten_config_select_options
- syntax_language
- theme.rs
- ServiceRegistry
- graphql.rs
- String
- user/terminal.rs
- corpus_inventory.rs
- kotlin.rs
- common.rs
- lua.rs
- nix.rs
- JobResult
- perl.rs
- user/browser.rs
- php.rs
- treesittercontext_shared.rs
- `user`
- client.rs
- predicate_capture_text
- r.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- ruby.rs
- package
- ancestor_contexts_for_cursor
- rainbow_parens.rs
- .oil_directory_sections
- solidity.rs
- Database Explorer PRD
- index_syntax_lines
- .from_text
- AbiOilKeyAction
- GitStashEntry
- AcpManager
- .from_hook_detail
- .acp_client_by_id
- markdown.rs
- bash.rs
- clojure.rs
- elixir.rs
- syntax_language
- hcl.rs
- java.rs
- .git_command_for_chord
- 0004-markdown-pretty-pipeline.md
- package
- proto.rs
- scala.rs
- .autocomplete_providers
- swift.rs
- lang/vim.rs
- xml.rs
- Language
- .browser_feature_spec
- Domain Docs
- Issue tracker: GitHub
- .context_help_specs
- .db_feature_spec
- .debug_adapters
- .ghost_text_lines
- .git_feature_spec
- .hover_providers
- rainbow_paren.rs
- AbiPickerTruncateStrategy
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pdf_open_mode
- user/workspace_dock.rs
- AbiPdfOpenMode
- .picker_layout
- LspLogEntry
- .picker_truncate_strategy
- main
- .show_paren_config
- keymap.rs
- .terminal_config
- shell/mod.rs
- .terminal_feature_spec
- .workspace_roots
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- syntax_language
- Agent skills
- debug_adapters
- syntax_languages
- package
- ligatures.rs
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 767 edges
2. `ShellBuffer` - 377 edges
3. `shell_ui_mut()` - 342 edges
4. `register_shell_hooks()` - 263 edges
5. `shell_ui()` - 230 edges
6. `ShellError` - 192 edges
7. `shell_buffer()` - 183 edges
8. `shell_buffer_mut()` - 183 edges
9. `TextBuffer` - 180 edges
10. `ShellUiState` - 174 edges

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

## Communities (316 total, 33 thin omitted)

### Community 0 - "load_font_set_with_mode"
Cohesion: 0.08
Nodes (28): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode(), load_icon_font() (+20 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (27): CodeActionParams, code_action_params(), inline_completion_params(), is_copilot_server(), LspClientError, LspClientManager, LspSessionHandle, path_to_uri() (+19 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (63): load_font_set(), rasterize_icon_glyph_for_cell(), acp_agent_markdown_uses_shared_pipeline_pretty(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji() (+55 more)

### Community 3 - ".new"
Cohesion: 0.12
Nodes (72): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+64 more)

### Community 4 - "ShellError"
Cohesion: 0.05
Nodes (37): Display, Error, From, ShellError, clear_key_sequence(), active_lsp_workspace_loaded(), active_runtime_surface(), alt_mod() (+29 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.02
Nodes (120): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+112 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.11
Nodes (63): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+55 more)

### Community 7 - "sync_quickfix_popup_buffer"
Cohesion: 0.11
Nodes (17): quickfix_clear_marks(), quickfix_entry_for_cursor(), quickfix_mark_all(), quickfix_open_current_list(), quickfix_open_from_one_shot(), quickfix_open_picker_matches(), quickfix_state(), quickfix_state_mut() (+9 more)

### Community 8 - "shell/browser.rs"
Cohesion: 0.05
Nodes (74): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+66 more)

### Community 9 - "render.rs"
Cohesion: 0.04
Nodes (113): acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), acp_buffer_layout(), acp_chat_bubble_width_px(), acp_chat_origin_x(), acp_pane_body_visible_rows(), acp_prefix_columns(), acp_slice_chars() (+105 more)

### Community 10 - "AcpEvent"
Cohesion: 0.10
Nodes (24): AcpEvent, choose_permission_outcome(), coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), format_permission_option_kind(), PendingPermission (+16 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (42): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+34 more)

### Community 12 - "Self"
Cohesion: 0.07
Nodes (11): browser_item(), default_action(), AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, PickerActionSpec (+3 more)

### Community 13 - "Path"
Cohesion: 0.07
Nodes (57): parse_log_oneline(), build_git_fringe_snapshot(), command_output_transcript(), commit_git_buffer(), create_git_worktree_from_query(), git_branch_merge(), git_branch_push_remote(), git_branch_remote() (+49 more)

### Community 14 - "String"
Cohesion: 0.06
Nodes (73): shell_ui(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer(), acp_slash_picker_backspace_can_delete_leading_slash(), acp_slash_picker_text_input_updates_acp_input(), acp_switch_pane_command_changes_internal_pane_without_changing_workspace_pane(), browser_buffer_submit_tracks_requested_navigation(), browser_escape_from_insert_keeps_input_cursor_position() (+65 more)

### Community 15 - ".spawn"
Cohesion: 0.09
Nodes (20): Keycode, Mod, terminal_key_for_event(), live_terminal_session_spawns_and_terminates(), LiveTerminalError, Display, Error, Formatter (+12 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.11
Nodes (40): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+32 more)

### Community 17 - "editor-git/src/lib.rs"
Cohesion: 0.13
Nodes (25): configure_background_command(), detect_in_progress(), git_available(), GitStatusError, list_repository_files(), parse_header(), parse_stash_list(), parse_status() (+17 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.05
Nodes (114): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+106 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.04
Nodes (18): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GhostTextLine, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig (+10 more)

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
Cohesion: 0.08
Nodes (32): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+24 more)

### Community 24 - "String"
Cohesion: 0.12
Nodes (19): DbActionOutcome, DbBrowserBufferView, DbService, DbSessionSummary, DisabledSecretStore, InMemorySecretStore, redact_error(), redact_key_value_segments() (+11 more)

### Community 25 - "Result"
Cohesion: 0.06
Nodes (111): active_runtime_popup(), ctrl_mod(), install_mark_list_state_for_test(), open_workspace_from_project(), add_linked_worktree(), browser_host_new_window_event_routes_into_browser_popup(), browser_insert_mode_ctrl_enter_binding_submits_current_url(), browser_popup_command_focuses_the_popup_surface() (+103 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.07
Nodes (51): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+43 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "FontSet"
Cohesion: 0.08
Nodes (47): Canvas, RenderColor, FontSet, alpha_bitmap_surface(), cached_primary_text_runs(), CachedLigatureGlyphPlacement, CachedLigatureLayout, compose_emoji_surface() (+39 more)

### Community 32 - "String"
Cohesion: 0.09
Nodes (63): run_command(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), create_git_worktree(), delete_git_status_targets(), ensure_rebase_in_progress(), fetch_git_all() (+55 more)

### Community 33 - ".new"
Cohesion: 0.12
Nodes (34): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command(), git_status_log_all_command() (+26 more)

### Community 34 - "shell/pdf.rs"
Cohesion: 0.10
Nodes (52): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_delete_page(), pdf_fit_mode_label(), pdf_header_lines() (+44 more)

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
Cohesion: 0.04
Nodes (256): EditorRuntime, Default, focus_active_browser_popup(), focus_browser_input_section(), Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete() (+248 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (97): absolute_path_hint(), acp_decode_image(), acp_tool_call_from_partial_update(), active_buffer_revision_key(), active_shell_workspace_id(), advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_line_indent() (+89 more)

### Community 42 - "shell_buffer"
Cohesion: 0.07
Nodes (88): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+80 more)

### Community 43 - ".new"
Cohesion: 0.04
Nodes (109): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_escape_from_insert_keeps_input_cursor_position(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_plan_entries_normalize_completed_prefix_when_later_step_is_active() (+101 more)

### Community 44 - "PluginBuffer"
Cohesion: 0.08
Nodes (6): PickerKeybindingContext, PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate, RVec

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (67): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+59 more)

### Community 47 - "Option"
Cohesion: 0.07
Nodes (51): BufRead, completion_documentation(), completion_level_for_message(), configuration_item_section(), CopilotDeviceCodePrompt, csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params() (+43 more)

### Community 48 - "render_workspace_dock"
Cohesion: 0.14
Nodes (21): refresh_workspace_dock_branches(), render_workspace_dock(), Arc, HashMap, Instant, Mutex, Option, Path (+13 more)

### Community 49 - "clipboard.rs"
Cohesion: 0.19
Nodes (13): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+5 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.15
Nodes (28): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+20 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (35): AtomicUsize, active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc (+27 more)

### Community 52 - "TextPoint"
Cohesion: 0.08
Nodes (7): advance_point_by_text(), Selection, TextPoint, TextSnapshot, InlineCompletionState, UndoSnapshot, UndoTree

### Community 53 - ".with_install_root"
Cohesion: 0.10
Nodes (20): asset_path_from_parts(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), finalize_language_install_removes_compiler_sidecars(), install_plan_compile_command_prefers_cpp_scanner(), install_plan_compile_command_uses_windows_msvc_for_c_scanner(), install_plan_reports_missing_grammar_sources_before_compile() (+12 more)

### Community 54 - "Result"
Cohesion: 0.08
Nodes (66): active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), diff_git_commit_at_point(), ensure_no_rebase_in_progress(), fetch_git_pushremote(), fetch_git_upstream(), finish_oil_worktree_branch_selection() (+58 more)

### Community 55 - ".from"
Cohesion: 0.09
Nodes (113): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root() (+105 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (18): apply_input_operator_motion(), ascii_control_caret_notation(), byte_index_for_char_column(), display_columns_for_character(), input_charwise_motion_range(), InputField, is_wide_display_character(), is_zero_width_display_character() (+10 more)

### Community 57 - "shell/git.rs"
Cohesion: 0.08
Nodes (43): ActiveBufferEventContext, begin_oil_worktree_request(), git_branch_list(), git_remote_worktree_branch_list(), git_status_branches_command(), git_status_checkout_file_command(), git_status_command_name(), git_status_diff_paths_command() (+35 more)

### Community 58 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 59 - "BufferId"
Cohesion: 0.12
Nodes (44): active_and_secondary_buffer_ids(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status() (+36 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "TextRange"
Cohesion: 0.12
Nodes (10): delimiter_partner(), find_matching_close_tag(), is_inline_whitespace(), parse_tag_token(), parse_tag_token_at(), Fn, Option, ShowParenMatch (+2 more)

### Community 62 - ".new"
Cohesion: 0.13
Nodes (21): build_tokio_runtime(), DbExecutionOutput, default_volt_state_dir(), execute_postgres(), execute_sql_server(), execute_sqlite(), initialize_native_keyring(), render_rows() (+13 more)

### Community 63 - "TextBuffer"
Cohesion: 0.08
Nodes (13): BufferStats, EditRecord, is_sentence_closer(), LineEnding, Default, String, Vec, TextBuffer (+5 more)

### Community 64 - "PluginCommand"
Cohesion: 0.06
Nodes (29): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+21 more)

### Community 65 - "PickerOverlay"
Cohesion: 0.08
Nodes (17): GitBranchActionKind, GitCommitActionKind, keydown_chord_token(), KeydownChordToken, normalize_named_key_token(), PickerAction, PickerKind, PickerOverlay (+9 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.08
Nodes (49): apply_text_edits_to_span(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), create_parser(), desired_indent_for_loaded_language(), highlight_inline_language_per_line() (+41 more)

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
Nodes (18): CaptureThemeMapping, command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport, LanguageConfiguration, LanguageLoader, normalize_unique_entries(), I (+10 more)

### Community 72 - "Option"
Cohesion: 0.09
Nodes (17): append_query_source(), buffer_text_for_byte_range(), GrammarSource, io_error(), load_language(), maybe_read_bundled_query_source(), optional_query_source(), parse_query_inherits() (+9 more)

### Community 73 - ".send"
Cohesion: 0.16
Nodes (36): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession, AcpTerminal (+28 more)

### Community 74 - "DbSessionId"
Cohesion: 0.24
Nodes (8): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbSessionId, insert_test_session(), sqls_initialization_options_for_query_buffer_use_attached_session(), sqls_workspace_settings_for_query_buffer_use_attached_session(), sqls_workspace_settings_preserve_mssql_data_source_name(), test_service()

### Community 75 - "state.rs"
Cohesion: 0.14
Nodes (25): multicursor_cursor_points(), BlockInsertState, DirectoryYankEntry, LastFind, LastSearch, MulticursorState, BTreeMap, BufferId (+17 more)

### Community 76 - "Section"
Cohesion: 0.14
Nodes (14): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+6 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (43): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing() (+35 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (63): PickerEntry, search_is_case_sensitive(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), file_context_preview() (+55 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (39): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+31 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.05
Nodes (68): ProjectCandidate, exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search() (+60 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.10
Nodes (31): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, LspState, panic_payload_message(), parse_launch_options() (+23 more)

### Community 83 - "Option"
Cohesion: 0.11
Nodes (30): cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_stash_at_point(), git_commit_at_point(), git_line_is_untracked(), git_status_apply_commit_command(), git_status_cherry_pick_apply_command(), git_status_cherry_pick_command() (+22 more)

### Community 85 - "LanguageServerSpec"
Cohesion: 0.05
Nodes (63): Client, csharp_language_server(), dev_extension_server(), directory_contains_extension(), directory_matches_root_marker(), dockerfile_language_server(), document_language_id_for_extension(), document_language_id_for_glob() (+55 more)

### Community 86 - "ShellUiState"
Cohesion: 0.02
Nodes (125): BufferKind, InputPromptOverlay, ActiveLspBufferContext, default_vim_target(), WorkspaceId, active_lsp_code_action_range(), active_runtime_buffer(), active_window_id() (+117 more)

### Community 87 - "AbiPaneConfig"
Cohesion: 0.05
Nodes (25): exported_ligature_config(), exported_pane_config(), LigatureConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig, config(), AbiLigatureConfig (+17 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.12
Nodes (26): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+18 more)

### Community 89 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 90 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

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

### Community 96 - "WorkspaceConfigurationValue"
Cohesion: 0.10
Nodes (20): sanitize_transport_message(), transport_key_is_sensitive(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, From, I, Number, T (+12 more)

### Community 97 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - ".start"
Cohesion: 0.16
Nodes (19): ChildStdin, diagnostic_matches_request_range(), launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), AtomicBool (+11 more)

### Community 101 - ".load_from_path"
Cohesion: 0.08
Nodes (19): normalize_inline_text(), AsRef, Drop, Into, Item, Iterator, Path, PathBuf (+11 more)

### Community 102 - "PathBuf"
Cohesion: 0.13
Nodes (17): diagnostics_parser_maps_lsp_fields(), file_uri_to_path(), language_server_session_in_workspace_scope(), LspClientState, LspInlineCompletionItem, LspLiveSession, normalize_path_for_compare(), normalize_session_root() (+9 more)

### Community 103 - "Self"
Cohesion: 0.02
Nodes (134): GitCommandBinding, GitPrefixBinding, GitStashEntry, exported_debug_adapters(), exported_syntax_languages(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag() (+126 more)

### Community 104 - "Vec"
Cohesion: 0.07
Nodes (16): CommandPaletteState, CompilationState, DapState, EventLog, format_micros_as_millis(), AcpClient, AutocompleteProvider, ContextHelpSpec (+8 more)

### Community 105 - "Diagnostic"
Cohesion: 0.09
Nodes (23): formatting_parser_maps_text_edits(), full_document_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), lsp_formatting_options(), lsp_position_from_text_point(), lsp_range_from_text_range(), lsp_text_edit_from_lsp() (+15 more)

### Community 106 - "main"
Cohesion: 0.12
Nodes (16): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), Arc, DebugAdapterSpec, Error (+8 more)

### Community 107 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 108 - "refresh_pending_syntax"
Cohesion: 0.12
Nodes (11): built_user_library_path_for_command(), command_builds_user_library(), lsp_diagnostic_belongs_to_workspace(), normalize_tabs(), refresh_buffer_syntax(), refresh_pending_syntax(), Cow, HashSet (+3 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (37): UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars(), picker_overlay_from_spec() (+29 more)

### Community 110 - "GitStatusSnapshot"
Cohesion: 0.17
Nodes (5): GitLogEntry, GitStatusSnapshot, Option, Self, Vec

### Community 111 - ".default"
Cohesion: 0.09
Nodes (52): Self, load_persisted_state(), load_postgres_schema(), Path, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section() (+44 more)

### Community 112 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 113 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - "String"
Cohesion: 0.13
Nodes (4): RepositoryStatus, Into, String, StatusEntry

### Community 116 - "String"
Cohesion: 0.11
Nodes (13): append_lines(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self, String (+5 more)

### Community 117 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (41): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), db_browser_action_from_spec(), DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbColumn (+33 more)

### Community 118 - "PickerItem"
Cohesion: 0.20
Nodes (7): match_item(), PickerItem, PickerMatch, Into, Option, Self, String

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - ".move_object_end_forward"
Cohesion: 0.15
Nodes (7): is_object_separator(), is_punctuation_char(), is_word_char(), matches_word_kind(), word_motion_class(), WordKind, WordMotionClass

### Community 121 - "record_runtime_error"
Cohesion: 0.21
Nodes (18): apply_browser_location_updates(), ensure_browser_popup_buffer(), navigate_browser_buffer(), normalize_browser_url(), open_active_buffer_in_browser_split(), open_browser_buffer_in_popup(), open_browser_buffer_in_split(), open_detected_browser_url() (+10 more)

### Community 122 - "AbiKeymapConfig"
Cohesion: 0.32
Nodes (5): exported_keymap_config(), KeymapConfig, AbiKeymapConfig, KeymapConfig, KeymapConfig

### Community 123 - "Vec"
Cohesion: 0.18
Nodes (30): SectionRenderLine, oil_directory_line_spans(), find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token() (+22 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.09
Nodes (20): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitPrefixState, GitSummarySnapshot (+12 more)

### Community 125 - "DirectoryViewState"
Cohesion: 0.19
Nodes (17): apply_directory_state(), directory_cd_from_cursor(), directory_entry_at_cursor(), directory_entry_label(), directory_root_for_entry(), directory_visible_entries(), DirectoryOpenMode, DirectoryViewState (+9 more)

### Community 127 - "LiveTerminalSession"
Cohesion: 0.12
Nodes (12): AlacrittyEvent, Self, LiveTerminalSession, QueuedEventListener, Arc, Drop, Receiver, Sender (+4 more)

### Community 128 - "JobSpec"
Cohesion: 0.10
Nodes (26): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobKind (+18 more)

### Community 129 - "Option"
Cohesion: 0.13
Nodes (6): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "AbiOilFeatureSpec"
Cohesion: 0.11
Nodes (16): AbiIconFontSymbol, AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeybindings, AbiOilSortMode, IconFontSymbol, OilDefaults, OilFeatureSpec (+8 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 134 - "load_sql_server_schema"
Cohesion: 0.13
Nodes (16): ColumnData, Compat, connect_sql_server(), load_sql_server_schema(), OsSecretStore, postgres_columns_by_table(), BTreeMap, sql_server_cell() (+8 more)

### Community 135 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - ".get"
Cohesion: 0.18
Nodes (8): DbEngine, DbHistoryEntry, DbQueryBufferMeta, DbSnippet, pad_row(), PersistedDbState, QualifiedName, RememberedConnection

### Community 138 - "String"
Cohesion: 0.22
Nodes (17): apply_directory_edit_actions(), apply_directory_edit_queue(), create_dir_action_creates_empty_directory(), diff_directory_lines(), directory_edit_actions(), directory_edit_lines(), DirectoryEditAction, parse_directory_lines() (+9 more)

### Community 139 - "LspLocation"
Cohesion: 0.16
Nodes (11): definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), location_from_link(), location_from_lsp(), location_sorting_deduplicates_reference_results(), LspLocation, parse_definition_response(), parse_reference_response() (+3 more)

### Community 140 - "String"
Cohesion: 0.09
Nodes (52): acp_complete_slash(), acp_connected(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_picker_closed(), acp_permission_picker_submitted(), acp_pick_mode(), acp_pick_model() (+44 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (9): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, BufferId, Into, Option, Self, String (+1 more)

### Community 142 - ".query"
Cohesion: 0.19
Nodes (10): compile_query_source(), DeferredQuery, html_language(), query_capture_property_value(), query_capture_property_value_returns_set_property(), query_compiler_accepts_vim_case_insensitive_regex_prefix(), Language, rust_language() (+2 more)

### Community 143 - "directory.rs"
Cohesion: 0.29
Nodes (12): copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), DirectoryLine, oil_directory_theme_token(), oil_entry_theme_token(), oil_file_theme_token(), parse_directory_line() (+4 more)

### Community 144 - "update_acp_input_hint"
Cohesion: 0.21
Nodes (10): acp_permission_approve(), acp_permission_deny(), build_acp_input_hint(), format_acp_mode_label(), format_acp_model_label(), PermissionDecision, resolve_permission(), update_acp_input_hint() (+2 more)

### Community 145 - ".path"
Cohesion: 0.21
Nodes (12): db_query_buffer_receives_sql_highlighting_without_blocking(), db_table_preview_buffer_exposes_hidden_sqls_path_without_file_open_hooks(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir() (+4 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 148 - "aligned_indent_column"
Cohesion: 0.21
Nodes (12): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column(), query_property_is_set() (+4 more)

### Community 149 - "String"
Cohesion: 0.06
Nodes (31): active_parameter_label(), documentation_lines(), hover_marked_string(), hover_marked_string_markdown_text(), hover_text(), hover_text_lines(), LspCodeAction, LspDocumentTextEdits (+23 more)

### Community 150 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 151 - "headerline_lines"
Cohesion: 0.31
Nodes (6): build_headerline_lines(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (36): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+28 more)

### Community 153 - "UserLibraryModule"
Cohesion: 0.13
Nodes (20): browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord() (+12 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".new"
Cohesion: 0.22
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 156 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 157 - "AbiStatuslineContext"
Cohesion: 0.31
Nodes (6): exported_statusline_render(), statusline_context_from_abi(), AbiLspDiagnosticsInfo, AbiStatuslineContext, LspDiagnosticsInfo, LspDiagnosticsInfo

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

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

### Community 165 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 170 - "common.rs"
Cohesion: 0.10
Nodes (27): binding_suffix(), GrammarSourceSpec, GrammarSourceSpec<'a>, package(), package_with_path_matchers(), CaptureThemeMapping, GrammarSource, LanguageConfiguration (+19 more)

### Community 171 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 172 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 173 - "JobResult"
Cohesion: 0.16
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

### Community 179 - "client.rs"
Cohesion: 0.04
Nodes (94): ClientCapabilities, apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), close_buffer_keeps_session_alive_for_next_file() (+86 more)

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

### Community 186 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 187 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 188 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 189 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 190 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 191 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 194 - ".from_text"
Cohesion: 0.10
Nodes (44): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings(), large_buffers_expose_line_windows_without_full_materialization() (+36 more)

### Community 195 - "AbiOilKeyAction"
Cohesion: 0.60
Nodes (3): AbiOilKeyAction, OilKeyAction, OilKeyAction

### Community 197 - "AcpManager"
Cohesion: 0.12
Nodes (26): AcpClientConfig, AvailableCommand, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_set_mode(), acp_set_model() (+18 more)

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

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

### Community 233 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 234 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 236 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 242 - "shell/mod.rs"
Cohesion: 0.01
Nodes (310): DbAutocompleteCandidate, acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter(), acp_multiline_text_lines() (+302 more)

### Community 248 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 251 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 252 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 255 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

## Knowledge Gaps
- **142 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+137 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **33 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `shell/tests.rs`, `ShellError`, `sync_quickfix_popup_buffer`, `AcpEvent`, `String`, `String`, `Path`, `String`, `update_acp_input_hint`, `.path`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `Result`, `command_stream.rs`, `String`, `.new`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `shell_buffer`, `Result`, `.len`, `shell/git.rs`, `BufferId`, `PickerOverlay`, `shell/acp.rs`, `AcpManager`, `workspace_search.rs`, `shell/terminal.rs`, `Option`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `main`, `refresh_pending_syntax`, `shell/picker.rs`, `shell/mod.rs`, `record_runtime_error`, `GitSummaryState`, `DirectoryViewState`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `Option`, `ShellError`, `sync_quickfix_popup_buffer`, `shell/browser.rs`, `render.rs`, `String`, `Path`, `.new`, `shell/pdf.rs`, `EditorRuntime`, `shell_buffer`, `.new`, `TextPoint`, `.from`, `.len`, `TextBuffer`, `PickerOverlay`, `shell/acp.rs`, `state.rs`, `shell/terminal.rs`, `Option`, `ShellUiState`, `draw_diagnostic_underlines_for_segment`, `refresh_pending_syntax`, `shell/picker.rs`, `shell/mod.rs`, `Vec`, `GitSummaryState`, `DirectoryViewState`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `.from` to `load_font_set_with_mode`, `shell/tests.rs`, `ShellError`, `user/lib.rs`, `shell/browser.rs`, `String`, `directory.rs`, `ShellConfig`, `DynamicUserLibrary`, `editor-render/src/lib.rs`, `HoverOverlay`, `Option`, `shell_buffer`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `shell/git.rs`, `volt/src/main.rs`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `main`, `shell/picker.rs`, `shell/mod.rs`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.043) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _142 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `load_font_set_with_mode` be split into smaller, more focused modules?**
  _Cohesion score 0.08418367346938775 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.07915057915057915 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.030952380952380953 - nodes in this community are weakly interconnected._