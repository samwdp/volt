# Graph Report - volt  (2026-08-14)

## Corpus Check
- 232 files · ~588,581 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9338 nodes · 38189 edges · 281 communities (269 shown, 12 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3183 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `ba3a4c35`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- Option
- Path
- .new
- .new
- Result
- user/lib.rs
- editor-syntax/src/lib.rs
- ShellUiState
- AbiOilKeyAction
- render.rs
- AcpManager
- PluginPackage
- Self
- Path
- PluginBuffer
- LiveTerminalError
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- Result
- .from
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- render_text_with_fonts
- EditorRuntime
- PaneConfig
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- String
- Self
- ShellBuffer
- shell/tests.rs
- editor-icons/src/lib.rs
- shell_ui_mut
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- TextBuffer
- UserLibrary
- PathBuf
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- LanguageServerSpec
- sync_quickfix_popup_buffer
- BufferId
- ShellError
- InputField
- Option
- AcpPickerItemSpec
- buffer_footer_layout_with_command_line
- editor-lsp/src/lib.rs
- Result
- .new
- LspNotification
- PluginCommand
- String
- SyntaxRegistry
- StatuslineContext
- shell/acp.rs
- DebugConfiguration
- capture_mappings
- LanguageConfiguration
- String
- .send
- LanguageInstallPlan
- editor-path/src/lib.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- volt/src/main.rs
- LiveTerminalSession
- Section
- Option
- build_job_command
- editor-db/src/lib.rs
- draw_diagnostic_underlines_for_segment
- shell/browser.rs
- main
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- WorkspaceConfigurationValue
- editor-picker/src/lib.rs
- GitEditorState
- Self
- AcpClient
- RString
- Option
- common.rs
- String
- RVec
- Vec
- bash.rs
- clipboard.rs
- shell/picker.rs
- active_runtime_popup
- .default
- treesittercontext_ghosttext.rs
- clojure.rs
- PluginKeyBinding
- .get
- .spawn
- elixir.rs
- java.rs
- process_supervisor.rs
- hcl.rs
- DbSessionId
- kotlin.rs
- shell/git.rs
- Option
- modeline.rs
- DynamicUserLibrary
- latex.rs
- JobError
- TerminalRenderSnapshot
- user/config.rs
- lua.rs
- key_sequence.rs
- nix.rs
- .new_with_secret_store
- perl.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- normalize_inline_text
- php.rs
- r.rs
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- graphql.rs
- JobSpec
- ShellConfig
- volt/build.rs
- .new
- ruby.rs
- client.rs
- scala.rs
- ancestor_contexts_for_cursor
- oil.rs
- DbBrowserContext
- lsp.rs
- .oil_directory_sections
- swift.rs
- lang/vim.rs
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- ROption
- flatten_config_select_options
- .next_token
- theme.rs
- ServiceRegistry
- aligned_indent_column
- String
- user/terminal.rs
- build_output.rs
- proto.rs
- AbiPdfOpenMode
- solidity.rs
- package
- JobResult
- syntax_language
- user/browser.rs
- xml.rs
- .oil_keybindings
- `user`
- treesittercontext_shared.rs
- DbService
- AbiPickerTruncateStrategy
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- syntax_language
- AbiWorkspaceRoot
- PluginHookBinding
- update_acp_input_hint
- syntax_language
- index_syntax_lines
- Database Explorer PRD
- .new
- .load_from_path
- syntax_language
- UserLibraryModule
- String
- VimActionContext
- markdown.rs
- AbiSectionTree
- cargo
- user/workspace_dock.rs
- FontSet
- 0004-markdown-pretty-pipeline.md
- package
- build_headerline_lines
- Language
- Domain Docs
- Issue tracker: GitHub
- load
- debug_adapters
- main
- AbiGitFeatureSpec
- shell/mod.rs
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- Agent skills
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 765 edges
2. `ShellBuffer` - 373 edges
3. `shell_ui_mut()` - 342 edges
4. `register_shell_hooks()` - 261 edges
5. `shell_ui()` - 230 edges
6. `ShellError` - 185 edges
7. `shell_buffer()` - 181 edges
8. `shell_buffer_mut()` - 178 edges
9. `ShellUiState` - 174 edges
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

## Communities (281 total, 12 thin omitted)

### Community 0 - "Option"
Cohesion: 0.02
Nodes (102): active_buffer_revision_key(), active_shell_workspace_id(), ActiveTypingFrameProfile, apply_lsp_notifications(), ascii_control_caret_notation(), block_comment_toggle_removal_lens(), buffer_context_overlay_snapshot(), BufferContextOverlayCacheKey (+94 more)

### Community 1 - "Path"
Cohesion: 0.07
Nodes (29): inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, LspSessionHandle (+21 more)

### Community 2 - ".new"
Cohesion: 0.05
Nodes (94): buffer_footer_layout(), acp_multiline_text_lines_strip_carriage_returns(), acp_output_scroll_reaches_wrapped_tail(), acp_plan_entries_normalize_completed_prefix_when_later_step_is_active(), acp_plan_entries_normalize_completed_prefix_without_active_step(), acp_plan_entries_populate_static_plan_pane(), acp_plan_height_caps_wrapped_content_at_ten_rows(), acp_scroll_output_to_end_reaches_last_rendered_line() (+86 more)

### Community 3 - ".new"
Cohesion: 0.11
Nodes (74): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+66 more)

### Community 4 - "Result"
Cohesion: 0.04
Nodes (69): clear_key_sequence(), active_buffer_event_context(), active_lsp_buffer_context(), active_runtime_surface(), alt_mod(), apply_copilot_auth_notification(), apply_sqls_workspace_settings_for_active_buffer_context(), apply_sqls_workspace_settings_for_buffer() (+61 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (108): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+100 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.10
Nodes (73): vim_search_entries_trim_whitespace_from_labels(), additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+65 more)

### Community 7 - "ShellUiState"
Cohesion: 0.03
Nodes (94): BufferKind, default_vim_target(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_window_id(), apply_lsp_text_edits(), apply_pending_lsp_state(), buffer_interaction() (+86 more)

### Community 8 - "AbiOilKeyAction"
Cohesion: 0.47
Nodes (4): exported_oil_chord_action(), AbiOilKeyAction, OilKeyAction, OilKeyAction

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (117): acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), advance_point_by_text(), multicursor_selection_offsets(), acp_buffer_layout(), acp_chat_bubble_width_px(), acp_chat_origin_x(), acp_pane_body_visible_rows() (+109 more)

### Community 10 - "AcpManager"
Cohesion: 0.12
Nodes (26): AcpClientConfig, AvailableCommand, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_set_mode(), acp_set_model() (+18 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (42): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+34 more)

### Community 12 - "Self"
Cohesion: 0.06
Nodes (19): browser_item(), default_action(), hook_command(), Option, AcpActionSpec, AcpPickerOption, ContextHelpEntry, DbActionSpec (+11 more)

### Community 13 - "Path"
Cohesion: 0.07
Nodes (57): parse_log_oneline(), begin_oil_worktree_request(), build_git_fringe_snapshot(), command_output_transcript(), create_git_worktree_from_query(), fetch_git_prune(), git_branch_list(), git_branch_merge() (+49 more)

### Community 14 - "PluginBuffer"
Cohesion: 0.09
Nodes (6): PickerKeybindingContext, PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate, RVec

### Community 15 - "LiveTerminalError"
Cohesion: 0.09
Nodes (17): Keycode, Mod, terminal_key_for_event(), LiveTerminalError, Display, Error, Formatter, From (+9 more)

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
Nodes (16): DynamicUserLibrary, AcpClient, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig (+8 more)

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
Cohesion: 0.20
Nodes (10): DbSchemaCache, DbSession, InMemorySecretStore, load_postgres_schema(), load_sqlite_columns(), load_sqlite_schema(), Result, sqlite_index_table() (+2 more)

### Community 25 - ".from"
Cohesion: 0.07
Nodes (36): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GhostTextLine, GhostTextLine, exported_ghost_text_lines(), GhostTextLine, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag() (+28 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (45): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+37 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.09
Nodes (46): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+38 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "render_text_with_fonts"
Cohesion: 0.07
Nodes (45): Canvas, RenderColor, Self, alpha_bitmap_surface(), CachedLigatureGlyphPlacement, CachedLigatureLayout, compose_emoji_surface(), compose_ligature_surface() (+37 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.07
Nodes (130): EditorRuntime, Default, run_command(), active_git_status_command_context(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit() (+122 more)

### Community 33 - "PaneConfig"
Cohesion: 0.08
Nodes (14): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, MarkdownPrettyConfig, config(), AbiKeymapConfig (+6 more)

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

### Community 39 - "String"
Cohesion: 0.04
Nodes (193): Cow, write_system_clipboard(), yank_from_clipboard_text(), yank_to_clipboard_text(), accept_autocomplete(), activate_db_browser_line(), active_directory_root(), active_lsp_code_action_range() (+185 more)

### Community 40 - "Self"
Cohesion: 0.10
Nodes (22): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+14 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.03
Nodes (44): buffer_uses_browser_host_surface(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_tool_call_from_partial_update() (+36 more)

### Community 42 - "shell/tests.rs"
Cohesion: 0.03
Nodes (59): ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size(), focused_hover_ctrl_scroll_motions_are_bounded(), focused_hover_gg_and_g_scroll_to_expected_bounds() (+51 more)

### Community 43 - "editor-icons/src/lib.rs"
Cohesion: 0.15
Nodes (11): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+3 more)

### Community 44 - "shell_ui_mut"
Cohesion: 0.05
Nodes (75): ctrl_mod(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger() (+67 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.05
Nodes (58): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, decode_modeline() (+50 more)

### Community 47 - "TextBuffer"
Cohesion: 0.04
Nodes (62): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), EditRecord (+54 more)

### Community 48 - "UserLibrary"
Cohesion: 0.09
Nodes (34): browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Instant (+26 more)

### Community 49 - "PathBuf"
Cohesion: 0.03
Nodes (97): acp_decode_image(), active_theme_state_path(), active_workspace_root(), asset_path_from_parts(), BackingFileFingerprint, built_user_library_path_for_command(), canonicalize_project_root_path(), cleanup_formatter_temp() (+89 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.11
Nodes (31): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+23 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (59): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage() (+51 more)

### Community 52 - "LanguageServerSpec"
Cohesion: 0.13
Nodes (9): LanguageServerSpec, normalize_optional_string(), normalize_unique_entries(), Into, IntoIterator, Item, LanguageServerRootStrategy, Self (+1 more)

### Community 53 - "sync_quickfix_popup_buffer"
Cohesion: 0.11
Nodes (19): buffer_is_quickfix(), FontSetInit, quickfix_clear_marks(), quickfix_entry_for_cursor(), quickfix_mark_all(), quickfix_open_current_list(), quickfix_open_entry(), quickfix_open_from_one_shot() (+11 more)

### Community 54 - "BufferId"
Cohesion: 0.11
Nodes (27): ActiveBufferEventContext, apply_git_status_snapshot(), cancel_git_commit_buffer(), commit_git_buffer(), finish_oil_worktree_branch_selection(), git_command_output_owned(), git_commit_message(), git_commit_temp_path() (+19 more)

### Community 55 - "ShellError"
Cohesion: 0.12
Nodes (89): Display, Error, From, ShellError, render_browser_buffer_body(), Color, adjust_color(), blend_color() (+81 more)

### Community 57 - "Option"
Cohesion: 0.18
Nodes (6): CommandPaletteState, CompilationState, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 58 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 59 - "buffer_footer_layout_with_command_line"
Cohesion: 0.07
Nodes (38): acp_rendered_text_segments(), display_columns_for_character(), is_wide_display_character(), is_zero_width_display_character(), LineCharMap, LineWrapSegment, resolved_tab_width(), segment_index_for_column() (+30 more)

### Community 60 - "editor-lsp/src/lib.rs"
Cohesion: 0.15
Nodes (33): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, LspError, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server() (+25 more)

### Community 61 - "Result"
Cohesion: 0.08
Nodes (95): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+87 more)

### Community 62 - ".new"
Cohesion: 0.09
Nodes (41): diff_git_commit_at_point(), diff_git_dwim(), diff_git_stash_at_point(), git_action_detail(), git_args_with_no_pager(), git_log_args(), git_status_cherry_open_command(), git_status_diff_commit_command() (+33 more)

### Community 63 - "LspNotification"
Cohesion: 0.04
Nodes (41): BufRead, ChildStdin, completion_level_for_message(), diagnostic_matches_request_range(), launch_summary(), log_message_error_from_ols_does_not_become_ui_notification(), LspLogDirection, LspLogEntry (+33 more)

### Community 64 - "PluginCommand"
Cohesion: 0.10
Nodes (22): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+14 more)

### Community 65 - "String"
Cohesion: 0.07
Nodes (93): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk(), buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus(), buffer_save_still_writes_when_format_on_save_fails() (+85 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.06
Nodes (69): append_query_source(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), command_failure_message(), compile_query_source() (+61 more)

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
Cohesion: 0.08
Nodes (15): PathMatcher, Vec, CaptureThemeMapping, cmake_configuration(), LanguageConfiguration, LanguageLoader, load_language(), I (+7 more)

### Community 72 - "String"
Cohesion: 0.30
Nodes (21): apply_language_options_table(), apply_options_table(), parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option() (+13 more)

### Community 73 - ".send"
Cohesion: 0.16
Nodes (36): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession, AcpTerminal (+28 more)

### Community 74 - "LanguageInstallPlan"
Cohesion: 0.09
Nodes (17): asset_path_from_parts(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), GrammarSource, InstallCommandSpec, io_error(), LanguageInstallPlan, remove_legacy_grammar_install_directory() (+9 more)

### Community 75 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (17): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathPattern, PathPatternKind (+9 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (70): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+62 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (37): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+29 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.08
Nodes (66): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+58 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.15
Nodes (35): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+27 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.06
Nodes (64): acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items(), hook_command() (+56 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.11
Nodes (28): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), format_micros_as_millis(), LaunchMode, LaunchOptions, LspState, parse_launch_options() (+20 more)

### Community 83 - "LiveTerminalSession"
Cohesion: 0.12
Nodes (13): AlacrittyEvent, Self, terminal_scroll_for_motion(), LiveTerminalSession, QueuedEventListener, Arc, Drop, Receiver (+5 more)

### Community 84 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 85 - "Option"
Cohesion: 0.09
Nodes (17): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LanguageServerSession, path_is_solution(), resolve_single_solution_path() (+9 more)

### Community 86 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 87 - "editor-db/src/lib.rs"
Cohesion: 0.08
Nodes (47): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), current_statement(), DbColumn, DbExecutionOutput, DbTable (+39 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.15
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "shell/browser.rs"
Cohesion: 0.05
Nodes (79): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+71 more)

### Community 90 - "main"
Cohesion: 0.10
Nodes (21): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), panic_payload_message(), print_shell_summary(), Any (+13 more)

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
Cohesion: 0.14
Nodes (36): default_install_root(), csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting() (+28 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (16): sanitize_transport_message(), transport_key_is_sensitive(), document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From (+8 more)

### Community 97 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (46): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+38 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "Self"
Cohesion: 0.05
Nodes (49): GitStashEntry, AbiCaptureThemeMapping, AbiContextHelpEntry, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot (+41 more)

### Community 100 - "AcpClient"
Cohesion: 0.07
Nodes (19): AsyncRead, AcpClient, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, KillTerminalRequest, KillTerminalResponse, ReadTextFileRequest (+11 more)

### Community 101 - "RString"
Cohesion: 0.11
Nodes (17): AbiAcpClient, AbiColor, AbiStringPair, AbiTheme, AbiThemeOption, AbiThemeOptionEntry, AbiThemeToken, AcpClient (+9 more)

### Community 102 - "Option"
Cohesion: 0.05
Nodes (58): configuration_item_section(), CopilotDeviceCodePrompt, csharp_metadata_request_params(), diagnostics_parser_maps_lsp_fields(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item(), file_uri_to_path() (+50 more)

### Community 103 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 104 - "String"
Cohesion: 0.15
Nodes (16): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferState, DbEngine, DbHistoryEntry, DbIndex, DbQueryBufferMeta (+8 more)

### Community 105 - "RVec"
Cohesion: 0.14
Nodes (13): AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, DebugAdapterSpec, HoverProvider, HoverProviderTopic, DebugAdapterSpec (+5 more)

### Community 106 - "Vec"
Cohesion: 0.13
Nodes (7): EventLog, AutocompleteProvider, ContextHelpSpec, HoverProvider, String, Vec, WorkspaceRoot

### Community 107 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 108 - "clipboard.rs"
Cohesion: 0.19
Nodes (12): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+4 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (38): UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars(), picker_overlay() (+30 more)

### Community 110 - "active_runtime_popup"
Cohesion: 0.10
Nodes (58): active_runtime_popup(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean() (+50 more)

### Community 111 - ".default"
Cohesion: 0.06
Nodes (65): Self, definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), location_from_link(), location_from_lsp(), location_sorting_deduplicates_reference_results(), LspLocation, parse_definition_response() (+57 more)

### Community 112 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 113 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 114 - "PluginKeyBinding"
Cohesion: 0.13
Nodes (22): plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding(), normal_binding_commands() (+14 more)

### Community 115 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - ".spawn"
Cohesion: 0.11
Nodes (15): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+7 more)

### Community 117 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 118 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 121 - "DbSessionId"
Cohesion: 0.26
Nodes (11): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbBrowserBufferKind, DbSessionId, DbSessionSummary, insert_test_session(), remembered_connections_store_metadata_separately_from_secret(), sqls_initialization_options_for_query_buffer_use_attached_session() (+3 more)

### Community 122 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 123 - "shell/git.rs"
Cohesion: 0.10
Nodes (51): apply_git_view(), find_paren_number_range(), format_section_line(), git_line_is_untracked(), git_status_checkout_file_command(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_delete_target_for_line() (+43 more)

### Community 124 - "Option"
Cohesion: 0.07
Nodes (36): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_command_output_background(), git_repository_present(), git_status_command_name(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot (+28 more)

### Community 125 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 126 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (23): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DebugAdapterSpec, DirectoryEntry (+15 more)

### Community 127 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "TerminalRenderSnapshot"
Cohesion: 0.15
Nodes (5): Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalCursorSnapshot, TerminalRenderLine, TerminalRenderSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 134 - ".new_with_secret_store"
Cohesion: 0.13
Nodes (14): default_volt_state_dir(), initialize_native_keyring(), load_persisted_state(), OsSecretStore, Arc, Into, Path, PathBuf (+6 more)

### Community 135 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "normalize_inline_text"
Cohesion: 0.20
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 138 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 139 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 140 - "AcpEvent"
Cohesion: 0.10
Nodes (24): AcpEvent, choose_permission_outcome(), coalesce_acp_events(), coalesce_acp_events_merges_adjacent_agent_text_chunks(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), format_permission_option_kind(), PendingPermission (+16 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 144 - "JobSpec"
Cohesion: 0.21
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "ShellConfig"
Cohesion: 0.16
Nodes (12): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+4 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 148 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 149 - "client.rs"
Cohesion: 0.04
Nodes (88): ClientCapabilities, active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document() (+80 more)

### Community 150 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 151 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (37): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec(), help_entry() (+29 more)

### Community 153 - "DbBrowserContext"
Cohesion: 0.14
Nodes (19): browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), feature_spec(), hook_command(), package(), package_exports_required_commands(), query_buffer_exports_execute_chord() (+11 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 156 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 157 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "ROption"
Cohesion: 0.14
Nodes (13): exported_statusline_render(), statusline_context_from_abi(), AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiLspDiagnosticsInfo, AbiStatuslineContext, AutocompleteProvider, AutocompleteProviderItem (+5 more)

### Community 161 - "flatten_config_select_options"
Cohesion: 0.31
Nodes (9): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+1 more)

### Community 163 - "theme.rs"
Cohesion: 0.15
Nodes (30): assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages(), bundled_themes_use_pallet_sections_and_token_references(), list_theme_files() (+22 more)

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
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 169 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 170 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 171 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 172 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 173 - "JobResult"
Cohesion: 0.18
Nodes (3): CompilationResult, JobResult, Duration

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "treesittercontext_shared.rs"
Cohesion: 0.23
Nodes (18): find(), IconFontSymbol, Option, symbols(), collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword() (+10 more)

### Community 180 - "DbService"
Cohesion: 0.11
Nodes (15): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, DbAutocompleteCandidate, DbService, looks_like_postgres_connection_string(), looks_like_sql_server_connection_string(), parse_key_value(), parse_postgres_keyword() (+7 more)

### Community 181 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

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

### Community 187 - "AbiWorkspaceRoot"
Cohesion: 0.32
Nodes (5): exported_workspace_roots(), WorkspaceRoot, AbiWorkspaceRoot, WorkspaceRoot, WorkspaceRoot

### Community 189 - "update_acp_input_hint"
Cohesion: 0.21
Nodes (10): acp_permission_approve(), acp_permission_deny(), build_acp_input_hint(), format_acp_mode_label(), format_acp_model_label(), PermissionDecision, resolve_permission(), update_acp_input_hint() (+2 more)

### Community 190 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 191 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - ".new"
Cohesion: 0.06
Nodes (48): CodeActionParams, close_buffer_keeps_session_alive_for_next_file(), code_action_params(), code_action_params_use_flattened_lsp_shape(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), copilot_status_notifications_offer_sign_in_action(), default_workspace_lists_only_sessions_serving_open_buffers() (+40 more)

### Community 194 - ".load_from_path"
Cohesion: 0.12
Nodes (16): detect_preferred_line_ending(), from_reader_normalizes_crlf_and_tracks_line_endings(), LineEnding, must(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean(), AsRef, Drop (+8 more)

### Community 195 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 196 - "UserLibraryModule"
Cohesion: 0.08
Nodes (26): AbiBrowserFeatureSpec, AbiContextHelpSpec, AbiDbFeatureSpec, AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeybindings, AbiOilSortMode, AbiTerminalFeatureSpec (+18 more)

### Community 197 - "String"
Cohesion: 0.09
Nodes (52): acp_complete_slash(), acp_connected(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_picker_closed(), acp_permission_picker_submitted(), acp_pick_mode(), acp_pick_model() (+44 more)

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

### Community 201 - "AbiSectionTree"
Cohesion: 0.18
Nodes (9): exported_git_status_sections(), DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 202 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 203 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_popup_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 205 - "FontSet"
Cohesion: 0.09
Nodes (28): TextStyle, EmojiFont, FontSet, FontSet<'ttf>, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode() (+20 more)

### Community 212 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 215 - "build_headerline_lines"
Cohesion: 0.22
Nodes (8): packages(), LanguageConfiguration, Vec, syntax_languages(), build_headerline_lines(), headerline_lines(), String, Vec

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

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 239 - "AbiGitFeatureSpec"
Cohesion: 0.14
Nodes (13): GitCommandBinding, GitPrefixBinding, exported_git_command_for_chord(), AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding (+5 more)

### Community 242 - "shell/mod.rs"
Cohesion: 0.02
Nodes (247): ActiveLspBufferContext, WorkspaceId, absolute_path_hint(), acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat() (+239 more)

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **141 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+136 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `Option`, `Result`, `ShellUiState`, `AcpManager`, `AcpEvent`, `Path`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `command_stream.rs`, `shell/pdf.rs`, `ServiceRegistry`, `String`, `ShellBuffer`, `shell/tests.rs`, `shell_ui_mut`, `PathBuf`, `sync_quickfix_popup_buffer`, `BufferId`, `update_acp_input_hint`, `.new`, `Result`, `String`, `shell/acp.rs`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `shell/browser.rs`, `main`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `shell/picker.rs`, `active_runtime_popup`, `shell/mod.rs`, `shell/git.rs`, `Option`?**
  _High betweenness centrality (0.109) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `Option`, `TerminalRenderSnapshot`, `.new`, `Result`, `ShellUiState`, `render.rs`, `shell/pdf.rs`, `String`, `TextBuffer`, `UserLibrary`, `PathBuf`, `BufferId`, `ShellError`, `InputField`, `buffer_footer_layout_with_command_line`, `Result`, `shell/acp.rs`, `directory.rs`, `shell/terminal.rs`, `draw_diagnostic_underlines_for_segment`, `shell/browser.rs`, `shell/picker.rs`, `shell/mod.rs`, `shell/git.rs`, `Option`?**
  _High betweenness centrality (0.059) - this node is a cross-community bridge._
- **Why does `UserLibrary` connect `UserLibrary` to `Option`, `Result`, `user/lib.rs`, `ShellUiState`, `render.rs`, `ShellConfig`, `DynamicUserLibrary`, `editor-render/src/lib.rs`, `HoverOverlay`, `editor-markdown/src/lib.rs`, `sdk/src/lib.rs`, `PathBuf`, `HeaderlineTestUserLibrary`, `ShellError`, `buffer_footer_layout_with_command_line`, `Result`, `directory.rs`, `FontSet`, `volt/src/main.rs`, `shell/browser.rs`, `main`, `editor-plugin-host/src/lib.rs`, `shell/picker.rs`, `shell/mod.rs`, `Option`, `DynamicUserLibrary`?**
  _High betweenness centrality (0.057) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _141 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.01647483638004965 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.07290417166466746 - nodes in this community are weakly interconnected._
- **Should `.new` be split into smaller, more focused modules?**
  _Cohesion score 0.04638218923933209 - nodes in this community are weakly interconnected._