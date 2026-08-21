# Graph Report - volt  (2026-08-19)

## Corpus Check
- 237 files · ~603,113 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 9687 nodes · 39649 edges · 295 communities (285 shown, 10 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3270 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `b8022804`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- load_font_set_with_mode
- Path
- shell/tests.rs
- .new
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- PickerOverlay
- shell/browser.rs
- render.rs
- AcpEvent
- PluginPackage
- Self
- Option
- String
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- shell/issues.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- Result
- open_workspace_from_project
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- FontSet
- EditorRuntime
- shell/git.rs
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- String
- Self
- Option
- Result
- shell_ui_mut
- shell/mod.rs
- editor-markdown/src/lib.rs
- sdk/src/lib.rs
- Instant
- LanguageServerRegistry
- AbiContextHelpSpec
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- editor-lsp/src/lib.rs
- LanguageServerSpec
- Issue
- render_buffer_with_view_state
- TextPoint
- state.rs
- picker_items
- active_runtime_popup
- build_output.rs
- LspNotification
- .new
- TextBuffer
- PluginCommand
- WorkspaceDockBranchCache
- SyntaxRegistry
- editor-issues/src/lib.rs
- String
- DebugConfiguration
- capture_mappings
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
- PickerItemSpec
- volt/src/main.rs
- Section
- editor-git/src/lib.rs
- Option
- BufferId
- Self
- draw_diagnostic_underlines_for_segment
- show_paren.rs
- RString
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- AbiAutocompleteProvider
- editor-picker/src/lib.rs
- GitEditorState
- modeline.rs
- From
- .spawn
- .new
- .from
- String
- .from
- Path
- ShellConfig
- JobSpec
- shell/picker.rs
- RVec
- .default
- AbiGitFeatureSpec
- open_slash_command_picker
- PluginKeyBinding
- DbBrowserBufferView
- String
- String
- LspLogEntry
- process_supervisor.rs
- AbiDirectoryEntry
- IssueId
- TextRange
- LineSyntaxSpan
- GitSummaryState
- split_runtime_pane
- DynamicUserLibrary
- TerminalTranscript
- JobError
- Option
- user/config.rs
- UserLibraryModule
- key_sequence.rs
- editor-icons/src/lib.rs
- .get
- common.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- build_job_command
- treesittercontext_ghosttext.rs
- DbEngine
- shell/acp.rs
- CommandLineOverlay
- UserLibrary
- AbiPaneConfig
- resolve_permission
- TextEdit
- volt/build.rs
- treesittercontext_shared.rs
- headerline_lines
- client.rs
- ancestor_contexts_for_cursor
- AbiLanguageConfiguration
- oil.rs
- user/db.rs
- lsp.rs
- AbiSectionTree
- LspCompletionItem
- latex.rs
- load
- Copilot instructions for `volt`
- AbiSectionItem
- AcpManager
- centered_rect
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
- predicate_capture_text
- hcl.rs
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- ruby.rs
- java.rs
- perl.rs
- php.rs
- .new
- proto.rs
- Database Explorer PRD
- solidity.rs
- .from_text
- swift.rs
- lang/vim.rs
- Result
- xml.rs
- configure_file_buffer
- markdown.rs
- TerminalCursorSnapshot
- spawn_terminal_reader
- scala.rs
- cargo
- rainbow_parens.rs
- AbiKeymapConfig
- choose_permission_outcome
- 0004-markdown-pretty-pipeline.md
- debug_adapters
- syntax_language
- AbiDebugAdapterSpec
- AbiLigatureConfig
- I
- AbiWorkspaceRoot
- package
- Language
- package
- Domain Docs
- Issue tracker: GitHub
- rainbow_paren.rs
- user/workspace_dock.rs
- package
- main
- keymap.rs
- AcpPaneState
- 0002-lsp-stop-restart-session-picker.md
- 0003-external-command-stream-default.md
- VimActionContext
- syntax_language
- Agent skills
- ligatures.rs
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 790 edges
2. `ShellBuffer` - 384 edges
3. `shell_ui_mut()` - 352 edges
4. `register_shell_hooks()` - 265 edges
5. `shell_ui()` - 242 edges
6. `shell_buffer_mut()` - 194 edges
7. `ShellError` - 193 edges
8. `shell_buffer()` - 192 edges
9. `TextBuffer` - 180 edges
10. `ShellUiState` - 180 edges

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

## Communities (295 total, 10 thin omitted)

### Community 0 - "load_font_set_with_mode"
Cohesion: 0.08
Nodes (28): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode(), load_icon_font() (+20 more)

### Community 1 - "Path"
Cohesion: 0.08
Nodes (28): ClientCapabilities, client_capabilities(), inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), LspClientError, LspClientManager, LspInlineCompletionItem (+20 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.02
Nodes (111): cycle_hover_provider(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_output_speaker_roles_and_tool_chip(), acp_paste_image_inserts_mention_token_and_stores_bytes() (+103 more)

### Community 3 - ".new"
Cohesion: 0.11
Nodes (75): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+67 more)

### Community 4 - "ShellError"
Cohesion: 0.03
Nodes (72): Display, Error, From, ShellError, browser_sync_plan(), Instant, clear_key_sequence(), accept_autocomplete() (+64 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.02
Nodes (122): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_picker_items(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+114 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.07
Nodes (91): additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+83 more)

### Community 7 - "PickerOverlay"
Cohesion: 0.05
Nodes (31): absolute_path_hint(), buffer_is_quickfix(), GitBranchActionKind, GitCommitActionKind, keycode_name_token(), keydown_chord_token(), KeydownChordToken, normalize_named_key_token() (+23 more)

### Community 8 - "shell/browser.rs"
Cohesion: 0.05
Nodes (78): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+70 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (110): Rect, acp_buffer_layout(), acp_pane_body_visible_rows(), AcpBufferLayout, AcpPaneLayout, adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines() (+102 more)

### Community 10 - "AcpEvent"
Cohesion: 0.09
Nodes (37): AvailableCommand, acp_pick_mode(), AcpCommand, AcpEvent, AcpRuntime, AcpSessionInfo, active_command_input_hint(), build_acp_input_hint() (+29 more)

### Community 11 - "PluginPackage"
Cohesion: 0.07
Nodes (39): file_open_package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration, syntax_language() (+31 more)

### Community 12 - "Self"
Cohesion: 0.03
Nodes (36): browser_item(), browser_items(), dashboard_sections(), default_action(), sidebar_sections(), exported_db_browser_items(), hook_command(), Option (+28 more)

### Community 13 - "Option"
Cohesion: 0.07
Nodes (61): parse_log_oneline(), build_git_fringe_snapshot(), command_output_transcript(), create_git_worktree_from_query(), fetch_git_prune(), find_paren_number_range(), git_branch_list(), git_branch_merge() (+53 more)

### Community 14 - "String"
Cohesion: 0.06
Nodes (93): ctrl_mod(), shell_ui(), acp_at_symbol_opens_git_file_picker_and_return_inserts_mention(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range(), acp_input_field_o_and_o_open_new_lines(), acp_input_field_visual_line_delete_removes_selected_lines() (+85 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.06
Nodes (30): AlacrittyEvent, Keycode, Mod, Self, terminal_key_for_event(), terminal_scroll_for_motion(), LiveTerminalError, LiveTerminalSession (+22 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.06
Nodes (60): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+52 more)

### Community 17 - "GitStatusSnapshot"
Cohesion: 0.09
Nodes (10): GitLogEntry, GitStashEntry, GitStatusSnapshot, RepositoryStatus, Into, Option, Self, String (+2 more)

### Community 18 - "shell/issues.rs"
Cohesion: 0.17
Nodes (46): activate_issues_board_line(), apply_capture_report(), apply_rewrite_intent(), apply_scan_report(), begin_issues_create(), board_issue_id_at_row(), collect_scan_files(), enqueue_capture_after_save() (+38 more)

### Community 19 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (29): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DirectoryEntry, GitFeatureSpec (+21 more)

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
Cohesion: 0.07
Nodes (39): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+31 more)

### Community 24 - "Result"
Cohesion: 0.23
Nodes (7): InMemorySecretStore, redact_error(), remembered_connections_store_metadata_separately_from_secret(), HashMap, Result, snippets_and_history_persist(), unix_epoch_secs()

### Community 25 - "open_workspace_from_project"
Cohesion: 0.06
Nodes (74): active_window_id(), install_mark_list_state_for_test(), open_oil_directory(), open_workspace_file(), open_workspace_from_project(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), WindowId (+66 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.07
Nodes (75): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+67 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.08
Nodes (59): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches() (+51 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (29): RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind (+21 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "FontSet"
Cohesion: 0.06
Nodes (56): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, FontSet, acp_slice_chars() (+48 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.06
Nodes (148): EditorRuntime, Default, run_command(), active_git_status_command_context(), cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker() (+140 more)

### Community 33 - "shell/git.rs"
Cohesion: 0.07
Nodes (62): ActiveBufferEventContext, apply_git_status_snapshot(), apply_git_view(), commit_git_buffer(), finish_oil_worktree_branch_selection(), git_commit_message(), git_commit_temp_path(), git_line_is_untracked() (+54 more)

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
Nodes (228): Cow, write_system_clipboard(), yank_to_clipboard_text(), active_directory_root(), active_lsp_code_action_range(), active_shell_buffer_has_input(), active_shell_buffer_id(), active_shell_buffer_is_terminal() (+220 more)

### Community 40 - "Self"
Cohesion: 0.09
Nodes (24): ConfigOilSortMode, ConfigPickerTruncateStrategy, ConfigWorkspaceDockSide, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_oil_sort_mode(), default_pane_golden_ratio(), default_picker_truncate_strategy() (+16 more)

### Community 41 - "Option"
Cohesion: 0.02
Nodes (77): acp_output_header_title(), acp_tool_call_from_partial_update(), AcpBufferState, AcpPane, AcpPastedImage, active_buffer_revision_key(), advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab() (+69 more)

### Community 42 - "Result"
Cohesion: 0.08
Nodes (89): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), render_buffer(), acp_escape_from_insert_keeps_input_cursor_position(), browser_escape_from_insert_keeps_input_cursor_position(), db_dashboard_execute_replaces_output_and_concatenates_multiple_queries() (+81 more)

### Community 43 - "shell_ui_mut"
Cohesion: 0.16
Nodes (36): shell_ui_mut(), buffer_footer_layout(), acp_section_layout_orders_output_input_footer_and_statusline(), browser_input_layout_uses_symmetric_vertical_padding(), install_plugin_sections_test_buffer(), install_plugin_sections_test_buffer_with_update(), install_terminal_test_buffer(), plugin_sections_layout_keeps_output_pane_at_bottom_with_single_row_start() (+28 more)

### Community 44 - "shell/mod.rs"
Cohesion: 0.02
Nodes (263): ActiveLspBufferContext, WorkspaceId, AcpDecodedImage, AcpRenderedImageLine, active_lsp_buffer_context(), active_project_workspace_root(), active_theme_state_path(), active_workspace_open_buffer_paths() (+255 more)

### Community 45 - "editor-markdown/src/lib.rs"
Cohesion: 0.07
Nodes (71): anti_conceal_detects_cursor_and_visual(), apply_link_pretty(), apply_structure_node(), atx_heading_marker(), cfg(), conceal_line_text(), ConcealRange, default_icon_map() (+63 more)

### Community 46 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (67): WorkspaceDockTestUserLibrary, AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec (+59 more)

### Community 47 - "Instant"
Cohesion: 0.03
Nodes (57): ActiveTypingFrameProfile, append_hover_rendered_content(), apply_markdown_code_fence_syntax(), average_duration(), command_builds_user_library(), compute_buffer_syntax(), DirectoryPrefixState, finalize_hover_overlay() (+49 more)

### Community 48 - "LanguageServerRegistry"
Cohesion: 0.18
Nodes (7): LanguageServerRegistry, LspError, Display, Error, Formatter, Result, Vec

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.11
Nodes (15): AbiBrowserFeatureSpec, AbiContextHelpEntry, AbiContextHelpSpec, AbiDbFeatureSpec, AbiTerminalFeatureSpec, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec (+7 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.21
Nodes (21): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), push_terminal_render_run(), resolve_terminal_background(), resolve_terminal_foreground(), resolve_terminal_index_color() (+13 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.02
Nodes (79): AtomicUsize, load_font_set(), acp_agent_markdown_uses_shared_pipeline_pretty(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), codicon_glyphs_fit_inside_one_editor_cell(), CommandLog (+71 more)

### Community 52 - "editor-lsp/src/lib.rs"
Cohesion: 0.20
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 53 - "LanguageServerSpec"
Cohesion: 0.16
Nodes (7): LanguageServerSpec, Into, IntoIterator, Item, LanguageServerRootStrategy, Self, String

### Community 54 - "Issue"
Cohesion: 0.08
Nodes (27): board_issues(), CaptureItem, CaptureReport, civil_from_days(), CodeReference, escape_yaml_scalar(), format_utc_timestamp(), Issue (+19 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.13
Nodes (90): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+82 more)

### Community 56 - "TextPoint"
Cohesion: 0.04
Nodes (49): TextPoint, TextSnapshot, apply_input_operator_motion(), ascii_control_caret_notation(), char_at_index(), char_immediately_before(), chars_immediately_before(), charwise_motion_range() (+41 more)

### Community 57 - "state.rs"
Cohesion: 0.10
Nodes (30): multicursor_selection_offsets(), statusline_mode_label(), multicursor_cursor_points(), multicursor_ranges_for_line(), BlockInsertState, DirectoryYankEntry, FormatterRegistry, FormatterSpec (+22 more)

### Community 58 - "picker_items"
Cohesion: 0.19
Nodes (17): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+9 more)

### Community 59 - "active_runtime_popup"
Cohesion: 0.11
Nodes (53): active_runtime_popup(), add_linked_worktree(), fetch_git_prune_is_silent_command_without_popup(), git_pull_upstream_streams_into_popup_buffer(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items(), git_status_ctrl_v_visual_u_unstages_selected_items() (+45 more)

### Community 60 - "build_output.rs"
Cohesion: 0.18
Nodes (17): link_root_user_library(), main(), create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option (+9 more)

### Community 61 - "LspNotification"
Cohesion: 0.06
Nodes (28): ChildStdin, completion_level_for_message(), launch_summary(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel, LspNotificationLog (+20 more)

### Community 62 - ".new"
Cohesion: 0.13
Nodes (21): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbAutocompleteCandidate, default_volt_state_dir(), insert_test_session(), redact_key_value_segments(), Arc, PathBuf (+13 more)

### Community 63 - "TextBuffer"
Cohesion: 0.05
Nodes (27): delimiter_partner(), EditRecord, find_matching_close_tag(), is_object_separator(), is_punctuation_char(), is_sentence_closer(), is_word_char(), matches_word_kind() (+19 more)

### Community 64 - "PluginCommand"
Cohesion: 0.11
Nodes (21): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+13 more)

### Community 65 - "WorkspaceDockBranchCache"
Cohesion: 0.14
Nodes (19): refresh_workspace_dock_branches(), Arc, HashMap, Instant, Mutex, Option, Path, PathBuf (+11 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.05
Nodes (69): asset_path_from_parts(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), collect_structure_nodes(), create_parser(), DeferredQuery (+61 more)

### Community 67 - "editor-issues/src/lib.rs"
Cohesion: 0.20
Nodes (31): board_hides_closed_by_default(), capture_can_finish_after_caller_continues(), capture_file(), capture_ignores_hack_and_xxx(), capture_mints_and_rewrites_todo_and_fixme(), comment_prefix_for_path(), confirm_rewrite_applied(), confirm_rewrite_skipped() (+23 more)

### Community 68 - "String"
Cohesion: 0.11
Nodes (55): acp_image_mention_token(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+47 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (27): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+19 more)

### Community 70 - "capture_mappings"
Cohesion: 0.16
Nodes (18): capture_mappings(), jsx_syntax_language(), package(), CaptureThemeMapping, LanguageConfiguration, Vec, syntax_language(), capture_mappings() (+10 more)

### Community 71 - "WorkspaceConfigurationValue"
Cohesion: 0.13
Nodes (13): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), BTreeMap, From, Number, T (+5 more)

### Community 72 - "String"
Cohesion: 0.06
Nodes (29): append_query_source(), CaptureThemeMapping, command_failure_message(), GrammarRecompileFailure, GrammarRecompileReport, GrammarSource, LanguageConfiguration, LanguageLoader (+21 more)

### Community 73 - ".send"
Cohesion: 0.11
Nodes (40): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpRuntimeState, AcpSession, AcpTerminal, connect_acp_client() (+32 more)

### Community 74 - "DbService"
Cohesion: 0.13
Nodes (15): db_browser_action_from_spec(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbIndex, DbQueryBufferMeta, DbService (+7 more)

### Community 75 - "clipboard.rs"
Cohesion: 0.13
Nodes (34): ClipboardUtil, clipboard_data_for_mime(), clipboard_image_from_path(), clipboard_image_from_path_loads_named_png(), clipboard_image_from_path_text(), clipboard_image_from_uri_list(), clipboard_text_for_mime(), clipboard_video_ready() (+26 more)

### Community 76 - "directory.rs"
Cohesion: 0.12
Nodes (47): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+39 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.16
Nodes (37): command_candidate_names(), default_process_supervisor_executable(), enrich_env_with_node_manager(), enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing(), environment_value(), explicit_windows_env_value(), is_launch_candidate(), lookup_env_value() (+29 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (65): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.15
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.05
Nodes (72): workspace_picker_item(), exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search() (+64 more)

### Community 82 - "volt/src/main.rs"
Cohesion: 0.06
Nodes (55): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), command_palette_items(), CommandPaletteState, CompilationState, DapState, dynamic_user_library_can_wrap_exported_module(), EventLog (+47 more)

### Community 83 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 84 - "editor-git/src/lib.rs"
Cohesion: 0.13
Nodes (25): configure_background_command(), detect_in_progress(), git_available(), GitStatusError, list_repository_files(), parse_header(), parse_stash_list(), parse_status() (+17 more)

### Community 85 - "Option"
Cohesion: 0.13
Nodes (6): LanguageServerSession, normalize_optional_string(), AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 86 - "BufferId"
Cohesion: 0.04
Nodes (74): BufferKind, browser_state_for_kind(), buffer_uses_browser_host_surface(), default_vim_target(), activate_db_browser_line(), active_buffer_event_context(), active_dashboard_editor_buffer(), active_or_open_dashboard_buffer() (+66 more)

### Community 87 - "Self"
Cohesion: 0.13
Nodes (16): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiIconFontCategory, AbiStatusEntry, GitLogEntry, GitStashEntry (+8 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.15
Nodes (22): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+14 more)

### Community 89 - "show_paren.rs"
Cohesion: 0.40
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), ShowParenConfig

### Community 90 - "RString"
Cohesion: 0.11
Nodes (17): AbiAcpClient, AbiColor, AbiStringPair, AbiTheme, AbiThemeOption, AbiThemeOptionEntry, AbiThemeToken, AcpClient (+9 more)

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
Cohesion: 0.14
Nodes (37): default_install_root(), default_query_asset_root(), csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config() (+29 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "AbiAutocompleteProvider"
Cohesion: 0.29
Nodes (6): AbiAutocompleteProvider, AbiAutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem, AutocompleteProvider, AutocompleteProviderItem

### Community 97 - "editor-picker/src/lib.rs"
Cohesion: 0.05
Nodes (46): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+38 more)

### Community 98 - "GitEditorState"
Cohesion: 0.21
Nodes (19): abort_git_editor_buffer(), confirm_git_editor_buffer(), finish_git_editor_buffer(), GitEditorSession, GitEditorState, inject_git_editor_env(), open_git_editor_buffer(), refresh_pending_git_editor() (+11 more)

### Community 99 - "modeline.rs"
Cohesion: 0.17
Nodes (23): buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_multipart_segment(), compose_includes_macro_recording_register(), compose_joins_default_left_and_right_segments(), compose_modeline(), compose_places_position_and_lsp_on_the_right() (+15 more)

### Community 100 - "From"
Cohesion: 0.12
Nodes (16): AbiLanguageServerRootStrategy, AbiLspDiagnosticsInfo, AbiOilKeyAction, AbiPdfOpenMode, AbiPickerTruncateStrategy, LanguageServerRootStrategy, LspDiagnosticsInfo, OilKeyAction (+8 more)

### Community 101 - ".spawn"
Cohesion: 0.14
Nodes (17): live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, must(), push_snapshot_line(), E, Into, IntoIterator, Item (+9 more)

### Community 102 - ".new"
Cohesion: 0.18
Nodes (22): begin_oil_worktree_request(), diff_git_dwim(), git_args_with_no_pager(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), merge_git_preview(), oil_git_worktree_command(), open_git_cherry_buffer() (+14 more)

### Community 103 - ".from"
Cohesion: 0.09
Nodes (27): lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GhostTextLine, GhostTextLine, exported_ghost_text_lines(), GhostTextLine, abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag() (+19 more)

### Community 104 - "String"
Cohesion: 0.27
Nodes (23): apply_language_options_table(), apply_options_table(), parse_color_part(), parse_hex_channel(), parse_hex_color(), parse_hex_color_value(), parse_language_options_table(), parse_option() (+15 more)

### Community 105 - ".from"
Cohesion: 0.20
Nodes (16): close_buffer_keeps_session_alive_for_next_file(), file_uri_roundtrip_handles_windows_paths(), live_session_picker_label_includes_server_and_root(), path_to_file_uri(), Arc, BTreeMap, session_labels_ignore_stale_tracked_session_keys(), stop_buffer_shuts_down_session() (+8 more)

### Community 106 - "Path"
Cohesion: 0.22
Nodes (10): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LspWorkspaceDiagnostic, path_is_solution(), resolve_single_solution_path(), Path (+2 more)

### Community 107 - "ShellConfig"
Cohesion: 0.17
Nodes (12): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+4 more)

### Community 108 - "JobSpec"
Cohesion: 0.20
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 109 - "shell/picker.rs"
Cohesion: 0.10
Nodes (39): ShellTestUserLibrary, UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_fringe_width_chars() (+31 more)

### Community 110 - "RVec"
Cohesion: 0.18
Nodes (10): AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, HoverProvider, HoverProviderTopic, HoverProvider, HoverProviderTopic, RVec (+2 more)

### Community 111 - ".default"
Cohesion: 0.09
Nodes (52): Self, browser_display_url_prefers_requested_navigation(), Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec() (+44 more)

### Community 112 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 113 - "open_slash_command_picker"
Cohesion: 0.20
Nodes (13): acp_complete_slash(), acp_slash_completion_query(), AcpUiAction, CompletionTrigger, handle_acp_ui_action(), maybe_open_acp_input_completion(), open_file_mention_picker(), open_slash_command_picker() (+5 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (24): plugin_buffer_binding_scope_active(), plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding() (+16 more)

### Community 115 - "DbBrowserBufferView"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, section_count_label(), summarize_sql(), DbBrowserItemRenderer

### Community 116 - "String"
Cohesion: 0.05
Nodes (48): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, diagnostics_parser_maps_lsp_fields(), documentation_lines(), explicit_windows_env_value() (+40 more)

### Community 117 - "String"
Cohesion: 0.07
Nodes (56): Compat, box_row(), box_rule(), BoxRuleKind, build_tokio_runtime(), CellAlign, connect_sql_server(), connection_descriptor_detects_all_supported_engines() (+48 more)

### Community 118 - "LspLogEntry"
Cohesion: 0.17
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "AbiDirectoryEntry"
Cohesion: 0.13
Nodes (13): exported_oil_directory_sections(), AbiDirectoryEntry, AbiDirectoryEntryKind, AbiOilDefaults, AbiOilSortMode, DirectoryEntry, DirectoryEntryKind, OilDefaults (+5 more)

### Community 121 - "IssueId"
Cohesion: 0.21
Nodes (10): IssueId, linked_issue_id_on_line(), parse_code_reference_line(), parse_linked_and_unlinked_forms(), ParsedCodeReference, Display, Error, Formatter (+2 more)

### Community 122 - "TextRange"
Cohesion: 0.07
Nodes (25): CodeActionParams, TextRange, code_action_params(), diagnostic_matches_request_range(), formatting_parser_maps_text_edits(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), lsp_formatting_options() (+17 more)

### Community 123 - "LineSyntaxSpan"
Cohesion: 0.12
Nodes (46): browser_header_and_table_lines_use_distinct_tokens(), cell_theme_token(), connection_line_spans(), db_browser_line_spans(), db_results_error_spans(), db_results_line_spans(), db_results_syntax_lines(), db_results_table_row_spans() (+38 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.08
Nodes (23): apply_git_fringe_hunk(), build_git_summary_snapshot(), git_command_output_background(), git_repository_present(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState (+15 more)

### Community 125 - "split_runtime_pane"
Cohesion: 0.20
Nodes (15): cycle_runtime_pane(), split_runtime_pane(), browser_open_buffer_command_uses_existing_split_pane(), focus_test_buffer(), inactive_split_render_reads_saved_buffer_input_mode(), insert_mode_is_buffer_local_across_buffer_switches(), insert_mode_is_buffer_local_across_split_focus_changes(), install_scratch_test_buffer() (+7 more)

### Community 126 - "DynamicUserLibrary"
Cohesion: 0.03
Nodes (26): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DebugAdapterSpec, DirectoryEntry (+18 more)

### Community 127 - "TerminalTranscript"
Cohesion: 0.17
Nodes (5): append_lines(), TerminalLine, TerminalSession, TerminalStream, TerminalTranscript

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.13
Nodes (7): Option, Vec, terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot

### Community 130 - "user/config.rs"
Cohesion: 0.18
Nodes (23): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+15 more)

### Community 131 - "UserLibraryModule"
Cohesion: 0.15
Nodes (13): AbiIconFontSymbol, AbiOilFeatureSpec, AbiOilKeybindings, AbiStatuslineContext, IconFontSymbol, OilFeatureSpec, OilKeybindings, IconFontSymbol (+5 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "editor-icons/src/lib.rs"
Cohesion: 0.11
Nodes (16): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+8 more)

### Community 134 - ".get"
Cohesion: 0.16
Nodes (22): ColumnData, column_is_numeric(), DbColumn, DbSchemaCache, DbTable, is_numeric_cell(), load_postgres_schema(), load_sql_server_schema() (+14 more)

### Community 135 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "build_job_command"
Cohesion: 0.43
Nodes (7): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), configure_background_command(), Command, configure_background_command(), Command

### Community 138 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 139 - "DbEngine"
Cohesion: 0.24
Nodes (7): DbEngine, DbHistoryEntry, DbSnippet, load_persisted_state(), PersistedDbState, RememberedConnection, Path

### Community 140 - "shell/acp.rs"
Cohesion: 0.10
Nodes (43): acp_file_mention_at_cursor(), acp_file_mention_at_cursor_requires_token_start(), acp_file_uri(), acp_insert_file_mention(), acp_permission_picker_closed(), acp_picker_entry(), acp_resolve_permission_option(), buffer_lookup_is_scoped_to_workspace() (+35 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.14
Nodes (6): CommandLineCompletionState, CommandLineOverlay, InputPromptOverlay, Option, String, Vec

### Community 142 - "UserLibrary"
Cohesion: 0.08
Nodes (33): browser_buffer_layout(), browser_host_viewport_rect(), browser_viewport_rect(), BrowserBufferLayout, Rect, acp_rendered_text_segments(), LineCharMap, LineWrapSegment (+25 more)

### Community 143 - "AbiPaneConfig"
Cohesion: 0.09
Nodes (17): AbiMarkdownPrettyConfig, AbiMarkdownPrettyIcon, AbiPaneConfig, AbiPickerLayout, AbiShowParenConfig, AbiWorkspaceDockSide, fraction_to_hundredths(), hundredths_to_fraction() (+9 more)

### Community 144 - "resolve_permission"
Cohesion: 0.40
Nodes (4): acp_permission_approve(), acp_permission_deny(), PermissionDecision, resolve_permission()

### Community 145 - "TextEdit"
Cohesion: 0.67
Nodes (4): TextEdit, apply_text_edits_to_span(), text_edit_to_input_edit(), InputEdit

### Community 146 - "volt/build.rs"
Cohesion: 0.14
Nodes (46): add_standalone_workspace_root(), build_windows_icon(), copy_assets_directory(), copy_dir_recursive(), copy_file_with_retry(), copy_user_directory(), create_dir_all_with_retry(), inline_workspace_package_fields() (+38 more)

### Community 147 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

### Community 148 - "headerline_lines"
Cohesion: 0.19
Nodes (11): packages(), LanguageConfiguration, Vec, syntax_languages(), build_headerline_lines(), db_buffer_headerline(), headerline_lines(), Option (+3 more)

### Community 149 - "client.rs"
Cohesion: 0.04
Nodes (74): active_parameter_label(), char_to_byte_offset(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+66 more)

### Community 150 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 151 - "AbiLanguageConfiguration"
Cohesion: 0.19
Nodes (9): AbiCaptureThemeMapping, AbiGrammarSource, AbiLanguageConfiguration, CaptureThemeMapping, GrammarSource, LanguageConfiguration, CaptureThemeMapping, GrammarSource (+1 more)

### Community 152 - "oil.rs"
Cohesion: 0.08
Nodes (41): seti_directory_icon(), DirectoryEntry, OilSortMode, Path, chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label() (+33 more)

### Community 153 - "user/db.rs"
Cohesion: 0.15
Nodes (21): browser_items_shape_table_rows_from_user_config(), browser_key_bindings(), connect_buffer_binds_enter_to_submit_command(), connect_buffer_lines(), dashboard_buffer_declares_nested_layout_and_execute_chord(), dashboard_key_bindings(), engine_icon(), feature_spec() (+13 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "AbiSectionTree"
Cohesion: 0.25
Nodes (7): exported_git_status_sections(), AbiSection, AbiSectionTree, Section, SectionTree, Section, SectionTree

### Community 156 - "LspCompletionItem"
Cohesion: 0.22
Nodes (3): LspCompletionItem, LspCompletionKind, parse_completion_kind()

### Community 157 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 158 - "load"
Cohesion: 0.17
Nodes (22): ConfigFingerprint, CachedUserConfig, config_cache(), config_fingerprint_for_files(), config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files() (+14 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "AbiSectionItem"
Cohesion: 0.29
Nodes (6): AbiSectionAction, AbiSectionItem, SectionAction, SectionItem, SectionAction, SectionItem

### Community 161 - "AcpManager"
Cohesion: 0.16
Nodes (9): acp_connected(), acp_open_permission_request(), acp_session_buffer_name(), AcpManager, AcpPendingPermissionUi, open_permission_picker(), HashMap, Receiver (+1 more)

### Community 162 - "centered_rect"
Cohesion: 0.19
Nodes (9): centered_rect(), acp_chat_bubble_cols(), acp_rendered_text_wrap_cols(), acp_chat_bubble_width_px(), acp_chat_origin_x(), acp_prefix_columns(), acp_spinner_frame(), picker_card_rect() (+1 more)

### Community 163 - "theme.rs"
Cohesion: 0.15
Nodes (28): assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages(), bundled_themes_use_pallet_sections_and_token_references(), list_theme_files() (+20 more)

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 166 - "String"
Cohesion: 0.29
Nodes (6): call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "user/terminal.rs"
Cohesion: 0.19
Nodes (12): default_terminal_args(), default_terminal_program(), exported_terminal_config(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback() (+4 more)

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
Cohesion: 0.18
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
Cohesion: 0.08
Nodes (59): BufRead, code_action_params_use_flattened_lsp_shape(), completion_documentation(), configuration_item_section(), copilot_status_notifications_offer_sign_in_action(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item() (+51 more)

### Community 180 - "predicate_capture_text"
Cohesion: 0.25
Nodes (12): evaluate_general_predicate(), general_predicates_match(), lua_class_matches(), lua_item_matches(), lua_item_span(), lua_match_here(), lua_pattern_matches(), lua_set_matches() (+4 more)

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
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

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

### Community 190 - ".new"
Cohesion: 0.31
Nodes (4): CommandLinePurpose, BufferId, Into, Self

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
Cohesion: 0.04
Nodes (71): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), edits_since_returns_contiguous_forward_edits() (+63 more)

### Community 195 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 196 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 197 - "Result"
Cohesion: 0.09
Nodes (35): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_submitted(), acp_pick_model() (+27 more)

### Community 198 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 199 - "configure_file_buffer"
Cohesion: 0.52
Nodes (7): active_and_secondary_buffer_ids(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), record_file_reload_event(), wait_for_file_reload_worker()

### Community 200 - "markdown.rs"
Cohesion: 0.21
Nodes (14): default_pretty_icons(), inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), pretty_config(), pretty_config_ships_consistent_icon_map(), pretty_icon_map(), BTreeMap (+6 more)

### Community 201 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 202 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 203 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 204 - "cargo"
Cohesion: 0.43
Nodes (6): cargo(), I, Path, Result, String, run()

### Community 205 - "rainbow_parens.rs"
Cohesion: 0.47
Nodes (4): config(), package(), package_exports_toggle_command_and_binding(), rainbow_config_load_stays_cheap_for_frame_budget()

### Community 206 - "AbiKeymapConfig"
Cohesion: 0.60
Nodes (3): AbiKeymapConfig, KeymapConfig, KeymapConfig

### Community 207 - "choose_permission_outcome"
Cohesion: 0.40
Nodes (6): choose_permission_outcome(), format_permission_option_kind(), PendingPermission, PermissionOption, PermissionOptionKind, RequestPermissionOutcome

### Community 209 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 210 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 211 - "AbiDebugAdapterSpec"
Cohesion: 0.60
Nodes (3): AbiDebugAdapterSpec, DebugAdapterSpec, DebugAdapterSpec

### Community 212 - "AbiLigatureConfig"
Cohesion: 0.60
Nodes (3): AbiLigatureConfig, LigatureConfig, LigatureConfig

### Community 214 - "AbiWorkspaceRoot"
Cohesion: 0.60
Nodes (3): AbiWorkspaceRoot, WorkspaceRoot, WorkspaceRoot

### Community 215 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 216 - "Language"
Cohesion: 0.22
Nodes (8): Database, External commands, Issues, Language, Language servers, Markdown presentation, Volt, Workspace

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

### Community 234 - "package"
Cohesion: 0.28
Nodes (7): exported_pdf_open_mode(), PdfOpenMode, open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 238 - "main"
Cohesion: 0.25
Nodes (8): escape_rust_string(), main(), parse_symbol_line(), Box, Error, Option, Result, String

### Community 242 - "AcpPaneState"
Cohesion: 0.05
Nodes (52): acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_diff_display_lines(), acp_icon_segment(), acp_mark_chat(), acp_mark_gutter(), acp_multiline_text_lines() (+44 more)

### Community 248 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **143 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+138 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **10 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `shell/tests.rs`, `ShellError`, `PickerOverlay`, `shell/browser.rs`, `AcpEvent`, `shell/acp.rs`, `Option`, `String`, `resolve_permission`, `shell/issues.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `open_workspace_from_project`, `command_stream.rs`, `AcpManager`, `shell/git.rs`, `shell/pdf.rs`, `ServiceRegistry`, `String`, `Option`, `Result`, `shell_ui_mut`, `shell/mod.rs`, `Instant`, `TextPoint`, `active_runtime_popup`, `String`, `Result`, `configure_file_buffer`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `volt/src/main.rs`, `BufferId`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `GitEditorState`, `.new`, `shell/picker.rs`, `open_slash_command_picker`, `GitSummaryState`, `split_runtime_pane`?**
  _High betweenness centrality (0.125) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `.new`, `UserLibraryModule`, `user/lib.rs`, `common.rs`, `Self`, `headerline_lines`, `calculator.rs`, `oil.rs`, `user/db.rs`, `lsp.rs`, `latex.rs`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `lua.rs`, `user/terminal.rs`, `kotlin.rs`, `nix.rs`, `r.rs`, `shell/mod.rs`, `bash.rs`, `clojure.rs`, `user/browser.rs`, `elixir.rs`, `graphql.rs`, `sdk/src/lib.rs`, `HeaderlineTestUserLibrary`, `hcl.rs`, `picker_items`, `java.rs`, `perl.rs`, `php.rs`, `ruby.rs`, `proto.rs`, `PluginCommand`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `capture_mappings`, `xml.rs`, `markdown.rs`, `scala.rs`, `rainbow_parens.rs`, `debug_adapters`, `volt/src/main.rs`, `syntax_language`, `PickerItemSpec`, `package`, `package`, `show_paren.rs`, `editor-plugin-host/src/lib.rs`, `user/workspace_dock.rs`, `package`, `.default`, `PluginKeyBinding`, `syntax_language`?**
  _High betweenness centrality (0.056) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Option` to `Option`, `ShellError`, `PickerOverlay`, `shell/browser.rs`, `render.rs`, `UserLibrary`, `shell/git.rs`, `shell/pdf.rs`, `String`, `Result`, `shell_ui_mut`, `shell/mod.rs`, `Instant`, `render_buffer_with_view_state`, `TextPoint`, `state.rs`, `TextBuffer`, `directory.rs`, `shell/terminal.rs`, `BufferId`, `draw_diagnostic_underlines_for_segment`, `shell/picker.rs`, `open_slash_command_picker`, `AcpPaneState`, `LineSyntaxSpan`, `GitSummaryState`?**
  _High betweenness centrality (0.046) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _143 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `load_font_set_with_mode` be split into smaller, more focused modules?**
  _Cohesion score 0.08418367346938775 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.08103975535168195 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.024870554282318987 - nodes in this community are weakly interconnected._