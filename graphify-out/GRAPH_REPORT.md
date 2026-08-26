# Graph Report - volt  (2026-08-26)

## Corpus Check
- 263 files · ~655,315 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10983 nodes · 44903 edges · 336 communities (324 shown, 12 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3582 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `cdaabca5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- DebugConfiguration
- Path
- Option
- .new
- ShellUiState
- user/lib.rs
- editor-syntax/src/lib.rs
- String
- LanguageServerSpec
- render.rs
- draw.rs
- editor-git/src/lib.rs
- sdk/src/lib.rs
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
- FontSet
- .path
- String
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- Option
- .len
- shell/acp.rs
- .new
- editor-markdown/src/lib.rs
- user/workspace_dock.rs
- WorkspaceConfigurationValue
- Diagnostic
- DebugConfigurationCandidate
- Path
- HeaderlineTestUserLibrary
- Self
- String
- lsp.rs
- ShellError
- .new
- DapClientManager
- probe.rs
- active_runtime_popup
- build_output.rs
- key_sequence.rs
- DbService
- String
- PluginCommand
- BufferId
- SyntaxRegistry
- Section
- AbiGitStatusSnapshot
- Result
- .new
- .new
- LineSyntaxSpan
- PluginPackage
- common.rs
- theme.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- shell_buffer
- .get
- repository_files.rs
- tool_install.rs
- InstallCommand
- editor-lsp/src/client.rs
- editor-picker/src/lib.rs
- LineWrapSegment
- PluginBufferSection
- resolve_picker_extra
- editor-plugin-host/src/lib.rs
- CommandSource
- Vec
- registered_queries.rs
- workspace_nav.rs
- AbiLanguageConfiguration
- FontSet<'ttf>
- GitEditorState
- modeline.rs
- editor-lsp/src/lib.rs
- browser_host.rs
- shell/tests.rs
- .from
- TextSnapshot
- shim.rs
- git_probe_snapshot
- .spawn
- JobSpec
- shell/picker.rs
- idle.rs
- .default
- AbiGitFeatureSpec
- build_job_command
- PluginKeyBinding
- DebugSessionPlan
- LspCodeAction
- editor-db/src/lib.rs
- TextBuffer
- process_supervisor.rs
- Vec
- shell/browser.rs
- DapSessionHandle
- DebugAdapterSpec
- Option
- String
- StoredBreakpoint
- String
- JobError
- Option
- user/config.rs
- editor-dap/src/client.rs
- LanguageServerRegistry
- PickerItemSpec
- ROption
- .recompute_matches
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- editor-terminal/src/lib.rs
- git_worktree_dashboard_picker_overlay
- load_user_library
- .new
- CommandLineOverlay
- Result
- .new
- String
- treesitter_install.rs
- volt/build.rs
- .load_from_path
- PickerSession
- String
- syntax_indent_for_buffer
- aligned_indent_column
- oil.rs
- user/db.rs
- .send
- PathBuf
- show_paren.rs
- workspace.rs
- load
- Copilot instructions for `volt`
- .new
- AbiPaneConfig
- InstallRecipe
- package
- ServiceRegistry
- browser_sync_plan
- String
- user/terminal.rs
- corpus_inventory.rs
- DapLogEntry
- shell/workspace_dock.rs
- editor-path/src/lib.rs
- user/dap.rs
- JobResult
- GhostTextContext
- user/browser.rs
- predicate_capture_text
- editor-icons/src/lib.rs
- `user`
- DynamicUserLibrary
- HighlightSpan
- open_slash_command_picker
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- treesittercontext_ghosttext.rs
- AcpManager
- treesittercontext_shared.rs
- volt/src/main.rs
- LspLogEntry
- TerminalCursorSnapshot
- .path
- Database Explorer PRD
- AbiSectionTree
- .from_text
- PickerItem
- cmake.rs
- resolve_permission
- clojure.rs
- Option
- markdown.rs
- Self
- bash.rs
- ancestor_contexts_for_cursor
- Option
- elixir.rs
- TerminalEventWake
- java.rs
- 0004-markdown-pretty-pipeline.md
- graphql.rs
- main
- capture_mappings
- hcl.rs
- dap-client-spec.md
- kotlin.rs
- highlight.rs
- Language
- Option
- Domain Docs
- Issue tracker: GitHub
- main
- AcpEvent
- lua.rs
- nix.rs
- latex.rs
- rainbow_paren.rs
- index_syntax_lines
- evaluate_expression
- BufferId
- perl.rs
- php.rs
- panic_payload_message
- scala.rs
- String
- AbiPickerTruncateStrategy
- UserLibraryModule
- .oil_directory_sections
- LspFormattingOptions
- rainbow_parens.rs
- syntax_language
- syntax_language
- clipboard.rs
- proto.rs
- AcpPickerItemSpec
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- .request
- .workspaces
- Agent skills
- syntax_language
- r.rs
- VimActionContext
- 0005-dap-session-and-client.md
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- ruby.rs
- solidity.rs
- swift.rs
- lang/vim.rs
- DapSessionInfo
- ligatures.rs
- 0006-language-server-and-debug-adapter-install.md
- Self
- syntax_languages
- .terminal_output
- package
- editor-core/src/lib.rs
- shell/git.rs
- buffer_footer_layout_with_command_line
- user/keymap.rs

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 884 edges
2. `ShellBuffer` - 393 edges
3. `shell_ui_mut()` - 393 edges
4. `register_shell_hooks()` - 274 edges
5. `shell_ui()` - 271 edges
6. `shell_buffer()` - 207 edges
7. `shell_buffer_mut()` - 202 edges
8. `ShellError` - 197 edges
9. `ShellUiState` - 194 edges
10. `ShellState` - 169 edges

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

## Communities (336 total, 12 thin omitted)

### Community 0 - "DebugConfiguration"
Cohesion: 0.13
Nodes (10): DebugConfiguration, DebugRequestKind, Into, IntoIterator, Item, Iterator, Option, PathBuf (+2 more)

### Community 1 - "Path"
Cohesion: 0.07
Nodes (28): inline_completion_params(), is_copilot_server(), LspClientError, LspClientManager, LspSessionHandle, parse_definition_response(), parse_text_edit_response(), path_to_uri() (+20 more)

### Community 2 - "Option"
Cohesion: 0.08
Nodes (51): begin_oil_worktree_request(), build_git_fringe_snapshot_with_cache(), build_git_summary_snapshot(), classify_head_blob(), command_output_transcript(), git_branch_list(), git_branch_merge(), git_branch_push_remote() (+43 more)

### Community 3 - ".new"
Cohesion: 0.10
Nodes (84): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), begin_project_discovery_test(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), discovery_fixture(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change() (+76 more)

### Community 4 - "ShellUiState"
Cohesion: 0.02
Nodes (96): clear_key_sequence(), active_buffer_event_context(), active_lsp_workspace_loaded(), active_runtime_surface(), ActiveTypingFrameProfile, alt_mod(), apply_lsp_notifications(), apply_pending_lsp_state() (+88 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (105): bundled_highlight_query(), cached_syntax_languages(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+97 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.09
Nodes (78): B, additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+70 more)

### Community 7 - "String"
Cohesion: 0.05
Nodes (79): shell_ui(), acp_second_escape_returns_hjkl_and_visual_mode_to_output_buffer(), acp_switch_pane_command_changes_internal_pane_without_changing_workspace_pane(), browser_buffer_submit_tracks_requested_navigation(), browser_escape_from_insert_keeps_input_cursor_position(), browser_host_focus_parent_event_returns_to_normal_mode(), browser_host_new_window_event_routes_into_browser_popup(), browser_host_open_devtools_event_is_ignored_without_a_live_webview() (+71 more)

### Community 8 - "LanguageServerSpec"
Cohesion: 0.10
Nodes (13): LanguageServerRootStrategy, LanguageServerSpec, normalize_unique_entries(), InstallRecipe, Into, IntoIterator, Item, Iterator (+5 more)

### Community 9 - "render.rs"
Cohesion: 0.04
Nodes (126): WrapCollect, acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), resolved_tab_width(), acp_bubble_remaining_rows(), acp_buffer_layout(), acp_chat_bubble_width_px(), acp_chat_origin_x() (+118 more)

### Community 10 - "draw.rs"
Cohesion: 0.09
Nodes (50): AcpBufferDraw, AcpPaneDraw, AcpPrefixDraw, BrowserBufferDraw, BrowserSyncView, BufferBodyPalette, BufferChrome, BufferChrome<'a> (+42 more)

### Community 11 - "editor-git/src/lib.rs"
Cohesion: 0.17
Nodes (31): cached_repository_file_listing_is_keyed_by_workspace_root(), cached_repository_file_listing_refreshes_after_index_or_head_change(), cached_repository_file_listing_reuses_paths_until_identity_changes(), configure_git_identity(), detect_in_progress(), git_available(), git_stdout(), GitStatusError (+23 more)

### Community 12 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (63): WorkspaceDockTestUserLibrary, Vec, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+55 more)

### Community 13 - "GitSummaryState"
Cohesion: 0.07
Nodes (23): git_status_command_name(), git_summary_changed_tracks_head_updates(), git_summary_refresh_due_until_interval_or_stale(), GitFringeState, GitHeadBlobCache, GitHeadBlobKey, GitPrefixState, GitSummarySnapshot (+15 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.16
Nodes (31): collect_configuration_candidates(), configuration_holes(), configuration_holes_detect_missing_launch_program(), DapConfigError, DebugInferContext, deep_inference_finds_cargo_binary_and_heuristic(), deep_inference_finds_dotnet_dll(), default_workspace_skips_deep_inference() (+23 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.08
Nodes (20): Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, Display, Drop, Error (+12 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.05
Nodes (92): Condvar, compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind (+84 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.09
Nodes (11): GitLogEntry, GitStashEntry, GitStatusSnapshot, parse_header(), RepositoryStatus, Into, Option, Self (+3 more)

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
Nodes (24): Buffer, EditorModel, ModelError, Pane, Popup, BTreeMap, BufferId, Display (+16 more)

### Community 22 - "KeymapScope"
Cohesion: 0.10
Nodes (34): autocomplete_overrides_workspace_while_active(), BindingKey, ChordModifier, dap_mode_overrides_global_f5_while_session_live(), duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeyBinding (+26 more)

### Community 23 - "calculator.rs"
Cohesion: 0.07
Nodes (39): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+31 more)

### Community 24 - "paths.rs"
Cohesion: 0.17
Nodes (23): is_volt_install_path(), locate_program(), ProgramLocation, Path, PathBuf, apply_install_bins_to_process_path(), bin_dir(), effective_path() (+15 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.04
Nodes (121): ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_backspace_can_delete_leading_slash() (+113 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.10
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.10
Nodes (49): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+41 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.06
Nodes (72): centered_rect(), default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names() (+64 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, notification_severity(), RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.09
Nodes (26): text_style_from_theme_style(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display (+18 more)

### Community 31 - "FontSet"
Cohesion: 0.06
Nodes (58): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, PathBuf, ThemeRuntimeSlots (+50 more)

### Community 32 - ".path"
Cohesion: 0.04
Nodes (50): append_hover_rendered_content(), apply_markdown_code_fence_syntax(), comment_style_for_buffer(), comment_style_for_language_path(), CommentStyle, compute_buffer_syntax(), FileReloadWorkerCommand, FileReloadWorkerOutcome (+42 more)

### Community 33 - "String"
Cohesion: 0.05
Nodes (30): TextRange, active_parameter_label(), CopilotDeviceCodePrompt, documentation_lines(), location_from_link(), log_and_notification_revision_skip_cloning_unchanged_snapshots(), LspCompletionItem, LspCompletionKind (+22 more)

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
Nodes (395): EditorRuntime, Default, Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), activate_db_browser_line(), active_buffer_revision_key() (+387 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (112): TextPoint, acp_output_header_title(), acp_tool_call_from_partial_update(), advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_input_operator_motion(), apply_line_indent(), apply_markdown_table_update() (+104 more)

### Community 42 - ".len"
Cohesion: 0.04
Nodes (34): acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_rendered_text_segments(), AcpPaneState (+26 more)

### Community 43 - "shell/acp.rs"
Cohesion: 0.09
Nodes (63): acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_image_mention_token(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), background_command_candidates() (+55 more)

### Community 44 - ".new"
Cohesion: 0.16
Nodes (23): browser_host_event_for_ipc(), BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, BrowserSurfacePlan, BrowserSyncPlan (+15 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.05
Nodes (102): anti_conceal_overlay_reuses_cached_plan(), build_plan(), cached_plan_rebuilds_when_revision_changes(), cached_plan_reuses_arc_for_same_revision(), CachedMarkdownPretty, cfg(), disabled_pretty_sentinel_skips_build(), fixture_text() (+94 more)

### Community 46 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_workspace_dock_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 47 - "WorkspaceConfigurationValue"
Cohesion: 0.13
Nodes (14): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From, I, Number (+6 more)

### Community 48 - "Diagnostic"
Cohesion: 0.14
Nodes (11): CodeActionParams, code_action_params(), code_action_params_use_flattened_lsp_shape(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), LspDiagnostic, LspDiagnosticSeverity, sample_diagnostic() (+3 more)

### Community 49 - "DebugConfigurationCandidate"
Cohesion: 0.12
Nodes (12): DebugConfigurationCandidate, DebugConfigurationSource, DebugStartHistory, DebugStartRecord, default_request(), history_records_last_and_recent(), Into, Item (+4 more)

### Community 50 - "Path"
Cohesion: 0.07
Nodes (29): asset_path_from_parts(), command_failure_message(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), GrammarSource, install_plan_requests_generate_when_parser_is_missing(), InstallCommandSpec (+21 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (32): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, AutocompleteProvider, DebugAdapterSpec (+24 more)

### Community 52 - "Self"
Cohesion: 0.05
Nodes (17): default_action(), hook_command(), AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, PickerActionSpec (+9 more)

### Community 53 - "String"
Cohesion: 0.06
Nodes (136): run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit() (+128 more)

### Community 54 - "lsp.rs"
Cohesion: 0.17
Nodes (23): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), clojure_lsp_recipe(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), install_recipe_for_language_server(), language_servers() (+15 more)

### Community 55 - "ShellError"
Cohesion: 0.12
Nodes (94): Display, Error, From, ShellError, render_browser_buffer_body(), CellMetrics, TextMetrics, adjust_color() (+86 more)

### Community 56 - ".new"
Cohesion: 0.01
Nodes (240): BufferKind, browser_state_for_kind(), ActiveLspBufferContext, default_vim_target(), WorkspaceId, ping_shell_wakeup(), absolute_path_hint(), acp_build_output_lines() (+232 more)

### Community 57 - "DapClientManager"
Cohesion: 0.16
Nodes (13): active_thread_id(), clear_stopped_snapshot(), connect_tcp(), DapClientError, DapClientManager, Display, Error, Formatter (+5 more)

### Community 58 - "probe.rs"
Cohesion: 0.11
Nodes (40): CachedProbe, compute_identity_snapshot(), fallback_rev_parse(), fill_numstat(), git_probe_snapshot_with_numstat(), GitProbeSnapshot, HeadParse, identity_revision() (+32 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (54): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+46 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 62 - "DbService"
Cohesion: 0.11
Nodes (21): db_browser_action_from_spec(), db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbActionOutcome, DbAutocompleteCandidate, DbBrowserAction, DbBrowserBufferState, DbEngine (+13 more)

### Community 63 - "String"
Cohesion: 0.11
Nodes (48): AcpClientConfig, acp_complete_slash(), acp_cycle_mode(), acp_disconnect(), acp_insert_file_mention(), acp_insert_slash_command(), acp_load_session(), acp_new_session() (+40 more)

### Community 64 - "PluginCommand"
Cohesion: 0.09
Nodes (25): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+17 more)

### Community 65 - "BufferId"
Cohesion: 0.04
Nodes (73): acp_decode_image(), active_or_open_dashboard_buffer(), active_runtime_buffer(), apply_lsp_text_edits(), BufferViewState, cleanup_formatter_temp(), clear_lsp_ui_for_stopped_paths(), collect_workspace_language_ids() (+65 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.09
Nodes (44): SyntaxText, buffer_text_for_byte_range(), changed_range_windows(), collect_injection_regions(), create_parser(), DeferredQuery, desired_indent_for_loaded_language(), general_predicates_match() (+36 more)

### Community 67 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 68 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 69 - "Result"
Cohesion: 0.04
Nodes (138): buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail() (+130 more)

### Community 70 - ".new"
Cohesion: 0.12
Nodes (34): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_head_blob_cache_reuses_text_for_same_head(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command() (+26 more)

### Community 71 - ".new"
Cohesion: 0.26
Nodes (10): default_volt_state_dir(), Arc, PathBuf, Self, Send, Sync, SecretStore, sqlite_batch_execution_concatenates_query_results() (+2 more)

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (49): dap_variable_line_spans(), browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines() (+41 more)

### Community 73 - "PluginPackage"
Cohesion: 0.06
Nodes (41): file_open_package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration, syntax_language() (+33 more)

### Community 74 - "common.rs"
Cohesion: 0.10
Nodes (27): binding_suffix(), GrammarSourceSpec, GrammarSourceSpec<'a>, package(), package_with_path_matchers(), CaptureThemeMapping, GrammarSource, LanguageConfiguration (+19 more)

### Community 75 - "theme.rs"
Cohesion: 0.14
Nodes (35): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+27 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (71): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+63 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.17
Nodes (35): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+27 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (45): LspWorkspaceDiagnostic, PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+37 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "shell_buffer"
Cohesion: 0.06
Nodes (94): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), markdown_pretty_paint_plan(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+86 more)

### Community 82 - ".get"
Cohesion: 0.11
Nodes (24): ColumnData, DbColumn, DbIndex, DbSchemaCache, DbSession, DbTable, load_postgres_schema(), load_sql_server_schema() (+16 more)

### Community 83 - "repository_files.rs"
Cohesion: 0.15
Nodes (36): configure_background_command(), Command, cache_key(), CachedRepoFileList, default_worktree_common_dir(), file_fingerprint(), FileFingerprint, invalidate_repository_file_list_cache_for() (+28 more)

### Community 84 - "tool_install.rs"
Cohesion: 0.19
Nodes (40): apply_tool_install_finish(), begin_explicit_install(), continue_tool_install(), fail_tool_install(), fail_tool_install_with_message(), handle_dap_install_hook(), handle_lsp_install_hook(), install_debug_adapter_by_id() (+32 more)

### Community 85 - "InstallCommand"
Cohesion: 0.09
Nodes (30): acp_tool_kind_icon(), Display, Error, Formatter, From, Result, Self, String (+22 more)

### Community 86 - "editor-lsp/src/client.rs"
Cohesion: 0.04
Nodes (96): ClientCapabilities, TextEdit, apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), completion_parser_handles_lists_and_docs() (+88 more)

### Community 87 - "editor-picker/src/lib.rs"
Cohesion: 0.16
Nodes (23): best_contiguous_substring_bonus(), capped_match_set_does_not_require_cloning_losing_row_previews(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), empty_query_with_result_limit_truncates(), fringe_metadata_survives_matching() (+15 more)

### Community 88 - "LineWrapSegment"
Cohesion: 0.14
Nodes (24): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+16 more)

### Community 89 - "PluginBufferSection"
Cohesion: 0.10
Nodes (10): DbBrowserKind, PickerKeybindingContext, plugin_buffer_sections_can_declare_nested_layout_tree(), PluginBufferLayout, PluginBufferLayoutAxis, PluginBufferLayoutNode, PluginBufferSection, PluginBufferSections (+2 more)

### Community 90 - "resolve_picker_extra"
Cohesion: 0.15
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "Vec"
Cohesion: 0.11
Nodes (9): EventLog, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, Vec (+1 more)

### Community 94 - "registered_queries.rs"
Cohesion: 0.16
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 97 - "FontSet<'ttf>"
Cohesion: 0.15
Nodes (6): EmojiFont, FontSet<'ttf>, IconFont, RasterFont, ShapeFace, Font

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "editor-lsp/src/lib.rs"
Cohesion: 0.18
Nodes (30): csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LspError, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+22 more)

### Community 101 - "browser_host.rs"
Cohesion: 0.11
Nodes (15): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+7 more)

### Community 102 - "shell/tests.rs"
Cohesion: 0.03
Nodes (60): load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), buffer_save_still_writes_when_format_on_save_fails(), codicon_glyphs_fit_inside_one_editor_cell() (+52 more)

### Community 103 - ".from"
Cohesion: 0.05
Nodes (54): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), exported_acp_clients(), abi_debug_adapter_spec_round_trips_install_recipe(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_install_recipe(), abi_language_server_spec_round_trips_path_matchers() (+46 more)

### Community 104 - "TextSnapshot"
Cohesion: 0.07
Nodes (9): BufferStats, EditRecord, HighlightDocument, String, Vec, TextSnapshot, trimmed_line(), visible_line_len() (+1 more)

### Community 105 - "shim.rs"
Cohesion: 0.29
Nodes (17): candidate_names(), ensure_unix_executable(), finalize_install(), find_named_file(), find_named_file_inner(), resolve_binary(), Option, Path (+9 more)

### Community 106 - "git_probe_snapshot"
Cohesion: 0.35
Nodes (20): git_available(), git_probe_generation(), git_probe_numstat_spawns_once_until_head_or_index_changes(), git_probe_snapshot(), git_probe_snapshot_hides_detached_head_from_dock(), git_probe_snapshot_matches_rev_parse_and_reuses_identity(), git_probe_snapshot_non_git_root_is_absent_without_spawn(), git_probe_snapshot_shares_cache_across_canonical_roots() (+12 more)

### Community 107 - ".spawn"
Cohesion: 0.10
Nodes (17): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+9 more)

### Community 108 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.10
Nodes (44): UserLibraryService, BoundedSourceLibrary, buffer_close_confirm_overlay(), buffer_picker_preview(), command_picker_overlay_uses_finite_result_limit(), ensure_picker_keybindings(), message_picker_overlay(), overlay_for_source() (+36 more)

### Community 110 - "idle.rs"
Cohesion: 0.11
Nodes (19): attach_shell_wakeup(), idle_wait_timeout(), idle_wait_timeout_ms(), ping_without_sdl_attach_is_noop(), Arc, AtomicBool, Duration, Event (+11 more)

### Community 111 - ".default"
Cohesion: 0.08
Nodes (59): Self, load_persisted_state(), Path, parse_status(), parser_extracts_branch_and_sections(), parser_extracts_unborn_branch_name(), browser_display_url_prefers_requested_navigation(), Self (+51 more)

### Community 112 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 113 - "build_job_command"
Cohesion: 0.29
Nodes (9): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, run_job(), windows_fnm_environment(), configure_background_command() (+1 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - "DebugSessionPlan"
Cohesion: 0.22
Nodes (3): DebugAdapterTransport, DebugSessionPlan, DapState

### Community 116 - "LspCodeAction"
Cohesion: 0.12
Nodes (8): code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), LspCodeAction, LspDocumentTextEdits, LspTextEdit, parse_code_action_response(), Error, windows_should_retry_spawn_error()

### Community 117 - "editor-db/src/lib.rs"
Cohesion: 0.07
Nodes (43): box_row(), box_rule(), BoxRuleKind, CellAlign, column_is_numeric(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement() (+35 more)

### Community 118 - "TextBuffer"
Cohesion: 0.07
Nodes (16): detect_preferred_line_ending(), is_inline_whitespace(), is_object_separator(), is_punctuation_char(), is_word_char(), LineEnding, matches_word_kind(), Default (+8 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 121 - "shell/browser.rs"
Cohesion: 0.13
Nodes (33): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_url_candidates(), browser_url_prefix_len(), BrowserBufferState, BrowserPane (+25 more)

### Community 122 - "DapSessionHandle"
Cohesion: 0.11
Nodes (33): DapReaderSession, DapSessionHandle, fake_adapter_loop(), fake_variables_for_reference(), mark_session_ended(), PendingResponse, read_frame(), record_transport_event_inner() (+25 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.09
Nodes (22): Client, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, gdb(), must() (+14 more)

### Community 124 - "Option"
Cohesion: 0.11
Nodes (9): advance_point_by_text(), delimiter_partner(), find_matching_close_tag(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at(), Option, ShowParenMatch (+1 more)

### Community 125 - "String"
Cohesion: 0.24
Nodes (7): DbBrowserBufferKind, DbBrowserBufferView, DisabledSecretStore, String, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (45): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+37 more)

### Community 127 - "String"
Cohesion: 0.13
Nodes (8): CommandPaletteState, CompilationState, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, String, TerminalState

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.14
Nodes (7): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "editor-dap/src/client.rs"
Cohesion: 0.09
Nodes (30): attach_arguments(), build_csharp_fixture(), configure_adapter_command(), connect_transport(), DapExecutionPosition, DapSessionEvent, DapStackFrameInfo, DapThreadInfo (+22 more)

### Community 132 - "LanguageServerRegistry"
Cohesion: 0.17
Nodes (11): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, path_is_solution(), resolve_single_solution_path(), Path (+3 more)

### Community 133 - "PickerItemSpec"
Cohesion: 0.07
Nodes (48): acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items(), height_fraction() (+40 more)

### Community 134 - "ROption"
Cohesion: 0.09
Nodes (20): exported_ghost_text_lines(), exported_headerline_lines(), GhostTextLine, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGhostTextContext, AbiHoverProvider, AbiSectionAction (+12 more)

### Community 135 - ".recompute_matches"
Cohesion: 0.22
Nodes (4): contiguous_substring_beats_split_path_match(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), push_ascii_lowercase()

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "editor-terminal/src/lib.rs"
Cohesion: 0.14
Nodes (28): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+20 more)

### Community 138 - "git_worktree_dashboard_picker_overlay"
Cohesion: 0.19
Nodes (16): git_common_dir(), git_worktree_dashboard_picker_overlay(), git_worktree_list(), git_worktree_list_parser_normalizes_windows_drive_paths(), GitWorktreeListEntry, parse_git_worktree_list(), PathBuf, worktree_dashboard_base_dir() (+8 more)

### Community 139 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 140 - ".new"
Cohesion: 0.09
Nodes (28): AsyncRead, buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), humanize_debug_label(), open_permission_request_reorders_queue_for_requested_picker(), permission_prompt_lines(), permission_prompt_lines_show_locations_and_choices() (+20 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "Result"
Cohesion: 0.10
Nodes (25): Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, execute_postgres(), execute_sql_server(), execute_sqlite(), execution_notice() (+17 more)

### Community 143 - ".new"
Cohesion: 0.14
Nodes (40): attach_session(), close_buffer_keeps_session_alive_for_next_file(), close_then_open_then_incremental_edits_work_again(), did_open_still_sends_full_text(), file_uri_roundtrip_handles_windows_paths(), full_sync_sends_null_range_and_full_text_even_with_edits(), incremental_did_change_emits_one_event_per_contiguous_edit(), incremental_did_change_includes_newline_in_range_and_text() (+32 more)

### Community 144 - "String"
Cohesion: 0.31
Nodes (18): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+10 more)

### Community 145 - "treesitter_install.rs"
Cohesion: 0.27
Nodes (26): apply_tree_sitter_recompile_notification(), continue_next_tree_sitter_recompile(), continue_tree_sitter_install(), continue_tree_sitter_install_after_clone(), continue_tree_sitter_install_after_generate(), continue_tree_sitter_recompile(), continue_tree_sitter_recompile_after_clone(), continue_tree_sitter_recompile_after_generate() (+18 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".load_from_path"
Cohesion: 0.15
Nodes (13): from_reader_normalizes_crlf_and_tracks_line_endings(), must(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean(), AsRef, Drop, E, Path (+5 more)

### Community 148 - "PickerSession"
Cohesion: 0.17
Nodes (6): PickerResultOrder, PickerSession, selection_skips_divider_rows(), selection_wraps_across_match_list(), set_items_preserves_selected_id_when_still_matched(), source_order_preserves_input_order()

### Community 149 - "String"
Cohesion: 0.38
Nodes (16): parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option(), parse_options_table(), parse_palette() (+8 more)

### Community 150 - "syntax_indent_for_buffer"
Cohesion: 0.20
Nodes (12): adjust_tag_child_indent(), closing_tag_name_after_cursor(), desired_indent_columns_for_text(), desired_indent_for_buffer(), desired_reindent_columns_for_line(), indent_string_from_columns(), is_tag_name_character(), leading_whitespace_info() (+4 more)

### Community 151 - "aligned_indent_column"
Cohesion: 0.18
Nodes (14): aligned_indent_column(), collect_structure_nodes(), current_line_starts_with_token(), delimiter_column(), first_content_column_after(), indent_begin_applies(), line_intersects_node(), line_starts_with_token_at_column() (+6 more)

### Community 152 - "oil.rs"
Cohesion: 0.08
Nodes (39): seti_directory_icon(), exported_oil_strip_entry_icon_prefix(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections() (+31 more)

### Community 153 - "user/db.rs"
Cohesion: 0.12
Nodes (26): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings() (+18 more)

### Community 154 - ".send"
Cohesion: 0.14
Nodes (37): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+29 more)

### Community 155 - "PathBuf"
Cohesion: 0.10
Nodes (33): ChildStdin, diagnostic_matches_request_range(), language_server_session_in_workspace_scope(), launch_summary(), LspClientState, LspReaderSession, LspSessionSharedState, normalize_path_for_compare() (+25 more)

### Community 156 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 157 - "workspace.rs"
Cohesion: 0.11
Nodes (48): begin_discovery_override(), discovery_test_lock(), DiscoveryOverrideGuard, existing_workspace_for_project(), git_available(), message_item(), override_project_search_roots_for_test(), package() (+40 more)

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
Cohesion: 0.05
Nodes (24): exported_ligature_config(), exported_pane_config(), LigatureConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig, config(), AbiLigatureConfig (+16 more)

### Community 162 - "InstallRecipe"
Cohesion: 0.21
Nodes (10): github_release_builds_latest_download_url(), InstallRecipe, AsRef, Into, IntoIterator, Item, Option, Self (+2 more)

### Community 163 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "browser_sync_plan"
Cohesion: 0.26
Nodes (14): BrowserViewportRect, browser_buffer_layout(), browser_host_viewport_rect(), browser_surface_buffer_at_point(), browser_sync_plan(), browser_viewport_contains_point(), browser_viewport_rect(), browser_viewport_rect_rect() (+6 more)

### Community 166 - "String"
Cohesion: 0.25
Nodes (8): lsp_location_uri_detail(), lsp_location_uri_label(), call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "DapLogEntry"
Cohesion: 0.26
Nodes (6): assert_control_requests_omit_nulls(), dap_log_text(), DapLogDirection, DapLogEntry, DapLogSnapshot, DapTransportLog

### Community 170 - "shell/workspace_dock.rs"
Cohesion: 0.15
Nodes (24): init_repo(), refresh_workspace_dock_branches(), Instant, Option, Path, PathBuf, Self, String (+16 more)

### Community 171 - "editor-path/src/lib.rs"
Cohesion: 0.11
Nodes (22): contains_wildcards(), glob_literal_count(), glob_matches(), grammar_install_root(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher (+14 more)

### Community 172 - "user/dap.rs"
Cohesion: 0.20
Nodes (17): adapter_preferences_match_language_defaults(), codelldb_recipe(), debug_adapters(), debug_adapters_attach_typed_install_recipes(), install_recipe_for_debug_adapter(), locals_buffer_declares_locals_and_expressions_sections(), locals_sections(), package() (+9 more)

### Community 173 - "JobResult"
Cohesion: 0.22
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "GhostTextContext"
Cohesion: 0.25
Nodes (8): GhostTextContext, build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "predicate_capture_text"
Cohesion: 0.27
Nodes (11): evaluate_general_predicate(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches(), predicate_capture_node() (+3 more)

### Community 177 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (32): current_theme_source_fingerprint(), DynamicUserLibrary, refresh_theme_registry_if_needed(), AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec (+24 more)

### Community 180 - "HighlightSpan"
Cohesion: 0.16
Nodes (13): apply_text_edits_to_span(), HighlightSpan, InjectionHighlights, InjectionRegion, intern_query_captures(), intern_theme_token(), Arc, BTreeMap (+5 more)

### Community 181 - "open_slash_command_picker"
Cohesion: 0.29
Nodes (9): acp_slash_completion_query(), AcpUiAction, CompletionTrigger, handle_acp_ui_action(), open_slash_command_picker(), pending_slash_completion_trigger(), pending_slash_trigger(), PendingSlashTrigger (+1 more)

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.18
Nodes (10): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+2 more)

### Community 185 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 186 - "AcpManager"
Cohesion: 0.10
Nodes (21): acp_connected(), acp_permission_picker_closed(), acp_session_buffer_name(), AcpManager, AcpPendingPermissionUi, apply_acp_notification(), config_option_is_mode(), config_option_is_model() (+13 more)

### Community 187 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 188 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 189 - "LspLogEntry"
Cohesion: 0.09
Nodes (12): last_did_open_text(), last_notification_params(), LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot (+4 more)

### Community 190 - "TerminalCursorSnapshot"
Cohesion: 0.29
Nodes (4): TerminalCursorDraw, terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 191 - ".path"
Cohesion: 0.17
Nodes (14): db_connect_enter_submits_pasted_connection_string(), db_dashboard_opens_and_writes_files_through_editor_section(), db_query_buffer_receives_sql_highlighting_without_blocking(), db_table_preview_buffer_exposes_hidden_sqls_path_without_file_open_hooks(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed() (+6 more)

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "AbiSectionTree"
Cohesion: 0.11
Nodes (15): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiDirectoryEntry, AbiDirectoryEntryKind (+7 more)

### Community 194 - ".from_text"
Cohesion: 0.11
Nodes (39): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), edits_since_returns_contiguous_forward_edits(), highlight_document_captures_edits_without_undo_history(), highlight_document_falls_back_to_full_parse_without_contiguous_edits(), large_buffers_expose_line_windows_without_full_materialization() (+31 more)

### Community 195 - "PickerItem"
Cohesion: 0.18
Nodes (7): PickerItem, PickerMatch, Into, Option, Self, String, set_item_preview_updates_selected_match_without_filling_other_rows()

### Community 196 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 197 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 198 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 199 - "Option"
Cohesion: 0.07
Nodes (25): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), collapse_variable_path(), DapStoppedSnapshot, DapVariableNode, DapVariablePath, DapVariableRow (+17 more)

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "Self"
Cohesion: 0.11
Nodes (10): normalize_inline_text(), Item, Iterator, Range, Self, Selection, TextByteChunks, TextByteChunks<'a> (+2 more)

### Community 202 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 203 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 204 - "Option"
Cohesion: 0.06
Nodes (63): BufRead, char_to_byte_offset(), completion_documentation(), completion_level_for_message(), configuration_item_section(), copilot_status_notifications_offer_sign_in_action(), csharp_metadata_request_params(), effective_workspace_configuration_settings() (+55 more)

### Community 205 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 206 - "TerminalEventWake"
Cohesion: 0.20
Nodes (12): AlacrittyEvent, QueuedEventListener, Arc, Debug, Fn, Send, Sender, Sync (+4 more)

### Community 207 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 209 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 210 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 211 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 212 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 214 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 215 - "highlight.rs"
Cohesion: 0.39
Nodes (8): bench_highlight_rust(), bench_highlight_rust_window(), Language, String, rust_fixture(), rust_language(), rust_registry(), Criterion

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "Option"
Cohesion: 0.12
Nodes (6): LanguageServerSession, normalize_optional_string(), AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "main"
Cohesion: 0.13
Nodes (17): bootstrap(), HostBootstrap, command_palette_items(), main(), DebugAdapterSpec, Error, LanguageConfiguration, LanguageServerSpec (+9 more)

### Community 221 - "AcpEvent"
Cohesion: 0.07
Nodes (35): AvailableCommand, AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), command_input_hint() (+27 more)

### Community 223 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 224 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 225 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 227 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 228 - "evaluate_expression"
Cohesion: 0.47
Nodes (4): DapEvaluateContext, evaluate_expression(), EvaluateArgumentsContext, From

### Community 229 - "BufferId"
Cohesion: 0.12
Nodes (29): ActiveBufferEventContext, apply_git_status_snapshot(), diff_git_commit_at_point(), fetch_git_pushremote(), fetch_git_remote(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_remote_list() (+21 more)

### Community 230 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 231 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 232 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 233 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 234 - "String"
Cohesion: 0.08
Nodes (19): append_query_source(), CaptureThemeMapping, GrammarRecompileFailure, GrammarRecompileReport, LanguageConfiguration, LanguageLoader, load_language(), normalize_unique_entries() (+11 more)

### Community 235 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 236 - "UserLibraryModule"
Cohesion: 0.07
Nodes (26): exported_oil_defaults(), exported_oil_keybindings(), exported_pdf_open_mode(), OilDefaults, OilKeybindings, PdfOpenMode, AbiIconFontSymbol, AbiOilDefaults (+18 more)

### Community 237 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 238 - "LspFormattingOptions"
Cohesion: 0.47
Nodes (3): lsp_formatting_options(), LspFormattingOptions, FormattingOptions

### Community 239 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 240 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 241 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 242 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 243 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 244 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 247 - ".request"
Cohesion: 0.40
Nodes (4): Arguments, parse_response_body(), strip_null_fields(), Response

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 250 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 251 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 285 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 286 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 334 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 335 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 340 - "Self"
Cohesion: 0.04
Nodes (57): DebugAdapterRootStrategy, exported_keymap_config(), exported_statusline_render(), exported_workspace_roots(), KeymapConfig, WorkspaceRoot, statusline_context_from_abi(), AbiBrowserFeatureSpec (+49 more)

### Community 341 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 344 - ".terminal_output"
Cohesion: 0.50
Nodes (3): apply_output_limit(), TerminalOutputRequest, TerminalOutputResponse

### Community 345 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 347 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 386 - "shell/git.rs"
Cohesion: 0.08
Nodes (46): apply_git_fringe_hunk(), commit_git_buffer(), FringeDiffOp, git_commit_message(), git_commit_temp_path(), git_fringe_snapshot_from_texts(), git_fringe_snapshot_from_texts_ignores_crlf_only_difference(), git_fringe_snapshot_from_texts_is_empty_when_identical() (+38 more)

### Community 409 - "buffer_footer_layout_with_command_line"
Cohesion: 0.25
Nodes (13): ScreenHit, debug_fringe_cell_count(), editor_fringe_width_px(), wrap_columns_for_width(), wrap_columns_for_width_with_fringe(), buffer_cursor_screen_anchor(), buffer_footer_layout_with_command_line(), buffer_point_at_screen() (+5 more)

## Knowledge Gaps
- **156 isolated node(s):** `BufferChrome<'a>`, `ShellWakeupEvent`, `StartupProfile`, `topbar`, `navToggle` (+151 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `Option`, `shell/git.rs`, `ShellUiState`, `String`, `git_worktree_dashboard_picker_overlay`, `GitSummaryState`, `String`, `treesitter_install.rs`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `syntax_indent_for_buffer`, `state_with_user_library`, `command_stream.rs`, `.path`, `shell/pdf.rs`, `ServiceRegistry`, `Option`, `DynamicUserLibrary`, `open_slash_command_picker`, `String`, `.new`, `AcpManager`, `active_runtime_popup`, `String`, `.path`, `BufferId`, `resolve_permission`, `.new`, `Result`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `shell_buffer`, `tool_install.rs`, `editor-plugin-host/src/lib.rs`, `editor-core/src/lib.rs`, `CommandSource`, `AcpEvent`, `main`, `GitEditorState`, `BufferId`, `shell/tests.rs`, `shell/picker.rs`, `shell/browser.rs`?**
  _High betweenness centrality (0.145) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `Option`, `shell/git.rs`, `ShellUiState`, `render.rs`, `draw.rs`, `GitSummaryState`, `buffer_footer_layout_with_command_line`, `state_with_user_library`, `.path`, `shell/pdf.rs`, `browser_sync_plan`, `shell/mod.rs`, `.len`, `shell/acp.rs`, `editor-markdown/src/lib.rs`, `open_slash_command_picker`, `ShellError`, `.new`, `BufferId`, `Result`, `.new`, `LineSyntaxSpan`, `directory.rs`, `shell/terminal.rs`, `shell_buffer`, `LineWrapSegment`, `shell/picker.rs`, `TextBuffer`, `shell/browser.rs`, `StoredBreakpoint`?**
  _High betweenness centrality (0.075) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `user/lib.rs`, `PickerItemSpec`, `sdk/src/lib.rs`, `calculator.rs`, `oil.rs`, `user/db.rs`, `show_paren.rs`, `ruby.rs`, `solidity.rs`, `workspace.rs`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `package`, `user/terminal.rs`, `user/dap.rs`, `user/workspace_dock.rs`, `user/browser.rs`, `HeaderlineTestUserLibrary`, `Self`, `lsp.rs`, `.new`, `PluginCommand`, `cmake.rs`, `clojure.rs`, `markdown.rs`, `bash.rs`, `common.rs`, `elixir.rs`, `swift.rs`, `java.rs`, `lang/vim.rs`, `graphql.rs`, `capture_mappings`, `hcl.rs`, `syntax_languages`, `kotlin.rs`, `package`, `PluginBufferSection`, `editor-plugin-host/src/lib.rs`, `main`, `lua.rs`, `nix.rs`, `latex.rs`, `perl.rs`, `php.rs`, `scala.rs`, `UserLibraryModule`, `.default`, `syntax_language`, `syntax_language`, `rainbow_parens.rs`, `proto.rs`, `AcpPickerItemSpec`, `PluginKeyBinding`, `r.rs`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **What connects `BufferChrome<'a>`, `ShellWakeupEvent`, `StartupProfile` to the rest of the system?**
  _156 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `DebugConfiguration` be split into smaller, more focused modules?**
  _Cohesion score 0.13306451612903225 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.06786492374727669 - nodes in this community are weakly interconnected._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.07878787878787878 - nodes in this community are weakly interconnected._