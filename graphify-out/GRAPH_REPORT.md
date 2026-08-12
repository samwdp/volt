# Graph Report - volt  (2026-08-12)

## Corpus Check
- 231 files · ~579,665 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9237 nodes · 37699 edges · 318 communities (287 shown, 31 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3141 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `0d36bbb1`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Instant
- Path
- shell/tests.rs
- .new
- Result
- user/lib.rs
- editor-syntax/src/lib.rs
- ShellUiState
- TextPoint
- render.rs
- Result
- PluginPackage
- Self
- TextBuffer
- shell/browser.rs
- .spawn
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- editor-db/src/lib.rs
- state.rs
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- FontSet
- EditorRuntime
- sync_quickfix_popup_buffer
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- Self
- Option
- DbService
- LanguageServerSpec
- LspCodeAction
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- .char_to_point
- .new
- workspace.rs
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- Result
- PaneConfig
- Section
- render_buffer_with_view_state
- .len
- String
- syntax_languages
- .from
- .handle_event
- String
- HighlightWindow
- LspNotification
- PluginCommand
- state_with_user_library
- SyntaxRegistry
- .new
- String
- DebugConfiguration
- capture_mappings
- String
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
- volt/src/main.rs
- UserLibraryModule
- editor-lsp/src/lib.rs
- Option
- AbiContextHelpSpec
- .new
- draw_diagnostic_underlines_for_segment
- SectionLineMeta
- main
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- Option
- LoadedLanguage
- GitEditorState
- WorkspaceConfigurationValue
- .new
- shell/acp.rs
- Option
- common.rs
- WorkspaceDockBranchCache
- shell/git.rs
- Vec
- WorkspaceConfiguration
- String
- shell/picker.rs
- active_runtime_popup
- user/git.rs
- treesittercontext_ghosttext.rs
- editor-picker/src/lib.rs
- PluginKeyBinding
- wrap_line_segments
- String
- RVec
- BufferId
- process_supervisor.rs
- Result
- DbEngine
- execute_oil_action
- Vec
- GitSummaryState
- statusline.rs
- AbiGitStatusSnapshot
- .new
- run_job
- Option
- user/config.rs
- editor-icons/src/lib.rs
- key_sequence.rs
- PathBuf
- sqlite_query_execution_and_schema_cache_work
- editor-buffer/src/lib.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- .default
- treesittercontext_shared.rs
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- browser_host.rs
- JobSpec
- ShellConfig
- standalone_user_manifest.rs
- AbiSectionTree
- Vec
- client.rs
- TerminalCursorSnapshot
- ancestor_contexts_for_cursor
- oil.rs
- DbBrowserContext
- lsp.rs
- .oil_directory_sections
- LspLogEntry
- AcpPickerItemSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- bash.rs
- clojure.rs
- elixir.rs
- graphql.rs
- ServiceRegistry
- aligned_indent_column
- String
- user/terminal.rs
- build_output.rs
- proto.rs
- hcl.rs
- java.rs
- kotlin.rs
- JobResult
- .new
- user/browser.rs
- latex.rs
- .oil_keybindings
- `user`
- .next_token
- nix.rs
- perl.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- syntax_language
- php.rs
- r.rs
- ruby.rs
- syntax_language
- scala.rs
- Database Explorer PRD
- solidity.rs
- swift.rs
- lang/vim.rs
- xml.rs
- db_service_mut
- LiveTerminalSession
- Option
- markdown.rs
- lua.rs
- cargo
- user/workspace_dock.rs
- .oil_directory_sections
- apply_lsp_notifications
- .acp_client_by_id
- .git_command_for_chord
- 0004-markdown-pretty-pipeline.md
- AbiLanguageConfiguration
- .autocomplete_providers
- .browser_feature_spec
- package
- .context_help_specs
- syntax_language
- .db_feature_spec
- Language
- .debug_adapters
- Domain Docs
- Issue tracker: GitHub
- load
- .git_feature_spec
- package
- package
- .hover_providers
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pdf_open_mode
- .picker_truncate_strategy
- .statusline_render
- .terminal_config
- .terminal_feature_spec
- .workspace_roots
- directory_entry_display_label_from_parts
- debug_adapters
- clipboard.rs
- main
- TextRange
- directory_edit_actions
- DirectoryViewState
- shell/mod.rs
- normalize_inline_text
- .path
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- Self
- browser_buffer_layout
- Agent skills
- index_syntax_lines
- panic_payload_message
- package
- git_remote_worktree_branch_list
- AbiPickerTruncateStrategy
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 764 edges
2. `ShellBuffer` - 369 edges
3. `shell_ui_mut()` - 341 edges
4. `register_shell_hooks()` - 261 edges
5. `shell_ui()` - 229 edges
6. `ShellError` - 183 edges
7. `shell_buffer()` - 179 edges
8. `shell_buffer_mut()` - 176 edges
9. `ShellUiState` - 173 edges
10. `TextBuffer` - 167 edges

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

## Communities (318 total, 31 thin omitted)

### Community 0 - "Instant"
Cohesion: 0.05
Nodes (28): ActiveTypingFrameProfile, average_duration(), DirectoryPrefixState, ensure_log_directory(), format_duration_ms(), FpsOverlayState, frame_pacing_deferred_for_typing(), frame_pacing_remaining() (+20 more)

### Community 1 - "Path"
Cohesion: 0.09
Nodes (24): inline_completion_params(), is_copilot_server(), LspClientError, LspClientManager, LspSessionHandle, parse_text_edit_response(), path_to_uri(), request_timeout_for_method() (+16 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (76): load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), buffer_save_still_writes_when_format_on_save_fails(), codicon_glyphs_fit_inside_one_editor_cell() (+68 more)

### Community 3 - ".new"
Cohesion: 0.12
Nodes (72): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+64 more)

### Community 4 - "Result"
Cohesion: 0.05
Nodes (51): Display, Error, From, ShellError, browser_sync_plan(), Instant, clear_key_sequence(), active_runtime_surface() (+43 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (111): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+103 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.11
Nodes (67): vim_search_entries_trim_whitespace_from_labels(), additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+59 more)

### Community 7 - "ShellUiState"
Cohesion: 0.03
Nodes (126): active_lsp_buffer_context(), active_lsp_code_action_range(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_window_id(), apply_copilot_auth_notification(), apply_lsp_text_edits(), apply_pending_lsp_state() (+118 more)

### Community 8 - "TextPoint"
Cohesion: 0.09
Nodes (10): advance_point_by_text(), Self, Selection, TextPoint, TextSnapshot, char_immediately_before(), chars_immediately_before(), normalize_completion_replacement() (+2 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (101): advance_point_by_text(), multicursor_selection_offsets(), acp_pane_body_visible_rows(), acp_slice_chars(), AcpBufferLayout, AcpPaneLayout, adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face() (+93 more)

### Community 10 - "Result"
Cohesion: 0.11
Nodes (33): AcpClientConfig, acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_closed() (+25 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (42): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+34 more)

### Community 12 - "Self"
Cohesion: 0.04
Nodes (34): AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, picker_provider_spec_accepts_extra_keybinds(), PickerAcpClientContext, PickerActionSpec, PickerBufferContext (+26 more)

### Community 13 - "TextBuffer"
Cohesion: 0.09
Nodes (13): BufferStats, large_buffers_expose_line_windows_without_full_materialization(), Default, Into, PathBuf, String, Vec, TextBuffer (+5 more)

### Community 14 - "shell/browser.rs"
Cohesion: 0.11
Nodes (40): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len() (+32 more)

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
Nodes (16): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, GitStatusPrefix, IconFontSymbol, KeymapConfig, LigatureConfig (+8 more)

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

### Community 24 - "editor-db/src/lib.rs"
Cohesion: 0.08
Nodes (39): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_db_browser_items(), default_db_browser_line(), escape_bracket() (+31 more)

### Community 25 - "state.rs"
Cohesion: 0.12
Nodes (24): BlockInsertState, DirectoryYankEntry, FormatterRegistry, FormatterSpec, LastFind, LastSearch, BTreeMap, BufferId (+16 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.10
Nodes (50): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+42 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.08
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.08
Nodes (44): centered_rect(), default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names() (+36 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (30): AutocompleteProviderKind, RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay, HoverProviderContent (+22 more)

### Community 30 - "Theme"
Cohesion: 0.08
Nodes (25): text_style_from_theme_style(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display (+17 more)

### Community 31 - "FontSet"
Cohesion: 0.08
Nodes (50): Canvas, DrawCommand, RenderColor, Arc, TextStyle, FontSet, alpha_bitmap_surface(), cached_primary_text_runs() (+42 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.07
Nodes (133): EditorRuntime, Default, run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker() (+125 more)

### Community 33 - "sync_quickfix_popup_buffer"
Cohesion: 0.13
Nodes (16): buffer_is_quickfix(), quickfix_clear_marks(), quickfix_entry_for_cursor(), quickfix_mark_all(), quickfix_open_current_list(), quickfix_open_from_one_shot(), quickfix_open_picker_matches(), quickfix_select_next() (+8 more)

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
Cohesion: 0.10
Nodes (22): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+14 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (58): acp_tool_call_from_partial_update(), active_buffer_revision_key(), active_shell_workspace_id(), apply_scroll_command(), block_comment_toggle_removal_lens(), buffer_context_overlay_snapshot(), BufferContextOverlayCacheKey, BufferContextOverlaySnapshot (+50 more)

### Community 42 - "DbService"
Cohesion: 0.16
Nodes (12): DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbQueryBufferMeta, DbService, DbSession, DbSessionId (+4 more)

### Community 43 - "LanguageServerSpec"
Cohesion: 0.12
Nodes (11): LanguageServerRootStrategy, LanguageServerSpec, normalize_optional_string(), normalize_unique_entries(), I, Into, IntoIterator, Item (+3 more)

### Community 44 - "LspCodeAction"
Cohesion: 0.11
Nodes (11): code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), LspCodeAction, LspDocumentTextEdits, LspTextEdit, parse_code_action_document_change(), parse_code_action_response(), parse_code_action_workspace_edit() (+3 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.05
Nodes (48): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+40 more)

### Community 47 - ".char_to_point"
Cohesion: 0.11
Nodes (4): is_inline_whitespace(), is_sentence_closer(), Fn, sentence_ranges_cover_inner_and_around_text_objects()

### Community 48 - ".new"
Cohesion: 0.03
Nodes (80): BufferKind, browser_state_for_kind(), buffer_uses_browser_host_surface(), ActiveLspBufferContext, default_vim_target(), WorkspaceId, absolute_path_hint(), active_buffer_event_context() (+72 more)

### Community 49 - "workspace.rs"
Cohesion: 0.13
Nodes (25): existing_workspace_for_project(), file_picker_preview(), message_item(), package(), package_exports_cycle_project_workspace_commands(), package_exports_format_command(), package_exports_mark_list_commands(), package_exports_marked_workspace_slot_jump_commands() (+17 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.15
Nodes (28): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+20 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (31): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc, AutocompleteProvider (+23 more)

### Community 52 - "Result"
Cohesion: 0.05
Nodes (95): shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_visual_yank_copies_selected_text(), acp_nonleading_double_slash_does_not_open_slash_picker() (+87 more)

### Community 53 - "PaneConfig"
Cohesion: 0.07
Nodes (17): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, config(), AbiKeymapConfig, AbiLigatureConfig (+9 more)

### Community 54 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.11
Nodes (91): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+83 more)

### Community 56 - ".len"
Cohesion: 0.05
Nodes (50): advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update(), ascii_control_caret_notation(), char_at_index(), detect_markdown_table(), display_columns_for_character(), exact_match_positions_in_chars() (+42 more)

### Community 57 - "String"
Cohesion: 0.27
Nodes (5): DbBrowserBufferView, DisabledSecretStore, String, summarize_sql(), DbBrowserItemRenderer

### Community 58 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 59 - ".from"
Cohesion: 0.05
Nodes (49): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GhostTextLine, GhostTextLine, main(), exported_ghost_text_lines(), exported_themes(), GhostTextLine, abi_language_configuration_round_trips_path_matchers() (+41 more)

### Community 60 - ".handle_event"
Cohesion: 0.11
Nodes (18): acp_open_permission_request(), AcpPendingPermissionUi, config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), open_permission_picker(), open_permission_request_reorders_queue_for_requested_picker() (+10 more)

### Community 61 - "String"
Cohesion: 0.08
Nodes (95): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range(), acp_input_field_o_and_o_open_new_lines() (+87 more)

### Community 62 - "HighlightWindow"
Cohesion: 0.10
Nodes (31): apply_text_edits_to_span(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), highlight_loaded_language(), highlight_loaded_language_with_tree() (+23 more)

### Community 63 - "LspNotification"
Cohesion: 0.07
Nodes (27): ChildStdin, diagnostic_matches_request_range(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel, LspNotificationLog (+19 more)

### Community 64 - "PluginCommand"
Cohesion: 0.09
Nodes (25): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+17 more)

### Community 65 - "state_with_user_library"
Cohesion: 0.05
Nodes (90): ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), active_input_prompt_text(), browser_insert_mode_ctrl_enter_binding_submits_current_url() (+82 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.07
Nodes (41): append_query_source(), asset_path_from_parts(), create_parser(), default_query_asset_root(), desired_indent_for_loaded_language(), ensure_cloned_grammar_dir_exists(), highlight_inline_language_per_line(), install_plan_requests_generate_when_parser_is_missing() (+33 more)

### Community 67 - ".new"
Cohesion: 0.10
Nodes (23): big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings(), move_word_backward_and_end_cover_word_navigation(), must(), reload_from_path_requires_a_backing_file(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean() (+15 more)

### Community 68 - "String"
Cohesion: 0.15
Nodes (45): active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+37 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "String"
Cohesion: 0.07
Nodes (25): CaptureThemeMapping, cmake_configuration(), command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport, GrammarSource, installable_rust_configuration(), InstallCommandSpec (+17 more)

### Community 72 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 73 - ".send"
Cohesion: 0.16
Nodes (35): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession, AcpTerminal (+27 more)

### Community 74 - ".new"
Cohesion: 0.06
Nodes (40): AsyncRead, acp_resolve_permission_option(), AcpClient, buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), humanize_debug_label() (+32 more)

### Community 75 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 76 - "directory.rs"
Cohesion: 0.21
Nodes (24): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), directory_cd_from_cursor() (+16 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), configure_background_command(), default_process_supervisor_executable(), environment_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.07
Nodes (74): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+66 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (39): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+31 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.10
Nodes (26): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+18 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 83 - "UserLibraryModule"
Cohesion: 0.10
Nodes (19): exported_icon_symbols(), IconFontSymbol, AbiIconFontSymbol, AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeybindings, AbiOilSortMode, IconFontSymbol (+11 more)

### Community 84 - "editor-lsp/src/lib.rs"
Cohesion: 0.21
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+20 more)

### Community 85 - "Option"
Cohesion: 0.12
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, path_is_solution() (+10 more)

### Community 86 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 87 - ".new"
Cohesion: 0.05
Nodes (78): buffer_footer_layout(), acp_multiline_text_lines_strip_carriage_returns(), acp_section_layout_orders_output_input_footer_and_statusline(), acp_wrapped_text_uses_full_width_on_continuation_rows(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow(), block_cursor_text_overlay_positions_multibyte_cursor_text() (+70 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.13
Nodes (23): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+15 more)

### Community 89 - "SectionLineMeta"
Cohesion: 0.14
Nodes (26): diff_git_commit_at_point(), diff_git_dwim(), diff_git_stash_at_point(), git_action_detail(), git_args_with_no_pager(), git_commit_at_point(), git_status_diff_commit_command(), git_status_diff_dwim_command() (+18 more)

### Community 90 - "main"
Cohesion: 0.12
Nodes (16): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), Arc, DebugAdapterSpec, Error (+8 more)

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
Cohesion: 0.14
Nodes (36): default_install_root(), csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting() (+28 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "Option"
Cohesion: 0.07
Nodes (62): parse_log_oneline(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), create_git_worktree_from_query(), find_paren_number_range(), git_branch_merge(), git_branch_push_remote() (+54 more)

### Community 97 - "LoadedLanguage"
Cohesion: 0.15
Nodes (14): compile_query_source(), DeferredQuery, html_language(), LoadedLanguage, query_capture_property_value(), query_capture_property_value_returns_set_property(), query_compiler_accepts_vim_case_insensitive_regex_prefix(), BTreeMap (+6 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.11
Nodes (13): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, Number (+5 more)

### Community 100 - ".new"
Cohesion: 0.16
Nodes (22): browser_host_event_for_ipc(), browser_navigation_retry_required(), BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, DesktopBrowserHostService, optional_non_empty_text() (+14 more)

### Community 101 - "shell/acp.rs"
Cohesion: 0.08
Nodes (35): acp_complete_slash(), acp_permission_approve(), acp_permission_deny(), acp_pick_mode(), acp_picker_entries(), acp_picker_entry(), acp_slash_completion_query(), AcpUiAction (+27 more)

### Community 102 - "Option"
Cohesion: 0.07
Nodes (52): BufRead, completion_documentation(), completion_level_for_message(), configuration_item_section(), csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item() (+44 more)

### Community 103 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 104 - "WorkspaceDockBranchCache"
Cohesion: 0.14
Nodes (19): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+11 more)

### Community 105 - "shell/git.rs"
Cohesion: 0.07
Nodes (52): apply_git_fringe_hunk(), apply_git_view(), git_log_args(), git_status_checkout_file_command(), git_status_cherry_open_command(), git_status_command_name(), git_status_diff_paths_command(), git_status_diff_range_command() (+44 more)

### Community 106 - "Vec"
Cohesion: 0.07
Nodes (16): CommandPaletteState, CompilationState, EventLog, format_micros_as_millis(), LspState, AcpClient, AutocompleteProvider, ContextHelpSpec (+8 more)

### Community 108 - "String"
Cohesion: 0.04
Nodes (164): Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), active_shell_buffer_has_input(), active_shell_buffer_id(), active_shell_buffer_is_terminal(), active_shell_buffer_mut() (+156 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.15
Nodes (34): buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars(), picker_overlay(), picker_overlay_from_spec() (+26 more)

### Community 110 - "active_runtime_popup"
Cohesion: 0.11
Nodes (54): active_runtime_popup(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean() (+46 more)

### Community 111 - "user/git.rs"
Cohesion: 0.11
Nodes (48): commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title(), help_entry() (+40 more)

### Community 112 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 113 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (46): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+38 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (22): plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding(), normal_binding_commands() (+14 more)

### Community 115 - "wrap_line_segments"
Cohesion: 0.13
Nodes (18): acp_rendered_text_segments(), acp_rendered_text_wrap_cols(), LineCharMap, LineWrapSegment, resolved_tab_width(), wrap_line_segments(), wrap_line_segments_for_line(), acp_prefix_columns() (+10 more)

### Community 116 - "String"
Cohesion: 0.11
Nodes (13): append_lines(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self, String (+5 more)

### Community 117 - "RVec"
Cohesion: 0.10
Nodes (18): exported_debug_adapters(), AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiTerminalConfig, DebugAdapterSpec (+10 more)

### Community 118 - "BufferId"
Cohesion: 0.14
Nodes (22): ActiveBufferEventContext, apply_git_status_snapshot(), finish_oil_worktree_branch_selection(), git_line_is_untracked(), git_status_action_targets(), git_status_delete_target_for_line(), git_status_delete_targets(), git_status_selected_lines() (+14 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Result"
Cohesion: 0.14
Nodes (17): Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, execute_postgres(), execute_sql_server(), execute_sqlite(), initialize_native_keyring() (+9 more)

### Community 121 - "DbEngine"
Cohesion: 0.21
Nodes (9): db_browser_action_from_spec(), DbEngine, DbHistoryEntry, DbIndex, DbSnippet, PersistedDbState, qualified_name_from_spec(), QualifiedName (+1 more)

### Community 122 - "execute_oil_action"
Cohesion: 0.15
Nodes (17): active_directory_root(), active_shell_buffer_path(), buffer_is_directory(), ensure_directory_buffer(), execute_oil_action(), handle_directory_chord(), handle_directory_keydown_chord(), oil_default_root() (+9 more)

### Community 123 - "Vec"
Cohesion: 0.22
Nodes (26): format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token(), git_status_entry_token_from_icon(), git_status_head_spans(), git_status_header_item_spans() (+18 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.13
Nodes (12): git_summary_changed_tracks_head_updates(), GitFringeState, GitSummarySnapshot, GitSummaryState, refresh_git_fringe(), refresh_pending_git_summary(), Arc, AtomicBool (+4 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 127 - ".new"
Cohesion: 0.08
Nodes (38): close_buffer_keeps_session_alive_for_next_file(), default_workspace_lists_only_sessions_serving_open_buffers(), full_document_range(), live_session_picker_label_includes_server_and_root(), live_sessions_for_workspace_includes_root_scoped_and_buffer_served(), log_message_error_from_ols_does_not_become_ui_notification(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+30 more)

### Community 128 - "run_job"
Cohesion: 0.14
Nodes (17): CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display, E (+9 more)

### Community 129 - "Option"
Cohesion: 0.13
Nodes (6): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "editor-icons/src/lib.rs"
Cohesion: 0.11
Nodes (16): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+8 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "PathBuf"
Cohesion: 0.07
Nodes (23): CopilotDeviceCodePrompt, diagnostics_parser_maps_lsp_fields(), file_uri_to_path(), language_server_session_in_workspace_scope(), location_from_link(), location_from_lsp(), LspClientState, LspLiveSession (+15 more)

### Community 134 - "sqlite_query_execution_and_schema_cache_work"
Cohesion: 0.13
Nodes (15): default_volt_state_dir(), InMemorySecretStore, load_persisted_state(), remembered_connections_store_metadata_separately_from_secret(), Arc, HashMap, Mutex, Path (+7 more)

### Community 135 - "editor-buffer/src/lib.rs"
Cohesion: 0.13
Nodes (17): around_word_ranges_at_line_end_exclude_newline(), detect_preferred_line_ending(), EditRecord, is_object_separator(), is_punctuation_char(), is_word_char(), line_ranges_and_char_searches_resolve_expected_points(), LineEnding (+9 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - ".default"
Cohesion: 0.09
Nodes (20): CodeActionParams, Self, code_action_params(), code_action_params_use_flattened_lsp_shape(), definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), location_sorting_deduplicates_reference_results(), lsp_formatting_options() (+12 more)

### Community 138 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 140 - "AcpEvent"
Cohesion: 0.10
Nodes (24): AvailableCommand, AcpEvent, build_acp_input_hint(), coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks(), command_input_hint(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work() (+16 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "browser_host.rs"
Cohesion: 0.11
Nodes (18): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+10 more)

### Community 144 - "JobSpec"
Cohesion: 0.27
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "AbiSectionTree"
Cohesion: 0.19
Nodes (9): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 148 - "Vec"
Cohesion: 0.17
Nodes (19): ColumnData, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns(), load_sqlite_schema() (+11 more)

### Community 149 - "client.rs"
Cohesion: 0.04
Nodes (85): ClientCapabilities, active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document() (+77 more)

### Community 150 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 151 - "ancestor_contexts_for_cursor"
Cohesion: 0.17
Nodes (15): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+7 more)

### Community 152 - "oil.rs"
Cohesion: 0.17
Nodes (18): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), feature_spec(), help_lines(), help_lines_reflect_current_keybindings(), keybindings(), keydown_action() (+10 more)

### Community 153 - "DbBrowserContext"
Cohesion: 0.13
Nodes (21): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), default_action(), feature_spec(), hook_command(), package() (+13 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 156 - "LspLogEntry"
Cohesion: 0.17
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 157 - "AcpPickerItemSpec"
Cohesion: 0.11
Nodes (19): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+11 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 161 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 162 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 163 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "aligned_indent_column"
Cohesion: 0.12
Nodes (24): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after(), general_predicates_match(), indent_begin_applies(), line_intersects_node() (+16 more)

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "build_output.rs"
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 170 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 171 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 172 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 175 - "user/browser.rs"
Cohesion: 0.21
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 180 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 181 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

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

### Community 186 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 187 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 188 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 189 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 190 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 191 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 194 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 195 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 196 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 197 - "db_service_mut"
Cohesion: 0.24
Nodes (16): activate_db_browser_line(), apply_db_browser_view(), buffer_is_db_browser(), db_service(), db_service_mut(), open_db_connections_buffer(), open_db_history_buffer(), open_db_query_for_table_preview() (+8 more)

### Community 198 - "LiveTerminalSession"
Cohesion: 0.12
Nodes (12): AlacrittyEvent, Self, LiveTerminalSession, QueuedEventListener, Arc, Drop, Receiver, Sender (+4 more)

### Community 199 - "Option"
Cohesion: 0.20
Nodes (8): delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), find_matching_close_tag(), is_tag_name_char(), parse_tag_token(), Option, TagToken

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 202 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 203 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 204 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 205 - "apply_lsp_notifications"
Cohesion: 0.15
Nodes (14): active_project_workspace_root(), apply_lsp_notifications(), canonicalize_project_root_path(), jump_to_marked_workspace_slot(), lsp_notification_action(), lsp_notification_body_lines(), mark_active_project_workspace(), marked_workspace_display_name() (+6 more)

### Community 209 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 212 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 214 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 216 - "Language"
Cohesion: 0.25
Nodes (7): External commands, Issues, Language, Language servers, Markdown presentation, Volt, Workspace

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "load"
Cohesion: 0.28
Nodes (5): load(), config(), KeymapConfig, config(), LigatureConfig

### Community 222 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 235 - "directory_entry_display_label_from_parts"
Cohesion: 0.17
Nodes (16): directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), help_entry(), oil_entry_icon(), oil_file_icon(), ContextHelpEntry, DirectoryEntry (+8 more)

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 237 - "clipboard.rs"
Cohesion: 0.19
Nodes (13): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+5 more)

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 240 - "directory_edit_actions"
Cohesion: 0.23
Nodes (12): diff_directory_lines(), directory_edit_actions(), oil_directory_line_spans(), oil_directory_theme_token(), oil_entry_theme_token(), oil_file_theme_token(), repeated_icons_do_not_turn_existing_entries_into_renames(), OilDefaults (+4 more)

### Community 241 - "DirectoryViewState"
Cohesion: 0.21
Nodes (11): directory_entry_label(), directory_visible_entries(), DirectoryEditAction, DirectoryViewState, parse_worktree_request(), PendingWorktreeRequest, DirectoryEntry, Option (+3 more)

### Community 242 - "shell/mod.rs"
Cohesion: 0.01
Nodes (277): DbAutocompleteCandidate, acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_icon_segment(), acp_multiline_text_lines(), acp_padding_prefix(), acp_pane_content_rows() (+269 more)

### Community 243 - "normalize_inline_text"
Cohesion: 0.20
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 244 - ".path"
Cohesion: 0.23
Nodes (11): db_query_buffer_receives_sql_highlighting_without_blocking(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test() (+3 more)

### Community 247 - "Self"
Cohesion: 0.05
Nodes (55): GitCommandBinding, GitPrefixBinding, exported_pdf_open_mode(), PdfOpenMode, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiDirectoryEntry, AbiDirectoryEntryKind (+47 more)

### Community 248 - "browser_buffer_layout"
Cohesion: 0.31
Nodes (9): browser_buffer_layout(), browser_host_viewport_rect(), browser_viewport_rect(), BrowserBufferLayout, Rect, input_panel_chrome_height(), plugin_section_panel_chrome_height(), text_panel_chrome_height() (+1 more)

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 250 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 251 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 253 - "git_remote_worktree_branch_list"
Cohesion: 0.28
Nodes (9): begin_oil_worktree_request(), git_remote_worktree_branch_list(), git_worktree_create_command(), oil_git_worktree_command(), open_git_worktree_branch_picker(), open_git_worktree_dashboard_create(), remote_and_branch_from_ref(), Into (+1 more)

### Community 254 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

## Knowledge Gaps
- **141 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+136 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **31 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `Instant`, `shell/tests.rs`, `Result`, `ShellUiState`, `Result`, `AcpEvent`, `shell/browser.rs`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `command_stream.rs`, `sync_quickfix_popup_buffer`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `.new`, `Result`, `.handle_event`, `String`, `state_with_user_library`, `String`, `db_service_mut`, `.new`, `directory.rs`, `apply_lsp_notifications`, `workspace_search.rs`, `shell/terminal.rs`, `SectionLineMeta`, `main`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `Option`, `GitEditorState`, `shell/acp.rs`, `shell/git.rs`, `String`, `shell/picker.rs`, `active_runtime_popup`, `shell/mod.rs`, `.path`, `BufferId`, `execute_oil_action`, `GitSummaryState`, `git_remote_worktree_branch_list`?**
  _High betweenness centrality (0.117) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `Instant`, `Option`, `Result`, `ShellUiState`, `TextPoint`, `render.rs`, `TextBuffer`, `shell/browser.rs`, `state.rs`, `shell/pdf.rs`, `.new`, `render_buffer_with_view_state`, `.len`, `String`, `db_service_mut`, `directory.rs`, `shell/terminal.rs`, `.new`, `draw_diagnostic_underlines_for_segment`, `Option`, `shell/acp.rs`, `shell/git.rs`, `String`, `shell/picker.rs`, `DirectoryViewState`, `shell/mod.rs`, `wrap_line_segments`, `BufferId`, `browser_buffer_layout`, `execute_oil_action`, `Vec`, `GitSummaryState`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `.new` to `shell/tests.rs`, `Result`, `user/lib.rs`, `ShellUiState`, `render.rs`, `DynamicUserLibrary`, `shell/browser.rs`, `ShellConfig`, `DynamicUserLibrary`, `HoverOverlay`, `Option`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `render_buffer_with_view_state`, `String`, `directory.rs`, `volt/src/main.rs`, `main`, `editor-plugin-host/src/lib.rs`, `shell/git.rs`, `directory_edit_actions`, `shell/mod.rs`?**
  _High betweenness centrality (0.048) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _141 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Instant` be split into smaller, more focused modules?**
  _Cohesion score 0.05185185185185185 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.08532364139840776 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.026470588235294117 - nodes in this community are weakly interconnected._