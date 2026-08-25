# Graph Report - volt  (2026-08-25)

## Corpus Check
- 259 files · ~647,523 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10736 nodes · 43928 edges · 315 communities (303 shown, 12 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3496 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `f1e68447`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- DebugConfiguration
- Path
- Option
- src/tests.rs
- ShellState
- user/lib.rs
- editor-syntax/src/lib.rs
- String
- LanguageServerSpec
- render.rs
- draw.rs
- editor-git/src/lib.rs
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
- paths.rs
- state_with_user_library
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- ThemeRegistry
- present_scene_to_canvas
- String
- Option
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- Option
- .len
- Option
- shell/browser.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- WorkspaceConfigurationValue
- Option
- DebugConfigurationCandidate
- Path
- HeaderlineTestUserLibrary
- PathBuf
- theme.rs
- lsp.rs
- ShellError
- .new
- DapClientManager
- KeymapError
- active_runtime_popup
- build_output.rs
- key_sequence.rs
- .get
- AcpManager
- PaneConfig
- ShellUiState
- SyntaxRegistry
- RVec
- AbiGitStatusSnapshot
- String
- .new
- DbBrowserBufferView
- LineSyntaxSpan
- PluginPackage
- aligned_indent_column
- acp_rendered_text_wrap_cols
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- Result
- Section
- repository_files.rs
- tool_install.rs
- InstallCommand
- RString
- editor-picker/src/lib.rs
- diagnostics.rs
- HighlightDocument
- resolve_picker_extra
- editor-plugin-host/src/lib.rs
- CommandSource
- Vec
- crates/editor-syntax/tests/registered_queries.rs
- workspace_nav.rs
- AbiLanguageConfiguration
- TextBuffer
- GitEditorState
- modeline.rs
- editor-lsp/src/lib.rs
- .spawn
- shell/tests.rs
- .from
- browser_host.rs
- shim.rs
- DapLogEntry
- TextPoint
- JobSpec
- shell/picker.rs
- DapSessionInfo
- .default
- AbiSectionTree
- build_job_command
- PluginCommand
- DebugSessionPlan
- LspCodeAction
- String
- From
- process_supervisor.rs
- Vec
- FontSet<'ttf>
- DapSessionHandle
- DebugAdapterSpec
- .move_object_end_forward
- DbService
- StoredBreakpoint
- Option
- run_job
- editor-terminal/src/lib.rs
- user/config.rs
- editor-dap/src/client.rs
- LanguageServerRegistry
- PickerItemSpec
- AbiAutocompleteProvider
- PickerSession
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- String
- AbiGitFeatureSpec
- load_user_library
- String
- CommandLineOverlay
- .next_token
- .new
- PickerOverlay
- AbiLanguageServerSpec
- volt/build.rs
- .load_from_path
- list_repository_files
- HighlightSpan
- cargo
- .new
- oil.rs
- user/db.rs
- .send
- LspSessionHandle
- show_paren.rs
- workspace.rs
- load
- Copilot instructions for `volt`
- .new
- AbiPaneConfig
- InstallRecipe
- package
- ServiceRegistry
- .request
- String
- user/terminal.rs
- corpus_inventory.rs
- RainbowParensConfig
- AbiSection
- editor-path/src/lib.rs
- user/dap.rs
- JobResult
- treesittercontext_ghosttext.rs
- user/browser.rs
- PickerItem
- connect_sql_server
- `user`
- PathBuf
- predicate_capture_text
- Result
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- SectionLineMeta
- shell/acp.rs
- DynamicUserLibrary
- volt/src/main.rs
- LspLogEntry
- flatten_config_select_options
- .new
- Database Explorer PRD
- centered_rect
- .from_text
- String
- GitStashEntry
- resolve_permission
- AbiIconFontCategory
- Option
- markdown.rs
- .byte_slice_chunks
- .terminal_output
- .fmt
- editor-lsp/src/client.rs
- 0004-markdown-pretty-pipeline.md
- DbEngine
- main
- .new
- dap-client-spec.md
- highlight.rs
- Language
- AbiContextHelpSpec
- Domain Docs
- Issue tracker: GitHub
- main
- AcpEvent
- rainbow_paren.rs
- evaluate_expression
- BufferId
- String
- .oil_directory_sections
- clipboard.rs
- UserLibraryModule
- .new
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- scala.rs
- Agent skills
- 0005-dap-session-and-client.md
- AbiLigatureConfig
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- ligatures.rs
- 0006-language-server-and-debug-adapter-install.md
- Self
- editor-core/src/lib.rs
- ShellConfig
- shell/git.rs
- buffer_footer_layout_with_command_line
- index_syntax_lines
- spawn_terminal_reader
- user/lang/kotlin.rs
- user/lang/lua.rs
- user/lang/perl.rs
- user/lang/php.rs
- user/lang/proto.rs
- user/lang/vim.rs
- WorkspaceDockConfig
- user/keymap.rs

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 882 edges
2. `shell_ui_mut()` - 390 edges
3. `ShellBuffer` - 389 edges
4. `register_shell_hooks()` - 274 edges
5. `shell_ui()` - 270 edges
6. `shell_buffer()` - 198 edges
7. `shell_buffer_mut()` - 196 edges
8. `ShellError` - 194 edges
9. `ShellUiState` - 189 edges
10. `ShellState` - 166 edges

## Surprising Connections (you probably didn't know these)
- `discover_projects()` --calls--> `workspace_project_picker_items()`  [INFERRED]
  crates/editor-fs/src/lib.rs → user/workspace.rs
- `discover_projects()` --calls--> `workspace_switch_picker_items()`  [INFERRED]
  crates/editor-fs/src/lib.rs → user/workspace.rs
- `workspace_project_picker_items()` --calls--> `project_discovery_snapshot()`  [INFERRED]
  user/workspace.rs → crates/editor-fs/src/project_discovery.rs
- `workspace_project_picker_items_keep_candidates_while_rescan_runs()` --calls--> `project_discovery_snapshot()`  [INFERRED]
  user/workspace.rs → crates/editor-fs/src/project_discovery.rs
- `workspace_project_picker_items_sort_open_workspace_first()` --calls--> `project_discovery_snapshot()`  [INFERRED]
  user/workspace.rs → crates/editor-fs/src/project_discovery.rs

## Import Cycles
- 2-file cycle: `crates/editor-tool-install/src/lib.rs -> crates/editor-tool-install/src/paths.rs -> crates/editor-tool-install/src/lib.rs`
- 2-file cycle: `crates/editor-render/src/lib.rs -> crates/editor-render/src/split_layout.rs -> crates/editor-render/src/lib.rs`

## Communities (315 total, 12 thin omitted)

### Community 0 - "DebugConfiguration"
Cohesion: 0.13
Nodes (10): DebugConfiguration, DebugRequestKind, Into, IntoIterator, Item, Iterator, Option, PathBuf (+2 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (24): inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, LspLocation, parse_definition_response() (+16 more)

### Community 2 - "Option"
Cohesion: 0.06
Nodes (70): begin_oil_worktree_request(), build_git_fringe_snapshot_with_cache(), build_git_summary_snapshot(), command_output_transcript(), create_git_worktree_from_query(), git_branch_list(), git_branch_merge(), git_branch_push_remote() (+62 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.12
Nodes (75): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), begin_project_discovery_test(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), discovery_fixture(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change() (+67 more)

### Community 4 - "ShellState"
Cohesion: 0.03
Nodes (62): clear_key_sequence(), active_runtime_surface(), ActiveTypingFrameProfile, alt_mod(), average_duration(), browser_devtools_shortcut_requested(), build_keydown_chord(), build_shell_summary() (+54 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.02
Nodes (128): bundled_highlight_query(), cached_syntax_languages(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_acp_picker_items() (+120 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.09
Nodes (71): B, additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+63 more)

### Community 7 - "String"
Cohesion: 0.06
Nodes (48): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, documentation_lines(), explicit_windows_env_value() (+40 more)

### Community 8 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (12): LanguageServerRootStrategy, LanguageServerSpec, normalize_unique_entries(), InstallRecipe, Into, IntoIterator, Item, Iterator (+4 more)

### Community 9 - "render.rs"
Cohesion: 0.04
Nodes (126): RenderColor, Self, acp_pane_content_rows(), advance_point_by_text(), FontSet, is_zero_width_display_character(), multicursor_selection_offsets(), Rect (+118 more)

### Community 10 - "draw.rs"
Cohesion: 0.08
Nodes (54): AcpBufferDraw, AcpPaneDraw, AcpPrefixDraw, BrowserBufferDraw, BrowserSyncView, BufferBodyPalette, BufferChrome, BufferChrome<'a> (+46 more)

### Community 11 - "editor-git/src/lib.rs"
Cohesion: 0.20
Nodes (30): cached_repository_file_listing_is_keyed_by_workspace_root(), cached_repository_file_listing_refreshes_after_index_or_head_change(), cached_repository_file_listing_reuses_paths_until_identity_changes(), configure_git_identity(), detect_in_progress(), git_available(), git_stdout(), GitStatusError (+22 more)

### Community 12 - "Self"
Cohesion: 0.03
Nodes (35): picker_items(), browser_item(), browser_items(), dashboard_sections(), default_action(), sidebar_sections(), exported_db_browser_items(), hook_command() (+27 more)

### Community 13 - "GitSummaryState"
Cohesion: 0.11
Nodes (13): git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, refresh_git_fringe(), refresh_pending_git_summary() (+5 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.16
Nodes (31): collect_configuration_candidates(), configuration_holes(), configuration_holes_detect_missing_launch_program(), DapConfigError, DebugInferContext, deep_inference_finds_cargo_binary_and_heuristic(), deep_inference_finds_dotnet_dll(), default_workspace_skips_deep_inference() (+23 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (25): AlacrittyEvent, Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc (+17 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.05
Nodes (92): Condvar, compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind (+84 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.10
Nodes (9): GitLogEntry, GitStatusSnapshot, RepositoryStatus, Into, Option, Self, String, Vec (+1 more)

### Community 18 - "editor-issues/src/lib.rs"
Cohesion: 0.06
Nodes (113): board_hides_closed_by_default(), board_issues(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), CaptureItem, CaptureReport (+105 more)

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
Cohesion: 0.13
Nodes (19): BindingKey, ChordModifier, KeyBinding, KeymapRegistry, KeymapScope, KeymapVimMode, normalize_chord(), normalize_chord_token() (+11 more)

### Community 23 - "calculator.rs"
Cohesion: 0.08
Nodes (32): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+24 more)

### Community 24 - "paths.rs"
Cohesion: 0.13
Nodes (25): acp_tool_kind_icon(), ToolKind, is_volt_install_path(), locate_program(), ProgramLocation, Path, PathBuf, apply_install_bins_to_process_path() (+17 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.05
Nodes (109): ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), browser_sync_plan_excludes_pdf_buffers(), browser_sync_plan_hides_surfaces_while_picker_is_visible() (+101 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.05
Nodes (80): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches() (+72 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.09
Nodes (25): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+17 more)

### Community 31 - "present_scene_to_canvas"
Cohesion: 0.09
Nodes (31): Canvas, DrawCommand, Arc, TextStyle, PathBuf, ThemeRuntimeSlots, acp_slice_chars(), cached_primary_text_runs() (+23 more)

### Community 32 - "String"
Cohesion: 0.06
Nodes (128): run_command(), active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer() (+120 more)

### Community 33 - "Option"
Cohesion: 0.09
Nodes (11): terminal_cursor_shape_for_input_mode(), Option, Vec, TerminalCursorShape, TerminalCursorSnapshot, TerminalRenderLine, TerminalRenderSnapshot, TerminalSnapshot (+3 more)

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
Nodes (388): EditorRuntime, Default, Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), activate_db_browser_line(), active_buffer_event_context() (+380 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (114): acp_output_header_title(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_tool_call_from_partial_update(), AcpPaneState, active_buffer_revision_key(), advance_markdown_table_insert_tab() (+106 more)

### Community 42 - ".len"
Cohesion: 0.06
Nodes (17): apply_input_operator_motion(), ascii_control_caret_notation(), byte_index_for_char_column(), display_columns_for_character(), format_undo_snapshot_diff(), input_charwise_motion_range(), InputField, is_wide_display_character() (+9 more)

### Community 43 - "Option"
Cohesion: 0.11
Nodes (49): apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), background_command_candidates(), background_command_names(), background_spawn_should_retry(), BackgroundCommandPipes (+41 more)

### Community 44 - "shell/browser.rs"
Cohesion: 0.09
Nodes (44): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len() (+36 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.08
Nodes (68): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+60 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (66): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, decode_modeline(), decode_modeline_segment() (+58 more)

### Community 47 - "WorkspaceConfigurationValue"
Cohesion: 0.12
Nodes (15): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, I (+7 more)

### Community 48 - "Option"
Cohesion: 0.09
Nodes (17): Diagnostic, DiagnosticSeverity, directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerSession, LspWorkspaceDiagnostic (+9 more)

### Community 49 - "DebugConfigurationCandidate"
Cohesion: 0.12
Nodes (12): DebugConfigurationCandidate, DebugConfigurationSource, DebugStartHistory, DebugStartRecord, default_request(), history_records_last_and_recent(), Into, Item (+4 more)

### Community 50 - "Path"
Cohesion: 0.08
Nodes (26): asset_path_from_parts(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), finalize_language_install_removes_compiler_sidecars(), GrammarSource, install_plan_compile_command_prefers_cpp_scanner(), install_plan_compile_command_uses_windows_msvc_for_c_scanner() (+18 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (60): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_input_prompt_text(), buffer_save_still_writes_when_format_on_save_fails(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji() (+52 more)

### Community 52 - "PathBuf"
Cohesion: 0.02
Nodes (151): BufferKind, NullUserLibrary, browser_state_for_kind(), default_vim_target(), is_issues_board_kind(), acp_decode_image(), active_directory_root(), active_shell_buffer_path() (+143 more)

### Community 53 - "theme.rs"
Cohesion: 0.13
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 54 - "lsp.rs"
Cohesion: 0.17
Nodes (23): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), clojure_lsp_recipe(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), install_recipe_for_language_server(), language_servers() (+15 more)

### Community 55 - "ShellError"
Cohesion: 0.13
Nodes (86): Display, Error, From, ShellError, render_browser_buffer_body(), adjust_color(), blend_color(), DrawTarget (+78 more)

### Community 56 - ".new"
Cohesion: 0.02
Nodes (181): ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter() (+173 more)

### Community 57 - "DapClientManager"
Cohesion: 0.16
Nodes (13): active_thread_id(), clear_stopped_snapshot(), connect_tcp(), DapClientError, DapClientManager, Display, Error, Formatter (+5 more)

### Community 58 - "KeymapError"
Cohesion: 0.25
Nodes (15): autocomplete_overrides_workspace_while_active(), dap_mode_overrides_global_f5_while_session_live(), duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeymapError, popup_mode_does_not_claim_workspace_dock_chords(), popup_overrides_workspace_and_global_while_active() (+7 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (54): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+46 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 62 - ".get"
Cohesion: 0.17
Nodes (19): column_is_numeric(), DbAutocompleteCandidate, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns() (+11 more)

### Community 63 - "AcpManager"
Cohesion: 0.13
Nodes (23): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_pick_session(), acp_set_mode(), AcpManager (+15 more)

### Community 64 - "PaneConfig"
Cohesion: 0.12
Nodes (9): exported_keymap_config(), exported_ligature_config(), KeymapConfig, LigatureConfig, config(), hook_command(), package(), package_exports_split_close_and_switch_commands() (+1 more)

### Community 65 - "ShellUiState"
Cohesion: 0.04
Nodes (57): active_lsp_code_action_range(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_window_id(), active_workspace_open_buffer_paths(), BufferViewState, close_buffer_immediate(), close_buffer_with_prompt() (+49 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.09
Nodes (50): SyntaxText, buffer_text_for_byte_range(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser(), DeferredQuery, desired_indent_for_loaded_language() (+42 more)

### Community 67 - "RVec"
Cohesion: 0.18
Nodes (10): AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic, RVec (+2 more)

### Community 68 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 69 - "String"
Cohesion: 0.03
Nodes (178): request_browser_buffer_navigation(), shell_ui(), buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker() (+170 more)

### Community 70 - ".new"
Cohesion: 0.10
Nodes (36): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_head_blob_cache_reuses_text_for_same_head(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command() (+28 more)

### Community 71 - "DbBrowserBufferView"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (47): browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines(), db_results_table_row_spans() (+39 more)

### Community 73 - "PluginPackage"
Cohesion: 0.02
Nodes (162): file_open_package(), package(), bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration (+154 more)

### Community 74 - "aligned_indent_column"
Cohesion: 0.18
Nodes (14): aligned_indent_column(), collect_structure_nodes(), current_line_starts_with_token(), delimiter_column(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column() (+6 more)

### Community 75 - "acp_rendered_text_wrap_cols"
Cohesion: 0.33
Nodes (6): acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), acp_chat_bubble_width_px(), acp_chat_origin_x(), acp_prefix_columns(), acp_spinner_frame()

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (71): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+63 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (35): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+27 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.07
Nodes (72): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+64 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "Result"
Cohesion: 0.06
Nodes (112): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+104 more)

### Community 82 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 83 - "repository_files.rs"
Cohesion: 0.21
Nodes (27): cache_key(), CachedRepoFileList, default_worktree_common_dir(), file_fingerprint(), FileFingerprint, invalidate_repository_file_list_cache_for(), lock_cache(), normalize_path() (+19 more)

### Community 84 - "tool_install.rs"
Cohesion: 0.19
Nodes (40): apply_tool_install_finish(), begin_explicit_install(), continue_tool_install(), fail_tool_install(), fail_tool_install_with_message(), handle_dap_install_hook(), handle_lsp_install_hook(), install_debug_adapter_by_id() (+32 more)

### Community 85 - "InstallCommand"
Cohesion: 0.12
Nodes (26): Display, Error, From, Self, String, ToolInstallError, program_is_available(), archive_commands() (+18 more)

### Community 86 - "RString"
Cohesion: 0.11
Nodes (17): AbiAcpClient, AbiColor, AbiStringPair, AbiTheme, AbiThemeOption, AbiThemeOptionEntry, AbiThemeToken, AcpClient (+9 more)

### Community 87 - "editor-picker/src/lib.rs"
Cohesion: 0.15
Nodes (18): best_contiguous_substring_bonus(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), is_match_boundary(), is_match_end_boundary() (+10 more)

### Community 88 - "diagnostics.rs"
Cohesion: 0.14
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "HighlightDocument"
Cohesion: 0.18
Nodes (3): BufferStats, HighlightDocument, Vec

### Community 90 - "resolve_picker_extra"
Cohesion: 0.14
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.15
Nodes (34): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+26 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "Vec"
Cohesion: 0.12
Nodes (8): EventLog, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, String, Vec, WorkspaceRoot

### Community 94 - "crates/editor-syntax/tests/registered_queries.rs"
Cohesion: 0.15
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 97 - "TextBuffer"
Cohesion: 0.09
Nodes (13): delimiter_partner(), EditRecord, large_buffers_expose_line_windows_without_full_materialization(), parse_tag_token_at(), Default, Option, String, ShowParenMatch (+5 more)

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
Cohesion: 0.11
Nodes (21): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+13 more)

### Community 102 - "shell/tests.rs"
Cohesion: 0.03
Nodes (55): active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), configure_file_buffer(), contextual_ligature_raster_size_keeps_changed_glyphs_at_base_size() (+47 more)

### Community 103 - ".from"
Cohesion: 0.13
Nodes (21): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), normalize_window_blur(), abi_debug_adapter_spec_round_trips_install_recipe(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_install_recipe(), abi_language_server_spec_round_trips_path_matchers() (+13 more)

### Community 104 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 105 - "shim.rs"
Cohesion: 0.29
Nodes (17): candidate_names(), ensure_unix_executable(), finalize_install(), find_named_file(), find_named_file_inner(), resolve_binary(), Option, Path (+9 more)

### Community 106 - "DapLogEntry"
Cohesion: 0.26
Nodes (6): assert_control_requests_omit_nulls(), dap_log_text(), DapLogDirection, DapLogEntry, DapLogSnapshot, DapTransportLog

### Community 107 - "TextPoint"
Cohesion: 0.07
Nodes (13): advance_point_by_text(), delimited_and_tag_ranges_cover_quickref_objects(), find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), Fn, Self (+5 more)

### Community 108 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (38): ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_entries() (+30 more)

### Community 111 - ".default"
Cohesion: 0.11
Nodes (48): Self, session_labels_ignore_stale_tracked_session_keys(), commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids() (+40 more)

### Community 112 - "AbiSectionTree"
Cohesion: 0.16
Nodes (11): exported_git_status_sections(), exported_oil_directory_sections(), AbiDirectoryEntry, AbiOilSortMode, AbiSectionTree, DirectoryEntry, OilSortMode, DirectoryEntry (+3 more)

### Community 113 - "build_job_command"
Cohesion: 0.32
Nodes (8): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, windows_fnm_environment(), configure_background_command(), Command

### Community 114 - "PluginCommand"
Cohesion: 0.08
Nodes (37): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+29 more)

### Community 115 - "DebugSessionPlan"
Cohesion: 0.22
Nodes (3): DebugAdapterTransport, DebugSessionPlan, DapState

### Community 116 - "LspCodeAction"
Cohesion: 0.10
Nodes (9): lsp_code_action_diagnostic(), lsp_diagnostic_severity(), LspCodeAction, LspDocumentTextEdits, LspTextEdit, Error, LspDiagnostic, LspDiagnosticSeverity (+1 more)

### Community 117 - "String"
Cohesion: 0.07
Nodes (55): ColumnData, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor (+47 more)

### Community 118 - "From"
Cohesion: 0.13
Nodes (17): DebugAdapterRootStrategy, AbiDebugAdapterRootStrategy, AbiDebugAdapterSpec, AbiDebugAdapterTransport, AbiDebugAdapterTransportKind, AbiLspDiagnosticsInfo, AbiPickerTruncateStrategy, AbiStatuslineContext (+9 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 121 - "FontSet<'ttf>"
Cohesion: 0.16
Nodes (7): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, RasterFont, ShapeFace, Font

### Community 122 - "DapSessionHandle"
Cohesion: 0.11
Nodes (33): DapReaderSession, DapSessionHandle, fake_adapter_loop(), fake_variables_for_reference(), mark_session_ended(), PendingResponse, read_frame(), record_transport_event_inner() (+25 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.09
Nodes (22): Client, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, gdb(), must() (+14 more)

### Community 124 - ".move_object_end_forward"
Cohesion: 0.13
Nodes (8): is_object_separator(), is_punctuation_char(), is_word_char(), matches_word_kind(), word_kind_ranges_cover_big_word_objects(), word_motion_class(), WordKind, WordMotionClass

### Community 125 - "DbService"
Cohesion: 0.13
Nodes (15): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbIndex, DbQueryBufferMeta, DbService (+7 more)

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (46): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+38 more)

### Community 127 - "Option"
Cohesion: 0.15
Nodes (7): CommandPaletteState, CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 128 - "run_job"
Cohesion: 0.15
Nodes (18): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+10 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.14
Nodes (23): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+15 more)

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "editor-dap/src/client.rs"
Cohesion: 0.09
Nodes (30): attach_arguments(), build_csharp_fixture(), configure_adapter_command(), connect_transport(), DapExecutionPosition, DapSessionEvent, DapStackFrameInfo, DapThreadInfo (+22 more)

### Community 132 - "LanguageServerRegistry"
Cohesion: 0.17
Nodes (8): LanguageServerRegistry, LspError, Display, Error, Formatter, Result, Vec, normalize_extension()

### Community 133 - "PickerItemSpec"
Cohesion: 0.07
Nodes (43): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+35 more)

### Community 134 - "AbiAutocompleteProvider"
Cohesion: 0.29
Nodes (6): AbiAutocompleteProvider, AbiAutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem

### Community 135 - "PickerSession"
Cohesion: 0.16
Nodes (6): contiguous_substring_beats_split_path_match(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), PickerSession, result_limit_caps_large_match_sets(), Vec

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "String"
Cohesion: 0.25
Nodes (20): FpsOverlaySnapshot, file_name_with_parent(), fit_picker_label_after_transform(), format_fps_overlay_text(), is_path_like(), join_path_segments(), parent_initial_file_name(), path_segments() (+12 more)

### Community 138 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 139 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 140 - "String"
Cohesion: 0.08
Nodes (47): acp_connected(), acp_image_mention_token(), acp_open_permission_request(), acp_permission_picker_closed(), acp_permission_picker_submitted(), acp_resolve_permission_option(), acp_session_buffer_name(), acp_set_model() (+39 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 143 - ".new"
Cohesion: 0.13
Nodes (44): attach_session(), close_buffer_keeps_session_alive_for_next_file(), close_then_open_then_incremental_edits_work_again(), did_open_still_sends_full_text(), file_uri_roundtrip_handles_windows_paths(), full_document_range(), full_sync_sends_null_range_and_full_text_even_with_edits(), incremental_did_change_emits_one_event_per_contiguous_edit() (+36 more)

### Community 144 - "PickerOverlay"
Cohesion: 0.04
Nodes (32): absolute_path_hint(), buffer_is_quickfix(), GitBranchActionKind, GitCommitActionKind, index_syntax_lines_with_rainbow_parens(), lsp_session_lifecycle_picker_entry(), lsp_session_lifecycle_picker_overlay(), LspSessionPickerAction (+24 more)

### Community 145 - "AbiLanguageServerSpec"
Cohesion: 0.17
Nodes (10): AbiInstallRecipe, AbiInstallRecipeKind, AbiLanguageServerRootStrategy, AbiLanguageServerSpec, InstallRecipe, LanguageServerRootStrategy, LanguageServerSpec, InstallRecipe (+2 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".load_from_path"
Cohesion: 0.08
Nodes (20): detect_preferred_line_ending(), from_reader_normalizes_crlf_and_tracks_line_endings(), LineEnding, must(), normalize_inline_text(), reload_from_path_returns_false_when_disk_state_is_unchanged(), AsRef, Drop (+12 more)

### Community 148 - "list_repository_files"
Cohesion: 0.17
Nodes (12): configure_background_command(), RepositoryFilesError, Command, Display, Formatter, list_generation(), list_repository_files(), list_repository_files_uncached() (+4 more)

### Community 149 - "HighlightSpan"
Cohesion: 0.16
Nodes (13): apply_text_edits_to_span(), HighlightSpan, InjectionHighlights, InjectionRegion, intern_query_captures(), intern_theme_token(), Arc, BTreeMap (+5 more)

### Community 150 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 151 - ".new"
Cohesion: 0.15
Nodes (20): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_volt_state_dir(), insert_test_session(), redact_key_value_segments(), Arc, PathBuf, Self (+12 more)

### Community 152 - "oil.rs"
Cohesion: 0.05
Nodes (52): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+44 more)

### Community 153 - "user/db.rs"
Cohesion: 0.15
Nodes (21): browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings(), engine_icon(), feature_spec() (+13 more)

### Community 154 - ".send"
Cohesion: 0.11
Nodes (43): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+35 more)

### Community 155 - "LspSessionHandle"
Cohesion: 0.06
Nodes (41): ChildStdin, CodeActionParams, TextEdit, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), full_sync_uses_null_range_change(), incremental_content_changes() (+33 more)

### Community 156 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 157 - "workspace.rs"
Cohesion: 0.11
Nodes (49): PickerWorkspaceContext, begin_discovery_override(), discovery_test_lock(), DiscoveryOverrideGuard, existing_workspace_for_project(), git_available(), message_item(), override_project_search_roots_for_test() (+41 more)

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.12
Nodes (15): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+7 more)

### Community 160 - ".new"
Cohesion: 0.31
Nodes (17): client_initialize_launch_disconnect_against_fake_tcp_adapter(), continue_step_pause_and_locals_against_fake_adapter(), continue_to_process_exit_queues_terminated(), debug_stop_after_attach_leaves_process_running(), expand_collapse_and_reapply_nested_locals_and_watches(), live_toggle_calls_set_breakpoints(), missing_adapter_binary_is_clear(), one_session_per_workspace_enforced() (+9 more)

### Community 161 - "AbiPaneConfig"
Cohesion: 0.10
Nodes (16): AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig, AbiPickerLayout, AbiShowParenConfig, AbiWorkspaceDockSide, fraction_to_hundredths(), hundredths_to_fraction() (+8 more)

### Community 162 - "InstallRecipe"
Cohesion: 0.21
Nodes (10): github_release_builds_latest_download_url(), InstallRecipe, AsRef, Into, IntoIterator, Item, Option, Self (+2 more)

### Community 163 - "package"
Cohesion: 0.28
Nodes (7): exported_pdf_open_mode(), PdfOpenMode, open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - ".request"
Cohesion: 0.40
Nodes (4): Arguments, parse_response_body(), strip_null_fields(), Response

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "RainbowParensConfig"
Cohesion: 0.22
Nodes (5): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget(), RainbowParensConfig

### Community 170 - "AbiSection"
Cohesion: 0.19
Nodes (9): AbiSection, AbiSectionAction, AbiSectionItem, Section, SectionAction, SectionItem, Section, SectionAction (+1 more)

### Community 171 - "editor-path/src/lib.rs"
Cohesion: 0.11
Nodes (22): contains_wildcards(), glob_literal_count(), glob_matches(), grammar_install_root(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher (+14 more)

### Community 172 - "user/dap.rs"
Cohesion: 0.20
Nodes (17): adapter_preferences_match_language_defaults(), codelldb_recipe(), debug_adapters(), debug_adapters_attach_typed_install_recipes(), install_recipe_for_debug_adapter(), locals_buffer_declares_locals_and_expressions_sections(), locals_sections(), package() (+9 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "treesittercontext_ghosttext.rs"
Cohesion: 0.07
Nodes (48): packages(), LanguageConfiguration, Vec, syntax_languages(), ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery (+40 more)

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "PickerItem"
Cohesion: 0.23
Nodes (5): match_item(), PickerItem, PickerMatch, Option, picker_fringe_width_chars()

### Community 177 - "connect_sql_server"
Cohesion: 0.50
Nodes (4): Compat, connect_sql_server(), TcpStream, SqlServerClient

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "PathBuf"
Cohesion: 0.17
Nodes (14): diagnostics_parser_maps_lsp_fields(), file_uri_to_path(), language_server_session_in_workspace_scope(), LspClientState, LspLiveSession, normalize_path_for_compare(), normalize_session_root(), parse_publish_diagnostics() (+6 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.27
Nodes (11): evaluate_general_predicate(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches(), predicate_capture_node() (+3 more)

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

### Community 185 - "SectionLineMeta"
Cohesion: 0.29
Nodes (13): cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), GitStatusCommandContext, open_git_commit_picker_with_action() (+5 more)

### Community 186 - "shell/acp.rs"
Cohesion: 0.10
Nodes (35): acp_complete_slash(), acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_insert_file_mention(), acp_insert_slash_command(), acp_pick_mode(), acp_pick_model() (+27 more)

### Community 187 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (26): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DirectoryEntry, GhostTextLine (+18 more)

### Community 188 - "volt/src/main.rs"
Cohesion: 0.11
Nodes (28): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), format_micros_as_millis(), LaunchMode, LaunchOptions, LspState, parse_launch_options() (+20 more)

### Community 189 - "LspLogEntry"
Cohesion: 0.10
Nodes (12): last_did_open_text(), last_notification_params(), LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot (+4 more)

### Community 190 - "flatten_config_select_options"
Cohesion: 0.27
Nodes (10): config_option_is_mode(), config_option_is_model(), config_option_matches(), flatten_config_select_options(), session_mode_state_from_config(), session_model_state_from_config(), SessionConfigOption, SessionConfigSelectOption (+2 more)

### Community 191 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "centered_rect"
Cohesion: 0.67
Nodes (3): centered_rect(), picker_card_rect(), PickerLayout

### Community 194 - ".from_text"
Cohesion: 0.12
Nodes (35): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_ranges_cover_quotes_and_brackets(), edits_since_returns_contiguous_forward_edits(), highlight_document_captures_edits_without_undo_history(), highlight_document_falls_back_to_full_parse_without_contiguous_edits(), line_ranges_and_char_searches_resolve_expected_points(), move_matching_delimiter_jumps_between_html_tags() (+27 more)

### Community 195 - "String"
Cohesion: 0.42
Nodes (3): Into, Self, String

### Community 196 - "GitStashEntry"
Cohesion: 0.33
Nodes (3): GitStashEntry, parse_stash_list(), parses_stash_list_entries()

### Community 197 - "resolve_permission"
Cohesion: 0.40
Nodes (3): acp_permission_approve(), acp_permission_deny(), resolve_permission()

### Community 198 - "AbiIconFontCategory"
Cohesion: 0.60
Nodes (3): AbiIconFontCategory, IconFontCategory, IconFontCategory

### Community 199 - "Option"
Cohesion: 0.07
Nodes (25): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), collapse_variable_path(), DapStoppedSnapshot, DapVariableNode, DapVariablePath, DapVariableRow (+17 more)

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - ".byte_slice_chunks"
Cohesion: 0.31
Nodes (6): Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 202 - ".terminal_output"
Cohesion: 0.50
Nodes (3): apply_output_limit(), TerminalOutputRequest, TerminalOutputResponse

### Community 204 - "editor-lsp/src/client.rs"
Cohesion: 0.03
Nodes (119): BufRead, ClientCapabilities, char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_documentation() (+111 more)

### Community 209 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbSnippet, load_persisted_state(), PersistedDbState, RememberedConnection, Path

### Community 210 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 211 - ".new"
Cohesion: 0.04
Nodes (57): package(), help_entry(), ContextHelpEntry, package(), package_exports_image_commands(), package_exports_image_keybindings(), hook_command(), package() (+49 more)

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 215 - "highlight.rs"
Cohesion: 0.39
Nodes (8): bench_highlight_rust(), bench_highlight_rust_window(), Language, String, rust_fixture(), rust_language(), rust_registry(), Criterion

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "main"
Cohesion: 0.13
Nodes (15): bootstrap(), HostBootstrap, command_palette_items(), main(), panic_payload_message(), Any, Box, DebugAdapterSpec (+7 more)

### Community 221 - "AcpEvent"
Cohesion: 0.09
Nodes (30): AvailableCommand, AcpEvent, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), command_input_hint(), drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work() (+22 more)

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 228 - "evaluate_expression"
Cohesion: 0.47
Nodes (4): DapEvaluateContext, evaluate_expression(), EvaluateArgumentsContext, From

### Community 229 - "BufferId"
Cohesion: 0.14
Nodes (23): ActiveBufferEventContext, fetch_git_pushremote(), fetch_git_remote(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_remote_list(), git_snapshot_for_buffer(), open_git_cherry_buffer() (+15 more)

### Community 234 - "String"
Cohesion: 0.08
Nodes (21): append_query_source(), CaptureThemeMapping, command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport, LanguageConfiguration, LanguageLoader, load_language() (+13 more)

### Community 237 - ".oil_directory_sections"
Cohesion: 0.29
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 242 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 243 - "UserLibraryModule"
Cohesion: 0.12
Nodes (16): AbiGhostTextContext, AbiIconFontSymbol, AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeybindings, IconFontSymbol, OilDefaults, OilFeatureSpec (+8 more)

### Community 244 - ".new"
Cohesion: 0.26
Nodes (13): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items_mark_current_models(), picker_items_preserve_slash_command_labels() (+5 more)

### Community 248 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 266 - "AbiLigatureConfig"
Cohesion: 0.60
Nodes (3): AbiLigatureConfig, LigatureConfig, LigatureConfig

### Community 340 - "Self"
Cohesion: 0.11
Nodes (17): AbiDirectoryEntryKind, AbiKeymapConfig, AbiOilKeyAction, AbiPdfOpenMode, AbiWorkspaceRoot, DirectoryEntryKind, KeymapConfig, OilKeyAction (+9 more)

### Community 347 - "editor-core/src/lib.rs"
Cohesion: 0.15
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 354 - "ShellConfig"
Cohesion: 0.15
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 386 - "shell/git.rs"
Cohesion: 0.07
Nodes (51): apply_git_fringe_hunk(), classify_head_blob(), FringeDiffOp, git_fringe_snapshot_from_texts(), git_fringe_snapshot_from_texts_ignores_crlf_only_difference(), git_fringe_snapshot_from_texts_is_empty_when_identical(), git_fringe_snapshot_from_texts_marks_all_lines_added_without_head(), git_fringe_snapshot_from_texts_marks_inserted_line_added() (+43 more)

### Community 409 - "buffer_footer_layout_with_command_line"
Cohesion: 0.08
Nodes (42): browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Rect (+34 more)

### Community 410 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 411 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 467 - "user/lang/kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 468 - "user/lang/lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 469 - "user/lang/perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 470 - "user/lang/php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 471 - "user/lang/proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 474 - "user/lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 476 - "WorkspaceDockConfig"
Cohesion: 0.18
Nodes (9): WorkspaceDockTestUserLibrary, WorkspaceDockConfig, WorkspaceDockSide, config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_workspace_dock_scope(), package_exports_dock_navigation_commands() (+1 more)

## Knowledge Gaps
- **155 isolated node(s):** `BufferChrome<'a>`, `StartupProfile`, `topbar`, `navToggle`, `pageSidebar` (+150 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `Option`, `shell/git.rs`, `ShellState`, `String`, `GitSummaryState`, `PickerOverlay`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `command_stream.rs`, `String`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `.len`, `Option`, `shell/browser.rs`, `PathBuf`, `.new`, `SectionLineMeta`, `shell/acp.rs`, `active_runtime_popup`, `AcpManager`, `ShellUiState`, `resolve_permission`, `.new`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `Result`, `tool_install.rs`, `editor-plugin-host/src/lib.rs`, `editor-core/src/lib.rs`, `CommandSource`, `AcpEvent`, `main`, `GitEditorState`, `BufferId`, `shell/tests.rs`, `shell/picker.rs`?**
  _High betweenness centrality (0.138) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `shell/git.rs`, `ShellState`, `render.rs`, `draw.rs`, `GitSummaryState`, `PickerOverlay`, `buffer_footer_layout_with_command_line`, `state_with_user_library`, `Option`, `shell/pdf.rs`, `shell/mod.rs`, `.len`, `shell/browser.rs`, `PathBuf`, `ShellError`, `.new`, `shell/acp.rs`, `ShellUiState`, `String`, `.new`, `LineSyntaxSpan`, `directory.rs`, `shell/terminal.rs`, `Result`, `diagnostics.rs`, `TextBuffer`, `shell/picker.rs`, `StoredBreakpoint`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `user/lib.rs`, `PickerItemSpec`, `Self`, `calculator.rs`, `oil.rs`, `user/db.rs`, `show_paren.rs`, `workspace.rs`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `package`, `user/terminal.rs`, `RainbowParensConfig`, `user/dap.rs`, `treesittercontext_ghosttext.rs`, `user/browser.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `lsp.rs`, `.new`, `PaneConfig`, `markdown.rs`, `.new`, `user/lang/kotlin.rs`, `user/lang/lua.rs`, `user/lang/perl.rs`, `user/lang/php.rs`, `user/lang/proto.rs`, `user/lang/vim.rs`, `editor-plugin-host/src/lib.rs`, `main`, `WorkspaceDockConfig`, `PluginCommand`, `UserLibraryModule`, `.new`, `scala.rs`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **What connects `BufferChrome<'a>`, `StartupProfile`, `topbar` to the rest of the system?**
  _155 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `DebugConfiguration` be split into smaller, more focused modules?**
  _Cohesion score 0.13306451612903225 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.07612509534706331 - nodes in this community are weakly interconnected._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.05837837837837838 - nodes in this community are weakly interconnected._