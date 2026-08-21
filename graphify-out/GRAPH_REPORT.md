# Graph Report - volt  (2026-08-21)

## Corpus Check
- 246 files · ~623,705 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10051 nodes · 41134 edges · 329 communities (300 shown, 29 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3327 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `415ab318`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- PathBuf
- Path
- String
- src/tests.rs
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- Option
- shell/browser.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- shell/git.rs
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
- Result
- state_with_user_library
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- ThemeRegistry
- FontSet
- String
- .new
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- ShellBuffer
- shell_buffer_mut
- Result
- Vec
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- String
- LanguageServerSession
- active_shell_buffer_mut
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- editor-lsp/src/lib.rs
- LanguageServerSpec
- AbiOilFeatureSpec
- ShellError
- .len
- DapClientError
- .new
- active_runtime_popup
- build_output.rs
- String
- .new
- TextBuffer
- PluginCommand
- workspace_dock_layout
- SyntaxRegistry
- PluginBufferSection
- Option
- DebugConfiguration
- .new
- WorkspaceConfigurationValue
- String
- .send
- DbService
- clipboard.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- workspace.rs
- volt/src/main.rs
- DynamicUserLibrary
- Option
- Option
- ShellUiState
- .new
- wrap_line_segments
- show_paren.rs
- buffer_footer_layout_with_command_line
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- treesittercontext_ghosttext.rs
- editor-picker/src/lib.rs
- GitEditorState
- modeline.rs
- Self
- .spawn
- editor-path/src/lib.rs
- abi.rs
- BufferId
- editor-lsp/src/client.rs
- DapSessionHandle
- main
- JobSpec
- shell/picker.rs
- TextEdit
- .default
- AbiLanguageConfiguration
- RVec
- PluginKeyBinding
- DbBrowserBufferView
- LspCodeAction
- String
- LspSessionHandle
- process_supervisor.rs
- AbiSectionTree
- shell/acp.rs
- PixelRect
- LineSyntaxSpan
- GitSummaryState
- treesittercontext_shared.rs
- StoredBreakpoint
- shell/tests.rs
- JobError
- Option
- user/config.rs
- ancestor_contexts_for_cursor
- key_sequence.rs
- editor-icons/src/lib.rs
- .get
- common.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- build_job_command
- Section
- DbEngine
- String
- CommandLineOverlay
- From
- AbiPaneConfig
- resolve_permission
- editor-dap/src/client.rs
- volt/build.rs
- .oil_directory_sections
- headerline_lines
- TextRange
- browser_host.rs
- xml.rs
- oil.rs
- user/db.rs
- lsp.rs
- browser_sync_plan
- cargo
- PickerItemSpec
- load
- Copilot instructions for `volt`
- find_font_by_name
- AbiContextHelpSpec
- TerminalTranscript
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
- Option
- aligned_indent_column
- hcl.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- ruby.rs
- java.rs
- perl.rs
- php.rs
- flatten_config_select_options
- proto.rs
- Database Explorer PRD
- solidity.rs
- .from_text
- swift.rs
- lang/vim.rs
- AcpManager
- UserLibraryModule
- TerminalDimensions
- markdown.rs
- TerminalCursorSnapshot
- .oil_directory_sections
- scala.rs
- .byte_slice_chunks
- rainbow_parens.rs
- configure_background_command
- load_user_library
- 0004-markdown-pretty-pipeline.md
- syntax_languages
- AbiKeymapConfig
- AbiPickerTruncateStrategy
- Vec
- dap-client-spec.md
- .acp_client_by_id
- package
- Language
- AbiLigatureConfig
- Domain Docs
- Issue tracker: GitHub
- AbiIconFontSymbol
- .git_command_for_chord
- .autocomplete_providers
- .browser_feature_spec
- .context_help_specs
- .db_feature_spec
- rainbow_paren.rs
- .ghost_text_lines
- .git_feature_spec
- .hover_providers
- .keymap_config
- .ligature_config
- .oil_feature_spec
- user/workspace_dock.rs
- package
- .oil_keybindings
- .pdf_open_mode
- .picker_layout
- main
- .picker_truncate_strategy
- keymap.rs
- .show_paren_config
- spawn_terminal_reader
- .terminal_feature_spec
- .workspace_roots
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- connect_sql_server
- LspLogEntry
- Agent skills
- ShellConfig
- ProjectCandidate
- AbiTheme
- .new
- dap.rs
- .next_token
- ligatures.rs
- 0005-dap-session-and-client.md
- latex.rs
- AbiPdfOpenMode
- syntax_language
- WorkspaceDockConfig
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 836 edges
2. `ShellBuffer` - 390 edges
3. `shell_ui_mut()` - 371 edges
4. `register_shell_hooks()` - 267 edges
5. `shell_ui()` - 258 edges
6. `shell_buffer_mut()` - 195 edges
7. `ShellError` - 194 edges
8. `shell_buffer()` - 192 edges
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

## Communities (329 total, 29 thin omitted)

### Community 0 - "PathBuf"
Cohesion: 0.02
Nodes (117): BufferKind, default_vim_target(), active_theme_state_path(), append_error_log(), asset_path_from_parts(), buffer_interaction(), buffer_is_dap_layout_side(), buffer_is_git_commit() (+109 more)

### Community 1 - "Path"
Cohesion: 0.09
Nodes (22): inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, parse_definition_response(), parse_text_edit_response() (+14 more)

### Community 2 - "String"
Cohesion: 0.05
Nodes (87): ctrl_mod(), shell_ui(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer(), acp_slash_picker_backspace_can_delete_leading_slash(), acp_slash_picker_text_input_updates_acp_input(), acp_switch_pane_command_changes_internal_pane_without_changing_workspace_pane(), browser_buffer_submit_tracks_requested_navigation() (+79 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.14
Nodes (64): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+56 more)

### Community 4 - "ShellState"
Cohesion: 0.03
Nodes (53): clear_key_sequence(), active_buffer_revision_key(), active_lsp_workspace_loaded(), active_runtime_surface(), ActiveTypingFrameProfile, alt_mod(), average_duration(), browser_devtools_shortcut_requested() (+45 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (97): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+89 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.07
Nodes (86): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), asset_path_from_parts(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+78 more)

### Community 7 - "Option"
Cohesion: 0.02
Nodes (79): absolute_path_hint(), block_comment_toggle_removal_lens(), closing_tag_name_after_cursor(), comment_style_for_buffer(), comment_style_for_language_path(), comment_toggle_removal_len(), CommentStyle, current_theme_source_fingerprint() (+71 more)

### Community 8 - "shell/browser.rs"
Cohesion: 0.11
Nodes (40): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_state_for_kind(), browser_surface_buffer_at_point(), browser_url_candidates() (+32 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (110): centered_rect(), advance_point_by_text(), multicursor_selection_offsets(), acp_slice_chars(), adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines(), autocomplete_visible_start() (+102 more)

### Community 10 - "AcpEvent"
Cohesion: 0.09
Nodes (29): AvailableCommand, AcpEvent, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks(), command_input_hint() (+21 more)

### Community 11 - "PluginPackage"
Cohesion: 0.05
Nodes (47): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+39 more)

### Community 12 - "Self"
Cohesion: 0.06
Nodes (17): picker_items(), AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, picker_provider_spec_accepts_extra_keybinds(), PickerActionSpec (+9 more)

### Community 13 - "shell/git.rs"
Cohesion: 0.06
Nodes (78): parse_log_oneline(), apply_git_fringe_hunk(), begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), fetch_git_prune(), git_branch_list() (+70 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.09
Nodes (37): collect_configuration_candidates(), configuration_holes(), configuration_holes_detect_missing_launch_program(), DapConfigError, DebugConfigurationCandidate, DebugConfigurationSource, DebugInferContext, DebugStartHistory (+29 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (25): AlacrittyEvent, Keycode, Mod, Self, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener (+17 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.13
Nodes (36): default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects(), discover_projects_finds_git_repositories_and_worktrees() (+28 more)

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
Cohesion: 0.08
Nodes (32): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+24 more)

### Community 24 - "Result"
Cohesion: 0.18
Nodes (8): DisabledSecretStore, InMemorySecretStore, redact_error(), remembered_connections_store_metadata_separately_from_secret(), HashMap, Result, snippets_and_history_persist(), unix_epoch_secs()

### Community 25 - "state_with_user_library"
Cohesion: 0.05
Nodes (91): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), active_input_prompt_text(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk(), buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus() (+83 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.14
Nodes (24): font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches(), font_style_rank(), golden_split_size(), horizontal_golden_ratio_grows_the_first_active_pane() (+16 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, notification_severity(), RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.08
Nodes (27): acp_chat_corner_radius(), acp_chat_rounded(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap (+19 more)

### Community 31 - "FontSet"
Cohesion: 0.07
Nodes (51): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, is_zero_width_display_character() (+43 more)

### Community 32 - "String"
Cohesion: 0.06
Nodes (130): run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer(), create_git_worktree() (+122 more)

### Community 33 - ".new"
Cohesion: 0.11
Nodes (36): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_status_diff_staged_command(), git_status_log_all_branches_command(), git_status_log_all_command() (+28 more)

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

### Community 39 - "shell/mod.rs"
Cohesion: 0.03
Nodes (377): EditorRuntime, Default, write_system_clipboard(), accept_autocomplete(), acp_decode_image(), activate_db_browser_line(), active_buffer_event_context(), active_dashboard_editor_buffer() (+369 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.02
Nodes (78): acp_output_header_title(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_tool_call_from_partial_update() (+70 more)

### Community 42 - "shell_buffer_mut"
Cohesion: 0.06
Nodes (89): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line() (+81 more)

### Community 43 - "Result"
Cohesion: 0.04
Nodes (125): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_paste_image_inserts_mention_token_and_stores_bytes() (+117 more)

### Community 44 - "Vec"
Cohesion: 0.03
Nodes (127): ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter() (+119 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.06
Nodes (55): Vec, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+47 more)

### Community 47 - "String"
Cohesion: 0.13
Nodes (9): CommandPaletteState, CompilationState, EventLog, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, String (+1 more)

### Community 48 - "LanguageServerSession"
Cohesion: 0.12
Nodes (19): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, normalize_unique_entries() (+11 more)

### Community 49 - "active_shell_buffer_mut"
Cohesion: 0.07
Nodes (83): Cow, yank_to_clipboard_text(), active_shell_buffer_mut(), active_shell_buffer_vim_targets_input(), add_next_multicursor_match(), adjust_tag_child_indent(), apply_block_operator(), apply_directory_edit_queue_if_needed() (+75 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.31
Nodes (14): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color(), resolve_terminal_named_color(), resolve_terminal_plain_color() (+6 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (58): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), contextual_ligature_raster_size_never_upscales_smaller_substitute_glyphs(), directory_view_state_uses_user_oil_defaults() (+50 more)

### Community 52 - "editor-lsp/src/lib.rs"
Cohesion: 0.20
Nodes (28): csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+20 more)

### Community 53 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (9): LanguageServerSpec, LspWorkspaceDiagnostic, Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String (+1 more)

### Community 54 - "AbiOilFeatureSpec"
Cohesion: 0.16
Nodes (10): exported_oil_defaults(), exported_oil_feature_spec(), OilDefaults, OilFeatureSpec, AbiOilDefaults, AbiOilFeatureSpec, OilDefaults, OilFeatureSpec (+2 more)

### Community 55 - "ShellError"
Cohesion: 0.10
Nodes (105): Display, Error, From, ShellError, render_browser_buffer_body(), Color, adjust_color(), blend_color() (+97 more)

### Community 56 - ".len"
Cohesion: 0.05
Nodes (31): ascii_control_caret_notation(), byte_index_for_char_column(), char_at_index(), display_columns_for_character(), exact_match_positions_in_chars(), find_char_forward(), fuzzy_match_end(), input_charwise_motion_range() (+23 more)

### Community 57 - "DapClientError"
Cohesion: 0.10
Nodes (26): Arguments, active_thread_id(), attach_arguments(), clear_stopped_snapshot(), connect_tcp(), connect_transport(), DapClientError, DapClientManager (+18 more)

### Community 58 - ".new"
Cohesion: 0.26
Nodes (13): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items_mark_current_models(), picker_items_preserve_slash_command_labels() (+5 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.10
Nodes (56): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+48 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "String"
Cohesion: 0.05
Nodes (44): active_parameter_label(), completion_level_for_message(), diagnostics_parser_maps_lsp_fields(), documentation_lines(), file_uri_to_path(), language_server_session_in_workspace_scope(), launch_summary(), LspClientState (+36 more)

### Community 62 - ".new"
Cohesion: 0.15
Nodes (20): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_volt_state_dir(), insert_test_session(), redact_key_value_segments(), Arc, PathBuf, Self (+12 more)

### Community 63 - "TextBuffer"
Cohesion: 0.04
Nodes (29): advance_point_by_text(), BufferStats, delimiter_partner(), EditRecord, find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token() (+21 more)

### Community 64 - "PluginCommand"
Cohesion: 0.08
Nodes (18): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+10 more)

### Community 65 - "workspace_dock_layout"
Cohesion: 0.14
Nodes (21): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+13 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.06
Nodes (63): append_query_source(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), compile_query_source(), create_parser() (+55 more)

### Community 67 - "PluginBufferSection"
Cohesion: 0.06
Nodes (17): browser_items(), browser_items_shape_table_rows_from_user_config(), dashboard_sections(), sidebar_sections(), exported_db_browser_items(), DbBrowserContext, DbBrowserItemSpec, DbBrowserKind (+9 more)

### Community 68 - "Option"
Cohesion: 0.11
Nodes (47): apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), background_command_candidates(), background_command_names(), BackgroundCommandPipes, build_background_command() (+39 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.06
Nodes (32): Client, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, DebugAdapterTransport, DebugConfiguration (+24 more)

### Community 70 - ".new"
Cohesion: 0.05
Nodes (50): help_entry(), ContextHelpEntry, hook_command(), package(), hook_command(), hook_command_detail(), package(), package() (+42 more)

### Community 71 - "WorkspaceConfigurationValue"
Cohesion: 0.14
Nodes (14): language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From, I, Number, T, WorkspaceConfigurationValue, K (+6 more)

### Community 72 - "String"
Cohesion: 0.07
Nodes (22): CaptureThemeMapping, command_failure_message(), DeferredQuery, GrammarRecompileFailure, GrammarRecompileReport, GrammarSource, InstallCommandSpec, LanguageConfiguration (+14 more)

### Community 73 - ".send"
Cohesion: 0.10
Nodes (45): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+37 more)

### Community 74 - "DbService"
Cohesion: 0.13
Nodes (15): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbIndex, DbQueryBufferMeta, DbService (+7 more)

### Community 75 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (70): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+62 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.17
Nodes (35): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+27 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (65): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (39): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+31 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "workspace.rs"
Cohesion: 0.13
Nodes (26): PickerWorkspaceContext, existing_workspace_for_project(), file_picker_preview(), message_item(), package(), package_exports_cycle_project_workspace_commands(), package_exports_format_command(), package_exports_mark_list_commands() (+18 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 84 - "Option"
Cohesion: 0.15
Nodes (26): cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), git_line_is_untracked(), git_status_command_name() (+18 more)

### Community 85 - "Option"
Cohesion: 0.11
Nodes (8): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), normalize_optional_string(), AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 86 - "ShellUiState"
Cohesion: 0.04
Nodes (40): active_runtime_buffer(), active_window_id(), apply_lsp_notifications(), BufferViewState, close_runtime_pane(), close_runtime_pane_by_id(), command_builds_user_library(), cycle_runtime_pane() (+32 more)

### Community 87 - ".new"
Cohesion: 0.16
Nodes (22): browser_additional_args(), browser_host_event_for_ipc(), BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, DesktopBrowserHostService (+14 more)

### Community 88 - "wrap_line_segments"
Cohesion: 0.07
Nodes (37): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+29 more)

### Community 89 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 90 - "buffer_footer_layout_with_command_line"
Cohesion: 0.13
Nodes (22): debug_fringe_cell_count(), editor_fringe_width_px(), wrap_columns_for_width(), wrap_columns_for_width_with_fringe(), buffer_cursor_screen_anchor(), buffer_footer_layout_with_command_line(), buffer_point_at_screen(), buffer_visible_headerline_lines() (+14 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.13
Nodes (37): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), bootstrap(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames() (+29 more)

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

### Community 96 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 97 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (46): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+38 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "Self"
Cohesion: 0.07
Nodes (29): DebugAdapterRootStrategy, AbiContextHelpEntry, AbiDebugAdapterRootStrategy, AbiDebugAdapterSpec, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiLanguageServerRootStrategy, AbiOilKeyAction (+21 more)

### Community 101 - ".spawn"
Cohesion: 0.14
Nodes (16): live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), E, Into, IntoIterator, Item, PathBuf (+8 more)

### Community 102 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 103 - "abi.rs"
Cohesion: 0.06
Nodes (41): GitStashEntry, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiAcpClient, AbiDebugAdapterTransport, AbiDebugAdapterTransportKind (+33 more)

### Community 104 - "BufferId"
Cohesion: 0.12
Nodes (26): ActiveBufferEventContext, apply_git_status_snapshot(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_snapshot_for_buffer(), git_status_action_targets(), git_status_delete_target_for_line(), git_status_delete_targets() (+18 more)

### Community 105 - "editor-lsp/src/client.rs"
Cohesion: 0.04
Nodes (111): ClientCapabilities, apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), close_buffer_keeps_session_alive_for_next_file(), code_action_parser_collects_active_file_edits() (+103 more)

### Community 106 - "DapSessionHandle"
Cohesion: 0.09
Nodes (30): capture_stopped_snapshot(), DapExecutionPosition, DapLocalVariable, DapSessionHandle, DapStoppedSnapshot, PendingResponse, record_transport_event(), record_transport_event_inner() (+22 more)

### Community 107 - "main"
Cohesion: 0.14
Nodes (14): command_palette_items(), main(), panic_payload_message(), print_shell_summary(), Any, Box, DebugAdapterSpec, Error (+6 more)

### Community 108 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.13
Nodes (35): ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars() (+27 more)

### Community 110 - "TextEdit"
Cohesion: 0.67
Nodes (4): TextEdit, apply_text_edits_to_span(), text_edit_to_input_edit(), InputEdit

### Community 111 - ".default"
Cohesion: 0.11
Nodes (47): Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title() (+39 more)

### Community 112 - "AbiLanguageConfiguration"
Cohesion: 0.17
Nodes (10): exported_syntax_languages(), AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping (+2 more)

### Community 113 - "RVec"
Cohesion: 0.18
Nodes (10): AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic, RVec (+2 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.10
Nodes (26): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, hook_command(), leader_binding() (+18 more)

### Community 115 - "DbBrowserBufferView"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - "LspCodeAction"
Cohesion: 0.14
Nodes (5): LspCodeAction, LspDocumentTextEdits, LspTextEdit, Error, windows_should_retry_spawn_error()

### Community 117 - "String"
Cohesion: 0.07
Nodes (55): ColumnData, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor (+47 more)

### Community 118 - "LspSessionHandle"
Cohesion: 0.09
Nodes (28): ChildStdin, LspSessionHandle, record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), request_timeout_for_method(), Arc (+20 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "AbiSectionTree"
Cohesion: 0.14
Nodes (12): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiOilSortMode, AbiSectionTree (+4 more)

### Community 121 - "shell/acp.rs"
Cohesion: 0.11
Nodes (31): acp_complete_slash(), acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_pick_mode(), acp_pick_model(), acp_picker_entries(), acp_picker_entry() (+23 more)

### Community 122 - "PixelRect"
Cohesion: 0.26
Nodes (19): PixelRect, rect_tuple(), along_size(), child_rect(), gap_is_inserted_between_siblings(), layout_child(), layout_node(), layout_split_tree() (+11 more)

### Community 123 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (47): browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines(), db_results_table_row_spans() (+39 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.11
Nodes (15): git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, refresh_git_fringe(), refresh_pending_git_summary() (+7 more)

### Community 125 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 126 - "StoredBreakpoint"
Cohesion: 0.16
Nodes (17): BreakpointState, BreakpointStore, BreakpointToggle, delete_removes_current_line_breakpoint(), paths_equal(), BTreeMap, Into, Option (+9 more)

### Community 127 - "shell/tests.rs"
Cohesion: 0.03
Nodes (54): active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), configure_file_buffer(), contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size() (+46 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.11
Nodes (13): push_snapshot_line(), push_terminal_render_run(), Option, Vec, terminal_render_snapshot(), terminal_render_snapshot_preserves_wide_character_widths(), terminal_render_snapshot_tracks_visible_cursor(), terminal_snapshot_lines() (+5 more)

### Community 130 - "user/config.rs"
Cohesion: 0.15
Nodes (26): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+18 more)

### Community 131 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.21
Nodes (23): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+15 more)

### Community 133 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 134 - ".get"
Cohesion: 0.17
Nodes (19): column_is_numeric(), DbAutocompleteCandidate, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns() (+11 more)

### Community 135 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "build_job_command"
Cohesion: 0.29
Nodes (9): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, run_job(), windows_fnm_environment(), configure_background_command() (+1 more)

### Community 138 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 139 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbSnippet, load_persisted_state(), PersistedDbState, RememberedConnection, Path

### Community 140 - "String"
Cohesion: 0.07
Nodes (54): acp_connected(), acp_image_mention_token(), acp_insert_file_mention(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_picker_closed(), acp_permission_picker_submitted(), acp_resolve_permission_option() (+46 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "From"
Cohesion: 0.06
Nodes (35): GitCommandBinding, GitPrefixBinding, exported_statusline_render(), statusline_context_from_abi(), AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextLine, AbiGitCommandBinding (+27 more)

### Community 143 - "AbiPaneConfig"
Cohesion: 0.06
Nodes (20): exported_pane_config(), MarkdownPrettyConfig, PickerLayout, ShowParenConfig, config(), AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig (+12 more)

### Community 144 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 145 - "editor-dap/src/client.rs"
Cohesion: 0.13
Nodes (26): client_initialize_launch_disconnect_against_fake_tcp_adapter(), continue_step_pause_and_locals_against_fake_adapter(), DapLogDirection, DapLogEntry, DapLogSnapshot, DapSessionInfo, DapTransportLog, debug_stop_after_attach_leaves_process_running() (+18 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 148 - "headerline_lines"
Cohesion: 0.29
Nodes (7): build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 149 - "TextRange"
Cohesion: 0.08
Nodes (18): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), location_from_link(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+10 more)

### Community 150 - "browser_host.rs"
Cohesion: 0.12
Nodes (14): allow_browser_drag_drop(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests(), browser_navigation_retry_required() (+6 more)

### Community 151 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 153 - "user/db.rs"
Cohesion: 0.15
Nodes (22): browser_item(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings(), default_action(), engine_icon() (+14 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "browser_sync_plan"
Cohesion: 0.24
Nodes (14): BrowserSurfacePlan, BrowserSyncPlan, BrowserViewportRect, browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect() (+6 more)

### Community 156 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 157 - "PickerItemSpec"
Cohesion: 0.08
Nodes (38): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+30 more)

### Community 158 - "load"
Cohesion: 0.17
Nodes (23): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+15 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "find_font_by_name"
Cohesion: 0.26
Nodes (15): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), pick_best_matching_font_path(), preferred_berkeley_mono_font(), preferred_berkeley_mono_font_candidates(), preferred_font_search_roots(), RenderError (+7 more)

### Community 161 - "AbiContextHelpSpec"
Cohesion: 0.08
Nodes (22): exported_browser_feature_spec(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_terminal_feature_spec(), BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec (+14 more)

### Community 162 - "TerminalTranscript"
Cohesion: 0.19
Nodes (5): append_lines(), TerminalLine, TerminalSession, TerminalStream, TerminalTranscript

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
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.24
Nodes (10): exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package(), package_exports_terminal_commands_and_binding() (+2 more)

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
Cohesion: 0.20
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

### Community 179 - "Option"
Cohesion: 0.07
Nodes (48): BufRead, char_to_byte_offset(), completion_documentation(), configuration_item_section(), CopilotDeviceCodePrompt, csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params() (+40 more)

### Community 180 - "aligned_indent_column"
Cohesion: 0.12
Nodes (24): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after(), general_predicates_match(), indent_begin_applies(), line_intersects_node() (+16 more)

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
Cohesion: 0.36
Nodes (7): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), Vec, WorkspaceRootConfig, WorkspaceSection

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

### Community 190 - "flatten_config_select_options"
Cohesion: 0.27
Nodes (10): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+2 more)

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
Nodes (67): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings(), is_object_separator() (+59 more)

### Community 195 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 196 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 197 - "AcpManager"
Cohesion: 0.13
Nodes (23): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_pick_session(), acp_set_mode(), AcpManager (+15 more)

### Community 198 - "UserLibraryModule"
Cohesion: 0.23
Nodes (5): exported_acp_picker_items(), UserLibraryModule, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind

### Community 199 - "TerminalDimensions"
Cohesion: 0.25
Nodes (4): From, TerminalDimensions, WindowSize, Dimensions

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "TerminalCursorSnapshot"
Cohesion: 0.25
Nodes (4): map_terminal_cursor_shape(), TerminalCursorShape, TerminalCursorSnapshot, CursorShape

### Community 202 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 203 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 204 - ".byte_slice_chunks"
Cohesion: 0.22
Nodes (7): Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 205 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 206 - "configure_background_command"
Cohesion: 0.28
Nodes (8): configure_background_command(), Box, Command, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 207 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 209 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 210 - "AbiKeymapConfig"
Cohesion: 0.32
Nodes (5): exported_keymap_config(), KeymapConfig, AbiKeymapConfig, KeymapConfig, KeymapConfig

### Community 211 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 212 - "Vec"
Cohesion: 0.10
Nodes (9): DapState, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, Vec (+1 more)

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 215 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "AbiLigatureConfig"
Cohesion: 0.32
Nodes (5): exported_ligature_config(), LigatureConfig, AbiLigatureConfig, LigatureConfig, LigatureConfig

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "AbiIconFontSymbol"
Cohesion: 0.14
Nodes (12): exported_oil_keybindings(), OilKeybindings, AbiIconFontCategory, AbiIconFontSymbol, AbiOilKeybindings, IconFontCategory, IconFontSymbol, OilKeybindings (+4 more)

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

### Community 242 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 247 - "connect_sql_server"
Cohesion: 0.50
Nodes (4): Compat, connect_sql_server(), TcpStream, SqlServerClient

### Community 248 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 250 - "ShellConfig"
Cohesion: 0.16
Nodes (12): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+4 more)

### Community 251 - "ProjectCandidate"
Cohesion: 0.23
Nodes (5): compact_project_path(), ProjectCandidate, ProjectKind, String, worktree_parent_name()

### Community 252 - "AbiTheme"
Cohesion: 0.23
Nodes (8): exported_themes(), AbiColor, AbiTheme, AbiThemeToken, Color, Color, Theme, Theme

### Community 253 - ".new"
Cohesion: 0.22
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 254 - "dap.rs"
Cohesion: 0.27
Nodes (10): adapter_preferences_match_language_defaults(), debug_adapters(), locals_buffer_declares_locals_and_expressions_sections(), locals_sections(), package(), package_exports_debug_layout_buffers(), package_exports_start_family_commands(), package_exports_stepping_and_restart_commands() (+2 more)

### Community 258 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 259 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 264 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 265 - "WorkspaceDockConfig"
Cohesion: 0.10
Nodes (12): WorkspaceDockTestUserLibrary, KeymapConfig, OilDefaults, OilFeatureSpec, OilKeybindings, OilSortMode, PickerLayout, Default (+4 more)

## Knowledge Gaps
- **152 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+147 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **29 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `PathBuf`, `String`, `ShellState`, `key_sequence.rs`, `Option`, `shell/browser.rs`, `AcpEvent`, `String`, `shell/git.rs`, `resolve_permission`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `command_stream.rs`, `String`, `.new`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `shell_buffer_mut`, `Vec`, `active_shell_buffer_mut`, `active_runtime_popup`, `Option`, `AcpManager`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `Option`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `BufferId`, `main`, `shell/picker.rs`, `shell/acp.rs`, `GitSummaryState`, `shell/tests.rs`?**
  _High betweenness centrality (0.125) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `PathBuf` to `ShellState`, `user/lib.rs`, `Option`, `shell/browser.rs`, `render.rs`, `WorkspaceDockConfig`, `DynamicUserLibrary`, `browser_sync_plan`, `HoverOverlay`, `shell/mod.rs`, `ShellBuffer`, `shell_buffer_mut`, `Vec`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `ShellError`, `workspace_dock_layout`, `directory.rs`, `load_user_library`, `volt/src/main.rs`, `DynamicUserLibrary`, `Option`, `ShellUiState`, `buffer_footer_layout_with_command_line`, `editor-plugin-host/src/lib.rs`, `shell/picker.rs`, `ShellConfig`?**
  _High betweenness centrality (0.051) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `PathBuf`, `Option`, `ShellState`, `Option`, `shell/browser.rs`, `render.rs`, `browser_sync_plan`, `.new`, `shell/pdf.rs`, `shell/mod.rs`, `shell_buffer_mut`, `Result`, `Vec`, `active_shell_buffer_mut`, `ShellError`, `.len`, `TextBuffer`, `directory.rs`, `shell/terminal.rs`, `ShellUiState`, `wrap_line_segments`, `buffer_footer_layout_with_command_line`, `BufferId`, `shell/picker.rs`, `shell/acp.rs`, `LineSyntaxSpan`, `GitSummaryState`, `StoredBreakpoint`, `shell/tests.rs`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _152 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `PathBuf` be split into smaller, more focused modules?**
  _Cohesion score 0.02211420330110571 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.094045865768988 - nodes in this community are weakly interconnected._
- **Should `String` be split into smaller, more focused modules?**
  _Cohesion score 0.051201671891327065 - nodes in this community are weakly interconnected._