# Graph Report - volt  (2026-08-17)

## Corpus Check
- 234 files · ~630,340 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9422 nodes · 38479 edges · 311 communities (281 shown, 30 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3199 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `08df3fe0`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Option
- Path
- shell/tests.rs
- src/tests.rs
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- ShellUiState
- shell/browser.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Path
- PickerSession
- LiveTerminalSession
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
- render_text_with_fonts
- EditorRuntime
- WorkspaceDockConfig
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- ShellBuffer
- Result
- String
- AbiContextHelpSpec
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- buffer_cursor_screen_anchor
- WorkspaceDockBranchCache
- state.rs
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- LanguageServerSpec
- Path
- BufferId
- render_buffer_with_view_state
- TextPoint
- PluginBuffer
- AcpPickerItemSpec
- active_runtime_popup
- editor-lsp/src/lib.rs
- shell_ui_mut
- .new
- LspNotification
- PluginCommand
- From
- SyntaxRegistry
- .new
- shell/acp.rs
- DebugConfiguration
- .new
- Option
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
- AbiGitStatusSnapshot
- Option
- build_job_command
- AbiPaneConfig
- draw_diagnostic_underlines_for_segment
- browser_host.rs
- .new
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- WorkspaceConfigurationValue
- resolve_picker_extra
- GitEditorState
- ModelineSegment
- editor-picker/src/lib.rs
- RString
- client.rs
- .from
- Vec
- LspCodeAction
- main
- RVec
- Self
- shell/picker.rs
- PickerItem
- .default
- .new
- ShellConfig
- PluginKeyBinding
- .get
- .spawn
- editor-db/src/lib.rs
- .next_token
- process_supervisor.rs
- AbiLanguageConfiguration
- TextRange
- Self
- shell/git.rs
- Option
- TerminalTranscript
- DynamicUserLibrary
- .new
- JobError
- Option
- user/config.rs
- WorkspaceConfiguration
- key_sequence.rs
- treesittercontext_ghosttext.rs
- String
- common.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- text_document_content_change
- AbiAutocompleteProvider
- debug_adapters
- String
- CommandLineOverlay
- corpus_inventory.rs
- Option
- JobSpec
- AbiSectionTree
- volt/build.rs
- Default
- bash.rs
- String
- clojure.rs
- syntax_languages
- oil.rs
- db.rs
- lsp.rs
- graphql.rs
- hcl.rs
- AbiGitFeatureSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- java.rs
- flatten_config_select_options
- elixir.rs
- theme.rs
- ServiceRegistry
- AbiSectionItem
- String
- user/terminal.rs
- build_output.rs
- kotlin.rs
- latex.rs
- lua.rs
- nix.rs
- JobResult
- perl.rs
- user/browser.rs
- php.rs
- .oil_keybindings
- `user`
- proto.rs
- aligned_indent_column
- r.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- TerminalCursorSnapshot
- ruby.rs
- scala.rs
- .oil_directory_sections
- configure_file_buffer
- solidity.rs
- Database Explorer PRD
- .from
- TextBuffer
- swift.rs
- UserLibraryModule
- AcpManager
- lang/vim.rs
- .oil_directory_sections
- markdown.rs
- .oil_directory_sections
- LspFormattingOptions
- xml.rs
- syntax_language
- AbiDebugAdapterSpec
- AbiKeymapConfig
- .db_feature_spec
- 0004-markdown-pretty-pipeline.md
- AbiLanguageServerRootStrategy
- AbiLspDiagnosticsInfo
- panic_payload_message
- .keymap_config
- VimActionContext
- AbiOilKeyAction
- AbiTerminalFeatureSpec
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
- package
- .context_help_specs
- .debug_adapters
- .ghost_text_lines
- .git_feature_spec
- .hover_providers
- user/workspace_dock.rs
- package
- .ligature_config
- LspLogEntry
- .oil_feature_spec
- main
- .oil_keybindings
- load
- .picker_layout
- Vec
- .picker_truncate_strategy
- .terminal_config
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- Agent skills
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
8. `shell_buffer_mut()` - 179 edges
9. `ShellUiState` - 174 edges
10. `TextBuffer` - 168 edges

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

## Communities (311 total, 30 thin omitted)

### Community 0 - "Option"
Cohesion: 0.02
Nodes (108): active_buffer_revision_key(), ActiveTypingFrameProfile, closing_tag_name_after_cursor(), comment_style_for_buffer(), comment_style_for_language_path(), CommentStyle, current_theme_source_fingerprint(), current_user_config_source_fingerprint() (+100 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (28): inline_completion_params(), is_copilot_server(), LspClientError, LspClientManager, LspSessionHandle, parse_text_edit_response(), path_to_uri(), request_timeout_for_method() (+20 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.02
Nodes (111): cycle_hover_provider(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_output_speaker_roles_and_tool_chip(), acp_plan_entries_normalize_completed_prefix_when_later_step_is_active(), acp_plan_entries_normalize_completed_prefix_without_active_step(), acp_plan_entries_populate_static_plan_pane() (+103 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.14
Nodes (63): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+55 more)

### Community 4 - "ShellError"
Cohesion: 0.05
Nodes (49): Display, Error, From, ShellError, clear_key_sequence(), active_buffer_event_context(), active_lsp_workspace_loaded(), active_runtime_surface() (+41 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.02
Nodes (122): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+114 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.09
Nodes (80): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names(), vim_search_entries_trim_whitespace_from_labels(), additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root() (+72 more)

### Community 7 - "ShellUiState"
Cohesion: 0.04
Nodes (41): active_lsp_code_action_range(), active_runtime_buffer(), active_shell_workspace_id(), buffer_is_oil_preview(), BufferViewState, close_popup_buffer_and_restore_focus(), command_builds_user_library(), create_db_query_buffer() (+33 more)

### Community 8 - "shell/browser.rs"
Cohesion: 0.06
Nodes (65): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_buffer_layout(), browser_display_url(), browser_host_viewport_rect(), browser_surface_buffer_at_point(), browser_sync_plan() (+57 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (119): acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), acp_buffer_layout(), acp_chat_bubble_width_px(), acp_chat_corner_radius(), acp_chat_origin_x(), acp_chat_rounded(), acp_pane_body_visible_rows() (+111 more)

### Community 10 - "AcpEvent"
Cohesion: 0.09
Nodes (30): AvailableCommand, acp_pick_mode(), AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome() (+22 more)

### Community 11 - "PluginPackage"
Cohesion: 0.05
Nodes (48): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+40 more)

### Community 12 - "Self"
Cohesion: 0.07
Nodes (16): browser_item(), default_action(), AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserContext, DbBrowserItemContext, DbBrowserItemKind (+8 more)

### Community 13 - "Path"
Cohesion: 0.07
Nodes (57): parse_log_oneline(), begin_oil_worktree_request(), build_git_fringe_snapshot(), command_output_transcript(), create_git_worktree_from_query(), fetch_git_prune(), git_branch_list(), git_branch_merge() (+49 more)

### Community 14 - "PickerSession"
Cohesion: 0.14
Nodes (7): PickerResultOrder, PickerSession, Self, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 15 - "LiveTerminalSession"
Cohesion: 0.06
Nodes (30): AlacrittyEvent, Keycode, Mod, Self, terminal_key_for_event(), terminal_scroll_for_motion(), LiveTerminalError, LiveTerminalSession (+22 more)

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
Nodes (16): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig, MarkdownPrettyConfig (+8 more)

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

### Community 24 - "DbService"
Cohesion: 0.14
Nodes (17): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbService, DbSession (+9 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.05
Nodes (96): ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), switch_runtime_workspace(), acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer() (+88 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.09
Nodes (48): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+40 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (31): AutocompleteProviderKind, RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay, HoverProviderContent (+23 more)

### Community 30 - "Theme"
Cohesion: 0.08
Nodes (25): text_style_from_theme_style(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display (+17 more)

### Community 31 - "render_text_with_fonts"
Cohesion: 0.07
Nodes (48): Canvas, RenderColor, Self, TextStyle, alpha_bitmap_surface(), cached_primary_text_runs(), CachedLigatureLayout, compose_emoji_surface() (+40 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.07
Nodes (130): EditorRuntime, Default, run_command(), active_git_status_command_context(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit() (+122 more)

### Community 33 - "WorkspaceDockConfig"
Cohesion: 0.06
Nodes (13): WorkspaceDockTestUserLibrary, exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, MarkdownPrettyConfig, PickerLayout (+5 more)

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
Nodes (305): Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), activate_db_browser_line(), active_directory_root(), active_project_workspace_root(), active_shell_buffer_has_input() (+297 more)

### Community 40 - "Self"
Cohesion: 0.12
Nodes (16): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_picker_truncate_strategy(), default_rainbow_parens_enabled() (+8 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.02
Nodes (54): acp_output_header_title(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_rendered_text_segments() (+46 more)

### Community 42 - "Result"
Cohesion: 0.08
Nodes (90): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line() (+82 more)

### Community 43 - "String"
Cohesion: 0.07
Nodes (81): load_font_set(), buffer_footer_layout(), acp_agent_markdown_uses_shared_pipeline_pretty(), acp_section_layout_orders_output_input_footer_and_statusline(), browser_input_layout_uses_symmetric_vertical_padding(), codicon_glyphs_fit_inside_one_editor_cell(), command_line_footer_layout_reserves_row_below_statusline(), compose_emoji_surface_rasterizes_simple_emoji() (+73 more)

### Community 44 - "AbiContextHelpSpec"
Cohesion: 0.14
Nodes (12): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+4 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (62): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec, decode_modeline() (+54 more)

### Community 47 - "buffer_cursor_screen_anchor"
Cohesion: 0.12
Nodes (23): LineCharMap, LineWrapSegment, resolved_tab_width(), wrap_columns_for_width(), wrap_line_segments_for_line(), block_cursor_text_overlay(), buffer_cursor_screen_anchor(), buffer_point_at_screen() (+15 more)

### Community 48 - "WorkspaceDockBranchCache"
Cohesion: 0.13
Nodes (19): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+11 more)

### Community 49 - "state.rs"
Cohesion: 0.07
Nodes (40): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+32 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.21
Nodes (21): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+13 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (30): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), HeaderlineTestUserLibrary, AcpClient, Arc, AutocompleteProvider, DebugAdapterSpec (+22 more)

### Community 52 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (10): LanguageServerRootStrategy, LanguageServerSpec, LspWorkspaceDiagnostic, normalize_unique_entries(), Into, IntoIterator, Item, LanguageServerRootStrategy (+2 more)

### Community 53 - "Path"
Cohesion: 0.06
Nodes (61): acp_decode_image(), active_lsp_buffer_context(), apply_lsp_text_edits(), apply_sqls_workspace_settings_for_active_buffer_context(), begin_copilot_sign_in(), cancel_lsp_sync_for_path(), cleanup_formatter_temp(), clear_lsp_ui_for_stopped_paths() (+53 more)

### Community 54 - "BufferId"
Cohesion: 0.11
Nodes (27): ActiveBufferEventContext, apply_git_status_snapshot(), cancel_git_commit_buffer(), commit_git_buffer(), finish_oil_worktree_branch_selection(), git_command_output_owned(), git_commit_message(), git_commit_temp_path() (+19 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.12
Nodes (87): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+79 more)

### Community 56 - "TextPoint"
Cohesion: 0.03
Nodes (76): TextPoint, TextSnapshot, advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_input_operator_motion(), apply_markdown_table_update(), ascii_control_caret_notation(), char_at_index() (+68 more)

### Community 57 - "PluginBuffer"
Cohesion: 0.10
Nodes (4): PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate

### Community 58 - "AcpPickerItemSpec"
Cohesion: 0.14
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (55): active_runtime_popup(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items(), git_status_ctrl_v_visual_u_unstages_selected_items() (+47 more)

### Community 60 - "editor-lsp/src/lib.rs"
Cohesion: 0.22
Nodes (27): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+19 more)

### Community 61 - "shell_ui_mut"
Cohesion: 0.07
Nodes (54): cycle_runtime_pane(), shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_backspace_can_delete_leading_slash() (+46 more)

### Community 62 - ".new"
Cohesion: 0.12
Nodes (26): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, default_db_browser_items(), execute_postgres(), execute_sql_server() (+18 more)

### Community 63 - "LspNotification"
Cohesion: 0.06
Nodes (30): ChildStdin, ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry (+22 more)

### Community 64 - "PluginCommand"
Cohesion: 0.09
Nodes (21): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+13 more)

### Community 65 - "From"
Cohesion: 0.13
Nodes (16): AbiGhostTextLine, AbiIconFontCategory, AbiLigatureConfig, AbiPdfOpenMode, AbiWorkspaceRoot, GhostTextLine, IconFontCategory, LigatureConfig (+8 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.07
Nodes (51): buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), compile_query_source(), create_parser(), desired_indent_for_loaded_language() (+43 more)

### Community 67 - ".new"
Cohesion: 0.05
Nodes (37): AsyncRead, acp_pick_model(), acp_picker_entries(), acp_picker_entry(), acp_resolve_permission_option(), AcpClient, coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks() (+29 more)

### Community 68 - "shell/acp.rs"
Cohesion: 0.10
Nodes (57): acp_slash_completion_query(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+49 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - ".new"
Cohesion: 0.05
Nodes (53): path_to_file_url_encodes_spaces(), feature_spec(), DbFeatureSpec, help_entry(), ContextHelpEntry, hook_command(), package(), hook_command() (+45 more)

### Community 71 - "Option"
Cohesion: 0.04
Nodes (47): append_query_source(), asset_path_from_parts(), CaptureThemeMapping, command_failure_message(), default_install_root(), default_query_asset_root(), DeferredQuery, ensure_cloned_grammar_dir_exists() (+39 more)

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
Cohesion: 0.06
Nodes (62): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+54 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (37): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+29 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.07
Nodes (73): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+65 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.15
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.06
Nodes (66): workspace_picker_item(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+58 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.16
Nodes (19): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), format_micros_as_millis(), LaunchMode, LaunchOptions, LspState, parse_launch_options(), parse_launch_options_accepts_fps_overlay() (+11 more)

### Community 83 - ".new"
Cohesion: 0.09
Nodes (41): diff_git_commit_at_point(), diff_git_dwim(), diff_git_stash_at_point(), git_action_detail(), git_args_with_no_pager(), git_log_args(), git_status_cherry_open_command(), git_status_diff_commit_command() (+33 more)

### Community 84 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 85 - "Option"
Cohesion: 0.10
Nodes (20): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, LspError, normalize_optional_string() (+12 more)

### Community 86 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 87 - "AbiPaneConfig"
Cohesion: 0.14
Nodes (12): AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig, AbiPickerLayout, AbiWorkspaceDockSide, fraction_to_hundredths(), MarkdownPrettyConfig, PickerLayout (+4 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.15
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 90 - ".new"
Cohesion: 0.16
Nodes (14): dynamic_user_library_can_wrap_exported_module(), load_user_library(), Arc, Instant, PathBuf, Self, StartupTrace, user_library_candidates() (+6 more)

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
Cohesion: 0.16
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (15): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, I (+7 more)

### Community 97 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "ModelineSegment"
Cohesion: 0.17
Nodes (25): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+17 more)

### Community 100 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 101 - "RString"
Cohesion: 0.13
Nodes (14): AbiColor, AbiStringPair, AbiTheme, AbiThemeOption, AbiThemeOptionEntry, AbiThemeToken, Color, Color (+6 more)

### Community 102 - "client.rs"
Cohesion: 0.05
Nodes (113): BufRead, char_to_byte_offset(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_documentation(), completion_level_for_message(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range() (+105 more)

### Community 103 - ".from"
Cohesion: 0.14
Nodes (19): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiFiniteF64, AbiLanguageServerSpec, AbiWorkspaceConfiguration (+11 more)

### Community 104 - "Vec"
Cohesion: 0.12
Nodes (8): EventLog, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, String, Vec, WorkspaceRoot

### Community 105 - "LspCodeAction"
Cohesion: 0.14
Nodes (5): LspCodeAction, LspDocumentTextEdits, LspTextEdit, Error, windows_should_retry_spawn_error()

### Community 106 - "main"
Cohesion: 0.13
Nodes (17): bootstrap(), HostBootstrap, command_palette_items(), main(), DebugAdapterSpec, Error, LanguageConfiguration, LanguageServerSpec (+9 more)

### Community 107 - "RVec"
Cohesion: 0.18
Nodes (10): AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic, RVec (+2 more)

### Community 108 - "Self"
Cohesion: 0.13
Nodes (8): Iterator, Range, Self, Selection, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 109 - "shell/picker.rs"
Cohesion: 0.12
Nodes (36): buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay(), picker_overlay_from_spec(), picker_preview_is_opt_in() (+28 more)

### Community 110 - "PickerItem"
Cohesion: 0.18
Nodes (7): match_item(), PickerItem, PickerMatch, Into, Option, String, picker_fringe_width_chars()

### Community 111 - ".default"
Cohesion: 0.10
Nodes (49): Self, browser_display_url_prefers_requested_navigation(), Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec() (+41 more)

### Community 112 - ".new"
Cohesion: 0.04
Nodes (50): BufferKind, browser_state_for_kind(), default_vim_target(), absolute_path_hint(), append_error_log(), buffer_interaction(), buffer_is_browser(), buffer_is_quickfix() (+42 more)

### Community 113 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - ".spawn"
Cohesion: 0.14
Nodes (17): live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator, Item (+9 more)

### Community 117 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (33): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn, DbIndex, DbSchemaCache, DbTable, default_db_browser_line() (+25 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 121 - "TextRange"
Cohesion: 0.07
Nodes (17): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), lsp_range_from_text_range() (+9 more)

### Community 122 - "Self"
Cohesion: 0.11
Nodes (20): exported_oil_directory_sections(), AbiDirectoryEntry, AbiDirectoryEntryKind, AbiOilDefaults, AbiOilFeatureSpec, AbiOilSortMode, AbiPickerTruncateStrategy, DirectoryEntry (+12 more)

### Community 123 - "shell/git.rs"
Cohesion: 0.10
Nodes (51): apply_git_view(), find_paren_number_range(), format_section_line(), git_line_is_untracked(), git_status_checkout_file_command(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_delete_target_for_line() (+43 more)

### Community 124 - "Option"
Cohesion: 0.07
Nodes (36): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_command_output_background(), git_repository_present(), git_status_command_name(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot (+28 more)

### Community 125 - "TerminalTranscript"
Cohesion: 0.17
Nodes (5): append_lines(), TerminalLine, TerminalSession, TerminalStream, TerminalTranscript

### Community 127 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.13
Nodes (7): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "treesittercontext_ghosttext.rs"
Cohesion: 0.07
Nodes (43): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+35 more)

### Community 134 - "String"
Cohesion: 0.14
Nodes (17): db_browser_action_from_spec(), DisabledSecretStore, initialize_native_keyring(), InMemorySecretStore, load_postgres_schema(), OsSecretStore, qualified_name_from_spec(), redact_error() (+9 more)

### Community 135 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "text_document_content_change"
Cohesion: 0.22
Nodes (8): full_sync_uses_null_range_change(), incremental_sync_uses_full_document_replacement_range(), text_document_content_change(), text_document_sync_kind(), InitializeResult, TextDocumentContentChangeEvent, TextDocumentSyncCapability, TextDocumentSyncKind

### Community 138 - "AbiAutocompleteProvider"
Cohesion: 0.29
Nodes (6): AbiAutocompleteProvider, AbiAutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem

### Community 139 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 140 - "String"
Cohesion: 0.09
Nodes (45): acp_complete_slash(), acp_connected(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny(), acp_permission_picker_closed(), acp_permission_picker_submitted() (+37 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "Option"
Cohesion: 0.15
Nodes (7): CommandPaletteState, CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 144 - "JobSpec"
Cohesion: 0.20
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "AbiSectionTree"
Cohesion: 0.25
Nodes (7): exported_git_status_sections(), AbiSection, AbiSectionTree, Section, SectionTree, Section, SectionTree

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "Default"
Cohesion: 0.29
Nodes (8): default_pane_golden_ratio(), default_workspace_dock_docked(), KeymapSection, PaneSection, Default, TerminalSection, UiSection, WorkspaceDockSection

### Community 148 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 149 - "String"
Cohesion: 0.05
Nodes (56): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, documentation_lines(), explicit_windows_env_value() (+48 more)

### Community 150 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 151 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 152 - "oil.rs"
Cohesion: 0.05
Nodes (52): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+44 more)

### Community 153 - "db.rs"
Cohesion: 0.22
Nodes (13): browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord(), query_buffer_lines() (+5 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 156 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 157 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 161 - "flatten_config_select_options"
Cohesion: 0.31
Nodes (9): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+1 more)

### Community 162 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 163 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "AbiSectionItem"
Cohesion: 0.29
Nodes (6): AbiSectionAction, AbiSectionItem, SectionAction, SectionItem, SectionAction, SectionItem

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 169 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 170 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 171 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 172 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

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

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 180 - "aligned_indent_column"
Cohesion: 0.12
Nodes (24): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after(), general_predicates_match(), indent_begin_applies(), line_intersects_node() (+16 more)

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
Cohesion: 0.36
Nodes (7): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), Vec, WorkspaceRootConfig, WorkspaceSection

### Community 186 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 187 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 188 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 189 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 190 - "configure_file_buffer"
Cohesion: 0.52
Nodes (7): active_and_secondary_buffer_ids(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), record_file_reload_event(), wait_for_file_reload_worker()

### Community 191 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - ".from"
Cohesion: 0.14
Nodes (20): close_buffer_keeps_session_alive_for_next_file(), default_workspace_lists_only_sessions_serving_open_buffers(), file_uri_roundtrip_handles_windows_paths(), live_session_picker_label_includes_server_and_root(), live_sessions_for_workspace_includes_root_scoped_and_buffer_served(), path_to_file_uri(), BTreeMap, session_in_scope_when_open_buffer_is_tracked() (+12 more)

### Community 194 - "TextBuffer"
Cohesion: 0.03
Nodes (72): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending() (+64 more)

### Community 195 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 196 - "UserLibraryModule"
Cohesion: 0.14
Nodes (14): AbiAcpClient, AbiGhostTextContext, AbiIconFontSymbol, AbiOilKeybindings, AbiStatuslineContext, AcpClient, IconFontSymbol, OilKeybindings (+6 more)

### Community 197 - "AcpManager"
Cohesion: 0.13
Nodes (25): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_set_mode(), acp_set_model(), AcpManager (+17 more)

### Community 198 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 199 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 202 - "LspFormattingOptions"
Cohesion: 0.47
Nodes (3): lsp_formatting_options(), LspFormattingOptions, FormattingOptions

### Community 203 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 204 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 205 - "AbiDebugAdapterSpec"
Cohesion: 0.60
Nodes (3): AbiDebugAdapterSpec, DebugAdapterSpec, DebugAdapterSpec

### Community 206 - "AbiKeymapConfig"
Cohesion: 0.60
Nodes (3): AbiKeymapConfig, KeymapConfig, KeymapConfig

### Community 209 - "AbiLanguageServerRootStrategy"
Cohesion: 0.60
Nodes (3): AbiLanguageServerRootStrategy, LanguageServerRootStrategy, LanguageServerRootStrategy

### Community 210 - "AbiLspDiagnosticsInfo"
Cohesion: 0.60
Nodes (3): AbiLspDiagnosticsInfo, LspDiagnosticsInfo, LspDiagnosticsInfo

### Community 211 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 214 - "AbiOilKeyAction"
Cohesion: 0.60
Nodes (3): AbiOilKeyAction, OilKeyAction, OilKeyAction

### Community 215 - "AbiTerminalFeatureSpec"
Cohesion: 0.60
Nodes (3): AbiTerminalFeatureSpec, TerminalFeatureSpec, TerminalFeatureSpec

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
Cohesion: 0.13
Nodes (25): apply_rainbow_delimiter_spans(), bracket_tokens(), BracketSpan, delimiter_family(), delimiter_text(), DelimiterFamily, depth_face_index(), depth_theme_token() (+17 more)

### Community 227 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 233 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 234 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 236 - "LspLogEntry"
Cohesion: 0.17
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 240 - "load"
Cohesion: 0.24
Nodes (6): load(), UserConfig, config(), KeymapConfig, config(), LigatureConfig

### Community 242 - "Vec"
Cohesion: 0.02
Nodes (165): ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter() (+157 more)

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **141 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+136 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `Option`, `shell/tests.rs`, `ShellError`, `ShellUiState`, `shell/browser.rs`, `AcpEvent`, `String`, `Path`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `command_stream.rs`, `shell/pdf.rs`, `ServiceRegistry`, `shell/mod.rs`, `ShellBuffer`, `Result`, `Path`, `BufferId`, `TextPoint`, `active_runtime_popup`, `shell_ui_mut`, `configure_file_buffer`, `.new`, `shell/acp.rs`, `AcpManager`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `.new`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `main`, `shell/picker.rs`, `.new`, `Vec`, `shell/git.rs`, `Option`?**
  _High betweenness centrality (0.102) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `.new` to `Option`, `ShellError`, `user/lib.rs`, `ShellUiState`, `shell/browser.rs`, `render.rs`, `DynamicUserLibrary`, `HoverOverlay`, `WorkspaceDockConfig`, `shell/mod.rs`, `ShellBuffer`, `Result`, `String`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `buffer_cursor_screen_anchor`, `HeaderlineTestUserLibrary`, `render_buffer_with_view_state`, `directory.rs`, `volt/src/main.rs`, `.new`, `editor-plugin-host/src/lib.rs`, `ShellConfig`, `Vec`, `Option`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `Option`, `Option`, `ShellError`, `ShellUiState`, `shell/browser.rs`, `render.rs`, `shell/pdf.rs`, `shell/mod.rs`, `Result`, `String`, `buffer_cursor_screen_anchor`, `state.rs`, `Path`, `BufferId`, `render_buffer_with_view_state`, `TextPoint`, `TextBuffer`, `shell/acp.rs`, `directory.rs`, `shell/terminal.rs`, `draw_diagnostic_underlines_for_segment`, `shell/picker.rs`, `.new`, `Vec`, `shell/git.rs`, `Option`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _141 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.019998984823105425 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.08290638099673964 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.023292836196062004 - nodes in this community are weakly interconnected._