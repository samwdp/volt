# Graph Report - volt  (2026-08-26)

## Corpus Check
- 264 files · ~659,281 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 11075 nodes · 45327 edges · 319 communities (306 shown, 13 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3603 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `1a649cd5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- DebugConfiguration
- Path
- Option
- .new
- ShellState
- user/lib.rs
- .new
- shell_ui
- String
- FontSet
- draw.rs
- editor-git/src/lib.rs
- Vec
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
- paths.rs
- state_with_user_library
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- ThemeRegistry
- TextPoint
- .new
- SectionLineMeta
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- Option
- clipboard.rs
- shell/acp.rs
- .new
- editor-markdown/src/lib.rs
- WorkspaceDockConfig
- trigger_autocomplete
- TextSnapshot
- DebugConfigurationCandidate
- LanguageInstallPlan
- HeaderlineTestUserLibrary
- Self
- String
- lsp.rs
- ShellError
- browser_host.rs
- DapClientManager
- probe.rs
- BufferId
- build_output.rs
- key_sequence.rs
- .len
- String
- Vec
- ShellUiState
- SyntaxRegistry
- .load_from_path
- AbiGitStatusSnapshot
- String
- .new
- test_service
- LineSyntaxSpan
- PluginPackage
- user/db.rs
- theme.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- Result
- Vec
- repository_files.rs
- tool_install.rs
- InstallCommand
- editor-lsp/src/client.rs
- editor-picker/src/lib.rs
- wrap_line_segments
- Option
- resolve_picker_extra
- editor-plugin-host/src/lib.rs
- CommandSource
- Result
- registered_queries.rs
- workspace_nav.rs
- PickerOverlay
- DynamicUserLibrary
- GitEditorState
- modeline.rs
- editor-lsp/src/lib.rs
- render.rs
- shell/tests.rs
- .from
- .line_count
- shim.rs
- PluginCommand
- .spawn
- .path
- shell/picker.rs
- idle.rs
- .default
- AbiGitFeatureSpec
- LanguageServerSpec
- PluginKeyBinding
- DebugSessionPlan
- TextRange
- String
- normalize_extension
- process_supervisor.rs
- Vec
- shell/browser.rs
- DapSessionHandle
- DebugAdapterSpec
- TextBuffer
- LspCodeAction
- StoredBreakpoint
- Option
- JobError
- rainbow_parens.rs
- user/config.rs
- editor-dap/src/client.rs
- capture_mappings
- ROption
- git_probe_snapshot
- Vec
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- editor-terminal/src/lib.rs
- git_worktree_dashboard_picker_overlay
- AcpPaneState
- .new
- CommandLineOverlay
- DbService
- .new
- I
- treesitter_install.rs
- volt/build.rs
- String
- PickerSession
- HighlightDocument
- JobSpec
- editor-syntax/src/lib.rs
- oil.rs
- install_test_lsp_manager
- .send
- LspSessionHandle
- show_paren.rs
- workspace.rs
- load
- Copilot instructions for `volt`
- .new
- Self
- InstallRecipe
- resolve_permission
- ServiceRegistry
- PaneConfig
- String
- user/terminal.rs
- corpus_inventory.rs
- DapLogEntry
- shell/workspace_dock.rs
- editor-path/src/lib.rs
- user/dap.rs
- TerminalEventWake
- headerline_lines
- user/browser.rs
- build_job_command
- editor-icons/src/lib.rs
- `user`
- .recompute_matches
- Vec
- open_slash_command_picker
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- treesittercontext_ghosttext.rs
- AcpManager
- treesittercontext_shared.rs
- volt/src/main.rs
- LspNotification
- .move_object_end_forward
- JobResult
- Database Explorer PRD
- .new
- .from_text
- PickerItem
- AbiSectionTree
- begin_oil_worktree_request
- DbEngine
- Option
- markdown.rs
- .byte_slice_chunks
- AbiPdfOpenMode
- ancestor_contexts_for_cursor
- Value
- TerminalCursorSnapshot
- OilDefaultsSection
- UserLibraryModule
- 0004-markdown-pretty-pipeline.md
- clojure.rs
- main
- graphql.rs
- hcl.rs
- dap-client-spec.md
- latex.rs
- highlight.rs
- Language
- Option
- Domain Docs
- Issue tracker: GitHub
- main
- AcpEvent
- ruby.rs
- cargo
- syntax_language
- rainbow_paren.rs
- index_syntax_lines
- evaluate_expression
- syntax_language
- syntax_language
- .workspaces
- StartupTrace
- DapSessionInfo
- String
- RVec
- .terminal_output
- .oil_directory_sections
- ligatures.rs
- .default
- picker_items
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- .request
- Agent skills
- 0005-dap-session-and-client.md
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- keymap.rs
- 0006-language-server-and-debug-adapter-install.md
- AbiContextHelpSpec
- syntax_languages
- editor-core/src/lib.rs
- buffer_footer_layout_with_command_line

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 885 edges
2. `ShellBuffer` - 399 edges
3. `shell_ui_mut()` - 395 edges
4. `register_shell_hooks()` - 274 edges
5. `shell_ui()` - 272 edges
6. `shell_buffer_mut()` - 211 edges
7. `shell_buffer()` - 210 edges
8. `ShellError` - 197 edges
9. `ShellUiState` - 194 edges
10. `ShellState` - 170 edges

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

## Communities (319 total, 13 thin omitted)

### Community 0 - "DebugConfiguration"
Cohesion: 0.13
Nodes (10): DebugConfiguration, DebugRequestKind, Into, IntoIterator, Item, Iterator, Option, PathBuf (+2 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (23): inline_completion_params(), is_copilot_server(), lsp_formatting_options(), LspClientError, LspClientManager, LspFormattingOptions, parse_definition_response(), parse_text_edit_response() (+15 more)

### Community 2 - "Option"
Cohesion: 0.06
Nodes (50): build_git_fringe_snapshot_with_cache(), build_git_summary_snapshot(), classify_head_blob(), command_output_transcript(), create_git_worktree_from_query(), git_branch_merge(), git_branch_push_remote(), git_branch_remote() (+42 more)

### Community 3 - ".new"
Cohesion: 0.10
Nodes (84): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), begin_project_discovery_test(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), discovery_fixture(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change() (+76 more)

### Community 4 - "ShellState"
Cohesion: 0.03
Nodes (61): clear_key_sequence(), active_buffer_event_context(), active_runtime_surface(), ActiveTypingFrameProfile, alt_mod(), average_duration(), browser_devtools_shortcut_requested(), buffer_is_git_commit() (+53 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (105): bundled_highlight_query(), cached_syntax_languages(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers() (+97 more)

### Community 6 - ".new"
Cohesion: 0.11
Nodes (65): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+57 more)

### Community 7 - "shell_ui"
Cohesion: 0.06
Nodes (42): shell_ui(), browser_open_buffer_command_opens_split_with_file_url(), browser_open_buffer_command_uses_existing_split_pane(), browser_popup_command_focuses_the_popup_surface(), browser_url_command_opens_split_browser_with_detected_url(), closing_streamed_command_popup_kills_worker(), copilot_auth_notification_shows_device_code_and_stays_active(), dap_install_server_opens_recipe_picker() (+34 more)

### Community 8 - "String"
Cohesion: 0.12
Nodes (14): language_server_spec_exposes_workspace_configuration_builders(), normalize_optional_string(), AsRef, From, Into, IntoIterator, Item, Number (+6 more)

### Community 9 - "FontSet"
Cohesion: 0.07
Nodes (47): RenderColor, Self, TextStyle, FontSet, alpha_bitmap_surface(), cached_emoji_layout(), cached_primary_text_runs(), CachedLigatureGlyphPlacement (+39 more)

### Community 10 - "draw.rs"
Cohesion: 0.08
Nodes (53): AcpBufferDraw, AcpPaneDraw, AcpPrefixDraw, BrowserBufferDraw, BrowserSyncView, BufferBodyPalette, BufferChrome, BufferChrome<'a> (+45 more)

### Community 11 - "editor-git/src/lib.rs"
Cohesion: 0.17
Nodes (31): cached_repository_file_listing_is_keyed_by_workspace_root(), cached_repository_file_listing_refreshes_after_index_or_head_change(), cached_repository_file_listing_reuses_paths_until_identity_changes(), configure_git_identity(), detect_in_progress(), git_available(), git_stdout(), GitStatusError (+23 more)

### Community 12 - "Vec"
Cohesion: 0.07
Nodes (39): user_modeline_context(), AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec, decode_modeline() (+31 more)

### Community 13 - "shell/git.rs"
Cohesion: 0.07
Nodes (49): ActiveBufferEventContext, apply_git_fringe_hunk(), FringeDiffOp, git_fringe_snapshot_from_texts(), git_fringe_snapshot_from_texts_ignores_crlf_only_difference(), git_fringe_snapshot_from_texts_is_empty_when_identical(), git_fringe_snapshot_from_texts_marks_all_lines_added_without_head(), git_fringe_snapshot_from_texts_marks_inserted_line_added() (+41 more)

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
Cohesion: 0.09
Nodes (25): buffer_sections(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_package_binds_ctrl_c_ctrl_c(), calculator_package_binds_ctrl_tab_to_switch_panes(), calculator_package_declares_its_buffer_through_package_metadata(), calculator_package_exports_open_and_evaluate_commands(), calculator_package_has_no_hook_declarations() (+17 more)

### Community 24 - "paths.rs"
Cohesion: 0.17
Nodes (23): is_volt_install_path(), locate_program(), ProgramLocation, Path, PathBuf, apply_install_bins_to_process_path(), bin_dir(), effective_path() (+15 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.07
Nodes (89): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk() (+81 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.10
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.10
Nodes (49): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+41 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.06
Nodes (74): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+66 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (31): AutocompleteProviderKind, RankedAutocompleteEntry, AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay, HoverProviderContent (+23 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.09
Nodes (25): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+17 more)

### Community 31 - "TextPoint"
Cohesion: 0.06
Nodes (80): TextPoint, Cow, yank_to_clipboard_text(), active_shell_buffer_mut(), active_shell_buffer_vim_targets_input(), advance_point_by_text(), apply_block_operator(), apply_directory_edit_queue_if_needed() (+72 more)

### Community 32 - ".new"
Cohesion: 0.02
Nodes (224): BufferKind, DbAutocompleteCandidate, browser_state_for_kind(), ActiveLspBufferContext, default_vim_target(), WorkspaceId, ping_shell_wakeup(), acp_build_output_lines() (+216 more)

### Community 33 - "SectionLineMeta"
Cohesion: 0.15
Nodes (24): cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), git_commit_at_point(), git_line_is_untracked(), git_sequence_in_progress(), git_status_apply_commit_command(), git_status_cherry_pick_apply_command(), git_status_cherry_pick_command() (+16 more)

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
Nodes (384): EditorRuntime, Default, write_system_clipboard(), accept_autocomplete(), acp_decode_image(), activate_db_browser_line(), active_buffer_revision_key(), active_dashboard_editor_buffer() (+376 more)

### Community 40 - "Self"
Cohesion: 0.12
Nodes (19): ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), default_rainbow_parens_enabled(), default_show_paren_enabled() (+11 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (94): buffer_uses_browser_host_surface(), acp_output_header_title(), acp_tool_call_from_partial_update(), apply_db_browser_view_to_section(), apply_markdown_code_fence_syntax(), apply_pending_block_insert(), block_comment_toggle_removal_lens(), buffer_context_overlay_snapshot() (+86 more)

### Community 42 - "clipboard.rs"
Cohesion: 0.06
Nodes (61): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+53 more)

### Community 43 - "shell/acp.rs"
Cohesion: 0.09
Nodes (63): acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_image_mention_token(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), background_command_candidates() (+55 more)

### Community 44 - ".new"
Cohesion: 0.13
Nodes (26): browser_host_event_for_ipc(), browser_host_needs_web_context(), browser_host_starts_without_a_live_web_context(), browser_navigation_retry_required(), BrowserBufferPlan, BrowserHostEvent, BrowserHostService, BrowserInstance (+18 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.05
Nodes (102): anti_conceal_overlay_reuses_cached_plan(), build_plan(), cached_plan_rebuilds_when_revision_changes(), cached_plan_reuses_arc_for_same_revision(), CachedMarkdownPretty, cfg(), disabled_pretty_sentinel_skips_build(), fixture_text() (+94 more)

### Community 46 - "WorkspaceDockConfig"
Cohesion: 0.18
Nodes (9): WorkspaceDockTestUserLibrary, WorkspaceDockConfig, WorkspaceDockSide, config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_workspace_dock_scope(), package_exports_dock_navigation_commands() (+1 more)

### Community 47 - "trigger_autocomplete"
Cohesion: 0.14
Nodes (20): adjust_tag_child_indent(), apply_sqls_workspace_settings_for_buffer(), buffer_is_db_query(), closing_tag_name_after_cursor(), desired_indent_columns_for_text(), desired_indent_for_buffer(), desired_reindent_columns_for_line(), indent_string_from_columns() (+12 more)

### Community 48 - "TextSnapshot"
Cohesion: 0.11
Nodes (41): TextSnapshot, add_counts(), add_delta(), apply_edits_to_counts(), assert_counts_match_rescan(), AutocompleteTokenCache, AutocompleteTokenScan, AutocompleteTokenScanKind (+33 more)

### Community 49 - "DebugConfigurationCandidate"
Cohesion: 0.12
Nodes (12): DebugConfigurationCandidate, DebugConfigurationSource, DebugStartHistory, DebugStartRecord, default_request(), history_records_last_and_recent(), Into, Item (+4 more)

### Community 50 - "LanguageInstallPlan"
Cohesion: 0.09
Nodes (16): asset_path_from_parts(), default_install_root(), default_query_asset_root(), ensure_cloned_grammar_dir_exists(), InstallCommandSpec, io_error(), LanguageInstallPlan, resolve_query_asset_root_from_roots() (+8 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (38): AtomicUsize, active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, render_shell_state_scene_with_docked_runtime_popup(), render_shell_state_scene_with_notification_overlay() (+30 more)

### Community 52 - "Self"
Cohesion: 0.02
Nodes (71): browser_items(), dashboard_sections(), sidebar_sections(), exported_acp_picker_items(), exported_db_browser_items(), AcpActionSpec, AcpPickerContext, AcpPickerItemSpec (+63 more)

### Community 53 - "String"
Cohesion: 0.05
Nodes (146): run_command(), active_git_status_command_context(), apply_git_status_snapshot(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer() (+138 more)

### Community 54 - "lsp.rs"
Cohesion: 0.17
Nodes (23): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), clojure_lsp_recipe(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), install_recipe_for_language_server(), language_servers() (+15 more)

### Community 55 - "ShellError"
Cohesion: 0.12
Nodes (87): Canvas, Display, Error, From, ShellError, render_browser_buffer_body(), CellMetrics, adjust_color() (+79 more)

### Community 56 - "browser_host.rs"
Cohesion: 0.12
Nodes (15): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+7 more)

### Community 57 - "DapClientManager"
Cohesion: 0.16
Nodes (13): active_thread_id(), clear_stopped_snapshot(), connect_tcp(), DapClientError, DapClientManager, Display, Error, Formatter (+5 more)

### Community 58 - "probe.rs"
Cohesion: 0.11
Nodes (40): CachedProbe, compute_identity_snapshot(), fallback_rev_parse(), fill_numstat(), git_probe_snapshot_with_numstat(), GitProbeSnapshot, HeadParse, identity_revision() (+32 more)

### Community 59 - "BufferId"
Cohesion: 0.11
Nodes (53): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+45 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 62 - ".len"
Cohesion: 0.04
Nodes (52): advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update(), ascii_control_caret_notation(), byte_index_for_char_column(), char_at_index(), detect_markdown_table(), display_columns_for_character() (+44 more)

### Community 63 - "String"
Cohesion: 0.11
Nodes (48): AcpClientConfig, acp_complete_slash(), acp_cycle_mode(), acp_disconnect(), acp_insert_file_mention(), acp_insert_slash_command(), acp_load_session(), acp_new_session() (+40 more)

### Community 64 - "Vec"
Cohesion: 0.08
Nodes (38): packages(), AutocompleteProvider, HoverProvider, Vec, WorkspaceRoot, user_library_contains_unique_packages_with_behavior(), user_library_exports_calculator_manual_providers(), user_library_keybindings_do_not_conflict() (+30 more)

### Community 65 - "ShellUiState"
Cohesion: 0.04
Nodes (59): active_lsp_workspace_loaded(), active_or_open_dashboard_buffer(), active_runtime_buffer(), apply_pending_lsp_state(), BufferViewState, close_popup_buffer_and_restore_focus(), command_builds_user_library(), create_db_query_like_buffer() (+51 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.10
Nodes (38): SyntaxText, buffer_text_for_byte_range(), collect_injection_regions(), compile_query_source(), create_parser(), DeferredQuery, desired_indent_for_loaded_language(), highlight_inline_language_per_line() (+30 more)

### Community 67 - ".load_from_path"
Cohesion: 0.07
Nodes (13): LineEnding, reload_from_path_returns_false_when_disk_state_is_unchanged(), AsRef, Drop, Into, Path, PathBuf, Result (+5 more)

### Community 68 - "AbiGitStatusSnapshot"
Cohesion: 0.19
Nodes (9): GitStashEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitStashEntry, GitStatusSnapshot, GitStatusSnapshot, StatusEntry (+1 more)

### Community 69 - "String"
Cohesion: 0.04
Nodes (162): default_error_log_path(), buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail() (+154 more)

### Community 70 - ".new"
Cohesion: 0.10
Nodes (35): diff_git_commit_at_point(), diff_git_dwim(), diff_git_stash_at_point(), git_args_with_no_pager(), git_fringe_snapshot_ignores_crlf_only_difference(), git_fringe_snapshot_is_empty_when_buffer_matches_head(), git_head_blob_cache_reuses_text_for_same_head(), git_push_remote_name_prefers_branch_push_remote_for_slashy_branch_names() (+27 more)

### Community 71 - "test_service"
Cohesion: 0.13
Nodes (19): db_browser_renderer_customizes_rows_and_preserves_actions(), default_volt_state_dir(), InMemorySecretStore, insert_test_session(), remembered_connections_store_metadata_separately_from_secret(), Arc, HashMap, Mutex (+11 more)

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.10
Nodes (52): browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines(), db_results_table_row_spans() (+44 more)

### Community 73 - "PluginPackage"
Cohesion: 0.02
Nodes (185): file_open_package(), package(), package(), package_exports_image_commands(), package_exports_image_keybindings(), bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter() (+177 more)

### Community 74 - "user/db.rs"
Cohesion: 0.14
Nodes (23): browser_item(), browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings(), default_action() (+15 more)

### Community 75 - "theme.rs"
Cohesion: 0.16
Nodes (31): apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages(), bundled_themes_use_pallet_sections_and_token_references() (+23 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (65): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+57 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.17
Nodes (35): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+27 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.08
Nodes (65): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "Result"
Cohesion: 0.06
Nodes (114): shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), markdown_pretty_paint_plan(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+106 more)

### Community 82 - "Vec"
Cohesion: 0.16
Nodes (21): Compat, connect_sql_server(), DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns() (+13 more)

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
Nodes (73): ClientCapabilities, apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range() (+65 more)

### Community 87 - "editor-picker/src/lib.rs"
Cohesion: 0.16
Nodes (23): best_contiguous_substring_bonus(), capped_match_set_does_not_require_cloning_losing_row_previews(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), empty_query_with_result_limit_truncates(), fringe_metadata_survives_matching() (+15 more)

### Community 88 - "wrap_line_segments"
Cohesion: 0.09
Nodes (34): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+26 more)

### Community 89 - "Option"
Cohesion: 0.05
Nodes (61): TextEdit, active_parameter_label(), char_to_byte_offset(), completion_documentation(), documentation_lines(), explicit_windows_env_value(), file_uri_to_path(), hover_marked_string() (+53 more)

### Community 90 - "resolve_picker_extra"
Cohesion: 0.15
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "Result"
Cohesion: 0.20
Nodes (8): db_browser_renderer_rejects_row_count_mismatch(), DbBrowserBufferView, OsSecretStore, Result, section_count_label(), summarize_sql(), DbBrowserItemRenderer, Entry

### Community 94 - "registered_queries.rs"
Cohesion: 0.15
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "PickerOverlay"
Cohesion: 0.05
Nodes (16): absolute_path_hint(), dap_log_buffer_lines(), ErrorSeverity, GitBranchActionKind, GitCommitActionKind, PickerAction, PickerKind, PickerOverlay (+8 more)

### Community 97 - "DynamicUserLibrary"
Cohesion: 0.02
Nodes (55): PathBuf, ThemeRuntimeSlots, DynamicUserLibrary, EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font() (+47 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+20 more)

### Community 101 - "render.rs"
Cohesion: 0.04
Nodes (133): WrapCollect, is_zero_width_display_character(), acp_buffer_layout(), acp_chat_corner_radius(), acp_chat_rounded(), acp_pane_body_visible_rows(), acp_slice_chars(), adjusted_contextual_ligature_pixel_size() (+125 more)

### Community 102 - "shell/tests.rs"
Cohesion: 0.02
Nodes (90): ctrl_mod(), load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_and_secondary_buffer_ids(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_insert_identifier_appears_and_delete_drops_last_occurrence(), autocomplete_or_group_uses_first_provider_with_results() (+82 more)

### Community 103 - ".from"
Cohesion: 0.05
Nodes (57): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GhostTextLine, GhostTextLine, abi_debug_adapter_spec_round_trips_install_recipe(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_install_recipe() (+49 more)

### Community 104 - ".line_count"
Cohesion: 0.12
Nodes (5): EditRecord, String, trimmed_line(), visible_line_len(), RopeSlice

### Community 105 - "shim.rs"
Cohesion: 0.29
Nodes (17): candidate_names(), ensure_unix_executable(), finalize_install(), find_named_file(), find_named_file_inner(), resolve_binary(), Option, Path (+9 more)

### Community 106 - "PluginCommand"
Cohesion: 0.08
Nodes (23): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+15 more)

### Community 107 - ".spawn"
Cohesion: 0.09
Nodes (22): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+14 more)

### Community 108 - ".path"
Cohesion: 0.21
Nodes (12): db_connect_enter_submits_pasted_connection_string(), db_query_buffer_receives_sql_highlighting_without_blocking(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir() (+4 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.11
Nodes (44): UserLibraryService, BoundedSourceLibrary, buffer_close_confirm_overlay(), buffer_picker_preview(), command_picker_overlay_uses_finite_result_limit(), ensure_picker_keybindings(), message_picker_overlay(), overlay_for_source() (+36 more)

### Community 110 - "idle.rs"
Cohesion: 0.11
Nodes (19): attach_shell_wakeup(), idle_wait_timeout(), idle_wait_timeout_ms(), ping_without_sdl_attach_is_noop(), Arc, AtomicBool, Duration, Event (+11 more)

### Community 111 - ".default"
Cohesion: 0.08
Nodes (56): Self, load_persisted_state(), Path, sqlite_query_execution_and_schema_cache_work(), parse_status(), parser_extracts_branch_and_sections(), parser_extracts_unborn_branch_name(), commit_buffer_template() (+48 more)

### Community 112 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 113 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (6): LanguageServerRegistry, LanguageServerSpec, InstallRecipe, Iterator, LanguageServerRootStrategy, Vec

### Community 114 - "PluginKeyBinding"
Cohesion: 0.11
Nodes (25): plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, hook_command(), leader_binding(), normal_binding() (+17 more)

### Community 115 - "DebugSessionPlan"
Cohesion: 0.22
Nodes (3): DebugAdapterTransport, DebugSessionPlan, DapState

### Community 116 - "TextRange"
Cohesion: 0.07
Nodes (20): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), diagnostics_parser_maps_lsp_fields(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+12 more)

### Community 117 - "String"
Cohesion: 0.07
Nodes (58): ColumnData, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, column_is_numeric(), connection_descriptor_detects_all_supported_engines() (+50 more)

### Community 118 - "normalize_extension"
Cohesion: 0.31
Nodes (5): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), BTreeMap, normalize_extension()

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 121 - "shell/browser.rs"
Cohesion: 0.13
Nodes (37): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len(), browser_viewport_contains_point() (+29 more)

### Community 122 - "DapSessionHandle"
Cohesion: 0.11
Nodes (33): DapReaderSession, DapSessionHandle, fake_adapter_loop(), fake_variables_for_reference(), mark_session_ended(), PendingResponse, read_frame(), record_transport_event_inner() (+25 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.09
Nodes (22): Client, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, gdb(), must() (+14 more)

### Community 124 - "TextBuffer"
Cohesion: 0.08
Nodes (12): delimiter_partner(), find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at(), Default, Fn (+4 more)

### Community 125 - "LspCodeAction"
Cohesion: 0.11
Nodes (12): code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), LspCodeAction, LspDocumentTextEdits, LspTextEdit, parse_code_action_document_change(), parse_code_action_item(), parse_code_action_response() (+4 more)

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (46): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+38 more)

### Community 127 - "Option"
Cohesion: 0.14
Nodes (7): CommandPaletteState, CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "editor-dap/src/client.rs"
Cohesion: 0.09
Nodes (30): attach_arguments(), build_csharp_fixture(), configure_adapter_command(), connect_transport(), DapExecutionPosition, DapSessionEvent, DapStackFrameInfo, DapThreadInfo (+22 more)

### Community 132 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 133 - "ROption"
Cohesion: 0.10
Nodes (19): exported_statusline_render(), statusline_context_from_abi(), AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiLspDiagnosticsInfo, AbiSectionAction, AbiSectionItem, AbiStatuslineContext (+11 more)

### Community 134 - "git_probe_snapshot"
Cohesion: 0.35
Nodes (20): git_available(), git_probe_generation(), git_probe_numstat_spawns_once_until_head_or_index_changes(), git_probe_snapshot(), git_probe_snapshot_hides_detached_head_from_dock(), git_probe_snapshot_matches_rev_parse_and_reuses_identity(), git_probe_snapshot_non_git_root_is_absent_without_spawn(), git_probe_snapshot_shares_cache_across_canonical_roots() (+12 more)

### Community 135 - "Vec"
Cohesion: 0.09
Nodes (13): EventLog, format_micros_as_millis(), LspState, panic_payload_message(), Any, AutocompleteProvider, Box, ContextHelpSpec (+5 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "editor-terminal/src/lib.rs"
Cohesion: 0.09
Nodes (30): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+22 more)

### Community 138 - "git_worktree_dashboard_picker_overlay"
Cohesion: 0.16
Nodes (18): git_commit_temp_path(), git_common_dir(), git_worktree_dashboard_picker_overlay(), git_worktree_list(), git_worktree_list_parser_normalizes_windows_drive_paths(), GitHeadBlobKey, GitWorktreeListEntry, parse_git_worktree_list() (+10 more)

### Community 139 - "AcpPaneState"
Cohesion: 0.16
Nodes (8): acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), AcpPaneState, Default

### Community 140 - ".new"
Cohesion: 0.09
Nodes (28): AsyncRead, buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), humanize_debug_label(), open_permission_request_reorders_queue_for_requested_picker(), permission_prompt_lines(), permission_prompt_lines_show_locations_and_choices() (+20 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "DbService"
Cohesion: 0.13
Nodes (15): DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbQueryBufferMeta, DbService, DbSession, DbSessionId (+7 more)

### Community 143 - ".new"
Cohesion: 0.14
Nodes (39): attach_session(), close_buffer_keeps_session_alive_for_next_file(), close_then_open_then_incremental_edits_work_again(), did_open_still_sends_full_text(), file_uri_roundtrip_handles_windows_paths(), full_sync_sends_null_range_and_full_text_even_with_edits(), incremental_did_change_emits_one_event_per_contiguous_edit(), incremental_did_change_includes_newline_in_range_and_text() (+31 more)

### Community 145 - "treesitter_install.rs"
Cohesion: 0.27
Nodes (26): apply_tree_sitter_recompile_notification(), continue_next_tree_sitter_recompile(), continue_tree_sitter_install(), continue_tree_sitter_install_after_clone(), continue_tree_sitter_install_after_generate(), continue_tree_sitter_recompile(), continue_tree_sitter_recompile_after_clone(), continue_tree_sitter_recompile_after_generate() (+18 more)

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "String"
Cohesion: 0.31
Nodes (20): apply_language_options_table(), parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option(), parse_options_table() (+12 more)

### Community 148 - "PickerSession"
Cohesion: 0.17
Nodes (6): PickerResultOrder, PickerSession, selection_skips_divider_rows(), selection_wraps_across_match_list(), set_items_preserves_selected_id_when_still_matched(), source_order_preserves_input_order()

### Community 149 - "HighlightDocument"
Cohesion: 0.18
Nodes (3): BufferStats, HighlightDocument, Vec

### Community 150 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 151 - "editor-syntax/src/lib.rs"
Cohesion: 0.05
Nodes (65): B, aligned_indent_column(), apply_text_edits_to_span(), capture_requires_theme_token(), changed_range_windows(), collect_structure_nodes(), command_failure_message(), current_line_starts_with_token() (+57 more)

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 153 - "install_test_lsp_manager"
Cohesion: 0.27
Nodes (10): apply_pending_lsp_state_clears_diagnostics_after_session_disconnect(), apply_pending_lsp_state_does_nothing_without_lsp_enabled_buffers(), apply_pending_lsp_state_refreshes_attached_server_label_when_session_set_changes(), apply_pending_lsp_state_refreshes_only_paths_whose_diagnostics_changed(), apply_pending_lsp_state_skips_diagnostic_lookups_when_generation_unchanged(), apply_pending_lsp_state_skips_log_snapshot_until_revision_moves(), apply_pending_lsp_state_toasts_only_when_notification_revision_moves(), install_lsp_enabled_file_buffer() (+2 more)

### Community 154 - ".send"
Cohesion: 0.14
Nodes (37): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+29 more)

### Community 155 - "LspSessionHandle"
Cohesion: 0.08
Nodes (38): ChildStdin, LspClientState, LspReaderSession, LspSessionHandle, LspSessionSharedState, note_session_disconnect_diagnostics(), record_published_diagnostics(), record_transport_entry() (+30 more)

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

### Community 161 - "Self"
Cohesion: 0.04
Nodes (51): DebugAdapterRootStrategy, AbiCaptureThemeMapping, AbiDebugAdapterRootStrategy, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGitLogEntry, AbiGrammarSource, AbiIconFontCategory (+43 more)

### Community 162 - "InstallRecipe"
Cohesion: 0.21
Nodes (10): github_release_builds_latest_download_url(), InstallRecipe, AsRef, Into, IntoIterator, Item, Option, Self (+2 more)

### Community 163 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "PaneConfig"
Cohesion: 0.07
Nodes (16): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, MarkdownPrettyConfig, PickerLayout, ShowParenConfig (+8 more)

### Community 166 - "String"
Cohesion: 0.54
Nodes (4): call_function(), Parser<'a, 'b>, Result, String

### Community 167 - "user/terminal.rs"
Cohesion: 0.21
Nodes (11): default_terminal_args(), default_terminal_program(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package() (+3 more)

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

### Community 173 - "TerminalEventWake"
Cohesion: 0.20
Nodes (12): AlacrittyEvent, QueuedEventListener, Arc, Debug, Fn, Send, Sender, Sync (+4 more)

### Community 174 - "headerline_lines"
Cohesion: 0.29
Nodes (7): build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option, String, Vec, special_buffer_headerline()

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "build_job_command"
Cohesion: 0.29
Nodes (9): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, run_job(), windows_fnm_environment(), configure_background_command() (+1 more)

### Community 177 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - ".recompute_matches"
Cohesion: 0.22
Nodes (4): contiguous_substring_beats_split_path_match(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), push_ascii_lowercase()

### Community 180 - "Vec"
Cohesion: 0.18
Nodes (14): autocomplete_items(), autocomplete_provider(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_symbols(), CalculatorSymbol, hover_lines(), hover_provider() (+6 more)

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

### Community 189 - "LspNotification"
Cohesion: 0.06
Nodes (19): completion_level_for_message(), copilot_status_notifications_offer_sign_in_action(), last_notification_params(), log_and_notification_revision_skip_cloning_unchanged_snapshots(), LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotification (+11 more)

### Community 190 - ".move_object_end_forward"
Cohesion: 0.24
Nodes (7): is_object_separator(), is_punctuation_char(), is_word_char(), matches_word_kind(), word_motion_class(), WordKind, WordMotionClass

### Community 191 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - ".new"
Cohesion: 0.29
Nodes (3): Lexer<'a>, Self, Token

### Community 194 - ".from_text"
Cohesion: 0.09
Nodes (48): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits(), from_reader_normalizes_crlf_and_tracks_line_endings() (+40 more)

### Community 195 - "PickerItem"
Cohesion: 0.18
Nodes (7): PickerItem, PickerMatch, Into, Option, Self, String, set_item_preview_updates_selected_match_without_filling_other_rows()

### Community 196 - "AbiSectionTree"
Cohesion: 0.18
Nodes (9): exported_git_status_sections(), DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 197 - "begin_oil_worktree_request"
Cohesion: 0.24
Nodes (10): begin_oil_worktree_request(), git_branch_list(), git_remote_worktree_branch_list(), git_worktree_create_command(), oil_git_worktree_command(), open_git_worktree_branch_picker(), open_git_worktree_dashboard_create(), remote_and_branch_from_ref() (+2 more)

### Community 198 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbIndex, DbSnippet, PersistedDbState, QualifiedName, RememberedConnection

### Community 199 - "Option"
Cohesion: 0.07
Nodes (25): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), collapse_variable_path(), DapStoppedSnapshot, DapVariableNode, DapVariablePath, DapVariableRow (+17 more)

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

### Community 201 - ".byte_slice_chunks"
Cohesion: 0.24
Nodes (7): Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 202 - "AbiPdfOpenMode"
Cohesion: 0.24
Nodes (7): exported_pdf_open_mode(), PdfOpenMode, open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 203 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 204 - "Value"
Cohesion: 0.07
Nodes (39): BufRead, configuration_item_section(), CopilotDeviceCodePrompt, csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item(), format_transport_message() (+31 more)

### Community 205 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 206 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 207 - "UserLibraryModule"
Cohesion: 0.09
Nodes (22): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiIconFontSymbol, AbiOilDefaults, AbiOilFeatureSpec, AbiOilKeybindings, AbiOilSortMode, AbiPickerTruncateStrategy (+14 more)

### Community 209 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 210 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 211 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 212 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 214 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 215 - "highlight.rs"
Cohesion: 0.39
Nodes (8): bench_highlight_rust(), bench_highlight_rust_window(), Language, String, rust_fixture(), rust_language(), rust_registry(), Criterion

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "Option"
Cohesion: 0.11
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerSession, LspError, path_is_solution(), resolve_single_solution_path() (+10 more)

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "main"
Cohesion: 0.15
Nodes (13): bootstrap(), HostBootstrap, command_palette_items(), load_user_library(), main(), Arc, DebugAdapterSpec, Error (+5 more)

### Community 221 - "AcpEvent"
Cohesion: 0.07
Nodes (35): AvailableCommand, AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome(), command_input_hint() (+27 more)

### Community 223 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 224 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 225 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 227 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 228 - "evaluate_expression"
Cohesion: 0.47
Nodes (4): DapEvaluateContext, evaluate_expression(), EvaluateArgumentsContext, From

### Community 229 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 230 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 232 - "StartupTrace"
Cohesion: 0.50
Nodes (3): Instant, Self, StartupTrace

### Community 234 - "String"
Cohesion: 0.07
Nodes (26): append_query_source(), CaptureThemeMapping, cmake_configuration(), dockerfile_configuration(), GrammarSource, installable_rust_configuration(), LanguageConfiguration, LanguageLoader (+18 more)

### Community 235 - "RVec"
Cohesion: 0.13
Nodes (14): exported_terminal_config(), AbiAcpClient, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, AcpClient, HoverProvider, HoverProviderTopic (+6 more)

### Community 236 - ".terminal_output"
Cohesion: 0.50
Nodes (3): apply_output_limit(), TerminalOutputRequest, TerminalOutputResponse

### Community 237 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 244 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 247 - ".request"
Cohesion: 0.40
Nodes (4): Arguments, parse_response_body(), strip_null_fields(), Response

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 340 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 341 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 347 - "editor-core/src/lib.rs"
Cohesion: 0.17
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 409 - "buffer_footer_layout_with_command_line"
Cohesion: 0.17
Nodes (21): browser_buffer_layout(), browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Rect (+13 more)

## Knowledge Gaps
- **156 isolated node(s):** `BufferChrome<'a>`, `ShellWakeupEvent`, `StartupProfile`, `topbar`, `navToggle` (+151 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `Option`, `ShellState`, `shell_ui`, `git_worktree_dashboard_picker_overlay`, `shell/git.rs`, `treesitter_install.rs`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `state_with_user_library`, `install_test_lsp_manager`, `command_stream.rs`, `TextPoint`, `.new`, `SectionLineMeta`, `shell/pdf.rs`, `resolve_permission`, `ServiceRegistry`, `Option`, `trigger_autocomplete`, `open_slash_command_picker`, `String`, `AcpManager`, `BufferId`, `String`, `ShellUiState`, `begin_oil_worktree_request`, `.new`, `String`, `LineSyntaxSpan`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `Result`, `tool_install.rs`, `editor-plugin-host/src/lib.rs`, `editor-core/src/lib.rs`, `CommandSource`, `AcpEvent`, `main`, `PickerOverlay`, `GitEditorState`, `shell/tests.rs`, `.path`, `shell/picker.rs`, `shell/browser.rs`?**
  _High betweenness centrality (0.148) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `Option`, `ShellState`, `editor-terminal/src/lib.rs`, `draw.rs`, `shell/git.rs`, `buffer_footer_layout_with_command_line`, `state_with_user_library`, `TextPoint`, `.new`, `shell/pdf.rs`, `shell/mod.rs`, `clipboard.rs`, `shell/acp.rs`, `editor-markdown/src/lib.rs`, `trigger_autocomplete`, `open_slash_command_picker`, `String`, `ShellError`, `.len`, `ShellUiState`, `String`, `LineSyntaxSpan`, `directory.rs`, `shell/terminal.rs`, `Result`, `wrap_line_segments`, `PickerOverlay`, `render.rs`, `shell/picker.rs`, `shell/browser.rs`, `TextBuffer`, `StoredBreakpoint`?**
  _High betweenness centrality (0.054) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `rainbow_parens.rs`, `.new`, `capture_mappings`, `user/lib.rs`, `calculator.rs`, `oil.rs`, `show_paren.rs`, `workspace.rs`, `.new`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `user/terminal.rs`, `user/dap.rs`, `WorkspaceDockConfig`, `user/browser.rs`, `HeaderlineTestUserLibrary`, `Self`, `lsp.rs`, `Vec`, `markdown.rs`, `user/db.rs`, `UserLibraryModule`, `clojure.rs`, `graphql.rs`, `hcl.rs`, `syntax_languages`, `latex.rs`, `editor-plugin-host/src/lib.rs`, `main`, `ruby.rs`, `syntax_language`, `syntax_language`, `PluginCommand`, `.default`, `PluginKeyBinding`, `picker_items`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **What connects `BufferChrome<'a>`, `ShellWakeupEvent`, `StartupProfile` to the rest of the system?**
  _156 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `DebugConfiguration` be split into smaller, more focused modules?**
  _Cohesion score 0.13306451612903225 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.07944307944307945 - nodes in this community are weakly interconnected._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.05962732919254658 - nodes in this community are weakly interconnected._