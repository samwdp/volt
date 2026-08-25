# Graph Report - volt  (2026-08-25)

## Corpus Check
- 257 files · ~640,164 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10510 nodes · 42932 edges · 332 communities (298 shown, 34 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3444 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7e26b91a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- set_directory_root
- LspClientError
- Option
- src/tests.rs
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- editor-lsp/src/client.rs
- shell/tests.rs
- render.rs
- draw.rs
- PluginPackage
- Self
- GitSummaryState
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
- .get
- state_with_user_library
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- ThemeRegistry
- FontSet
- String
- .markdown_pretty_config
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
- shell/browser.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- LanguageServerSpec
- Option
- editor-dap/src/lib.rs
- Path
- HeaderlineTestUserLibrary
- editor-git/src/lib.rs
- theme.rs
- lsp.rs
- ShellError
- Option
- DapClientManager
- AcpPickerItemSpec
- active_runtime_popup
- build_output.rs
- shell/git.rs
- BufferId
- AbiGitFeatureSpec
- PluginCommand
- InputField
- SyntaxRegistry
- clipboard.rs
- DebugConfigurationCandidate
- .new
- .new
- WorkspaceConfigurationValue
- LineSyntaxSpan
- LanguageServerSession
- DbService
- buffer_footer_layout_with_command_line
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerSession
- Section
- DynamicUserLibrary
- tool_install.rs
- InstallCommand
- workspace_dock_layout
- editor-picker/src/lib.rs
- diagnostics.rs
- TextSnapshot
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
- editor-lsp/src/lib.rs
- .spawn
- DebugConfiguration
- .from
- browser_host.rs
- paths.rs
- editor-icons/src/lib.rs
- TextRange
- JobSpec
- PickerOverlay
- String
- user/git.rs
- PickerItem
- Option
- PluginKeyBinding
- TextPoint
- LspClientManager
- String
- nix.rs
- process_supervisor.rs
- Vec
- centered_rect
- DapSessionHandle
- DebugAdapterSpec
- bash.rs
- DbBrowserBufferView
- StoredBreakpoint
- ROption
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- connect_transport
- key_sequence.rs
- Self
- AbiStatuslineContext
- AbiSectionTree
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- hcl.rs
- .new
- java.rs
- shell/acp.rs
- CommandLineOverlay
- kotlin.rs
- .default
- xml.rs
- latex.rs
- volt/build.rs
- .load_from_path
- r.rs
- swift.rs
- .push
- .new
- oil.rs
- user/db.rs
- .path
- LspNotification
- show_paren.rs
- PickerItemSpec
- load
- Copilot instructions for `volt`
- editor-dap/src/client.rs
- AbiPaneConfig
- InstallRecipe
- package
- ServiceRegistry
- lua.rs
- String
- user/terminal.rs
- corpus_inventory.rs
- rainbow_parens.rs
- AbiGitStatusSnapshot
- Vec
- user/dap.rs
- JobResult
- AbiHoverProvider
- user/browser.rs
- shim.rs
- PluginBuffer
- `user`
- String
- aligned_indent_column
- Result
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- WorkspaceDockConfig
- OilDefaultsSection
- load_font_set_with_mode
- volt/src/main.rs
- document_language_id_for_extension
- common.rs
- Database Explorer PRD
- proto.rs
- .from_text
- Vec
- Path
- I
- connect_sql_server
- Option
- markdown.rs
- normalize_inline_text
- panic_payload_message
- lang/vim.rs
- Option
- package
- clojure.rs
- elixir.rs
- 0004-markdown-pretty-pipeline.md
- DbEngine
- main
- .move_object_end_forward
- graphql.rs
- dap-client-spec.md
- ShellConfig
- highlight.rs
- Language
- TerminalCursorSnapshot
- Domain Docs
- Issue tracker: GitHub
- main
- rainbow_paren.rs
- perl.rs
- evaluate_expression
- handle_git_status_chord
- AbiPdfOpenMode
- String
- cargo
- Option
- .oil_directory_sections
- .oil_directory_sections
- .acp_client_by_id
- .git_command_for_chord
- AbiLanguageConfiguration
- php.rs
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- ruby.rs
- scala.rs
- Agent skills
- solidity.rs
- .autocomplete_providers
- user/workspace_dock.rs
- .browser_feature_spec
- .context_help_specs
- .next_token
- 0005-dap-session-and-client.md
- .db_feature_spec
- .git_feature_spec
- syntax_language
- .hover_providers
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pdf_open_mode
- package
- keymap.rs
- ligatures.rs
- .fmt
- 0006-language-server-and-debug-adapter-install.md
- .picker_layout
- .picker_truncate_strategy
- .show_paren_config
- .terminal_feature_spec
- .workspace_roots

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 881 edges
2. `shell_ui_mut()` - 390 edges
3. `ShellBuffer` - 389 edges
4. `register_shell_hooks()` - 272 edges
5. `shell_ui()` - 268 edges
6. `shell_buffer()` - 198 edges
7. `shell_buffer_mut()` - 196 edges
8. `ShellError` - 192 edges
9. `ShellUiState` - 187 edges
10. `ShellState` - 164 edges

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
- 2-file cycle: `crates/editor-tool-install/src/lib.rs -> crates/editor-tool-install/src/paths.rs -> crates/editor-tool-install/src/lib.rs`
- 2-file cycle: `crates/editor-render/src/lib.rs -> crates/editor-render/src/split_layout.rs -> crates/editor-render/src/lib.rs`

## Communities (332 total, 34 thin omitted)

### Community 0 - "set_directory_root"
Cohesion: 0.20
Nodes (10): acp_decode_image(), dap_log_buffer_lines(), decode_raster_image_bytes(), decode_raster_image_path(), refresh_directory_buffer(), DecodedImage, OilDefaults, set_directory_error() (+2 more)

### Community 1 - "LspClientError"
Cohesion: 0.08
Nodes (25): BufRead, ClientCapabilities, client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), inline_completion_params(), is_csharp_metadata_uri(), LspClientError, LspInlineCompletionItem (+17 more)

### Community 2 - "Option"
Cohesion: 0.07
Nodes (64): parse_log_oneline(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), create_git_worktree_from_query(), git_branch_merge(), git_branch_push_remote(), git_branch_remote() (+56 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.14
Nodes (64): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+56 more)

### Community 4 - "ShellState"
Cohesion: 0.03
Nodes (58): clear_key_sequence(), active_runtime_surface(), alt_mod(), browser_devtools_shortcut_requested(), build_keydown_chord(), build_shell_summary(), ChordModifiers, cycle_vim_command_line_completion() (+50 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.02
Nodes (116): bundled_highlight_query(), cached_syntax_languages(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers() (+108 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.10
Nodes (70): B, additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+62 more)

### Community 7 - "editor-lsp/src/client.rs"
Cohesion: 0.04
Nodes (71): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+63 more)

### Community 8 - "shell/tests.rs"
Cohesion: 0.03
Nodes (57): load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage() (+49 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (119): acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), advance_point_by_text(), acp_buffer_layout(), acp_chat_bubble_width_px(), acp_chat_corner_radius(), acp_chat_origin_x(), acp_chat_rounded() (+111 more)

### Community 10 - "draw.rs"
Cohesion: 0.08
Nodes (54): AcpBufferDraw, AcpPaneDraw, AcpPrefixDraw, BrowserBufferDraw, BrowserSyncView, BufferBodyPalette, BufferChrome, BufferChrome<'a> (+46 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (43): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+35 more)

### Community 12 - "Self"
Cohesion: 0.08
Nodes (9): AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, PickerActionSpec, Into, RString (+1 more)

### Community 13 - "GitSummaryState"
Cohesion: 0.10
Nodes (18): apply_git_fringe_hunk(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, mark_git_fringe_snapshots_stale() (+10 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.17
Nodes (30): collect_configuration_candidates(), configuration_holes(), DapConfigError, DebugInferContext, deep_inference_finds_cargo_binary_and_heuristic(), deep_inference_finds_dotnet_dll(), default_workspace_skips_deep_inference(), find_csproj() (+22 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (25): AlacrittyEvent, Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc (+17 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.05
Nodes (63): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+55 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.09
Nodes (12): detect_in_progress(), GitLogEntry, GitStashEntry, GitStatusSnapshot, parse_stash_list(), RepositoryStatus, Into, Option (+4 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.05
Nodes (114): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+106 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.04
Nodes (17): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig, MarkdownPrettyConfig (+9 more)

### Community 20 - "HookBus"
Cohesion: 0.07
Nodes (23): HookBus, HookDefinition, HookError, HookEvent, HookSubscription, BTreeMap, BufferId, Default (+15 more)

### Community 21 - "EditorModel"
Cohesion: 0.07
Nodes (26): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+18 more)

### Community 22 - "KeymapScope"
Cohesion: 0.10
Nodes (34): autocomplete_overrides_workspace_while_active(), BindingKey, ChordModifier, dap_mode_overrides_global_f5_while_session_live(), duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeyBinding (+26 more)

### Community 23 - "calculator.rs"
Cohesion: 0.08
Nodes (29): autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c(), calculator_package_binds_ctrl_tab_to_switch_panes() (+21 more)

### Community 24 - ".get"
Cohesion: 0.17
Nodes (19): column_is_numeric(), DbAutocompleteCandidate, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns() (+11 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.06
Nodes (96): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk() (+88 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.08
Nodes (58): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches() (+50 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.06
Nodes (33): AutocompleteProviderKind, notification_severity(), RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry (+25 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.09
Nodes (25): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+17 more)

### Community 31 - "FontSet"
Cohesion: 0.07
Nodes (53): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, IconFont (+45 more)

### Community 32 - "String"
Cohesion: 0.07
Nodes (120): run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit() (+112 more)

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
Cohesion: 0.02
Nodes (414): EditorRuntime, Default, write_system_clipboard(), accept_autocomplete(), activate_db_browser_line(), active_buffer_event_context(), active_buffer_revision_key(), active_dashboard_editor_buffer() (+406 more)

### Community 40 - "Self"
Cohesion: 0.12
Nodes (19): ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), default_rainbow_parens_enabled(), default_show_paren_enabled() (+11 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.02
Nodes (80): acp_output_header_title(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_tool_call_from_partial_update(), AcpPaneState, advance_markdown_table_insert_tab() (+72 more)

### Community 42 - "Result"
Cohesion: 0.07
Nodes (103): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+95 more)

### Community 43 - "String"
Cohesion: 0.04
Nodes (97): ctrl_mod(), shell_ui(), split_runtime_pane(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_text_input_updates_acp_input(), browser_buffer_submit_tracks_requested_navigation(), browser_host_focus_parent_event_returns_to_normal_mode(), browser_host_new_window_event_routes_into_browser_popup() (+89 more)

### Community 44 - "shell/browser.rs"
Cohesion: 0.10
Nodes (41): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len(), browser_viewport_contains_point() (+33 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.08
Nodes (68): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+60 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (62): Vec, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+54 more)

### Community 47 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (11): LanguageServerRootStrategy, LanguageServerSpec, InstallRecipe, Into, IntoIterator, Item, Iterator, LanguageServerRootStrategy (+3 more)

### Community 48 - "Option"
Cohesion: 0.13
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LspError, normalize_optional_string(), path_is_solution() (+10 more)

### Community 49 - "editor-dap/src/lib.rs"
Cohesion: 0.11
Nodes (18): Client, codelldb(), DapError, DebugAdapterTransport, DebugSessionPlan, gdb(), must(), prepared_session_includes_configuration_and_launch_spec() (+10 more)

### Community 50 - "Path"
Cohesion: 0.07
Nodes (27): asset_path_from_parts(), command_failure_message(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), GrammarSource, install_plan_requests_generate_when_parser_is_missing(), InstallCommandSpec (+19 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (36): AtomicUsize, active_input_prompt_text(), browser_sync_plan_avoids_notification_overlays(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient (+28 more)

### Community 52 - "editor-git/src/lib.rs"
Cohesion: 0.13
Nodes (23): configure_background_command(), git_available(), GitStatusError, list_repository_files(), parse_header(), parse_status(), parser_extracts_branch_and_sections(), parser_extracts_unborn_branch_name() (+15 more)

### Community 53 - "theme.rs"
Cohesion: 0.13
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 54 - "lsp.rs"
Cohesion: 0.16
Nodes (24): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), clojure_lsp_recipe(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), hook_command(), install_recipe_for_language_server() (+16 more)

### Community 55 - "ShellError"
Cohesion: 0.13
Nodes (87): Display, Error, From, ShellError, render_browser_buffer_body(), CellMetrics, adjust_color(), blend_color() (+79 more)

### Community 56 - "Option"
Cohesion: 0.01
Nodes (332): BufferKind, browser_state_for_kind(), ActiveLspBufferContext, default_vim_target(), WorkspaceId, absolute_path_hint(), acp_build_output_lines(), acp_build_plan_lines() (+324 more)

### Community 57 - "DapClientManager"
Cohesion: 0.17
Nodes (11): active_thread_id(), clear_stopped_snapshot(), DapClientError, DapClientManager, Display, Error, Formatter, Path (+3 more)

### Community 58 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.10
Nodes (59): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers() (+51 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "shell/git.rs"
Cohesion: 0.08
Nodes (44): begin_oil_worktree_request(), ensure_no_rebase_in_progress(), git_branch_list(), git_merge_in_progress(), git_rebase_in_progress(), git_remote_worktree_branch_list(), git_status_branches_command(), git_status_checkout_file_command() (+36 more)

### Community 62 - "BufferId"
Cohesion: 0.14
Nodes (24): ActiveBufferEventContext, apply_git_status_snapshot(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_snapshot_for_buffer(), git_status_delete_targets(), git_status_selected_lines(), handle_git_status_tab() (+16 more)

### Community 63 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 64 - "PluginCommand"
Cohesion: 0.09
Nodes (19): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+11 more)

### Community 65 - "InputField"
Cohesion: 0.05
Nodes (73): Cow, yank_to_clipboard_text(), active_shell_buffer_mut(), active_shell_buffer_vim_targets_input(), add_next_multicursor_match(), adjust_tag_child_indent(), apply_block_operator(), apply_directory_edit_queue_if_needed() (+65 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.08
Nodes (46): SyntaxText, apply_text_edits_to_span(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), create_parser() (+38 more)

### Community 67 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 68 - "DebugConfigurationCandidate"
Cohesion: 0.12
Nodes (13): configuration_holes_detect_missing_launch_program(), DebugConfigurationCandidate, DebugConfigurationSource, DebugStartHistory, DebugStartRecord, default_request(), history_records_last_and_recent(), Into (+5 more)

### Community 69 - ".new"
Cohesion: 0.04
Nodes (111): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_paste_image_inserts_mention_token_and_stores_bytes() (+103 more)

### Community 70 - ".new"
Cohesion: 0.11
Nodes (37): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command() (+29 more)

### Community 71 - "WorkspaceConfigurationValue"
Cohesion: 0.11
Nodes (17): sanitize_transport_message(), transport_key_is_sensitive(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, Number, T (+9 more)

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (49): dap_variable_line_spans(), browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines() (+41 more)

### Community 73 - "LanguageServerSession"
Cohesion: 0.10
Nodes (6): Diagnostic, DiagnosticSeverity, LanguageServerSession, LspWorkspaceDiagnostic, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 74 - "DbService"
Cohesion: 0.13
Nodes (15): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbIndex, DbQueryBufferMeta, DbService (+7 more)

### Community 75 - "buffer_footer_layout_with_command_line"
Cohesion: 0.06
Nodes (48): browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Rect (+40 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (70): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+62 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.13
Nodes (44): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing() (+36 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (65): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerSession"
Cohesion: 0.14
Nodes (7): PickerResultOrder, PickerSession, Self, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 82 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 84 - "tool_install.rs"
Cohesion: 0.19
Nodes (40): apply_tool_install_finish(), begin_explicit_install(), continue_tool_install(), fail_tool_install(), fail_tool_install_with_message(), handle_dap_install_hook(), handle_lsp_install_hook(), install_debug_adapter_by_id() (+32 more)

### Community 85 - "InstallCommand"
Cohesion: 0.12
Nodes (26): Display, Error, From, Self, String, ToolInstallError, program_is_available(), archive_commands() (+18 more)

### Community 86 - "workspace_dock_layout"
Cohesion: 0.13
Nodes (20): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+12 more)

### Community 87 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 88 - "diagnostics.rs"
Cohesion: 0.14
Nodes (23): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+15 more)

### Community 89 - "TextSnapshot"
Cohesion: 0.08
Nodes (14): BufferStats, EditRecord, HighlightDocument, large_buffers_expose_line_windows_without_full_materialization(), String, Vec, TextEdit, TextSnapshot (+6 more)

### Community 90 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.08
Nodes (18): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+10 more)

### Community 93 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 94 - ".from_grammar"
Cohesion: 0.16
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "treesittercontext_ghosttext.rs"
Cohesion: 0.07
Nodes (48): packages(), LanguageConfiguration, Vec, syntax_languages(), ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery (+40 more)

### Community 97 - "TextBuffer"
Cohesion: 0.10
Nodes (13): delimiter_partner(), find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at(), Default, Fn (+5 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "editor-lsp/src/lib.rs"
Cohesion: 0.25
Nodes (25): csharp_language_server(), dev_extension_server(), dockerfile_language_server(), must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared(), prepare_sessions_for_path_returns_filename_matches_without_extensions() (+17 more)

### Community 101 - ".spawn"
Cohesion: 0.10
Nodes (18): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+10 more)

### Community 102 - "DebugConfiguration"
Cohesion: 0.18
Nodes (7): DebugConfiguration, DebugRequestKind, Into, Option, PathBuf, Self, PendingDapStartPrompt

### Community 103 - ".from"
Cohesion: 0.05
Nodes (51): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), abi_debug_adapter_spec_round_trips_install_recipe(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_install_recipe(), abi_language_server_spec_round_trips_path_matchers(), AbiAcpClient (+43 more)

### Community 104 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 105 - "paths.rs"
Cohesion: 0.13
Nodes (25): acp_tool_kind_icon(), ToolKind, is_volt_install_path(), locate_program(), ProgramLocation, Path, PathBuf, apply_install_bins_to_process_path() (+17 more)

### Community 106 - "editor-icons/src/lib.rs"
Cohesion: 0.11
Nodes (16): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+8 more)

### Community 108 - "JobSpec"
Cohesion: 0.25
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "PickerOverlay"
Cohesion: 0.09
Nodes (39): PickerOverlay, ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec() (+31 more)

### Community 110 - "String"
Cohesion: 0.15
Nodes (16): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), collapse_variable_path(), DapSessionInfo, DapThreadInfo, DapVariablePath, expand_variable_path() (+8 more)

### Community 111 - "user/git.rs"
Cohesion: 0.11
Nodes (46): commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title(), in_progress_section() (+38 more)

### Community 112 - "PickerItem"
Cohesion: 0.18
Nodes (7): match_item(), PickerItem, PickerMatch, Into, Option, String, picker_fringe_width_chars()

### Community 113 - "Option"
Cohesion: 0.11
Nodes (10): Option, Vec, terminal_render_snapshot_preserves_wide_character_widths(), terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot (+2 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - "TextPoint"
Cohesion: 0.11
Nodes (6): advance_point_by_text(), Self, Selection, TextPoint, UndoSnapshot, UndoTree

### Community 116 - "LspClientManager"
Cohesion: 0.09
Nodes (18): code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), formatting_parser_maps_text_edits(), is_copilot_server(), lsp_formatting_options(), lsp_text_edit_from_lsp(), LspClientManager, LspCodeAction (+10 more)

### Community 117 - "String"
Cohesion: 0.07
Nodes (55): ColumnData, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor (+47 more)

### Community 118 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 121 - "centered_rect"
Cohesion: 0.67
Nodes (3): centered_rect(), picker_card_rect(), PickerLayout

### Community 122 - "DapSessionHandle"
Cohesion: 0.10
Nodes (28): Arguments, attach_arguments(), DapLogDirection, DapReaderSession, DapSessionHandle, mark_session_ended(), parse_response_body(), PendingResponse (+20 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.11
Nodes (11): DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, normalize_extension(), BTreeMap, InstallRecipe, IntoIterator, Item (+3 more)

### Community 124 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 125 - "DbBrowserBufferView"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (45): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+37 more)

### Community 127 - "ROption"
Cohesion: 0.12
Nodes (13): GhostTextLine, GhostTextLine, exported_ghost_text_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AutocompleteProvider (+5 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.18
Nodes (24): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+16 more)

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "connect_transport"
Cohesion: 0.22
Nodes (10): configure_adapter_command(), connect_tcp(), connect_transport(), Child, Command, DebugAdapterSpec, DebugAdapterTransport, TcpStream (+2 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "Self"
Cohesion: 0.04
Nodes (73): DebugAdapterRootStrategy, exported_ligature_config(), exported_terminal_config(), AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiDebugAdapterRootStrategy (+65 more)

### Community 134 - "AbiStatuslineContext"
Cohesion: 0.31
Nodes (6): exported_statusline_render(), statusline_context_from_abi(), AbiLspDiagnosticsInfo, AbiStatuslineContext, LspDiagnosticsInfo, LspDiagnosticsInfo

### Community 135 - "AbiSectionTree"
Cohesion: 0.19
Nodes (9): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 138 - ".new"
Cohesion: 0.05
Nodes (53): path_to_file_url_encodes_spaces(), feature_spec(), BrowserFeatureSpec, help_entry(), ContextHelpEntry, hook_command(), package(), hook_command() (+45 more)

### Community 139 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 140 - "shell/acp.rs"
Cohesion: 0.03
Nodes (248): AcpClientConfig, AsyncRead, AvailableCommand, ChildStderr, ClientSideConnection, acp_complete_slash(), acp_connected(), acp_cycle_mode() (+240 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 143 - ".default"
Cohesion: 0.06
Nodes (36): CodeActionParams, Self, close_buffer_keeps_session_alive_for_next_file(), code_action_params(), code_action_params_use_flattened_lsp_shape(), definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), file_uri_roundtrip_handles_windows_paths() (+28 more)

### Community 144 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 145 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".load_from_path"
Cohesion: 0.09
Nodes (20): detect_preferred_line_ending(), from_reader_normalizes_crlf_and_tracks_line_endings(), LineEnding, must(), reload_from_path_requires_a_backing_file(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean(), AsRef (+12 more)

### Community 148 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 149 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 150 - ".push"
Cohesion: 0.26
Nodes (5): assert_control_requests_omit_nulls(), dap_log_text(), DapLogEntry, DapLogSnapshot, DapTransportLog

### Community 151 - ".new"
Cohesion: 0.15
Nodes (20): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_volt_state_dir(), insert_test_session(), redact_key_value_segments(), Arc, PathBuf, Self (+12 more)

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 153 - "user/db.rs"
Cohesion: 0.12
Nodes (27): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings() (+19 more)

### Community 154 - ".path"
Cohesion: 0.21
Nodes (12): db_connect_enter_submits_pasted_connection_string(), db_query_buffer_receives_sql_highlighting_without_blocking(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir() (+4 more)

### Community 155 - "LspNotification"
Cohesion: 0.06
Nodes (33): ChildStdin, completion_level_for_message(), diagnostic_matches_request_range(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel (+25 more)

### Community 156 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 157 - "PickerItemSpec"
Cohesion: 0.06
Nodes (72): acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items(), height_fraction() (+64 more)

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.12
Nodes (15): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+7 more)

### Community 160 - "editor-dap/src/client.rs"
Cohesion: 0.14
Nodes (42): client_initialize_launch_disconnect_against_fake_tcp_adapter(), continue_step_pause_and_locals_against_fake_adapter(), continue_to_process_exit_queues_terminated(), DapSessionEvent, debug_stop_after_attach_leaves_process_running(), expand_collapse_and_reapply_nested_locals_and_watches(), fake_adapter_loop(), fake_variables_for_reference() (+34 more)

### Community 161 - "AbiPaneConfig"
Cohesion: 0.06
Nodes (21): exported_pane_config(), MarkdownPrettyConfig, PickerLayout, ShowParenConfig, AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig, AbiPickerLayout (+13 more)

### Community 162 - "InstallRecipe"
Cohesion: 0.21
Nodes (10): github_release_builds_latest_download_url(), InstallRecipe, AsRef, Into, IntoIterator, Item, Option, Self (+2 more)

### Community 163 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

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

### Community 169 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 170 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 171 - "Vec"
Cohesion: 0.27
Nodes (10): autocomplete_items(), calculator_symbols(), CalculatorSymbol, hover_lines(), hover_topics(), initial_buffer_lines(), initial_buffer_lines_only_seed_input_examples(), AutocompleteProviderItem (+2 more)

### Community 172 - "user/dap.rs"
Cohesion: 0.20
Nodes (17): adapter_preferences_match_language_defaults(), codelldb_recipe(), debug_adapters(), debug_adapters_attach_typed_install_recipes(), install_recipe_for_debug_adapter(), locals_buffer_declares_locals_and_expressions_sections(), locals_sections(), package() (+9 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "AbiHoverProvider"
Cohesion: 0.29
Nodes (6): AbiHoverProvider, AbiHoverProviderTopic, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic

### Community 175 - "user/browser.rs"
Cohesion: 0.29
Nodes (8): buffer_lines(), buffer_lines_include_current_url_when_present(), input_hint(), package(), package_exports_browser_open_command(), Option, String, Vec

### Community 176 - "shim.rs"
Cohesion: 0.29
Nodes (17): candidate_names(), ensure_unix_executable(), finalize_install(), find_named_file(), find_named_file_inner(), resolve_binary(), Option, Path (+9 more)

### Community 177 - "PluginBuffer"
Cohesion: 0.07
Nodes (11): dashboard_sections(), sidebar_sections(), DbBrowserKind, plugin_buffer_sections_can_declare_nested_layout_tree(), PluginBuffer, PluginBufferLayout, PluginBufferLayoutAxis, PluginBufferLayoutNode (+3 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "String"
Cohesion: 0.05
Nodes (36): CopilotDeviceCodePrompt, documentation_lines(), hover_marked_string(), hover_marked_string_markdown_text(), hover_parser_formats_marked_string_language_blocks_as_markdown(), hover_parser_keeps_plaintext_markup_plain(), hover_parser_preserves_markdown_content(), hover_text() (+28 more)

### Community 180 - "aligned_indent_column"
Cohesion: 0.12
Nodes (23): aligned_indent_column(), current_line_starts_with_token(), delimiter_column(), evaluate_general_predicate(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column() (+15 more)

### Community 181 - "Result"
Cohesion: 0.18
Nodes (8): DisabledSecretStore, InMemorySecretStore, redact_error(), remembered_connections_store_metadata_separately_from_secret(), HashMap, Result, snippets_and_history_persist(), unix_epoch_secs()

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.18
Nodes (10): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+2 more)

### Community 185 - "WorkspaceDockConfig"
Cohesion: 0.29
Nodes (3): WorkspaceDockTestUserLibrary, WorkspaceDockConfig, WorkspaceDockSide

### Community 186 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 187 - "load_font_set_with_mode"
Cohesion: 0.08
Nodes (29): PathBuf, ThemeRuntimeSlots, EmojiFont, FontSet<'ttf>, FontSetInit, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode() (+21 more)

### Community 188 - "volt/src/main.rs"
Cohesion: 0.11
Nodes (28): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), format_micros_as_millis(), LaunchMode, LaunchOptions, LspState, parse_launch_options() (+20 more)

### Community 190 - "document_language_id_for_extension"
Cohesion: 0.40
Nodes (3): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path()

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
Cohesion: 0.11
Nodes (38): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_ranges_cover_quotes_and_brackets(), edits_since_returns_contiguous_forward_edits(), highlight_document_captures_edits_without_undo_history(), highlight_document_falls_back_to_full_parse_without_contiguous_edits(), line_ranges_and_char_searches_resolve_expected_points(), move_matching_delimiter_jumps_between_html_tags() (+30 more)

### Community 195 - "Vec"
Cohesion: 0.08
Nodes (14): CommandPaletteState, CompilationState, EventLog, AcpClient, AutocompleteProvider, ContextHelpSpec, GitStatusPrefix, HoverProvider (+6 more)

### Community 196 - "Path"
Cohesion: 0.13
Nodes (15): language_server_session_in_workspace_scope(), LspClientState, LspLiveSession, normalize_path_for_compare(), normalize_session_root(), path_equals_or_under(), BTreeSet, Path (+7 more)

### Community 198 - "connect_sql_server"
Cohesion: 0.50
Nodes (4): Compat, connect_sql_server(), TcpStream, SqlServerClient

### Community 199 - "Option"
Cohesion: 0.06
Nodes (16): build_csharp_fixture(), DapExecutionPosition, DapStackFrameInfo, DapStoppedSnapshot, DapVariableNode, DapVariableRow, DapWatchExpression, expand_variable_node() (+8 more)

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "normalize_inline_text"
Cohesion: 0.22
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 202 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 203 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 204 - "Option"
Cohesion: 0.07
Nodes (55): json_value_contains_null(), completion_documentation(), configuration_item_section(), copilot_status_notifications_offer_sign_in_action(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item(), file_uri_to_path() (+47 more)

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

### Community 211 - ".move_object_end_forward"
Cohesion: 0.13
Nodes (8): is_object_separator(), is_punctuation_char(), is_word_char(), matches_word_kind(), word_kind_ranges_cover_big_word_objects(), word_motion_class(), WordKind, WordMotionClass

### Community 212 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 214 - "ShellConfig"
Cohesion: 0.15
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 215 - "highlight.rs"
Cohesion: 0.39
Nodes (8): bench_highlight_rust(), bench_highlight_rust_window(), Language, String, rust_fixture(), rust_language(), rust_registry(), Criterion

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "main"
Cohesion: 0.12
Nodes (16): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), Arc, DebugAdapterSpec, Error (+8 more)

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 227 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 228 - "evaluate_expression"
Cohesion: 0.50
Nodes (4): DapEvaluateContext, evaluate_expression(), EvaluateArgumentsContext, From

### Community 229 - "handle_git_status_chord"
Cohesion: 0.48
Nodes (6): git_status_command_name(), GitPrefixState, handle_git_status_chord(), set_git_prefix(), take_git_prefix(), GitPrefix

### Community 232 - "AbiPdfOpenMode"
Cohesion: 0.15
Nodes (10): exported_pdf_open_mode(), exported_picker_truncate_strategy(), PdfOpenMode, PickerTruncateStrategy, AbiPdfOpenMode, AbiPickerTruncateStrategy, PdfOpenMode, PickerTruncateStrategy (+2 more)

### Community 234 - "String"
Cohesion: 0.09
Nodes (17): CaptureThemeMapping, cmake_configuration(), GrammarRecompileFailure, GrammarRecompileReport, LanguageConfiguration, LanguageLoader, normalize_extension(), normalize_unique_entries() (+9 more)

### Community 235 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 236 - "Option"
Cohesion: 0.11
Nodes (24): append_query_source(), compile_query_source(), DeferredQuery, html_language(), intern_query_captures(), intern_theme_token(), load_language(), LoadedLanguage (+16 more)

### Community 237 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 238 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 241 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

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
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_workspace_dock_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 265 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 335 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

## Knowledge Gaps
- **155 isolated node(s):** `BufferChrome<'a>`, `StartupProfile`, `topbar`, `navToggle`, `pageSidebar` (+150 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **34 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `set_directory_root`, `Option`, `ShellState`, `shell/tests.rs`, `shell/acp.rs`, `GitSummaryState`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `.path`, `command_stream.rs`, `String`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `Result`, `String`, `shell/browser.rs`, `Option`, `active_runtime_popup`, `shell/git.rs`, `BufferId`, `InputField`, `.new`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `tool_install.rs`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `main`, `GitEditorState`, `handle_git_status_chord`, `PickerOverlay`?**
  _High betweenness centrality (0.178) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `user/lib.rs`, `Self`, `hcl.rs`, `.new`, `java.rs`, `Self`, `kotlin.rs`, `xml.rs`, `latex.rs`, `r.rs`, `swift.rs`, `calculator.rs`, `oil.rs`, `user/db.rs`, `show_paren.rs`, `PickerItemSpec`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `lua.rs`, `package`, `user/terminal.rs`, `rainbow_parens.rs`, `user/dap.rs`, `sdk/src/lib.rs`, `user/browser.rs`, `PluginBuffer`, `HeaderlineTestUserLibrary`, `lsp.rs`, `Option`, `AcpPickerItemSpec`, `common.rs`, `PluginCommand`, `proto.rs`, `markdown.rs`, `lang/vim.rs`, `package`, `clojure.rs`, `package`, `elixir.rs`, `graphql.rs`, `editor-plugin-host/src/lib.rs`, `main`, `treesittercontext_ghosttext.rs`, `perl.rs`, `php.rs`, `PluginKeyBinding`, `nix.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `bash.rs`, `user/workspace_dock.rs`?**
  _High betweenness centrality (0.077) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `set_directory_root`, `Option`, `ShellState`, `render.rs`, `draw.rs`, `shell/acp.rs`, `GitSummaryState`, `state_with_user_library`, `shell/pdf.rs`, `shell/mod.rs`, `Result`, `shell/browser.rs`, `ShellError`, `Option`, `InputField`, `.new`, `.new`, `LineSyntaxSpan`, `buffer_footer_layout_with_command_line`, `directory.rs`, `shell/terminal.rs`, `diagnostics.rs`, `TextBuffer`, `PickerOverlay`, `Option`, `TextPoint`, `StoredBreakpoint`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **What connects `BufferChrome<'a>`, `StartupProfile`, `topbar` to the rest of the system?**
  _155 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `LspClientError` be split into smaller, more focused modules?**
  _Cohesion score 0.07758031442241968 - nodes in this community are weakly interconnected._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.06526806526806526 - nodes in this community are weakly interconnected._
- **Should `src/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.1395961369622476 - nodes in this community are weakly interconnected._