# Graph Report - volt  (2026-08-18)

## Corpus Check
- 235 files · ~593,722 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9539 nodes · 38922 edges · 288 communities (258 shown, 30 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3229 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `4cbbcc53`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- FontSet
- Path
- shell/tests.rs
- .new
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- PickerOverlay
- browser_host.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Path
- String
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- String
- shell_ui_mut
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- render_text_with_fonts
- EditorRuntime
- String
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- ShellBuffer
- Result
- .new
- shell/acp.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- Option
- workspace_dock_layout
- AbiContextHelpSpec
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- editor-lsp/src/lib.rs
- LanguageServerSpec
- BufferId
- ShellError
- .is_empty
- shell/git.rs
- picker_items
- BufferId
- build_output.rs
- LspNotification
- .new_with_secret_store
- TextBuffer
- PluginCommand
- Option
- SyntaxRegistry
- spawn_terminal_reader
- Option
- DebugConfiguration
- capture_mappings
- GrammarRecompileReport
- String
- .send
- DbService
- clipboard.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- volt/src/main.rs
- Option
- .next_token
- LanguageServerRegistry
- ShellUiState
- AbiPaneConfig
- LineCharMap
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
- .spawn
- AbiSectionTree
- From
- Vec
- TextRange
- main
- editor-path/src/lib.rs
- JobSpec
- shell/picker.rs
- AbiGitFeatureSpec
- .default
- editor-picker/src/lib.rs
- PickerSession
- PluginKeyBinding
- .get
- TerminalTranscript
- editor-db/src/lib.rs
- PickerItem
- process_supervisor.rs
- RVec
- UserLibrary
- Self
- Vec
- GitSummaryState
- String
- DynamicUserLibrary
- Vec
- JobError
- Option
- user/config.rs
- UserLibraryModule
- key_sequence.rs
- treesittercontext_ghosttext.rs
- .new
- cmake.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- DbEngine
- build_job_command
- load_user_library
- String
- CommandLineOverlay
- OilDefaultsSection
- AbiLigatureConfig
- resolve_permission
- BufferKind
- volt/build.rs
- ShellConfig
- normalize_unique_entries
- String
- cargo
- GhostTextContext
- oil.rs
- db.rs
- lsp.rs
- compute_buffer_syntax
- .from
- latex.rs
- load
- Copilot instructions for `volt`
- TerminalCursorSnapshot
- flatten_config_select_options
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
- JobResult
- user/browser.rs
- `user`
- client.rs
- predicate_capture_text
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- ruby.rs
- .oil_directory_sections
- Database Explorer PRD
- .from_text
- AcpManager
- .acp_client_by_id
- markdown.rs
- scala.rs
- syntax_language
- .git_command_for_chord
- 0004-markdown-pretty-pipeline.md
- .autocomplete_providers
- Language
- .browser_feature_spec
- Domain Docs
- Issue tracker: GitHub
- .context_help_specs
- .db_feature_spec
- .debug_adapters
- .git_feature_spec
- .hover_providers
- rainbow_paren.rs
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
- .new
- .terminal_feature_spec
- .workspace_roots
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- syntax_language
- Agent skills
- ligatures.rs
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 773 edges
2. `ShellBuffer` - 379 edges
3. `shell_ui_mut()` - 347 edges
4. `register_shell_hooks()` - 263 edges
5. `shell_ui()` - 233 edges
6. `ShellError` - 192 edges
7. `shell_buffer_mut()` - 187 edges
8. `shell_buffer()` - 185 edges
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

## Communities (288 total, 30 thin omitted)

### Community 0 - "FontSet"
Cohesion: 0.11
Nodes (18): EmojiFont, FontSet, FontSet<'ttf>, IconFont, load_deferred_emoji_font(), load_font_set_with_mode(), load_next_deferred_icon_font(), normalize_display_scale() (+10 more)

### Community 1 - "Path"
Cohesion: 0.07
Nodes (31): inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, LspSessionHandle (+23 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (61): active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), browser_sync_plan_excludes_pdf_buffers(), codicon_glyphs_fit_inside_one_editor_cell(), configure_file_buffer() (+53 more)

### Community 3 - ".new"
Cohesion: 0.09
Nodes (81): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+73 more)

### Community 4 - "ShellState"
Cohesion: 0.04
Nodes (38): clear_key_sequence(), active_lsp_workspace_loaded(), active_runtime_surface(), alt_mod(), browser_devtools_shortcut_requested(), build_keydown_chord(), build_shell_summary(), ChordModifiers (+30 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (96): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+88 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.08
Nodes (86): additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+78 more)

### Community 7 - "PickerOverlay"
Cohesion: 0.07
Nodes (25): GitBranchActionKind, GitCommitActionKind, index_syntax_lines_with_rainbow_parens(), picker_preview_syntax_lines(), PickerAction, PickerKind, PickerOverlay, quickfix_clear_marks() (+17 more)

### Community 8 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (118): is_zero_width_display_character(), wrap_line_segments(), acp_buffer_layout(), acp_pane_body_visible_rows(), acp_slice_chars(), AcpBufferLayout, AcpPaneLayout, adjusted_contextual_ligature_pixel_size() (+110 more)

### Community 10 - "AcpEvent"
Cohesion: 0.09
Nodes (30): AvailableCommand, AcpEvent, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), command_input_hint(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work() (+22 more)

### Community 11 - "PluginPackage"
Cohesion: 0.02
Nodes (164): file_open_package(), package(), package(), package(), package_exports_image_commands(), package_exports_image_keybindings(), bash_package_auto_attaches_all_extensions(), bash_package_metadata() (+156 more)

### Community 12 - "Self"
Cohesion: 0.03
Nodes (37): browser_item(), browser_items(), default_action(), exported_acp_picker_items(), exported_db_browser_items(), hook_command(), Option, AcpActionSpec (+29 more)

### Community 13 - "Path"
Cohesion: 0.09
Nodes (45): parse_log_oneline(), build_git_fringe_snapshot(), create_git_worktree_from_query(), git_branch_merge(), git_branch_push_remote(), git_branch_remote(), git_commit_list(), git_commit_temp_path() (+37 more)

### Community 14 - "String"
Cohesion: 0.05
Nodes (89): cycle_hover_provider(), cycle_runtime_pane(), shell_ui(), split_runtime_pane(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_visual_yank_copies_selected_text(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail() (+81 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (24): AlacrittyEvent, LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc, Display, Drop, Error (+16 more)

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
Cohesion: 0.08
Nodes (29): autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c(), calculator_package_binds_ctrl_tab_to_switch_panes() (+21 more)

### Community 24 - "String"
Cohesion: 0.14
Nodes (17): db_browser_action_from_spec(), DisabledSecretStore, initialize_native_keyring(), InMemorySecretStore, load_postgres_schema(), OsSecretStore, qualified_name_from_spec(), redact_error() (+9 more)

### Community 25 - "shell_ui_mut"
Cohesion: 0.05
Nodes (122): active_runtime_popup(), ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), shell_ui_mut() (+114 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.09
Nodes (48): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+40 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "render_text_with_fonts"
Cohesion: 0.06
Nodes (53): Canvas, RenderColor, Self, TextStyle, alpha_bitmap_surface(), cached_primary_text_runs(), CachedLigatureGlyphPlacement, CachedLigatureLayout (+45 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.07
Nodes (105): EditorRuntime, Default, run_command(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer() (+97 more)

### Community 33 - "String"
Cohesion: 0.13
Nodes (43): apply_git_view(), command_output_transcript(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_read_command_output_allow_exit_codes(), git_status_cherry_open_command() (+35 more)

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

### Community 39 - "shell/mod.rs"
Cohesion: 0.03
Nodes (275): Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), active_directory_root(), active_project_workspace_root(), active_shell_buffer_has_input(), active_shell_buffer_id() (+267 more)

### Community 40 - "Self"
Cohesion: 0.12
Nodes (19): ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), default_rainbow_parens_enabled(), default_show_paren_enabled() (+11 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.03
Nodes (40): acp_output_header_title(), acp_tool_call_from_partial_update(), AcpBufferState, AcpPane, BackingFileFingerprint, ensure_buffer_has_line(), evaluate_active_plugin_buffer(), FileReloadWorkerOutcome (+32 more)

### Community 42 - "Result"
Cohesion: 0.08
Nodes (93): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+85 more)

### Community 43 - ".new"
Cohesion: 0.05
Nodes (80): buffer_footer_layout(), acp_multiline_text_lines_strip_carriage_returns(), acp_section_layout_orders_output_input_footer_and_statusline(), acp_wrapped_text_uses_full_width_on_continuation_rows(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow(), block_cursor_text_overlay_positions_multibyte_cursor_text() (+72 more)

### Community 44 - "shell/acp.rs"
Cohesion: 0.10
Nodes (35): acp_complete_slash(), acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_insert_file_mention(), acp_insert_slash_command(), acp_pick_mode(), acp_pick_model() (+27 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (62): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+54 more)

### Community 47 - "Option"
Cohesion: 0.03
Nodes (64): active_buffer_revision_key(), active_shell_workspace_id(), ascii_control_caret_notation(), buffer_context_overlay_snapshot(), BufferContextOverlayCacheKey, BufferContextOverlaySnapshot, built_user_library_path_for_command(), cached_context_overlay_snapshot() (+56 more)

### Community 48 - "workspace_dock_layout"
Cohesion: 0.12
Nodes (22): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+14 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.07
Nodes (26): exported_browser_feature_spec(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_oil_feature_spec(), exported_terminal_feature_spec(), BrowserFeatureSpec, ContextHelpSpec (+18 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.22
Nodes (17): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color(), resolve_terminal_named_color(), resolve_terminal_plain_color() (+9 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (60): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage(), contextual_ligature_raster_size_never_upscales_smaller_substitute_glyphs() (+52 more)

### Community 52 - "editor-lsp/src/lib.rs"
Cohesion: 0.20
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 53 - "LanguageServerSpec"
Cohesion: 0.15
Nodes (8): LanguageServerSpec, normalize_optional_string(), Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String

### Community 54 - "BufferId"
Cohesion: 0.15
Nodes (20): ActiveBufferEventContext, apply_git_status_snapshot(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_snapshot_for_buffer(), git_status_fetch_upstream_command(), git_status_pull_upstream_command(), git_status_rebase_pushremote_command() (+12 more)

### Community 55 - "ShellError"
Cohesion: 0.10
Nodes (105): Display, Error, From, ShellError, render_browser_buffer_body(), Color, adjust_color(), blend_color() (+97 more)

### Community 56 - ".is_empty"
Cohesion: 0.03
Nodes (75): TextPoint, TextSnapshot, acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count() (+67 more)

### Community 57 - "shell/git.rs"
Cohesion: 0.10
Nodes (35): begin_oil_worktree_request(), git_branch_list(), git_remote_worktree_branch_list(), git_status_checkout_file_command(), git_status_command_name(), git_status_diff_paths_command(), git_status_diff_range_command(), git_status_log_other_command() (+27 more)

### Community 58 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 59 - "BufferId"
Cohesion: 0.14
Nodes (38): acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items(), git_status_ctrl_v_visual_u_unstages_selected_items(), git_status_ctrl_v_visual_x_deletes_selected_items() (+30 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "LspNotification"
Cohesion: 0.09
Nodes (13): completion_level_for_message(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel, LspNotificationLog, LspNotificationProgress, LspNotificationSnapshot (+5 more)

### Community 62 - ".new_with_secret_store"
Cohesion: 0.27
Nodes (7): load_persisted_state(), Arc, Path, Self, Send, Sync, SecretStore

### Community 63 - "TextBuffer"
Cohesion: 0.05
Nodes (19): delimiter_partner(), EditRecord, find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at(), Default (+11 more)

### Community 64 - "PluginCommand"
Cohesion: 0.10
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 65 - "Option"
Cohesion: 0.10
Nodes (8): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), LanguageServerSession, AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 66 - "SyntaxRegistry"
Cohesion: 0.07
Nodes (58): TextEdit, apply_text_edits_to_span(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser() (+50 more)

### Community 67 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 68 - "Option"
Cohesion: 0.11
Nodes (49): apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), background_command_candidates(), background_command_names(), background_spawn_should_retry(), BackgroundCommandPipes (+41 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 72 - "String"
Cohesion: 0.05
Nodes (41): append_query_source(), asset_path_from_parts(), CaptureThemeMapping, command_failure_message(), default_install_root(), default_query_asset_root(), DeferredQuery, ensure_cloned_grammar_dir_exists() (+33 more)

### Community 73 - ".send"
Cohesion: 0.11
Nodes (43): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+35 more)

### Community 74 - "DbService"
Cohesion: 0.14
Nodes (16): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbService, DbSession, DbSessionId (+8 more)

### Community 75 - "clipboard.rs"
Cohesion: 0.06
Nodes (62): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+54 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (62): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+54 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (37): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+29 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.08
Nodes (66): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+58 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (37): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+29 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.06
Nodes (66): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+58 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 83 - "Option"
Cohesion: 0.09
Nodes (45): active_git_status_command_context(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), git_line_is_untracked() (+37 more)

### Community 85 - "LanguageServerRegistry"
Cohesion: 0.14
Nodes (17): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LspError, path_is_solution(), resolve_single_solution_path() (+9 more)

### Community 86 - "ShellUiState"
Cohesion: 0.02
Nodes (129): default_vim_target(), acp_decode_image(), activate_db_browser_line(), active_lsp_buffer_context(), active_lsp_code_action_range(), active_runtime_buffer(), active_window_id(), apply_db_browser_view() (+121 more)

### Community 87 - "AbiPaneConfig"
Cohesion: 0.06
Nodes (22): exported_keymap_config(), exported_pane_config(), KeymapConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig, config(), config() (+14 more)

### Community 88 - "LineCharMap"
Cohesion: 0.07
Nodes (34): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+26 more)

### Community 89 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

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
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 94 - "registered_queries.rs"
Cohesion: 0.16
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "WorkspaceConfigurationValue"
Cohesion: 0.17
Nodes (10): language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From, Number, T, workspace_configuration_value_round_trips_through_json(), WorkspaceConfigurationValue, K (+2 more)

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
Nodes (18): ChildStdin, launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), AtomicBool, Child (+10 more)

### Community 101 - ".spawn"
Cohesion: 0.16
Nodes (13): live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), E, Into, IntoIterator, Item, PathBuf (+5 more)

### Community 102 - "AbiSectionTree"
Cohesion: 0.11
Nodes (15): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiDirectoryEntry, AbiDirectoryEntryKind (+7 more)

### Community 103 - "From"
Cohesion: 0.04
Nodes (74): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GitStashEntry, exported_themes(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiAcpClient (+66 more)

### Community 104 - "Vec"
Cohesion: 0.11
Nodes (9): EventLog, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, Vec (+1 more)

### Community 105 - "TextRange"
Cohesion: 0.12
Nodes (14): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), lsp_range_from_text_range() (+6 more)

### Community 106 - "main"
Cohesion: 0.13
Nodes (15): bootstrap(), HostBootstrap, command_palette_items(), main(), panic_payload_message(), Any, Box, DebugAdapterSpec (+7 more)

### Community 107 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 108 - "JobSpec"
Cohesion: 0.20
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.10
Nodes (38): ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay() (+30 more)

### Community 110 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 111 - ".default"
Cohesion: 0.10
Nodes (51): Self, session_labels_ignore_stale_tracked_session_keys(), sync_buffer_preserves_manually_started_default_disabled_sessions(), sync_buffer_reopens_document_for_restarted_session_with_same_key(), test_session_handle(), commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head() (+43 more)

### Community 112 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 113 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - "TerminalTranscript"
Cohesion: 0.18
Nodes (5): append_lines(), TerminalLine, TerminalSession, TerminalStream, TerminalTranscript

### Community 117 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (33): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn, DbIndex, DbSchemaCache, DbTable, default_db_browser_line() (+25 more)

### Community 118 - "PickerItem"
Cohesion: 0.19
Nodes (8): match_item(), PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "RVec"
Cohesion: 0.17
Nodes (11): exported_terminal_config(), AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic (+3 more)

### Community 121 - "UserLibrary"
Cohesion: 0.09
Nodes (52): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_buffer_layout(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_host_viewport_rect(), browser_state_for_kind() (+44 more)

### Community 122 - "Self"
Cohesion: 0.07
Nodes (28): AbiContextHelpEntry, AbiIconFontCategory, AbiKeymapConfig, AbiLanguageServerRootStrategy, AbiLspDiagnosticsInfo, AbiOilKeyAction, AbiPickerLayout, AbiWorkspaceDockSide (+20 more)

### Community 123 - "Vec"
Cohesion: 0.20
Nodes (27): find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token(), git_status_entry_token_from_icon(), git_status_head_spans() (+19 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.08
Nodes (22): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_command_output_background(), git_repository_present(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState (+14 more)

### Community 125 - "String"
Cohesion: 0.13
Nodes (8): CommandPaletteState, CompilationState, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, String, TerminalState

### Community 127 - "Vec"
Cohesion: 0.27
Nodes (10): autocomplete_items(), calculator_symbols(), CalculatorSymbol, hover_lines(), hover_topics(), initial_buffer_lines(), initial_buffer_lines_only_seed_input_examples(), AutocompleteProviderItem (+2 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.12
Nodes (14): push_snapshot_line(), push_terminal_render_run(), Option, String, Vec, terminal_render_snapshot(), terminal_render_snapshot_preserves_wide_character_widths(), terminal_render_snapshot_tracks_visible_cursor() (+6 more)

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "UserLibraryModule"
Cohesion: 0.06
Nodes (28): exported_icon_symbols(), exported_oil_defaults(), exported_oil_keybindings(), exported_picker_truncate_strategy(), IconFontSymbol, OilDefaults, OilKeybindings, PickerTruncateStrategy (+20 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "treesittercontext_ghosttext.rs"
Cohesion: 0.06
Nodes (51): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+43 more)

### Community 134 - ".new"
Cohesion: 0.12
Nodes (26): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, default_db_browser_items(), execute_postgres(), execute_sql_server() (+18 more)

### Community 135 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "DbEngine"
Cohesion: 0.14
Nodes (13): DbActionOutcome, DbAutocompleteCandidate, DbEngine, DbHistoryEntry, DbQueryBufferMeta, DbSnippet, default_volt_state_dir(), PersistedDbState (+5 more)

### Community 138 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 139 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 140 - "String"
Cohesion: 0.07
Nodes (50): acp_connected(), acp_image_mention_token(), acp_open_permission_request(), acp_permission_picker_closed(), acp_permission_picker_submitted(), acp_resolve_permission_option(), acp_session_buffer_name(), acp_set_model() (+42 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 143 - "AbiLigatureConfig"
Cohesion: 0.32
Nodes (5): exported_ligature_config(), LigatureConfig, AbiLigatureConfig, LigatureConfig, LigatureConfig

### Community 144 - "resolve_permission"
Cohesion: 0.40
Nodes (3): acp_permission_approve(), acp_permission_deny(), resolve_permission()

### Community 145 - "BufferKind"
Cohesion: 0.09
Nodes (25): BufferKind, buffer_uses_browser_host_surface(), active_buffer_event_context(), apply_sqls_workspace_settings_for_buffer(), autocomplete_request_for_buffer(), buffer_is_acp(), buffer_is_browser(), buffer_is_command_output() (+17 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "ShellConfig"
Cohesion: 0.15
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 148 - "normalize_unique_entries"
Cohesion: 0.60
Nodes (3): normalize_unique_entries(), I, normalize_unique_entries()

### Community 149 - "String"
Cohesion: 0.05
Nodes (53): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, documentation_lines(), explicit_windows_env_value(), formatting_parser_maps_text_edits() (+45 more)

### Community 150 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 151 - "GhostTextContext"
Cohesion: 0.10
Nodes (18): GhostTextLine, exported_ghost_text_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AutocompleteProvider, AutocompleteProviderItem (+10 more)

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 153 - "db.rs"
Cohesion: 0.18
Nodes (14): browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord(), query_buffer_lines() (+6 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "compute_buffer_syntax"
Cohesion: 0.15
Nodes (19): append_hover_rendered_content(), apply_markdown_code_fence_syntax(), compute_buffer_syntax(), diagnostic_matches_cursor_line(), finalize_hover_overlay(), finalize_hover_provider_content(), hover_diagnostic_fragments_for_diagnostic(), hover_diagnostic_provider_fragments() (+11 more)

### Community 156 - ".from"
Cohesion: 0.18
Nodes (12): close_buffer_keeps_session_alive_for_next_file(), file_uri_roundtrip_handles_windows_paths(), live_session_picker_label_includes_server_and_root(), path_to_file_uri(), Error, stop_buffer_shuts_down_session(), stop_session_removes_live_session_and_returns_tracked_paths(), sync_buffer_onto_session_attaches_to_exact_root() (+4 more)

### Community 157 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "TerminalCursorSnapshot"
Cohesion: 0.24
Nodes (5): terminal_cursor_shape_for_input_mode(), map_terminal_cursor_shape(), TerminalCursorShape, TerminalCursorSnapshot, CursorShape

### Community 161 - "flatten_config_select_options"
Cohesion: 0.27
Nodes (10): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+2 more)

### Community 162 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 163 - "theme.rs"
Cohesion: 0.11
Nodes (55): packages(), LanguageConfiguration, Vec, syntax_languages(), apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors() (+47 more)

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
Cohesion: 0.21
Nodes (11): default_terminal_args(), default_terminal_program(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package() (+3 more)

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

### Community 173 - "JobResult"
Cohesion: 0.18
Nodes (3): CompilationResult, JobResult, Duration

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "client.rs"
Cohesion: 0.03
Nodes (136): BufRead, ClientCapabilities, active_parameter_label(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations() (+128 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

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

### Community 190 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 194 - ".from_text"
Cohesion: 0.04
Nodes (79): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits() (+71 more)

### Community 197 - "AcpManager"
Cohesion: 0.13
Nodes (23): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_pick_session(), acp_set_mode(), AcpManager (+15 more)

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 203 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 204 - "syntax_language"
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

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 233 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 234 - "AbiPdfOpenMode"
Cohesion: 0.24
Nodes (7): exported_pdf_open_mode(), PdfOpenMode, open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 236 - "LspLogEntry"
Cohesion: 0.16
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 242 - ".new"
Cohesion: 0.02
Nodes (198): ActiveLspBufferContext, WorkspaceId, absolute_path_hint(), acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat() (+190 more)

### Community 248 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **142 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+137 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `shell/tests.rs`, `ShellState`, `PickerOverlay`, `AcpEvent`, `String`, `Path`, `String`, `resolve_permission`, `BufferKind`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `shell_ui_mut`, `command_stream.rs`, `compute_buffer_syntax`, `String`, `shell/pdf.rs`, `ServiceRegistry`, `shell/mod.rs`, `ShellBuffer`, `Result`, `shell/acp.rs`, `Option`, `BufferId`, `.is_empty`, `shell/git.rs`, `Option`, `AcpManager`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `Option`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `main`, `shell/picker.rs`, `.new`, `UserLibrary`, `GitSummaryState`?**
  _High betweenness centrality (0.120) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `Option`, `ShellState`, `PickerOverlay`, `render.rs`, `BufferKind`, `compute_buffer_syntax`, `String`, `shell/pdf.rs`, `shell/mod.rs`, `Result`, `.new`, `shell/acp.rs`, `Option`, `ShellError`, `.is_empty`, `TextBuffer`, `clipboard.rs`, `directory.rs`, `shell/terminal.rs`, `Option`, `ShellUiState`, `LineCharMap`, `shell/picker.rs`, `.new`, `UserLibrary`, `Vec`, `GitSummaryState`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `UserLibrary` to `FontSet`, `ShellState`, `user/lib.rs`, `render.rs`, `load_user_library`, `ShellConfig`, `DynamicUserLibrary`, `compute_buffer_syntax`, `HoverOverlay`, `shell/mod.rs`, `ShellBuffer`, `Result`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `Option`, `workspace_dock_layout`, `HeaderlineTestUserLibrary`, `ShellError`, `shell/git.rs`, `directory.rs`, `volt/src/main.rs`, `ShellUiState`, `editor-plugin-host/src/lib.rs`, `shell/picker.rs`, `.new`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _142 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `FontSet` be split into smaller, more focused modules?**
  _Cohesion score 0.1092436974789916 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.07272727272727272 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0281743765467352 - nodes in this community are weakly interconnected._