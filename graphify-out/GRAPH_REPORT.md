# Graph Report - volt  (2026-08-25)

## Corpus Check
- 258 files · ~645,798 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 10680 nodes · 43711 edges · 329 communities (314 shown, 15 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3479 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `9ed3e6b5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- DebugConfiguration
- Path
- Option
- .new
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- editor-lsp/src/client.rs
- LanguageServerSpec
- render.rs
- draw.rs
- package
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
- FontSet
- String
- TerminalCursorSnapshot
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- shell/mod.rs
- Self
- ShellBuffer
- .len
- String
- shell/browser.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- String
- Option
- DebugConfigurationCandidate
- SyntaxRegistry
- HeaderlineTestUserLibrary
- PathBuf
- theme.rs
- lsp.rs
- render_buffer
- Vec
- DapClientManager
- KeymapError
- active_runtime_popup
- build_output.rs
- key_sequence.rs
- .get
- BufferId
- PluginCommand
- ShellUiState
- SyntaxText
- RVec
- AbiGitStatusSnapshot
- shell/tests.rs
- .new
- DbBrowserBufferView
- LineSyntaxSpan
- PluginPackage
- Option
- acp_rendered_text_wrap_cols
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- String
- Section
- PathBuf
- tool_install.rs
- InstallCommand
- WorkspaceDockBranchCache
- find_font_by_name
- diagnostics.rs
- HighlightDocument
- editor-picker/src/lib.rs
- editor-plugin-host/src/lib.rs
- CommandSource
- Vec
- crates/editor-syntax/tests/registered_queries.rs
- workspace_nav.rs
- AbiLanguageConfiguration
- .line_count
- GitEditorState
- modeline.rs
- editor-lsp/src/lib.rs
- .spawn
- Result
- .from
- .new
- ToolInstallError
- DapLogEntry
- TextBuffer
- JobSpec
- shell/picker.rs
- DapSessionInfo
- user/git.rs
- AbiSectionTree
- build_job_command
- PluginKeyBinding
- DebugSessionPlan
- LspCodeAction
- String
- nix.rs
- process_supervisor.rs
- Vec
- PixelRect
- DapSessionHandle
- DebugAdapterSpec
- Vec
- DbService
- StoredBreakpoint
- String
- run_job
- editor-terminal/src/lib.rs
- user/config.rs
- editor-dap/src/client.rs
- normalize_extension
- PickerItemSpec
- StatuslineContext
- TerminalTranscript
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- cmake.rs
- syntax_language
- load_user_library
- shell/acp.rs
- CommandLineOverlay
- .next_token
- .new
- UserLibrary
- OilDefaultsSection
- volt/build.rs
- .load_from_path
- swift.rs
- handle_git_status_chord
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
- rainbow_parens.rs
- LineEnding
- editor-path/src/lib.rs
- user/dap.rs
- JobResult
- treesittercontext_ghosttext.rs
- user/browser.rs
- I
- connect_sql_server
- `user`
- String
- predicate_capture_text
- Result
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- panic_payload_message
- Result
- Option
- volt/src/main.rs
- LspLogEntry
- .handle_event
- common.rs
- Database Explorer PRD
- centered_rect
- .from_text
- terminal_key_for_event
- Option
- markdown.rs
- normalize_inline_text
- Option
- 0004-markdown-pretty-pipeline.md
- DbEngine
- main
- capture_mappings
- dap-client-spec.md
- highlight.rs
- Language
- Self
- Domain Docs
- Issue tracker: GitHub
- main
- AcpEvent
- .default
- rainbow_paren.rs
- evaluate_expression
- BufferId
- String
- .oil_directory_sections
- clipboard.rs
- GhostTextContext
- picker_items
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- ruby.rs
- scala.rs
- Agent skills
- treesitter_install.rs
- 0005-dap-session-and-client.md
- AbiLigatureConfig
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- ligatures.rs
- 0006-language-server-and-debug-adapter-install.md
- AbiPdfOpenMode
- browser_host.rs
- editor-core/src/lib.rs
- TextPoint
- ShellConfig
- .path
- Diagnostic
- shell/git.rs
- browser_buffer_layout
- index_syntax_lines
- spawn_terminal_reader
- user/lang/bash.rs
- user/lang/clojure.rs
- user/lang/elixir.rs
- user/lang/graphql.rs
- user/lang/hcl.rs
- user/lang/java.rs
- user/lang/kotlin.rs
- user/lang/lua.rs
- user/lang/perl.rs
- user/lang/php.rs
- user/lang/proto.rs
- user/lang/r.rs
- user/lang/solidity.rs
- user/lang/vim.rs
- user/lang/xml.rs
- user/workspace_dock.rs
- syntax_language
- syntax_language
- package
- user/keymap.rs

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 882 edges
2. `shell_ui_mut()` - 390 edges
3. `ShellBuffer` - 389 edges
4. `register_shell_hooks()` - 272 edges
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

## Communities (329 total, 15 thin omitted)

### Community 0 - "DebugConfiguration"
Cohesion: 0.13
Nodes (10): DebugConfiguration, DebugRequestKind, Into, IntoIterator, Item, Iterator, Option, PathBuf (+2 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (19): BufRead, ClientCapabilities, client_capabilities(), is_copilot_server(), LspClientError, LspClientManager, LspSignatureHelpContents, read_message() (+11 more)

### Community 2 - "Option"
Cohesion: 0.07
Nodes (63): parse_log_oneline(), build_git_fringe_snapshot_with_cache(), build_git_summary_snapshot(), classify_head_blob(), command_output_transcript(), create_git_worktree_from_query(), git_branch_merge(), git_branch_push_remote() (+55 more)

### Community 3 - ".new"
Cohesion: 0.10
Nodes (83): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), begin_project_discovery_test(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), discovery_fixture(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change() (+75 more)

### Community 4 - "ShellError"
Cohesion: 0.03
Nodes (87): Display, Error, From, ShellError, browser_sync_plan(), clear_key_sequence(), active_lsp_workspace_loaded(), active_runtime_surface() (+79 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (117): bundled_highlight_query(), cached_syntax_languages(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers() (+109 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.09
Nodes (75): B, additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+67 more)

### Community 7 - "editor-lsp/src/client.rs"
Cohesion: 0.04
Nodes (94): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations() (+86 more)

### Community 8 - "LanguageServerSpec"
Cohesion: 0.11
Nodes (6): LanguageServerRegistry, LanguageServerSpec, InstallRecipe, Iterator, LanguageServerRootStrategy, Vec

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (114): WrapCollect, advance_point_by_text(), resolved_tab_width(), acp_pane_body_visible_rows(), adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines(), autocomplete_visible_start() (+106 more)

### Community 10 - "draw.rs"
Cohesion: 0.08
Nodes (52): AcpBufferDraw, AcpPaneDraw, AcpPrefixDraw, BrowserBufferDraw, BrowserSyncView, BufferBodyPalette, BufferChrome, BufferChrome<'a> (+44 more)

### Community 11 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 12 - "Self"
Cohesion: 0.03
Nodes (42): dashboard_sections(), sidebar_sections(), exported_acp_picker_items(), AcpActionSpec, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, AcpPickerOption (+34 more)

### Community 13 - "GitSummaryState"
Cohesion: 0.11
Nodes (15): git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, refresh_git_fringe(), refresh_pending_git_summary() (+7 more)

### Community 14 - "editor-dap/src/config.rs"
Cohesion: 0.16
Nodes (31): collect_configuration_candidates(), configuration_holes(), configuration_holes_detect_missing_launch_program(), DapConfigError, DebugInferContext, deep_inference_finds_cargo_binary_and_heuristic(), deep_inference_finds_dotnet_dll(), default_workspace_skips_deep_inference() (+23 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.08
Nodes (22): AlacrittyEvent, LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc, Display, Drop, Error (+14 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.05
Nodes (92): Condvar, compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind (+84 more)

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
Cohesion: 0.13
Nodes (19): BindingKey, ChordModifier, KeyBinding, KeymapRegistry, KeymapScope, KeymapVimMode, normalize_chord(), normalize_chord_token() (+11 more)

### Community 23 - "calculator.rs"
Cohesion: 0.08
Nodes (29): autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c(), calculator_package_binds_ctrl_tab_to_switch_panes() (+21 more)

### Community 24 - "paths.rs"
Cohesion: 0.13
Nodes (24): ToolKind, is_volt_install_path(), locate_program(), ProgramLocation, Path, PathBuf, apply_install_bins_to_process_path(), bin_dir() (+16 more)

### Community 25 - "state_with_user_library"
Cohesion: 0.07
Nodes (91): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), start_dap_for_active_workspace(), stop_dap_for_active_workspace(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk() (+83 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.09
Nodes (52): clear_window_surface(), overlay_window_surface_color(), window_surface_color(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested() (+44 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.10
Nodes (47): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+39 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.14
Nodes (24): font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches(), font_style_rank(), golden_split_size(), horizontal_golden_ratio_grows_the_first_active_pane() (+16 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (32): AutocompleteProviderKind, RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteProviderSpec, AutocompleteRegistry, HoverOverlay (+24 more)

### Community 30 - "ThemeRegistry"
Cohesion: 0.08
Nodes (27): acp_chat_corner_radius(), acp_chat_rounded(), amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap (+19 more)

### Community 31 - "FontSet"
Cohesion: 0.06
Nodes (63): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, PathBuf, ThemeRuntimeSlots (+55 more)

### Community 32 - "String"
Cohesion: 0.06
Nodes (129): run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit() (+121 more)

### Community 33 - "TerminalCursorSnapshot"
Cohesion: 0.31
Nodes (4): TerminalCursorDraw, terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

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
Nodes (400): EditorRuntime, Default, Cow, write_system_clipboard(), yank_to_clipboard_text(), resolve_external_invocation(), accept_autocomplete(), activate_db_browser_line() (+392 more)

### Community 40 - "Self"
Cohesion: 0.12
Nodes (19): ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), default_rainbow_parens_enabled(), default_show_paren_enabled() (+11 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.02
Nodes (96): acp_output_header_title(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_rendered_text_segments() (+88 more)

### Community 42 - ".len"
Cohesion: 0.07
Nodes (19): byte_index_for_char_column(), char_at_index(), find_char_forward(), fuzzy_match_end(), input_charwise_motion_range(), InputField, LineCharMap, matches_pattern_at() (+11 more)

### Community 43 - "String"
Cohesion: 0.11
Nodes (53): acp_image_mention_token(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+45 more)

### Community 44 - "shell/browser.rs"
Cohesion: 0.11
Nodes (40): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_state_for_kind(), browser_surface_buffer_at_point(), browser_url_candidates() (+32 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.08
Nodes (68): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+60 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (65): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+57 more)

### Community 47 - "String"
Cohesion: 0.12
Nodes (14): language_server_spec_exposes_workspace_configuration_builders(), normalize_optional_string(), AsRef, From, Into, IntoIterator, Item, Number (+6 more)

### Community 48 - "Option"
Cohesion: 0.11
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerSession, LspError, path_is_solution(), resolve_single_solution_path() (+10 more)

### Community 49 - "DebugConfigurationCandidate"
Cohesion: 0.12
Nodes (12): DebugConfigurationCandidate, DebugConfigurationSource, DebugStartHistory, DebugStartRecord, default_request(), history_records_last_and_recent(), Into, Item (+4 more)

### Community 50 - "SyntaxRegistry"
Cohesion: 0.10
Nodes (24): compile_query_source(), ensure_cloned_grammar_dir_exists(), html_language(), io_error(), LanguageInstallPlan, parse_query_inherits(), remove_compiler_sidecar_artifacts(), remove_legacy_grammar_install_directory() (+16 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.02
Nodes (76): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), active_input_prompt_text(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays() (+68 more)

### Community 52 - "PathBuf"
Cohesion: 0.03
Nodes (99): acp_decode_image(), active_theme_state_path(), asset_path_from_parts(), BackingFileFingerprint, cleanup_formatter_temp(), clear_saved_theme_selection(), collect_workspace_language_ids(), decode_raster_image_bytes() (+91 more)

### Community 53 - "theme.rs"
Cohesion: 0.11
Nodes (55): packages(), LanguageConfiguration, Vec, syntax_languages(), apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors() (+47 more)

### Community 54 - "lsp.rs"
Cohesion: 0.17
Nodes (23): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), clojure_lsp_recipe(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), install_recipe_for_language_server(), language_servers() (+15 more)

### Community 55 - "render_buffer"
Cohesion: 0.11
Nodes (91): render_browser_buffer_body(), CellMetrics, ScreenHit, adjust_color(), blend_color(), debug_fringe_cell_count(), DrawTarget, editor_fringe_width_px() (+83 more)

### Community 56 - "Vec"
Cohesion: 0.02
Nodes (167): ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter() (+159 more)

### Community 57 - "DapClientManager"
Cohesion: 0.16
Nodes (13): active_thread_id(), clear_stopped_snapshot(), connect_tcp(), DapClientError, DapClientManager, Display, Error, Formatter (+5 more)

### Community 58 - "KeymapError"
Cohesion: 0.25
Nodes (15): autocomplete_overrides_workspace_while_active(), dap_mode_overrides_global_f5_while_session_live(), duplicate_detection_uses_canonical_chords(), global_is_fallback_when_no_minor_mode_claims_chord(), hover_overrides_workspace_while_active(), KeymapError, popup_mode_does_not_claim_workspace_dock_chords(), popup_overrides_workspace_and_global_while_active() (+7 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.10
Nodes (59): active_runtime_popup(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), active_and_secondary_buffer_ids(), add_linked_worktree(), configure_file_buffer(), fetch_git_prune_is_silent_command_without_popup(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers() (+51 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 62 - ".get"
Cohesion: 0.17
Nodes (19): column_is_numeric(), DbAutocompleteCandidate, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns() (+11 more)

### Community 63 - "BufferId"
Cohesion: 0.19
Nodes (15): AcpClientConfig, acp_load_session(), acp_new_session(), active_acp_client(), close_acp_buffer(), close_acp_workspace_buffers(), create_acp_buffer(), focus_acp_buffer() (+7 more)

### Community 64 - "PluginCommand"
Cohesion: 0.07
Nodes (24): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+16 more)

### Community 65 - "ShellUiState"
Cohesion: 0.04
Nodes (44): active_or_open_dashboard_buffer(), active_runtime_buffer(), active_window_id(), apply_db_results_to_output_section(), buffer_is_dap_layout_side(), BufferViewState, close_db_multiview(), close_runtime_pane() (+36 more)

### Community 66 - "SyntaxText"
Cohesion: 0.14
Nodes (29): SyntaxText, apply_text_edits_to_span(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), highlight_inline_language_per_line(), highlight_loaded_language() (+21 more)

### Community 67 - "RVec"
Cohesion: 0.13
Nodes (14): exported_terminal_config(), AbiAcpClient, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, AcpClient, HoverProvider, HoverProviderTopic (+6 more)

### Community 68 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 69 - "shell/tests.rs"
Cohesion: 0.03
Nodes (137): acp_buffer_layout(), buffer_footer_layout(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_input_field_visual_yank_copies_selected_text(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail() (+129 more)

### Community 70 - ".new"
Cohesion: 0.10
Nodes (37): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_head_blob_cache_reuses_text_for_same_head(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command() (+29 more)

### Community 71 - "DbBrowserBufferView"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 72 - "LineSyntaxSpan"
Cohesion: 0.11
Nodes (50): dap_variable_line_spans(), browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines() (+42 more)

### Community 73 - "PluginPackage"
Cohesion: 0.06
Nodes (42): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+34 more)

### Community 74 - "Option"
Cohesion: 0.06
Nodes (38): aligned_indent_column(), append_query_source(), collect_structure_nodes(), create_parser(), current_line_starts_with_token(), DeferredQuery, delimiter_column(), desired_indent_for_loaded_language() (+30 more)

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
Nodes (74): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+66 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.12
Nodes (41): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+33 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "String"
Cohesion: 0.07
Nodes (102): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+94 more)

### Community 82 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 83 - "PathBuf"
Cohesion: 0.13
Nodes (9): asset_path_from_parts(), default_install_root(), default_query_asset_root(), GrammarSource, resolve_query_asset_root_from_roots(), Drop, PathBuf, shared_library_file_name() (+1 more)

### Community 84 - "tool_install.rs"
Cohesion: 0.19
Nodes (40): apply_tool_install_finish(), begin_explicit_install(), continue_tool_install(), fail_tool_install(), fail_tool_install_with_message(), handle_dap_install_hook(), handle_lsp_install_hook(), install_debug_adapter_by_id() (+32 more)

### Community 85 - "InstallCommand"
Cohesion: 0.16
Nodes (20): program_is_available(), archive_commands(), command(), commands_for_recipe(), dotnet_prerelease_passes_flag(), InstallCommand, InstallPlan, npm_recipe_uses_prefix() (+12 more)

### Community 86 - "WorkspaceDockBranchCache"
Cohesion: 0.13
Nodes (19): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+11 more)

### Community 87 - "find_font_by_name"
Cohesion: 0.26
Nodes (15): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), pick_best_matching_font_path(), preferred_berkeley_mono_font(), preferred_berkeley_mono_font_candidates(), preferred_font_search_roots(), RenderError (+7 more)

### Community 88 - "diagnostics.rs"
Cohesion: 0.14
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "HighlightDocument"
Cohesion: 0.15
Nodes (3): BufferStats, HighlightDocument, Vec

### Community 90 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (46): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+38 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.14
Nodes (35): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames(), file_open_hook_filters_match_globs() (+27 more)

### Community 92 - "CommandSource"
Cohesion: 0.09
Nodes (17): CommandHandler, CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, BTreeMap, Display (+9 more)

### Community 93 - "Vec"
Cohesion: 0.12
Nodes (8): EventLog, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, HoverProvider, Vec, WorkspaceRoot

### Community 94 - "crates/editor-syntax/tests/registered_queries.rs"
Cohesion: 0.15
Nodes (35): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+27 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 97 - ".line_count"
Cohesion: 0.15
Nodes (5): EditRecord, String, trimmed_line(), visible_line_len(), RopeSlice

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+20 more)

### Community 101 - ".spawn"
Cohesion: 0.11
Nodes (21): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator (+13 more)

### Community 102 - "Result"
Cohesion: 0.04
Nodes (97): ctrl_mod(), cycle_runtime_pane(), shell_ui(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_text_input_updates_acp_input(), browser_buffer_submit_tracks_requested_navigation(), browser_host_focus_parent_event_returns_to_normal_mode(), browser_host_new_window_event_routes_into_browser_popup() (+89 more)

### Community 103 - ".from"
Cohesion: 0.05
Nodes (56): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), abi_debug_adapter_spec_round_trips_install_recipe(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_install_recipe(), abi_language_server_spec_round_trips_path_matchers(), abi_language_server_spec_round_trips_workspace_configuration() (+48 more)

### Community 104 - ".new"
Cohesion: 0.17
Nodes (19): browser_host_event_for_ipc(), BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, DesktopBrowserHostService, optional_non_empty_text(), BTreeMap (+11 more)

### Community 105 - "ToolInstallError"
Cohesion: 0.16
Nodes (25): Display, Error, Formatter, From, Result, Self, String, ToolInstallError (+17 more)

### Community 106 - "DapLogEntry"
Cohesion: 0.26
Nodes (6): assert_control_requests_omit_nulls(), dap_log_text(), DapLogDirection, DapLogEntry, DapLogSnapshot, DapTransportLog

### Community 107 - "TextBuffer"
Cohesion: 0.08
Nodes (12): delimiter_partner(), find_matching_close_tag(), is_inline_whitespace(), is_sentence_closer(), parse_tag_token(), parse_tag_token_at(), Default, Fn (+4 more)

### Community 108 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.12
Nodes (37): UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_entries(), picker_fringe_width_chars() (+29 more)

### Community 111 - "user/git.rs"
Cohesion: 0.11
Nodes (48): commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title(), help_entry() (+40 more)

### Community 112 - "AbiSectionTree"
Cohesion: 0.18
Nodes (9): exported_git_status_sections(), DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree, AbiSectionTree, SectionTree (+1 more)

### Community 113 - "build_job_command"
Cohesion: 0.32
Nodes (8): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, windows_fnm_environment(), configure_background_command(), Command

### Community 114 - "PluginKeyBinding"
Cohesion: 0.10
Nodes (26): plugin_buffer_binding_scope_active(), plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, hook_command(), leader_binding() (+18 more)

### Community 115 - "DebugSessionPlan"
Cohesion: 0.22
Nodes (3): DebugAdapterTransport, DebugSessionPlan, DapState

### Community 116 - "LspCodeAction"
Cohesion: 0.10
Nodes (12): formatting_parser_maps_text_edits(), inline_completion_params(), lsp_formatting_options(), LspCodeAction, LspDocumentTextEdits, LspFormattingOptions, LspTextEdit, parse_text_edit_response() (+4 more)

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

### Community 121 - "PixelRect"
Cohesion: 0.24
Nodes (20): PixelRect, rect_tuple(), along_size(), child_rect(), gap_is_inserted_between_siblings(), layout_child(), layout_node(), layout_split_tree() (+12 more)

### Community 122 - "DapSessionHandle"
Cohesion: 0.11
Nodes (33): DapReaderSession, DapSessionHandle, fake_adapter_loop(), fake_variables_for_reference(), mark_session_ended(), PendingResponse, read_frame(), record_transport_event_inner() (+25 more)

### Community 123 - "DebugAdapterSpec"
Cohesion: 0.09
Nodes (22): Client, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterRootStrategy, DebugAdapterSpec, gdb(), must() (+14 more)

### Community 124 - "Vec"
Cohesion: 0.27
Nodes (10): autocomplete_items(), calculator_symbols(), CalculatorSymbol, hover_lines(), hover_topics(), initial_buffer_lines(), initial_buffer_lines_only_seed_input_examples(), AutocompleteProviderItem (+2 more)

### Community 125 - "DbService"
Cohesion: 0.13
Nodes (15): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbIndex, DbQueryBufferMeta, DbService (+7 more)

### Community 126 - "StoredBreakpoint"
Cohesion: 0.08
Nodes (45): BreakpointState, BreakpointStore, BreakpointToggle, debug_source_paths_eq(), delete_removes_current_line_breakpoint(), extras_persist_on_stored_breakpoint(), normalize_debug_source_path(), normalize_optional_text() (+37 more)

### Community 127 - "String"
Cohesion: 0.13
Nodes (8): CommandPaletteState, CompilationState, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, String, TerminalState

### Community 128 - "run_job"
Cohesion: 0.15
Nodes (18): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+10 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.10
Nodes (29): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+21 more)

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "editor-dap/src/client.rs"
Cohesion: 0.09
Nodes (30): attach_arguments(), build_csharp_fixture(), configure_adapter_command(), connect_transport(), DapExecutionPosition, DapSessionEvent, DapStackFrameInfo, DapThreadInfo (+22 more)

### Community 132 - "normalize_extension"
Cohesion: 0.31
Nodes (5): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), BTreeMap, normalize_extension()

### Community 133 - "PickerItemSpec"
Cohesion: 0.09
Nodes (31): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+23 more)

### Community 134 - "StatuslineContext"
Cohesion: 0.13
Nodes (11): statusline_context_from_abi(), AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiStatuslineContext, AutocompleteProvider, AutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem (+3 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 138 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 139 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 140 - "shell/acp.rs"
Cohesion: 0.10
Nodes (44): acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_insert_file_mention(), acp_picker_entry(), acp_resolve_permission_option(), buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state() (+36 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 143 - ".new"
Cohesion: 0.17
Nodes (35): attach_session(), close_buffer_keeps_session_alive_for_next_file(), close_then_open_then_incremental_edits_work_again(), did_open_still_sends_full_text(), file_uri_roundtrip_handles_windows_paths(), full_sync_sends_null_range_and_full_text_even_with_edits(), incremental_did_change_emits_one_event_per_contiguous_edit(), incremental_did_change_includes_newline_in_range_and_text() (+27 more)

### Community 144 - "UserLibrary"
Cohesion: 0.05
Nodes (49): BufferKind, buffer_uses_browser_host_surface(), default_vim_target(), active_buffer_event_context(), buffer_context_overlay_snapshot(), buffer_interaction(), buffer_is_acp(), buffer_is_browser() (+41 more)

### Community 145 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - ".load_from_path"
Cohesion: 0.13
Nodes (15): from_reader_normalizes_crlf_and_tracks_line_endings(), must(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean(), AsRef, Drop, E, Into (+7 more)

### Community 148 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 149 - "handle_git_status_chord"
Cohesion: 0.48
Nodes (6): git_status_command_name(), GitPrefixState, handle_git_status_chord(), set_git_prefix(), take_git_prefix(), GitPrefix

### Community 150 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 151 - ".new"
Cohesion: 0.15
Nodes (20): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_volt_state_dir(), insert_test_session(), redact_key_value_segments(), Arc, PathBuf, Self (+12 more)

### Community 152 - "oil.rs"
Cohesion: 0.09
Nodes (37): chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec(), help_entry() (+29 more)

### Community 153 - "user/db.rs"
Cohesion: 0.12
Nodes (27): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings() (+19 more)

### Community 154 - ".send"
Cohesion: 0.12
Nodes (38): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+30 more)

### Community 155 - "LspSessionHandle"
Cohesion: 0.08
Nodes (33): ChildStdin, TextEdit, diagnostic_matches_request_range(), incremental_content_changes(), inserted_text_after_edit(), launch_summary(), LspReaderSession, LspSessionHandle (+25 more)

### Community 156 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 157 - "workspace.rs"
Cohesion: 0.11
Nodes (45): begin_discovery_override(), discovery_test_lock(), DiscoveryOverrideGuard, existing_workspace_for_project(), file_picker_preview(), message_item(), override_project_search_roots_for_test(), package() (+37 more)

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
Cohesion: 0.06
Nodes (19): exported_pane_config(), MarkdownPrettyConfig, PickerLayout, ShowParenConfig, config(), AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig (+11 more)

### Community 162 - "InstallRecipe"
Cohesion: 0.21
Nodes (10): github_release_builds_latest_download_url(), InstallRecipe, AsRef, Into, IntoIterator, Item, Option, Self (+2 more)

### Community 163 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

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
Cohesion: 0.21
Nodes (11): default_terminal_args(), default_terminal_program(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package() (+3 more)

### Community 168 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 169 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

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
Cohesion: 0.06
Nodes (52): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+44 more)

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 177 - "connect_sql_server"
Cohesion: 0.50
Nodes (4): Compat, connect_sql_server(), TcpStream, SqlServerClient

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "String"
Cohesion: 0.09
Nodes (24): TextRange, CopilotDeviceCodePrompt, diagnostics_parser_maps_lsp_fields(), file_uri_to_path(), language_server_session_in_workspace_scope(), LspClientState, LspHoverContents, LspInlineCompletionItem (+16 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

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

### Community 185 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 186 - "Result"
Cohesion: 0.06
Nodes (44): acp_complete_slash(), acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+36 more)

### Community 187 - "Option"
Cohesion: 0.02
Nodes (89): absolute_path_hint(), apply_markdown_code_fence_syntax(), ascii_control_caret_notation(), closing_tag_name_after_cursor(), comment_style_for_buffer(), comment_style_for_language_path(), CommentStyle, compute_buffer_syntax() (+81 more)

### Community 188 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 189 - "LspLogEntry"
Cohesion: 0.10
Nodes (12): last_did_open_text(), last_notification_params(), LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot (+4 more)

### Community 190 - ".handle_event"
Cohesion: 0.12
Nodes (17): acp_pick_model(), acp_picker_entries(), acp_session_buffer_name(), AcpPendingPermissionUi, AcpSessionInfo, config_option_is_mode(), config_option_is_model(), config_option_matches() (+9 more)

### Community 191 - "common.rs"
Cohesion: 0.10
Nodes (27): binding_suffix(), GrammarSourceSpec, GrammarSourceSpec<'a>, package(), package_with_path_matchers(), CaptureThemeMapping, GrammarSource, LanguageConfiguration (+19 more)

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "centered_rect"
Cohesion: 0.67
Nodes (3): centered_rect(), picker_card_rect(), PickerLayout

### Community 194 - ".from_text"
Cohesion: 0.09
Nodes (46): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), edits_since_returns_contiguous_forward_edits(), highlight_document_captures_edits_without_undo_history(), highlight_document_falls_back_to_full_parse_without_contiguous_edits(), is_object_separator() (+38 more)

### Community 195 - "terminal_key_for_event"
Cohesion: 0.67
Nodes (3): Keycode, Mod, terminal_key_for_event()

### Community 199 - "Option"
Cohesion: 0.07
Nodes (25): apply_expanded_paths(), apply_expanded_watch_roots(), capture_stopped_snapshot(), collapse_variable_path(), DapStoppedSnapshot, DapVariableNode, DapVariablePath, DapVariableRow (+17 more)

### Community 200 - "markdown.rs"
Cohesion: 0.19
Nodes (15): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+7 more)

### Community 201 - "normalize_inline_text"
Cohesion: 0.22
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 204 - "Option"
Cohesion: 0.05
Nodes (63): completion_documentation(), completion_level_for_message(), configuration_item_section(), copilot_status_notifications_offer_sign_in_action(), csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item() (+55 more)

### Community 209 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbSnippet, load_persisted_state(), PersistedDbState, RememberedConnection, Path

### Community 210 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 211 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 213 - "dap-client-spec.md"
Cohesion: 0.25
Nodes (7): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories

### Community 215 - "highlight.rs"
Cohesion: 0.39
Nodes (8): bench_highlight_rust(), bench_highlight_rust_window(), Language, String, rust_fixture(), rust_language(), rust_registry(), Criterion

### Community 216 - "Language"
Cohesion: 0.20
Nodes (9): Database, Debugging, External commands, Issues, Language, Language servers, Markdown presentation, Volt (+1 more)

### Community 217 - "Self"
Cohesion: 0.03
Nodes (86): DebugAdapterRootStrategy, GitCommandBinding, GitPrefixBinding, exported_keymap_config(), exported_picker_truncate_strategy(), KeymapConfig, PickerTruncateStrategy, AbiBrowserFeatureSpec (+78 more)

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "main"
Cohesion: 0.17
Nodes (12): bootstrap(), HostBootstrap, command_palette_items(), main(), print_shell_summary(), DebugAdapterSpec, Error, LanguageConfiguration (+4 more)

### Community 221 - "AcpEvent"
Cohesion: 0.10
Nodes (31): AvailableCommand, acp_pick_mode(), AcpCommand, AcpEvent, AcpRuntime, active_command_input_hint(), build_acp_input_hint(), choose_permission_outcome() (+23 more)

### Community 224 - ".default"
Cohesion: 0.10
Nodes (17): CodeActionParams, Self, code_action_params(), code_action_params_use_flattened_lsp_shape(), definition_parser_supports_location_links(), location_from_link(), LspLocation, parse_definition_response() (+9 more)

### Community 226 - "rainbow_paren.rs"
Cohesion: 0.12
Nodes (32): apply_rainbow_delimiter_spans(), apply_rainbow_delimiter_spans_for_buffer(), apply_rainbow_delimiter_spans_inner(), bracket_tokens(), BracketSpan, buffer_apply_matches_contiguous_text_apply(), delimiter_kind(), DelimiterFamily (+24 more)

### Community 228 - "evaluate_expression"
Cohesion: 0.47
Nodes (4): DapEvaluateContext, evaluate_expression(), EvaluateArgumentsContext, From

### Community 229 - "BufferId"
Cohesion: 0.12
Nodes (28): ActiveBufferEventContext, apply_git_status_snapshot(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_line_is_untracked(), git_snapshot_for_buffer(), git_status_delete_target_for_line(), git_status_delete_targets() (+20 more)

### Community 234 - "String"
Cohesion: 0.09
Nodes (21): CaptureThemeMapping, command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport, installable_rust_configuration(), InstallCommandSpec, LanguageConfiguration, LanguageLoader (+13 more)

### Community 237 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 242 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 243 - "GhostTextContext"
Cohesion: 0.19
Nodes (10): GhostTextLine, AbiGhostTextContext, GhostTextContext, build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option, String (+2 more)

### Community 244 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 247 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 248 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

### Community 256 - "treesitter_install.rs"
Cohesion: 0.25
Nodes (27): StreamedCommandOutcome, apply_tree_sitter_recompile_notification(), continue_next_tree_sitter_recompile(), continue_tree_sitter_install(), continue_tree_sitter_install_after_clone(), continue_tree_sitter_install_after_generate(), continue_tree_sitter_recompile(), continue_tree_sitter_recompile_after_clone() (+19 more)

### Community 266 - "AbiLigatureConfig"
Cohesion: 0.32
Nodes (5): exported_ligature_config(), LigatureConfig, AbiLigatureConfig, LigatureConfig, LigatureConfig

### Community 340 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 342 - "browser_host.rs"
Cohesion: 0.11
Nodes (19): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+11 more)

### Community 347 - "editor-core/src/lib.rs"
Cohesion: 0.15
Nodes (20): command_registry_executes_commands_and_hooks_dispatch_events(), EventLog, model_closes_active_pane_without_closing_buffers(), model_focuses_existing_buffer_in_active_pane(), model_splits_pane_and_focuses(), model_switches_and_closes_workspaces(), F, Into (+12 more)

### Community 351 - "TextPoint"
Cohesion: 0.09
Nodes (7): advance_point_by_text(), Self, Selection, TextPoint, TextSnapshot, char_immediately_before(), Rope

### Community 354 - "ShellConfig"
Cohesion: 0.17
Nodes (12): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+4 more)

### Community 373 - ".path"
Cohesion: 0.18
Nodes (13): db_connect_enter_submits_pasted_connection_string(), db_dashboard_execute_replaces_output_and_concatenates_multiple_queries(), db_dashboard_opens_and_writes_files_through_editor_section(), db_query_buffer_receives_sql_highlighting_without_blocking(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir() (+5 more)

### Community 384 - "Diagnostic"
Cohesion: 0.20
Nodes (7): lsp_code_action_diagnostic(), lsp_diagnostic_severity(), parse_diagnostic(), LspDiagnostic, LspDiagnosticSeverity, Diagnostic, DiagnosticSeverity

### Community 386 - "shell/git.rs"
Cohesion: 0.07
Nodes (57): apply_git_fringe_hunk(), begin_oil_worktree_request(), FringeDiffOp, git_branch_list(), git_fringe_snapshot_from_texts(), git_fringe_snapshot_from_texts_ignores_crlf_only_difference(), git_fringe_snapshot_from_texts_is_empty_when_identical(), git_fringe_snapshot_from_texts_marks_all_lines_added_without_head() (+49 more)

### Community 409 - "browser_buffer_layout"
Cohesion: 0.36
Nodes (9): BrowserViewportRect, browser_buffer_layout(), browser_host_viewport_rect(), browser_viewport_rect(), browser_viewport_rect_rect(), BrowserBufferLayout, rects_intersect(), Rect (+1 more)

### Community 410 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 411 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 429 - "user/lang/bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 462 - "user/lang/clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 463 - "user/lang/elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 464 - "user/lang/graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 465 - "user/lang/hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 466 - "user/lang/java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

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

### Community 472 - "user/lang/r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 473 - "user/lang/solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 474 - "user/lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 475 - "user/lang/xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 476 - "user/workspace_dock.rs"
Cohesion: 0.48
Nodes (6): config(), config_defaults_to_left_undocked(), package(), package_binds_j_and_k_in_workspace_dock_scope(), package_exports_dock_navigation_commands(), package_exports_toggle_command()

### Community 481 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 482 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

## Knowledge Gaps
- **155 isolated node(s):** `BufferChrome<'a>`, `StartupProfile`, `topbar`, `navToggle`, `pageSidebar` (+150 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **15 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `shell/mod.rs` to `treesitter_install.rs`, `shell/git.rs`, `Option`, `ShellError`, `shell/acp.rs`, `GitSummaryState`, `UserLibrary`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `handle_git_status_chord`, `state_with_user_library`, `command_stream.rs`, `String`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `String`, `shell/browser.rs`, `PathBuf`, `Vec`, `Result`, `active_runtime_popup`, `Option`, `.handle_event`, `BufferId`, `ShellUiState`, `shell/tests.rs`, `.new`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `String`, `tool_install.rs`, `editor-plugin-host/src/lib.rs`, `editor-core/src/lib.rs`, `CommandSource`, `AcpEvent`, `main`, `GitEditorState`, `BufferId`, `Result`, `shell/picker.rs`, `.path`?**
  _High betweenness centrality (0.134) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `user/lib.rs`, `PickerItemSpec`, `cmake.rs`, `package`, `Self`, `swift.rs`, `calculator.rs`, `oil.rs`, `user/db.rs`, `show_paren.rs`, `workspace.rs`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `package`, `user/terminal.rs`, `rainbow_parens.rs`, `user/dap.rs`, `user/lang/bash.rs`, `sdk/src/lib.rs`, `user/browser.rs`, `HeaderlineTestUserLibrary`, `theme.rs`, `lsp.rs`, `Vec`, `common.rs`, `PluginCommand`, `markdown.rs`, `user/lang/clojure.rs`, `user/lang/elixir.rs`, `user/lang/graphql.rs`, `user/lang/hcl.rs`, `user/lang/java.rs`, `capture_mappings`, `user/lang/kotlin.rs`, `user/lang/lua.rs`, `user/lang/perl.rs`, `user/lang/php.rs`, `user/lang/proto.rs`, `user/lang/r.rs`, `user/lang/solidity.rs`, `editor-plugin-host/src/lib.rs`, `main`, `user/lang/vim.rs`, `user/lang/xml.rs`, `Self`, `user/workspace_dock.rs`, `syntax_language`, `syntax_language`, `package`, `PluginKeyBinding`, `picker_items`, `nix.rs`, `ruby.rs`, `scala.rs`?**
  _High betweenness centrality (0.066) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `editor-terminal/src/lib.rs`, `ShellError`, `render.rs`, `draw.rs`, `GitSummaryState`, `UserLibrary`, `browser_buffer_layout`, `state_with_user_library`, `shell/pdf.rs`, `shell/mod.rs`, `.len`, `shell/browser.rs`, `PathBuf`, `render_buffer`, `Vec`, `Result`, `Option`, `ShellUiState`, `shell/tests.rs`, `.new`, `LineSyntaxSpan`, `directory.rs`, `shell/terminal.rs`, `String`, `diagnostics.rs`, `BufferId`, `TextBuffer`, `shell/picker.rs`, `StoredBreakpoint`?**
  _High betweenness centrality (0.060) - this node is a cross-community bridge._
- **What connects `BufferChrome<'a>`, `StartupProfile`, `topbar` to the rest of the system?**
  _155 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `DebugConfiguration` be split into smaller, more focused modules?**
  _Cohesion score 0.13306451612903225 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.08348794063079777 - nodes in this community are weakly interconnected._
- **Should `Option` be split into smaller, more focused modules?**
  _Cohesion score 0.0673903211216644 - nodes in this community are weakly interconnected._