# Graph Report - volt  (2026-08-24)

## Corpus Check
- 247 files · ~648,843 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10235 nodes · 42155 edges · 316 communities (289 shown, 27 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3393 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4fd7b3ad`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- .new
- Path
- temp_dir
- .new
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- editor-lsp/src/client.rs
- shell_user_library
- Option
- AcpEvent
- syntax_language
- Self
- Option
- editor-dap/src/config.rs
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- .connect_raw
- String
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- render.rs
- shell/git.rs
- shell/mod.rs
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- EditorRuntime
- Self
- Option
- Result
- .new
- shell/browser.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- state.rs
- LanguageServerSpec
- editor-git/src/lib.rs
- user/picker.rs
- HeaderlineTestUserLibrary
- AbiContextHelpSpec
- String
- PluginPackage
- ShellError
- .len
- DapClientManager
- picker_items
- active_runtime_popup
- build_output.rs
- PluginBuffer
- BufferId
- Section
- PluginCommand
- UserLibrary
- SyntaxRegistry
- clipboard.rs
- shell/acp.rs
- editor-dap/src/lib.rs
- .new
- WorkspaceConfigurationValue
- LineSyntaxSpan
- .send
- DbService
- cargo
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- Option
- volt/src/main.rs
- DynamicUserLibrary
- String
- shell_ui
- ShellUiState
- start_struct_ctor_session
- draw_diagnostic_underlines_for_segment
- ProjectCandidate
- resolve_picker_extra
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- treesittercontext_ghosttext.rs
- TextBuffer
- GitEditorState
- modeline.rs
- .from_entries
- .spawn
- editor-path/src/lib.rs
- .ghost_text_lines
- browser_host.rs
- String
- editor-icons/src/lib.rs
- LspNotification
- JobSpec
- shell/picker.rs
- TerminalCursorSnapshot
- .default
- package
- Option
- PluginKeyBinding
- LspLocation
- Vec
- String
- nix.rs
- process_supervisor.rs
- Vec
- PixelRect
- DapSessionHandle
- DebugAdapterSpec
- bash.rs
- editor-picker/src/lib.rs
- StoredBreakpoint
- shell/tests.rs
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- DebugConfigurationCandidate
- key_sequence.rs
- PickerSession
- build_job_command
- AbiSectionTree
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- hcl.rs
- capture_mappings
- java.rs
- .new
- CommandLineOverlay
- kotlin.rs
- PaneConfig
- resolve_permission
- latex.rs
- volt/build.rs
- .oil_directory_sections
- r.rs
- swift.rs
- syntax_language
- test_service
- oil.rs
- user/db.rs
- xml.rs
- Vec
- show_paren.rs
- PickerItemSpec
- load
- Copilot instructions for `volt`
- editor-dap/src/client.rs
- Self
- PickerItem
- theme.rs
- ServiceRegistry
- lua.rs
- String
- user/terminal.rs
- corpus_inventory.rs
- rainbow_parens.rs
- DapEvaluateContext
- AbiGitFeatureSpec
- dap.rs
- JobResult
- treesittercontext_shared.rs
- user/browser.rs
- .request
- .oil_directory_sections
- `user`
- PathBuf
- predicate_capture_text
- Result
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- DebugConfiguration
- ancestor_contexts_for_cursor
- run_demo_shell
- html.rs
- LspLogEntry
- AcpManager
- common.rs
- Database Explorer PRD
- proto.rs
- .from_text
- .path
- connect_transport
- String
- TextEdit
- Option
- markdown.rs
- VimActionContext
- .acp_client_by_id
- lang/vim.rs
- Diagnostic
- headerline_lines
- clojure.rs
- elixir.rs
- 0004-markdown-pretty-pipeline.md
- DbEngine
- main
- .git_command_for_chord
- graphql.rs
- dap-client-spec.md
- terminal_key_for_event
- .autocomplete_providers
- Language
- .browser_feature_spec
- Domain Docs
- Issue tracker: GitHub
- setup_standalone_user_repository
- syntax_language
- .context_help_specs
- .db_feature_spec
- .git_feature_spec
- .hover_providers
- rainbow_paren.rs
- perl.rs
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pdf_open_mode
- .picker_layout
- String
- git_remote_worktree_branch_list
- .picker_truncate_strategy
- .show_paren_config
- .terminal_feature_spec
- .workspace_roots
- php.rs
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- ruby.rs
- scala.rs
- Agent skills
- solidity.rs
- user/workspace_dock.rs
- 0005-dap-session-and-client.md
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 861 edges
2. `ShellBuffer` - 391 edges
3. `shell_ui_mut()` - 382 edges
4. `register_shell_hooks()` - 272 edges
5. `shell_ui()` - 264 edges
6. `shell_buffer()` - 198 edges
7. `shell_buffer_mut()` - 196 edges
8. `ShellError` - 194 edges
9. `ShellUiState` - 186 edges
10. `TextBuffer` - 180 edges

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

## Communities (316 total, 27 thin omitted)

### Community 0 - ".new"
Cohesion: 0.02
Nodes (167): default_vim_target(), absolute_path_hint(), acp_build_output_lines(), acp_build_plan_lines(), acp_chat_bubble_cols(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat() (+159 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (29): ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), LspClientError, LspClientManager (+21 more)

### Community 2 - "temp_dir"
Cohesion: 0.11
Nodes (30): build_git_fringe_snapshot(), create_git_worktree_from_query(), git_commit_temp_path(), git_common_dir(), git_fringe_snapshot_ignores_crlf_only_difference(), git_fringe_snapshot_is_empty_when_buffer_matches_head(), git_fringe_temp_path(), git_push_remote_name_prefers_branch_push_remote_for_slashy_branch_names() (+22 more)

### Community 3 - ".new"
Cohesion: 0.12
Nodes (73): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+65 more)

### Community 4 - "ShellState"
Cohesion: 0.05
Nodes (27): clear_key_sequence(), active_runtime_surface(), alt_mod(), apply_multicursor_delete(), apply_multicursor_insert_text(), browser_devtools_shortcut_requested(), build_keydown_chord(), build_shell_summary() (+19 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (119): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+111 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.07
Nodes (87): additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+79 more)

### Community 7 - "editor-lsp/src/client.rs"
Cohesion: 0.05
Nodes (90): BufRead, char_to_byte_offset(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_documentation(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+82 more)

### Community 8 - "shell_user_library"
Cohesion: 0.06
Nodes (47): BufferKind, activate_db_browser_line(), active_buffer_event_context(), active_dashboard_editor_buffer(), active_or_open_dashboard_buffer(), apply_db_browser_view(), apply_db_browser_view_to_section(), buffer_is_acp() (+39 more)

### Community 9 - "Option"
Cohesion: 0.06
Nodes (67): acp_rendered_text_segments(), display_columns_for_character(), is_wide_display_character(), LineCharMap, LineWrapSegment, multicursor_selection_offsets(), resolved_tab_width(), segment_index_for_column() (+59 more)

### Community 10 - "AcpEvent"
Cohesion: 0.10
Nodes (29): AvailableCommand, AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), format_acp_mode_label() (+21 more)

### Community 11 - "syntax_language"
Cohesion: 0.08
Nodes (29): package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+21 more)

### Community 12 - "Self"
Cohesion: 0.05
Nodes (22): Self, default_action(), exported_acp_picker_items(), hook_command(), Option, AcpActionSpec, AcpPickerContext, AcpPickerItemSpec (+14 more)

### Community 13 - "Option"
Cohesion: 0.06
Nodes (54): parse_log_oneline(), apply_git_fringe_hunk(), build_git_summary_snapshot(), command_output_transcript(), find_paren_number_range(), git_branch_merge(), git_branch_push_remote(), git_branch_remote() (+46 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.12
Nodes (37): collect_configuration_candidates(), configuration_holes(), configuration_holes_detect_missing_launch_program(), DapConfigError, DebugInferContext, DebugStartHistory, DebugStartRecord, deep_inference_finds_cargo_binary_and_heuristic() (+29 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.08
Nodes (22): AlacrittyEvent, LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc, Display, Drop, Error (+14 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.13
Nodes (36): default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects(), discover_projects_finds_git_repositories_and_worktrees() (+28 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.09
Nodes (10): GitLogEntry, GitStashEntry, GitStatusSnapshot, RepositoryStatus, Into, Option, Self, String (+2 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.05
Nodes (114): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+106 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (36): CommandPaletteState, CompilationState, DynamicUserLibrary, EventLog, format_micros_as_millis(), panic_payload_message(), AcpClient, Any (+28 more)

### Community 20 - "HookBus"
Cohesion: 0.07
Nodes (23): HookBus, HookDefinition, HookError, HookEvent, HookSubscription, BTreeMap, BufferId, Default (+15 more)

### Community 21 - "EditorModel"
Cohesion: 0.07
Nodes (26): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+18 more)

### Community 22 - "KeymapScope"
Cohesion: 0.10
Nodes (33): autocomplete_overrides_workspace_while_active(), BindingKey, ChordModifier, dap_mode_overrides_global_f5_while_session_live(), duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeyBinding (+25 more)

### Community 23 - "calculator.rs"
Cohesion: 0.07
Nodes (39): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+31 more)

### Community 24 - ".connect_raw"
Cohesion: 0.14
Nodes (10): DbActionOutcome, DbSessionSummary, InMemorySecretStore, redact_error(), redact_key_value_segments(), remembered_connections_store_metadata_separately_from_secret(), HashMap, Mutex (+2 more)

### Community 25 - "String"
Cohesion: 0.07
Nodes (113): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk() (+105 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.08
Nodes (44): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+36 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "Theme"
Cohesion: 0.12
Nodes (21): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+13 more)

### Community 31 - "render.rs"
Cohesion: 0.04
Nodes (106): Canvas, RenderColor, FontSet, is_zero_width_display_character(), acp_spinner_frame(), adjusted_contextual_ligature_pixel_size(), alpha_bitmap_surface(), ascii_ligature_byte_ranges_with_face() (+98 more)

### Community 32 - "shell/git.rs"
Cohesion: 0.05
Nodes (167): run_command(), active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit() (+159 more)

### Community 33 - "shell/mod.rs"
Cohesion: 0.02
Nodes (245): ActiveLspBufferContext, WorkspaceId, AcpDecodedImage, AcpRenderedImageLine, active_lsp_buffer_context(), active_workspace_has_debug_session(), active_workspace_open_buffer_paths(), ActiveTypingFrameProfile (+237 more)

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

### Community 39 - "EditorRuntime"
Cohesion: 0.03
Nodes (322): EditorRuntime, Default, focus_active_browser_popup(), Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), active_directory_root() (+314 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (63): acp_output_header_title(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_tool_call_from_partial_update() (+55 more)

### Community 42 - "Result"
Cohesion: 0.07
Nodes (105): default_error_log_path(), format_current_line_indent(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode() (+97 more)

### Community 43 - ".new"
Cohesion: 0.04
Nodes (108): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_paste_image_inserts_mention_token_and_stores_bytes() (+100 more)

### Community 44 - "shell/browser.rs"
Cohesion: 0.09
Nodes (50): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_buffer_layout(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_host_viewport_rect(), browser_state_for_kind() (+42 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (73): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+65 more)

### Community 47 - "state.rs"
Cohesion: 0.12
Nodes (23): BlockInsertState, DirectoryYankEntry, FormatterRegistry, FormatterSpec, LastFind, LastSearch, BTreeMap, BufferId (+15 more)

### Community 48 - "LanguageServerSpec"
Cohesion: 0.05
Nodes (64): csharp_language_server(), dev_extension_server(), directory_contains_extension(), directory_matches_root_marker(), dockerfile_language_server(), document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path() (+56 more)

### Community 49 - "editor-git/src/lib.rs"
Cohesion: 0.13
Nodes (25): configure_background_command(), detect_in_progress(), git_available(), GitStatusError, list_repository_files(), parse_header(), parse_stash_list(), parse_status() (+17 more)

### Community 50 - "user/picker.rs"
Cohesion: 0.14
Nodes (29): acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items(), height_fraction() (+21 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (59): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage() (+51 more)

### Community 52 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 53 - "String"
Cohesion: 0.30
Nodes (21): apply_language_options_table(), apply_options_table(), parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option() (+13 more)

### Community 54 - "PluginPackage"
Cohesion: 0.09
Nodes (28): file_open_package(), package(), package(), package(), package_exports_image_commands(), package_exports_image_keybindings(), package(), LanguageConfiguration (+20 more)

### Community 55 - "ShellError"
Cohesion: 0.11
Nodes (96): Display, Error, From, ShellError, render_browser_buffer_body(), Color, adjust_color(), blend_color() (+88 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (31): apply_input_operator_motion(), ascii_control_caret_notation(), change_operator_word_motion(), char_at_index(), exact_match_positions_in_chars(), find_char_forward(), fuzzy_match_end(), fuzzy_match_end_in_chars() (+23 more)

### Community 57 - "DapClientManager"
Cohesion: 0.15
Nodes (13): active_thread_id(), clear_stopped_snapshot(), DapClientError, DapClientManager, Display, Error, Formatter, Path (+5 more)

### Community 58 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (52): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+44 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "PluginBuffer"
Cohesion: 0.06
Nodes (11): dashboard_sections(), sidebar_sections(), DbBrowserKind, plugin_buffer_sections_can_declare_nested_layout_tree(), PluginBuffer, PluginBufferLayout, PluginBufferLayoutAxis, PluginBufferLayoutNode (+3 more)

### Community 62 - "BufferId"
Cohesion: 0.16
Nodes (19): ActiveBufferEventContext, diff_git_commit_at_point(), diff_git_stash_at_point(), finish_oil_worktree_branch_selection(), git_action_detail(), git_line_is_untracked(), git_status_action_targets(), git_status_delete_target_for_line() (+11 more)

### Community 63 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 64 - "PluginCommand"
Cohesion: 0.10
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 65 - "UserLibrary"
Cohesion: 0.06
Nodes (48): buffer_context_overlay_snapshot(), debug_fringe_cell_count(), editor_fringe_width_px(), FpsOverlaySnapshot, pixel_rect_contains_point(), popup_window_height(), runtime_pane_rects(), RuntimePopupSnapshot (+40 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.05
Nodes (71): append_query_source(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), compile_query_source(), create_parser() (+63 more)

### Community 67 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 68 - "shell/acp.rs"
Cohesion: 0.08
Nodes (67): acp_file_uri(), acp_slash_completion_query(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates() (+59 more)

### Community 69 - "editor-dap/src/lib.rs"
Cohesion: 0.11
Nodes (18): Client, codelldb(), DapError, DebugAdapterTransport, DebugSessionPlan, gdb(), must(), prepared_session_includes_configuration_and_launch_spec() (+10 more)

### Community 70 - ".new"
Cohesion: 0.11
Nodes (37): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command() (+29 more)

### Community 71 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (15): sanitize_transport_message(), transport_key_is_sensitive(), AsRef, From, Number, T, WorkspaceConfigurationValue, K (+7 more)

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (47): dap_variable_line_spans(), browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines() (+39 more)

### Community 73 - ".send"
Cohesion: 0.13
Nodes (37): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+29 more)

### Community 74 - "DbService"
Cohesion: 0.14
Nodes (14): db_browser_action_from_spec(), DbAutocompleteCandidate, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbQueryBufferMeta, DbService, DbSession (+6 more)

### Community 75 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 76 - "directory.rs"
Cohesion: 0.12
Nodes (47): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+39 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (36): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+28 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.10
Nodes (45): PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank(), lsp_code_action_kind_matches() (+37 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "Option"
Cohesion: 0.20
Nodes (3): From, Option, ThemeOption

### Community 82 - "volt/src/main.rs"
Cohesion: 0.08
Nodes (42): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), command_palette_items(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, load_user_library(), LspState (+34 more)

### Community 84 - "String"
Cohesion: 0.31
Nodes (18): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+10 more)

### Community 85 - "shell_ui"
Cohesion: 0.06
Nodes (46): shell_ui(), browser_buffer_submit_tracks_requested_navigation(), browser_escape_from_insert_keeps_input_cursor_position(), browser_host_focus_parent_event_returns_to_normal_mode(), browser_host_new_window_event_routes_into_browser_popup(), browser_host_open_devtools_event_is_ignored_without_a_live_webview(), browser_input_layout_uses_symmetric_vertical_padding(), browser_location_updates_rename_buffer_with_current_url() (+38 more)

### Community 86 - "ShellUiState"
Cohesion: 0.03
Nodes (76): acp_decode_image(), active_buffer_revision_key(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_window_id(), buffer_is_oil_preview(), BufferViewState, close_db_multiview() (+68 more)

### Community 87 - "start_struct_ctor_session"
Cohesion: 0.21
Nodes (12): build_csharp_fixture(), DapStackFrameInfo, find_named_dll(), PathBuf, sharpdbg_double_step_over_struct_construction_keeps_session(), sharpdbg_expand_struct_local_keeps_session(), sharpdbg_step_into_from_entry_keeps_session_through_struct_ctor(), sharpdbg_step_into_struct_construction_keeps_session() (+4 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.15
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "ProjectCandidate"
Cohesion: 0.23
Nodes (5): compact_project_path(), ProjectCandidate, ProjectKind, String, worktree_parent_name()

### Community 90 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.13
Nodes (37): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), bootstrap(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames() (+29 more)

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

### Community 96 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 97 - "TextBuffer"
Cohesion: 0.04
Nodes (27): advance_point_by_text(), delimiter_partner(), EditRecord, find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at() (+19 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - ".from_entries"
Cohesion: 0.18
Nodes (12): ctrl_mod(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_text_input_updates_acp_input(), browser_sync_plan_hides_surfaces_while_picker_is_visible(), ctrl_q_with_non_quickfix_picker_does_not_quit(), paste_text_into_active_input_buffer_closes_acp_picker_for_multiline_text(), picker_extra_keybind_falls_through_for_shared_popup_navigation(), picker_extra_keybind_snapshots_context_closes_and_runs_command() (+4 more)

### Community 101 - ".spawn"
Cohesion: 0.11
Nodes (21): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+13 more)

### Community 102 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 104 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 105 - "String"
Cohesion: 0.07
Nodes (54): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), documentation_lines(), explicit_windows_env_value(), hover_marked_string() (+46 more)

### Community 106 - "editor-icons/src/lib.rs"
Cohesion: 0.15
Nodes (11): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+3 more)

### Community 107 - "LspNotification"
Cohesion: 0.06
Nodes (29): ChildStdin, completion_level_for_message(), diagnostic_matches_request_range(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel (+21 more)

### Community 108 - "JobSpec"
Cohesion: 0.17
Nodes (13): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), compilation_runner_marks_jobs_as_compilation(), job_manager_runs_commands_and_collects_output(), JobKind, JobSpec, must(), E (+5 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.10
Nodes (38): ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars() (+30 more)

### Community 110 - "TerminalCursorSnapshot"
Cohesion: 0.28
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 111 - ".default"
Cohesion: 0.10
Nodes (49): Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title() (+41 more)

### Community 112 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 113 - "Option"
Cohesion: 0.09
Nodes (10): Option, Vec, terminal_render_snapshot_preserves_wide_character_widths(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot, TerminalTranscript (+2 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (24): plugin_buffer_binding_scope_active(), plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding() (+16 more)

### Community 115 - "LspLocation"
Cohesion: 0.22
Nodes (5): definition_parser_preserves_uri_backed_locations(), location_sorting_deduplicates_reference_results(), LspLocation, parse_reference_response(), sort_locations()

### Community 116 - "Vec"
Cohesion: 0.07
Nodes (16): CopilotDeviceCodePrompt, formatting_parser_maps_text_edits(), lsp_formatting_options(), LspCodeAction, LspDocumentTextEdits, LspFormattingOptions, LspHoverContents, LspServerCommand (+8 more)

### Community 117 - "String"
Cohesion: 0.08
Nodes (52): box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, column_is_numeric(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor (+44 more)

### Community 118 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.36
Nodes (7): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), Vec, WorkspaceRootConfig, WorkspaceSection

### Community 121 - "PixelRect"
Cohesion: 0.11
Nodes (38): PixelRect, rect_tuple(), along_size(), child_rect(), gap_is_inserted_between_siblings(), layout_child(), layout_node(), layout_split_tree() (+30 more)

### Community 122 - "DapSessionHandle"
Cohesion: 0.10
Nodes (36): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), DapSessionEvent, DapSessionHandle, DapSessionInfo, evaluate_expression(), expand_variable_node() (+28 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.20
Nodes (5): DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, BTreeMap, Vec

### Community 124 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 125 - "editor-picker/src/lib.rs"
Cohesion: 0.17
Nodes (18): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+10 more)

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (45): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+37 more)

### Community 127 - "shell/tests.rs"
Cohesion: 0.03
Nodes (62): active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), configure_file_buffer(), contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size() (+54 more)

### Community 128 - "JobError"
Cohesion: 0.18
Nodes (12): CompilationRunner, JobError, JobHandle, JobManager, Display, Error, Formatter, From (+4 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.21
Nodes (21): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+13 more)

### Community 130 - "user/config.rs"
Cohesion: 0.17
Nodes (25): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+17 more)

### Community 131 - "DebugConfigurationCandidate"
Cohesion: 0.19
Nodes (6): DebugConfigurationCandidate, DebugConfigurationSource, default_request(), Into, Self, String

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 134 - "build_job_command"
Cohesion: 0.32
Nodes (8): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, windows_fnm_environment(), configure_background_command(), Command

### Community 135 - "AbiSectionTree"
Cohesion: 0.18
Nodes (9): exported_git_status_sections(), DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 138 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 139 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 140 - ".new"
Cohesion: 0.08
Nodes (33): AsyncRead, buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), install_acp_test_buffer(), pending_slash_completion_trigger_rejects_multiline_input(), permission_prompt_lines() (+25 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 143 - "PaneConfig"
Cohesion: 0.07
Nodes (16): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig (+8 more)

### Community 144 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 145 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 148 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 149 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 150 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 151 - "test_service"
Cohesion: 0.18
Nodes (15): db_browser_renderer_customizes_rows_and_preserves_actions(), default_volt_state_dir(), insert_test_session(), Arc, PathBuf, Self, Send, Sync (+7 more)

### Community 152 - "oil.rs"
Cohesion: 0.10
Nodes (35): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec(), help_entry() (+27 more)

### Community 153 - "user/db.rs"
Cohesion: 0.12
Nodes (26): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings() (+18 more)

### Community 154 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 155 - "Vec"
Cohesion: 0.12
Nodes (25): ColumnData, Compat, connect_sql_server(), DbColumn, DbIndex, DbSchemaCache, DbTable, load_postgres_schema() (+17 more)

### Community 156 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 157 - "PickerItemSpec"
Cohesion: 0.08
Nodes (33): PickerItemSpec, package(), package_exports_recompile_installed_command(), picker_items(), Vec, package(), picker_items(), Vec (+25 more)

### Community 158 - "load"
Cohesion: 0.11
Nodes (27): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+19 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.12
Nodes (15): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+7 more)

### Community 160 - "editor-dap/src/client.rs"
Cohesion: 0.14
Nodes (33): assert_control_requests_omit_nulls(), client_initialize_launch_disconnect_against_fake_tcp_adapter(), continue_step_pause_and_locals_against_fake_adapter(), continue_to_process_exit_queues_terminated(), dap_log_text(), DapLogDirection, DapLogEntry, DapLogSnapshot (+25 more)

### Community 161 - "Self"
Cohesion: 0.02
Nodes (159): DebugAdapterRootStrategy, GitStashEntry, exported_pdf_open_mode(), exported_picker_truncate_strategy(), PdfOpenMode, PickerTruncateStrategy, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers() (+151 more)

### Community 162 - "PickerItem"
Cohesion: 0.22
Nodes (6): PickerItem, PickerMatch, Into, Option, Self, String

### Community 163 - "theme.rs"
Cohesion: 0.15
Nodes (30): assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages(), bundled_themes_use_pallet_sections_and_token_references(), list_theme_files() (+22 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 166 - "String"
Cohesion: 0.27
Nodes (7): lsp_location_uri_detail(), call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "user/terminal.rs"
Cohesion: 0.20
Nodes (11): default_terminal_args(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package() (+3 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 170 - "DapEvaluateContext"
Cohesion: 0.67
Nodes (3): DapEvaluateContext, EvaluateArgumentsContext, From

### Community 171 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 172 - "dap.rs"
Cohesion: 0.24
Nodes (12): adapter_preferences_match_language_defaults(), debug_adapters(), locals_buffer_declares_locals_and_expressions_sections(), locals_sections(), package(), package_exports_debug_layout_buffers(), package_exports_polish_commands(), package_exports_start_family_commands() (+4 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "treesittercontext_shared.rs"
Cohesion: 0.23
Nodes (18): find(), IconFontSymbol, Option, symbols(), collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword() (+10 more)

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - ".request"
Cohesion: 0.40
Nodes (4): Arguments, parse_response_body(), strip_null_fields(), Response

### Community 177 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "PathBuf"
Cohesion: 0.08
Nodes (33): close_buffer_keeps_session_alive_for_next_file(), default_workspace_lists_only_sessions_serving_open_buffers(), file_uri_roundtrip_handles_windows_paths(), language_server_session_in_workspace_scope(), live_session_picker_label_includes_server_and_root(), live_sessions_for_workspace_includes_root_scoped_and_buffer_served(), LspClientState, LspLiveSession (+25 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

### Community 181 - "Result"
Cohesion: 0.20
Nodes (10): db_browser_renderer_rejects_row_count_mismatch(), DbBrowserBufferView, initialize_native_keyring(), OsSecretStore, Into, Result, section_count_label(), summarize_sql() (+2 more)

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.18
Nodes (10): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+2 more)

### Community 185 - "DebugConfiguration"
Cohesion: 0.13
Nodes (11): attach_arguments(), DebugConfiguration, DebugRequestKind, normalize_extension(), Into, IntoIterator, Item, Option (+3 more)

### Community 186 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 187 - "run_demo_shell"
Cohesion: 0.06
Nodes (39): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+31 more)

### Community 188 - "html.rs"
Cohesion: 0.60
Nodes (4): package(), package_registers_expected_html_bindings_and_formatter(), LanguageConfiguration, syntax_language()

### Community 189 - "LspLogEntry"
Cohesion: 0.16
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 190 - "AcpManager"
Cohesion: 0.09
Nodes (24): AcpManager, AcpPendingPermissionUi, AcpUiAction, config_option_is_mode(), config_option_is_model(), config_option_matches(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work() (+16 more)

### Community 191 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 194 - ".from_text"
Cohesion: 0.04
Nodes (76): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings() (+68 more)

### Community 195 - ".path"
Cohesion: 0.21
Nodes (11): db_connect_enter_submits_pasted_connection_string(), db_query_buffer_receives_sql_highlighting_without_blocking(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test() (+3 more)

### Community 196 - "connect_transport"
Cohesion: 0.22
Nodes (9): configure_adapter_command(), connect_tcp(), connect_transport(), Child, Command, DebugAdapterTransport, TcpStream, spawn_adapter_command() (+1 more)

### Community 197 - "String"
Cohesion: 0.09
Nodes (54): AcpClientConfig, acp_complete_slash(), acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_image_mention_token() (+46 more)

### Community 198 - "TextEdit"
Cohesion: 0.67
Nodes (4): TextEdit, apply_text_edits_to_span(), text_edit_to_input_edit(), InputEdit

### Community 199 - "Option"
Cohesion: 0.06
Nodes (21): collapse_variable_path(), DapExecutionPosition, DapStoppedSnapshot, DapThreadInfo, DapVariableNode, DapVariablePath, DapVariableRow, DapWatchExpression (+13 more)

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

### Community 203 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 204 - "Diagnostic"
Cohesion: 0.09
Nodes (22): CodeActionParams, code_action_params(), code_action_params_use_flattened_lsp_shape(), full_document_range(), full_sync_uses_null_range_change(), incremental_sync_uses_full_document_replacement_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+14 more)

### Community 205 - "headerline_lines"
Cohesion: 0.19
Nodes (11): packages(), LanguageConfiguration, Vec, syntax_languages(), build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option (+3 more)

### Community 206 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 207 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 209 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbSnippet, load_persisted_state(), PersistedDbState, RememberedConnection, Path

### Community 210 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 212 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 214 - "terminal_key_for_event"
Cohesion: 0.67
Nodes (3): Keycode, Mod, terminal_key_for_event()

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "setup_standalone_user_repository"
Cohesion: 0.33
Nodes (6): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 221 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 227 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 234 - "String"
Cohesion: 0.06
Nodes (27): asset_path_from_parts(), CaptureThemeMapping, command_failure_message(), default_install_root(), default_query_asset_root(), DeferredQuery, GrammarRecompileFailure, GrammarRecompileReport (+19 more)

### Community 235 - "git_remote_worktree_branch_list"
Cohesion: 0.38
Nodes (7): begin_oil_worktree_request(), git_remote_worktree_branch_list(), oil_git_worktree_command(), open_git_worktree_dashboard_create(), remote_and_branch_from_ref(), Into, trace_oil_worktree()

### Community 242 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 247 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 248 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 251 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 253 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

## Knowledge Gaps
- **153 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+148 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **27 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `.new`, `temp_dir`, `ShellState`, `shell_user_library`, `AcpEvent`, `Option`, `resolve_permission`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `String`, `command_stream.rs`, `shell/git.rs`, `shell/mod.rs`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `Result`, `shell/browser.rs`, `.len`, `active_runtime_popup`, `AcpManager`, `BufferId`, `.path`, `shell/acp.rs`, `String`, `.new`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `volt/src/main.rs`, `String`, `shell_ui`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `git_remote_worktree_branch_list`, `shell/picker.rs`, `shell/tests.rs`?**
  _High betweenness centrality (0.144) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `.new`, `ShellState`, `shell_user_library`, `Option`, `Option`, `String`, `shell/mod.rs`, `shell/pdf.rs`, `EditorRuntime`, `Result`, `.new`, `shell/browser.rs`, `state.rs`, `ShellError`, `.len`, `BufferId`, `UserLibrary`, `shell/acp.rs`, `.new`, `LineSyntaxSpan`, `directory.rs`, `shell/terminal.rs`, `ShellUiState`, `draw_diagnostic_underlines_for_segment`, `TextBuffer`, `shell/picker.rs`, `Option`, `PixelRect`, `StoredBreakpoint`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `UserLibrary` to `.new`, `ShellState`, `user/lib.rs`, `shell_user_library`, `Option`, `Option`, `DynamicUserLibrary`, `HoverOverlay`, `shell/mod.rs`, `Option`, `Result`, `shell/browser.rs`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `ShellError`, `run_demo_shell`, `directory.rs`, `volt/src/main.rs`, `DynamicUserLibrary`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `shell/picker.rs`?**
  _High betweenness centrality (0.055) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _153 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `.new` be split into smaller, more focused modules?**
  _Cohesion score 0.015981683912819015 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.0780650542118432 - nodes in this community are weakly interconnected._
- **Should `temp_dir` be split into smaller, more focused modules?**
  _Cohesion score 0.10685483870967742 - nodes in this community are weakly interconnected._