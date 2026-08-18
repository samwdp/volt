# Graph Report - volt  (2026-08-18)

## Corpus Check
- 237 files · ~602,788 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9678 nodes · 39596 edges · 308 communities (278 shown, 30 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3266 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `baf612d3`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- UserLibrary
- Path
- shell/tests.rs
- .new
- Result
- user/lib.rs
- editor-syntax/src/lib.rs
- PickerOverlay
- browser_host.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Path
- Result
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- Result
- state_with_user_library
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
- String
- Self
- Option
- String
- shell_ui_mut
- Instant
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- .buffer_mut
- db_service_mut
- AbiContextHelpSpec
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- editor-lsp/src/lib.rs
- LanguageServerSpec
- shell/git.rs
- render_buffer_with_view_state
- .len
- PluginBuffer
- picker_items
- active_runtime_popup
- build_output.rs
- state.rs
- DbSessionId
- TextBuffer
- PluginCommand
- Section
- SyntaxRegistry
- execute_oil_action
- shell/acp.rs
- DebugConfiguration
- capture_mappings
- WorkspaceConfigurationValue
- String
- .send
- Option
- clipboard.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- volt/src/main.rs
- EditorRuntime
- WorkspaceConfiguration
- Option
- ShellUiState
- Self
- draw_diagnostic_underlines_for_segment
- show_paren.rs
- .oil_directory_sections
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- GhostTextContext
- editor-picker/src/lib.rs
- GitEditorState
- modeline.rs
- LspSessionHandle
- .spawn
- wrap_line_segments
- .from
- LspLocation
- .new
- main
- editor-path/src/lib.rs
- JobSpec
- shell/picker.rs
- RVec
- .default
- String
- ShellConfig
- PluginKeyBinding
- DbService
- String
- String
- LspLogEntry
- process_supervisor.rs
- AbiSectionTree
- shell/browser.rs
- LspCodeAction
- LineSyntaxSpan
- Option
- Vec
- DynamicUserLibrary
- Option
- JobError
- TerminalRenderSnapshot
- user/config.rs
- UserLibraryModule
- key_sequence.rs
- editor-icons/src/lib.rs
- .get
- common.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- build_job_command
- treesittercontext_ghosttext.rs
- DbEngine
- .new
- CommandLineOverlay
- AbiLanguageConfiguration
- PaneConfig
- resolve_permission
- TextEdit
- volt/build.rs
- treesittercontext_shared.rs
- headerline_lines
- .null
- ancestor_contexts_for_cursor
- StatuslineContext
- oil.rs
- user/db.rs
- lsp.rs
- terminal_key_for_event
- TextRange
- latex.rs
- load
- Copilot instructions for `volt`
- AcpManager
- syntax_language
- theme.rs
- ServiceRegistry
- lua.rs
- String
- user/terminal.rs
- corpus_inventory.rs
- kotlin.rs
- nix.rs
- r.rs
- bash.rs
- JobResult
- clojure.rs
- user/browser.rs
- elixir.rs
- graphql.rs
- `user`
- client.rs
- predicate_capture_text
- hcl.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- ruby.rs
- java.rs
- perl.rs
- php.rs
- .oil_directory_sections
- proto.rs
- Database Explorer PRD
- solidity.rs
- .from_text
- swift.rs
- lang/vim.rs
- String
- xml.rs
- .acp_client_by_id
- markdown.rs
- AbiPickerTruncateStrategy
- scala.rs
- cargo
- rainbow_parens.rs
- .git_command_for_chord
- 0004-markdown-pretty-pipeline.md
- debug_adapters
- syntax_languages
- git_remote_worktree_branch_list
- .autocomplete_providers
- package
- Language
- .browser_feature_spec
- Domain Docs
- Issue tracker: GitHub
- .context_help_specs
- .db_feature_spec
- .debug_adapters
- AbiTheme
- .git_feature_spec
- .hover_providers
- rainbow_paren.rs
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pdf_open_mode
- user/workspace_dock.rs
- package
- .picker_layout
- package
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
- centered_rect
- ligatures.rs
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 788 edges
2. `ShellBuffer` - 384 edges
3. `shell_ui_mut()` - 351 edges
4. `register_shell_hooks()` - 265 edges
5. `shell_ui()` - 237 edges
6. `shell_buffer_mut()` - 194 edges
7. `ShellError` - 192 edges
8. `shell_buffer()` - 192 edges
9. `TextBuffer` - 180 edges
10. `ShellUiState` - 178 edges

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
- 2-file cycle: `crates/editor-render/src/lib.rs -> crates/editor-render/src/split_layout.rs -> crates/editor-render/src/lib.rs`

## Communities (308 total, 30 thin omitted)

### Community 0 - "UserLibrary"
Cohesion: 0.06
Nodes (47): BufferKind, browser_state_for_kind(), default_vim_target(), active_buffer_event_context(), buffer_interaction(), buffer_is_browser(), buffer_is_command_output(), buffer_is_db_connect() (+39 more)

### Community 1 - "Path"
Cohesion: 0.09
Nodes (22): inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, parse_definition_response(), parse_text_edit_response() (+14 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (56): load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), acp_output_speaker_roles_and_tool_chip(), acp_tool_diff_renders_added_and_removed_lines(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell() (+48 more)

### Community 3 - ".new"
Cohesion: 0.11
Nodes (74): path_to_file_url_encodes_spaces(), autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean() (+66 more)

### Community 4 - "Result"
Cohesion: 0.04
Nodes (63): Display, Error, From, ShellError, clear_key_sequence(), active_lsp_workspace_loaded(), active_runtime_surface(), alt_mod() (+55 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (90): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_acp_picker_items(), exported_autocomplete_providers() (+82 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.07
Nodes (87): additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+79 more)

### Community 7 - "PickerOverlay"
Cohesion: 0.05
Nodes (36): absolute_path_hint(), buffer_is_quickfix(), GitBranchActionKind, GitCommitActionKind, keycode_name_token(), keydown_chord_token(), KeydownChordToken, normalize_named_key_token() (+28 more)

### Community 8 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (111): Rect, acp_buffer_layout(), acp_pane_body_visible_rows(), AcpBufferLayout, AcpPaneLayout, adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines() (+103 more)

### Community 10 - "AcpEvent"
Cohesion: 0.10
Nodes (29): AvailableCommand, AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), format_acp_mode_label() (+21 more)

### Community 11 - "PluginPackage"
Cohesion: 0.07
Nodes (40): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+32 more)

### Community 12 - "Self"
Cohesion: 0.06
Nodes (18): browser_item(), browser_items(), default_action(), exported_db_browser_items(), AcpActionSpec, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind (+10 more)

### Community 13 - "Path"
Cohesion: 0.09
Nodes (45): parse_log_oneline(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), create_git_worktree_from_query(), git_command_output_background(), git_commit_list(), git_common_dir() (+37 more)

### Community 14 - "Result"
Cohesion: 0.06
Nodes (74): shell_ui(), split_runtime_pane(), active_and_secondary_buffer_ids(), browser_normal_mode_i_binding_focuses_input_without_inserting_text(), browser_open_buffer_command_uses_existing_split_pane(), browser_popup_command_focuses_the_popup_surface(), closing_streamed_command_popup_kills_worker(), configure_file_buffer() (+66 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.08
Nodes (22): AlacrittyEvent, LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc, Display, Drop, Error (+14 more)

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
Nodes (17): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig, MarkdownPrettyConfig (+9 more)

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

### Community 24 - "Result"
Cohesion: 0.15
Nodes (10): DbActionOutcome, DbSessionSummary, DisabledSecretStore, InMemorySecretStore, redact_error(), remembered_connections_store_metadata_separately_from_secret(), HashMap, Result (+2 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.05
Nodes (100): ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_text_input_updates_acp_input() (+92 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.05
Nodes (77): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches() (+69 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (31): AutocompleteProviderKind, RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay, HoverProviderContent (+23 more)

### Community 30 - "Theme"
Cohesion: 0.08
Nodes (25): text_style_from_theme_style(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display (+17 more)

### Community 31 - "FontSet"
Cohesion: 0.07
Nodes (51): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, acp_slice_chars() (+43 more)

### Community 32 - "String"
Cohesion: 0.07
Nodes (91): active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), diff_git_commit_at_point(), ensure_no_rebase_in_progress(), ensure_rebase_in_progress(), fetch_git_pushremote(), fetch_git_remote() (+83 more)

### Community 33 - ".new"
Cohesion: 0.17
Nodes (24): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_view_language_id(), git_view_lines(), git_view_lines_or_error() (+16 more)

### Community 34 - "shell/pdf.rs"
Cohesion: 0.10
Nodes (51): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_delete_page(), pdf_fit_mode_label(), pdf_header_lines() (+43 more)

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

### Community 39 - "String"
Cohesion: 0.04
Nodes (186): Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), active_lsp_code_action_range(), active_shell_buffer_has_input(), active_shell_buffer_id(), active_shell_buffer_is_terminal() (+178 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (75): acp_output_header_title(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_tool_call_from_partial_update(), AcpBufferState, AcpPane (+67 more)

### Community 42 - "String"
Cohesion: 0.07
Nodes (107): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+99 more)

### Community 43 - "shell_ui_mut"
Cohesion: 0.04
Nodes (106): shell_ui_mut(), buffer_footer_layout(), render_buffer(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail() (+98 more)

### Community 44 - "Instant"
Cohesion: 0.05
Nodes (26): ActiveTypingFrameProfile, average_duration(), DirectoryPrefixState, format_duration_ms(), FpsOverlayState, frame_pacing_remaining(), git_refresh_deferred_for_typing(), KeySequenceState (+18 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (67): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+59 more)

### Community 47 - ".buffer_mut"
Cohesion: 0.07
Nodes (46): active_lsp_buffer_context(), apply_copilot_auth_notification(), apply_lsp_notifications(), apply_lsp_text_edits(), apply_pending_lsp_state(), apply_sqls_workspace_settings_for_active_buffer_context(), apply_sqls_workspace_settings_for_buffer(), autocomplete_request_for_buffer() (+38 more)

### Community 48 - "db_service_mut"
Cohesion: 0.12
Nodes (32): activate_db_browser_line(), active_dashboard_editor_buffer(), active_or_open_dashboard_buffer(), apply_db_browser_view(), buffer_is_db_browser(), buffer_is_db_dashboard(), buffer_is_db_sidebar(), create_db_query_buffer() (+24 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.09
Nodes (18): exported_browser_feature_spec(), exported_db_feature_spec(), exported_terminal_feature_spec(), BrowserFeatureSpec, DbFeatureSpec, TerminalFeatureSpec, AbiBrowserFeatureSpec, AbiContextHelpSpec (+10 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.12
Nodes (26): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+18 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (33): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc, AutocompleteProvider (+25 more)

### Community 52 - "editor-lsp/src/lib.rs"
Cohesion: 0.22
Nodes (27): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+19 more)

### Community 53 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (14): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), LanguageServerRootStrategy, LanguageServerSpec, normalize_optional_string(), BTreeMap, Into (+6 more)

### Community 54 - "shell/git.rs"
Cohesion: 0.09
Nodes (43): ActiveBufferEventContext, commit_git_buffer(), git_command_output_owned(), git_commit_message(), git_commit_temp_path(), git_line_is_untracked(), git_status_action_targets(), git_status_checkout_file_command() (+35 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.11
Nodes (100): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+92 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (30): advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update(), ascii_control_caret_notation(), detect_markdown_table(), format_markdown_table_at_cursor(), input_charwise_motion_range(), InputField (+22 more)

### Community 57 - "PluginBuffer"
Cohesion: 0.06
Nodes (13): dashboard_sections(), sidebar_sections(), DbBrowserKind, PickerKeybindingContext, plugin_buffer_sections_can_declare_nested_layout_tree(), PluginBuffer, PluginBufferLayout, PluginBufferLayoutAxis (+5 more)

### Community 58 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (55): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+47 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "state.rs"
Cohesion: 0.09
Nodes (32): advance_point_by_text(), multicursor_selection_offsets(), statusline_mode_label(), markdown_inline_image_rows(), markdown_pretty_paint_plan(), multicursor_cursor_points(), multicursor_ranges_for_line(), BlockInsertState (+24 more)

### Community 62 - "DbSessionId"
Cohesion: 0.19
Nodes (17): db_browser_renderer_customizes_rows_and_preserves_actions(), DbQueryBufferMeta, DbSessionId, insert_test_session(), Arc, PathBuf, Self, Send (+9 more)

### Community 63 - "TextBuffer"
Cohesion: 0.04
Nodes (34): advance_point_by_text(), BufferStats, delimiter_partner(), EditRecord, find_matching_close_tag(), is_object_separator(), is_punctuation_char(), is_sentence_closer() (+26 more)

### Community 64 - "PluginCommand"
Cohesion: 0.07
Nodes (24): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+16 more)

### Community 65 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.06
Nodes (68): buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), compile_query_source(), create_parser(), DeferredQuery (+60 more)

### Community 67 - "execute_oil_action"
Cohesion: 0.15
Nodes (18): active_directory_root(), active_shell_buffer_path(), buffer_is_directory(), ensure_directory_buffer(), execute_oil_action(), handle_directory_chord(), handle_directory_keydown_chord(), oil_default_root() (+10 more)

### Community 68 - "shell/acp.rs"
Cohesion: 0.08
Nodes (67): acp_file_uri(), acp_slash_completion_query(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates() (+59 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "WorkspaceConfigurationValue"
Cohesion: 0.13
Nodes (11): language_server_spec_exposes_workspace_configuration_builders(), normalize_unique_entries(), AsRef, From, I, Number, T, WorkspaceConfigurationValue (+3 more)

### Community 72 - "String"
Cohesion: 0.05
Nodes (30): append_query_source(), asset_path_from_parts(), CaptureThemeMapping, command_failure_message(), default_install_root(), default_query_asset_root(), GrammarRecompileFailure, GrammarRecompileReport (+22 more)

### Community 73 - ".send"
Cohesion: 0.13
Nodes (37): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+29 more)

### Community 74 - "Option"
Cohesion: 0.17
Nodes (11): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, looks_like_postgres_connection_string(), looks_like_sql_server_connection_string(), parse_key_value(), parse_postgres_keyword(), parse_url_database(), parse_url_host() (+3 more)

### Community 75 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

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
Cohesion: 0.14
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.05
Nodes (71): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+63 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 83 - "EditorRuntime"
Cohesion: 0.08
Nodes (69): EditorRuntime, Default, run_command(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit() (+61 more)

### Community 85 - "Option"
Cohesion: 0.11
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, path_is_solution() (+10 more)

### Community 86 - "ShellUiState"
Cohesion: 0.04
Nodes (67): active_buffer_revision_key(), active_project_workspace_root(), active_runtime_buffer(), active_shell_workspace_id(), active_window_id(), buffer_is_oil_preview(), BufferViewState, close_acp_inline_picker_for() (+59 more)

### Community 87 - "Self"
Cohesion: 0.04
Nodes (55): GitCommandBinding, GitPrefixBinding, exported_git_feature_spec(), GitFeatureSpec, AbiContextHelpEntry, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGitCommandBinding (+47 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.14
Nodes (23): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+15 more)

### Community 89 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 90 - ".oil_directory_sections"
Cohesion: 0.33
Nodes (4): DirectoryEntry, OilSortMode, Path, SectionTree

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 94 - ".from_grammar"
Cohesion: 0.12
Nodes (41): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+33 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "GhostTextContext"
Cohesion: 0.12
Nodes (13): GhostTextLine, GhostTextLine, exported_ghost_text_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AutocompleteProvider (+5 more)

### Community 97 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (47): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+39 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "LspSessionHandle"
Cohesion: 0.07
Nodes (47): ChildStdin, ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), language_server_session_in_workspace_scope(), launch_summary(), LspClientState, LspSessionHandle (+39 more)

### Community 101 - ".spawn"
Cohesion: 0.09
Nodes (22): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+14 more)

### Community 102 - "wrap_line_segments"
Cohesion: 0.08
Nodes (29): acp_chat_bubble_cols(), acp_rendered_text_segments(), acp_rendered_text_wrap_cols(), display_columns_for_character(), is_wide_display_character(), is_zero_width_display_character(), LineCharMap, LineWrapSegment (+21 more)

### Community 103 - ".from"
Cohesion: 0.05
Nodes (51): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GitStashEntry, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiFiniteF64, AbiGhostTextLine (+43 more)

### Community 104 - "LspLocation"
Cohesion: 0.15
Nodes (8): definition_parser_preserves_uri_backed_locations(), location_from_link(), location_from_lsp(), location_sorting_deduplicates_reference_results(), LspLocation, parse_reference_response(), Location, LocationLink

### Community 105 - ".new"
Cohesion: 0.08
Nodes (38): CodeActionParams, close_buffer_keeps_session_alive_for_next_file(), code_action_params(), code_action_params_use_flattened_lsp_shape(), file_uri_roundtrip_handles_windows_paths(), full_document_range(), full_sync_uses_null_range_change(), incremental_sync_uses_full_document_replacement_range() (+30 more)

### Community 106 - "main"
Cohesion: 0.10
Nodes (20): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), panic_payload_message(), Any, Arc (+12 more)

### Community 107 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 108 - "JobSpec"
Cohesion: 0.20
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (37): ShellTestUserLibrary, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay(), picker_overlay_from_spec() (+29 more)

### Community 110 - "RVec"
Cohesion: 0.14
Nodes (13): AbiAcpClient, AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AcpClient, DebugAdapterSpec, HoverProvider, HoverProviderTopic (+5 more)

### Community 111 - ".default"
Cohesion: 0.09
Nodes (53): Self, load_persisted_state(), load_postgres_schema(), browser_display_url_prefers_requested_navigation(), Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head() (+45 more)

### Community 112 - "String"
Cohesion: 0.28
Nodes (19): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+11 more)

### Community 113 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.11
Nodes (25): plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, hook_command(), leader_binding(), normal_binding() (+17 more)

### Community 115 - "DbService"
Cohesion: 0.17
Nodes (10): DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbBrowserBufferView, DbService, HashSet, Path, section_count_label() (+2 more)

### Community 116 - "String"
Cohesion: 0.06
Nodes (25): active_parameter_label(), completion_level_for_message(), CopilotDeviceCodePrompt, documentation_lines(), execute_command_params(), LspHoverContents, LspLiveSession, LspNotification (+17 more)

### Community 117 - "String"
Cohesion: 0.07
Nodes (57): ColumnData, Compat, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, column_is_numeric() (+49 more)

### Community 118 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "AbiSectionTree"
Cohesion: 0.14
Nodes (12): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiOilSortMode, AbiSectionTree (+4 more)

### Community 121 - "shell/browser.rs"
Cohesion: 0.07
Nodes (66): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_buffer_layout(), browser_display_url(), browser_host_viewport_rect(), browser_surface_buffer_at_point(), browser_sync_plan() (+58 more)

### Community 122 - "LspCodeAction"
Cohesion: 0.14
Nodes (5): LspCodeAction, LspDocumentTextEdits, LspTextEdit, Error, windows_should_retry_spawn_error()

### Community 123 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (47): browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines(), db_results_table_row_spans() (+39 more)

### Community 124 - "Option"
Cohesion: 0.07
Nodes (34): apply_git_fringe_hunk(), find_paren_number_range(), git_repository_present(), git_status_command_name(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState (+26 more)

### Community 125 - "Vec"
Cohesion: 0.10
Nodes (11): CommandPaletteState, EventLog, format_micros_as_millis(), LspState, AutocompleteProvider, ContextHelpSpec, GitStatusSnapshot, HoverProvider (+3 more)

### Community 127 - "Option"
Cohesion: 0.14
Nodes (6): CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "TerminalRenderSnapshot"
Cohesion: 0.12
Nodes (7): terminal_cursor_shape_for_input_mode(), Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalCursorShape, TerminalCursorSnapshot, TerminalRenderLine, TerminalRenderSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "UserLibraryModule"
Cohesion: 0.06
Nodes (27): exported_icon_symbols(), exported_oil_defaults(), exported_oil_feature_spec(), exported_oil_keybindings(), exported_pdf_open_mode(), IconFontSymbol, OilDefaults, OilFeatureSpec (+19 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 134 - ".get"
Cohesion: 0.13
Nodes (20): DbAutocompleteCandidate, DbColumn, DbIndex, DbSchemaCache, DbSession, DbTable, load_sql_server_schema(), load_sqlite_columns() (+12 more)

### Community 135 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 138 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 139 - "DbEngine"
Cohesion: 0.38
Nodes (5): DbEngine, DbHistoryEntry, DbSnippet, PersistedDbState, RememberedConnection

### Community 140 - ".new"
Cohesion: 0.08
Nodes (33): AsyncRead, buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), install_acp_test_buffer(), pending_slash_completion_trigger_rejects_multiline_input(), permission_prompt_lines() (+25 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "AbiLanguageConfiguration"
Cohesion: 0.17
Nodes (10): exported_syntax_languages(), AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping (+2 more)

### Community 143 - "PaneConfig"
Cohesion: 0.09
Nodes (13): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig (+5 more)

### Community 144 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 145 - "TextEdit"
Cohesion: 0.67
Nodes (4): TextEdit, apply_text_edits_to_span(), text_edit_to_input_edit(), InputEdit

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 148 - "headerline_lines"
Cohesion: 0.29
Nodes (7): build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 149 - ".null"
Cohesion: 0.13
Nodes (24): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), explicit_windows_env_value(), Command, spawn_lsp_command(), temp_dir() (+16 more)

### Community 150 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 151 - "StatuslineContext"
Cohesion: 0.21
Nodes (9): exported_statusline_render(), statusline_context_from_abi(), user_modeline_context(), AbiLspDiagnosticsInfo, AbiStatuslineContext, LspDiagnosticsInfo, LspDiagnosticsInfo, LspDiagnosticsInfo (+1 more)

### Community 152 - "oil.rs"
Cohesion: 0.06
Nodes (45): seti_directory_icon(), exported_oil_chord_action(), exported_oil_keydown_action(), exported_oil_strip_entry_icon_prefix(), OilKeyAction, chord_action(), default_oil_keybindings_map_to_actions(), defaults() (+37 more)

### Community 153 - "user/db.rs"
Cohesion: 0.15
Nodes (20): browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings(), engine_icon(), feature_spec(), header_icon() (+12 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "terminal_key_for_event"
Cohesion: 0.67
Nodes (3): Keycode, Mod, terminal_key_for_event()

### Community 156 - "TextRange"
Cohesion: 0.09
Nodes (12): TextRange, diagnostic_matches_request_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), LspCompletionItem, LspCompletionKind, LspInlineCompletionItem, parse_completion_kind() (+4 more)

### Community 157 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 161 - "AcpManager"
Cohesion: 0.09
Nodes (24): AcpManager, AcpPendingPermissionUi, AcpUiAction, config_option_is_mode(), config_option_is_model(), config_option_matches(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work() (+16 more)

### Community 162 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 163 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 166 - "String"
Cohesion: 0.29
Nodes (6): call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 170 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 171 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 172 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 173 - "JobResult"
Cohesion: 0.18
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 177 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "client.rs"
Cohesion: 0.05
Nodes (95): BufRead, char_to_byte_offset(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_documentation(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+87 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

### Community 181 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

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

### Community 187 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 188 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 189 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 190 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 191 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 194 - ".from_text"
Cohesion: 0.05
Nodes (70): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings(), is_inline_whitespace() (+62 more)

### Community 195 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 196 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 197 - "String"
Cohesion: 0.09
Nodes (54): AcpClientConfig, acp_complete_slash(), acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_image_mention_token() (+46 more)

### Community 198 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

### Community 201 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 203 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 204 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 205 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 209 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 210 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 211 - "git_remote_worktree_branch_list"
Cohesion: 0.24
Nodes (10): begin_oil_worktree_request(), git_branch_list(), git_remote_worktree_branch_list(), git_worktree_create_command(), oil_git_worktree_command(), open_git_worktree_branch_picker(), open_git_worktree_dashboard_create(), remote_and_branch_from_ref() (+2 more)

### Community 215 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 216 - "Language"
Cohesion: 0.22
Nodes (8): Database, External commands, Issues, Language, Language servers, Markdown presentation, Volt, Workspace

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 223 - "AbiTheme"
Cohesion: 0.23
Nodes (8): exported_themes(), AbiColor, AbiTheme, AbiThemeToken, Color, Color, Theme, Theme

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 233 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 234 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 242 - "shell/mod.rs"
Cohesion: 0.02
Nodes (299): ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat() (+291 more)

### Community 248 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 253 - "centered_rect"
Cohesion: 0.67
Nodes (3): centered_rect(), picker_card_rect(), PickerLayout

## Knowledge Gaps
- **143 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+138 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `UserLibrary`, `shell/tests.rs`, `Result`, `PickerOverlay`, `AcpEvent`, `Path`, `Result`, `resolve_permission`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `command_stream.rs`, `String`, `AcpManager`, `.new`, `shell/pdf.rs`, `ServiceRegistry`, `String`, `Option`, `String`, `shell_ui_mut`, `Instant`, `.buffer_mut`, `db_service_mut`, `shell/git.rs`, `active_runtime_popup`, `execute_oil_action`, `shell/acp.rs`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `git_remote_worktree_branch_list`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `main`, `shell/picker.rs`, `String`, `shell/mod.rs`, `shell/browser.rs`, `Option`?**
  _High betweenness centrality (0.126) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `UserLibrary` to `shell/tests.rs`, `Result`, `user/lib.rs`, `DynamicUserLibrary`, `HoverOverlay`, `String`, `Option`, `String`, `shell_ui_mut`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `render_buffer_with_view_state`, `state.rs`, `directory.rs`, `volt/src/main.rs`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `main`, `shell/picker.rs`, `ShellConfig`, `shell/mod.rs`, `shell/browser.rs`, `Option`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `UserLibraryModule`, `user/lib.rs`, `common.rs`, `Self`, `calculator.rs`, `oil.rs`, `user/db.rs`, `lsp.rs`, `latex.rs`, `syntax_language`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `lua.rs`, `user/terminal.rs`, `kotlin.rs`, `nix.rs`, `r.rs`, `bash.rs`, `clojure.rs`, `user/browser.rs`, `elixir.rs`, `graphql.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `hcl.rs`, `PluginBuffer`, `picker_items`, `java.rs`, `perl.rs`, `php.rs`, `ruby.rs`, `proto.rs`, `PluginCommand`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `capture_mappings`, `xml.rs`, `markdown.rs`, `scala.rs`, `rainbow_parens.rs`, `debug_adapters`, `syntax_languages`, `PickerItemSpec`, `package`, `show_paren.rs`, `editor-plugin-host/src/lib.rs`, `user/workspace_dock.rs`, `main`, `package`, `package`, `shell/mod.rs`, `PluginKeyBinding`, `syntax_language`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _143 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `UserLibrary` be split into smaller, more focused modules?**
  _Cohesion score 0.05561105561105561 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.0898854591341487 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.032190195665895226 - nodes in this community are weakly interconnected._