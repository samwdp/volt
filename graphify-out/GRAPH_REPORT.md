# Graph Report - volt  (2026-07-30)

## Corpus Check
- 224 files · ~564,378 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8977 nodes · 36768 edges · 310 communities (280 shown, 30 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3030 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `2814edac`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- shell/mod.rs
- Path
- shell/tests.rs
- src/tests.rs
- ShellError
- user/lib.rs
- editor-syntax/src/lib.rs
- String
- AcpPaneState
- .new
- Result
- syntax_language
- sdk/src/lib.rs
- TextBuffer
- LanguageServerSpec
- LiveTerminalSession
- editor-fs/src/lib.rs
- GitStatusSnapshot
- editor-issues/src/lib.rs
- DynamicUserLibrary
- HookBus
- EditorModel
- KeymapScope
- calculator.rs
- editor-db/src/lib.rs
- state.rs
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- UserLibrary
- EditorRuntime
- workspace.rs
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- Self
- ShellBuffer
- UserLibraryModule
- clipboard.rs
- String
- ShellUiState
- render_text_with_fonts
- AbiGitFeatureSpec
- .new
- AbiContextHelpSpec
- editor-terminal/src/lib.rs
- HeaderlineTestUserLibrary
- shell_ui_mut
- Vec
- Section
- render_buffer_with_view_state
- .len
- String
- Option
- AbiTerminalConfig
- temp_dir
- String
- Self
- spawn_reader_thread
- AbiKeymapConfig
- state_with_user_library
- SyntaxRegistry
- .default
- String
- DebugConfiguration
- .new
- Option
- theme.rs
- .send
- .new
- editor-path/src/lib.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- AcpManager
- render.rs
- .from
- LanguageServerRegistry
- volt/src/main.rs
- .new
- draw_diagnostic_underlines_for_segment
- .new
- main
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- .from_grammar
- workspace_nav.rs
- Option
- PluginBuffer
- detect_markdown_table
- WorkspaceConfigurationValue
- shell/browser.rs
- abi.rs
- client.rs
- common.rs
- Option
- editor-lsp/src/lib.rs
- PickerSession
- editor-picker/src/lib.rs
- sync_quickfix_popup_buffer
- PickerOverlay
- Result
- buffer_is_git_status
- PathBuf
- resolve_picker_extra
- PluginKeyBinding
- AbiSectionTree
- .spawn
- PluginCommand
- DbService
- process_supervisor.rs
- .new
- DbEngine
- .get
- shell/git.rs
- GitSummaryState
- statusline.rs
- PickerItem
- find_font_by_name
- JobError
- Option
- user/config.rs
- oil.rs
- key_sequence.rs
- From
- .new_with_secret_store
- LspCodeAction
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- TerminalCursorSnapshot
- Diagnostic
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- AbiStatuslineContext
- JobSpec
- .fmt
- standalone_user_manifest.rs
- treesittercontext_ghosttext.rs
- String
- TextRange
- Instant
- load_user_library
- VimActionContext
- DbBrowserContext
- lsp.rs
- .oil_directory_sections
- LspLogEntry
- AcpPickerItemSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- .path
- git_remote_worktree_branch_list
- Vec
- .byte_slice_chunks
- ServiceRegistry
- aligned_indent_column
- String
- user/terminal.rs
- build_output.rs
- GhostTextContext
- OilDefaultsSection
- cmake.rs
- .next_token
- JobResult
- choose_permission_outcome
- user/browser.rs
- acp_buffer_layout
- .oil_keybindings
- `user`
- shell/acp.rs
- spawn_terminal_reader
- .oil_directory_sections
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- AbiPdfOpenMode
- .acp_client_by_id
- Option
- .git_command_for_chord
- GrammarRecompileReport
- .autocomplete_providers
- Database Explorer PRD
- .browser_feature_spec
- bash.rs
- clojure.rs
- elixir.rs
- setup_standalone_user_repository
- hcl.rs
- java.rs
- kotlin.rs
- PluginPackage
- lua.rs
- nix.rs
- perl.rs
- php.rs
- proto.rs
- r.rs
- ruby.rs
- scala.rs
- solidity.rs
- swift.rs
- lang/vim.rs
- xml.rs
- .context_help_specs
- .db_feature_spec
- Language
- .debug_adapters
- Domain Docs
- Issue tracker: GitHub
- load
- .git_feature_spec
- package
- .hover_providers
- .keymap_config
- .ligature_config
- .oil_feature_spec
- .oil_keybindings
- .pane_config
- .pdf_open_mode
- .picker_truncate_strategy
- .statusline_render
- .terminal_feature_spec
- .workspace_roots
- markdown.rs
- syntax_language
- debug_adapters
- Self
- index_syntax_lines
- syntax_language
- package
- TextEdit
- panic_payload_message
- .from_hook_detail
- panic_payload_message
- 0002-lsp-stop-restart-session-picker.md
- Agent skills
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 748 edges
2. `ShellBuffer` - 362 edges
3. `shell_ui_mut()` - 331 edges
4. `register_shell_hooks()` - 258 edges
5. `shell_ui()` - 220 edges
6. `ShellError` - 182 edges
7. `shell_buffer()` - 178 edges
8. `shell_buffer_mut()` - 173 edges
9. `TextBuffer` - 166 edges
10. `ShellUiState` - 161 edges

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

## Communities (310 total, 30 thin omitted)

### Community 0 - "shell/mod.rs"
Cohesion: 0.03
Nodes (340): Cow, write_system_clipboard(), yank_to_clipboard_text(), accept_autocomplete(), acp_decode_image(), activate_db_browser_line(), active_directory_root(), active_lsp_buffer_context() (+332 more)

### Community 1 - "Path"
Cohesion: 0.09
Nodes (22): inline_completion_params(), is_copilot_server(), LspClientError, LspClientManager, LspSessionHandle, path_to_uri(), request_timeout_for_method(), Arc (+14 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (73): load_font_set(), acp_wrapped_text_uses_full_width_on_continuation_rows(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji() (+65 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.14
Nodes (63): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+55 more)

### Community 4 - "ShellError"
Cohesion: 0.04
Nodes (46): RenderBackend, Arc, Debug, Default, Display, Error, From, Option (+38 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (93): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit() (+85 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.10
Nodes (74): vim_search_entries_trim_whitespace_from_labels(), additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust() (+66 more)

### Community 7 - "String"
Cohesion: 0.07
Nodes (64): active_git_status_command_context(), ActiveBufferEventContext, apply_git_status_snapshot(), ensure_no_rebase_in_progress(), fetch_git_pushremote(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_branch_list() (+56 more)

### Community 8 - "AcpPaneState"
Cohesion: 0.06
Nodes (40): acp_build_output_lines(), acp_build_plan_lines(), acp_icon_segment(), acp_multiline_text_lines(), acp_padding_prefix(), acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row() (+32 more)

### Community 9 - ".new"
Cohesion: 0.10
Nodes (52): FpsOverlaySnapshot, ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines(), buffer_point_at_screen(), buffer_visible_headerline_lines(), clamp_to_char_boundary(), collect_wrapped_lines(), file_name_with_parent() (+44 more)

### Community 10 - "Result"
Cohesion: 0.14
Nodes (26): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_submitted(), acp_pick_session() (+18 more)

### Community 11 - "syntax_language"
Cohesion: 0.07
Nodes (33): package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+25 more)

### Community 12 - "sdk/src/lib.rs"
Cohesion: 0.06
Nodes (41): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec, default_db_browser_line() (+33 more)

### Community 13 - "TextBuffer"
Cohesion: 0.03
Nodes (72): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending() (+64 more)

### Community 14 - "LanguageServerSpec"
Cohesion: 0.12
Nodes (8): LanguageServerSpec, normalize_optional_string(), normalize_unique_entries(), Into, IntoIterator, Item, LanguageServerRootStrategy, String

### Community 15 - "LiveTerminalSession"
Cohesion: 0.07
Nodes (25): AlacrittyEvent, Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc (+17 more)

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
Nodes (15): DynamicUserLibrary, BrowserFeatureSpec, DbFeatureSpec, GitFeatureSpec, IconFontSymbol, KeymapConfig, LigatureConfig, OilDefaults (+7 more)

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

### Community 24 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (33): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn, DbIndex, DbSchemaCache, DbTable, default_db_browser_line() (+25 more)

### Community 25 - "state.rs"
Cohesion: 0.14
Nodes (25): BlockInsertState, DirectoryYankEntry, LastFind, LastSearch, MulticursorState, BTreeMap, BufferId, Default (+17 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (47): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+39 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.09
Nodes (69): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+61 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.12
Nodes (29): centered_rect(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches(), font_style_rank(), golden_split_size() (+21 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (38): autocomplete_entries(), autocomplete_score(), AutocompleteProviderKind, AutocompleteWorkerRequest, buffer_autocomplete_entries(), db_autocomplete_entries(), lsp_autocomplete_entries(), manual_autocomplete_entries() (+30 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "UserLibrary"
Cohesion: 0.05
Nodes (51): BufferKind, browser_state_for_kind(), buffer_uses_browser_host_surface(), default_vim_target(), active_buffer_event_context(), buffer_interaction(), buffer_is_browser(), buffer_is_command_output() (+43 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.08
Nodes (93): EditorRuntime, Default, cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer(), create_git_worktree() (+85 more)

### Community 33 - "workspace.rs"
Cohesion: 0.14
Nodes (27): workspace_picker_item(), PickerWorkspaceContext, existing_workspace_for_project(), file_picker_preview(), message_item(), package(), package_exports_cycle_project_workspace_commands(), package_exports_format_command() (+19 more)

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

### Community 40 - "Self"
Cohesion: 0.15
Nodes (14): ConfigPickerTruncateStrategy, default_ambiguous_prefix_timeout_ms(), default_ligatures_enabled(), default_pane_golden_ratio(), default_picker_truncate_strategy(), KeymapSection, PaneSection, PickerTruncateStrategy (+6 more)

### Community 41 - "ShellBuffer"
Cohesion: 0.03
Nodes (43): acp_tool_call_from_partial_update(), apply_multicursor_delete(), apply_multicursor_insert_text(), buffer_context_overlay_snapshot(), BufferContextOverlayCacheKey, BufferContextOverlaySnapshot, cached_context_overlay_snapshot(), charwise_motion_range() (+35 more)

### Community 42 - "UserLibraryModule"
Cohesion: 0.08
Nodes (23): exported_icon_symbols(), exported_oil_feature_spec(), exported_picker_truncate_strategy(), IconFontSymbol, OilFeatureSpec, PickerTruncateStrategy, AbiIconFontSymbol, AbiOilFeatureSpec (+15 more)

### Community 43 - "clipboard.rs"
Cohesion: 0.19
Nodes (13): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+5 more)

### Community 44 - "String"
Cohesion: 0.06
Nodes (46): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), CopilotDeviceCodePrompt, documentation_lines(), explicit_windows_env_value() (+38 more)

### Community 45 - "ShellUiState"
Cohesion: 0.04
Nodes (64): InputPromptOverlay, active_lsp_workspace_loaded(), active_runtime_buffer(), active_window_id(), active_workspace_open_buffer_paths(), apply_pending_lsp_state(), BufferViewState, close_buffer_immediate() (+56 more)

### Community 46 - "render_text_with_fonts"
Cohesion: 0.10
Nodes (29): Canvas, DrawCommand, Arc, TextStyle, acp_slice_chars(), cached_primary_text_runs(), column_to_relative_byte_offset(), draw_primary_ligature_texture_if_available() (+21 more)

### Community 47 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 48 - ".new"
Cohesion: 0.02
Nodes (141): ActiveLspBufferContext, WorkspaceId, active_theme_state_path(), append_error_log(), append_hover_rendered_content(), apply_markdown_code_fence_syntax(), asset_path_from_parts(), AutocompleteBufferRequest (+133 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.08
Nodes (22): exported_browser_feature_spec(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_terminal_feature_spec(), BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec (+14 more)

### Community 50 - "editor-terminal/src/lib.rs"
Cohesion: 0.18
Nodes (24): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+16 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (34): AtomicUsize, active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc (+26 more)

### Community 52 - "shell_ui_mut"
Cohesion: 0.06
Nodes (68): ctrl_mod(), cycle_runtime_pane(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger() (+60 more)

### Community 53 - "Vec"
Cohesion: 0.11
Nodes (9): EventLog, LspState, AcpClient, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, Vec (+1 more)

### Community 54 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.12
Nodes (84): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, is_dark_color(), Color, segment_index_for_column() (+76 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (31): apply_input_operator_motion(), ascii_control_caret_notation(), char_at_index(), display_columns_for_character(), exact_match_positions_in_chars(), find_char_forward(), fuzzy_match_end(), fuzzy_match_end_in_chars() (+23 more)

### Community 57 - "String"
Cohesion: 0.14
Nodes (17): db_browser_action_from_spec(), DisabledSecretStore, initialize_native_keyring(), InMemorySecretStore, load_postgres_schema(), OsSecretStore, qualified_name_from_spec(), redact_error() (+9 more)

### Community 58 - "Option"
Cohesion: 0.06
Nodes (54): BufRead, completion_documentation(), completion_level_for_message(), configuration_item_section(), csharp_metadata_request_params(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item() (+46 more)

### Community 59 - "AbiTerminalConfig"
Cohesion: 0.47
Nodes (4): exported_terminal_config(), AbiTerminalConfig, TerminalConfig, TerminalConfig

### Community 60 - "temp_dir"
Cohesion: 0.11
Nodes (28): build_git_fringe_snapshot(), create_git_worktree_from_query(), git_commit_temp_path(), git_common_dir(), git_fringe_snapshot_ignores_crlf_only_difference(), git_fringe_snapshot_is_empty_when_buffer_matches_head(), git_fringe_temp_path(), git_push_remote_name_prefers_branch_push_remote_for_slashy_branch_names() (+20 more)

### Community 61 - "String"
Cohesion: 0.08
Nodes (95): default_error_log_path(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+87 more)

### Community 62 - "Self"
Cohesion: 0.08
Nodes (9): AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, PickerActionSpec, Into, RString (+1 more)

### Community 63 - "spawn_reader_thread"
Cohesion: 0.15
Nodes (18): ChildStdin, launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), AtomicBool, Mutex (+10 more)

### Community 64 - "AbiKeymapConfig"
Cohesion: 0.11
Nodes (14): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, PaneConfig, config(), PaneConfig (+6 more)

### Community 65 - "state_with_user_library"
Cohesion: 0.09
Nodes (66): install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), browser_sync_plan_excludes_pdf_buffers(), buffer_save_command_uses_shell_focused_buffer_when_runtime_focus_is_stale(), buffer_save_command_writes_edited_file_buffer_to_disk(), buffer_save_hook_prefers_explicit_event_buffer_over_shell_focus(), buffer_save_still_writes_when_format_on_save_fails() (+58 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.07
Nodes (54): buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser(), desired_indent_for_loaded_language(), ensure_cloned_grammar_dir_exists() (+46 more)

### Community 67 - ".default"
Cohesion: 0.10
Nodes (49): Self, browser_display_url_prefers_requested_navigation(), Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec() (+41 more)

### Community 68 - "String"
Cohesion: 0.14
Nodes (46): active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+38 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - ".new"
Cohesion: 0.05
Nodes (47): feature_spec(), DbFeatureSpec, help_entry(), ContextHelpEntry, hook_command(), package(), hook_command(), hook_command_detail() (+39 more)

### Community 71 - "Option"
Cohesion: 0.05
Nodes (41): append_query_source(), asset_path_from_parts(), CaptureThemeMapping, command_failure_message(), default_install_root(), default_query_asset_root(), DeferredQuery, GrammarSource (+33 more)

### Community 72 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 73 - ".send"
Cohesion: 0.10
Nodes (44): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+36 more)

### Community 74 - ".new"
Cohesion: 0.10
Nodes (29): buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), humanize_debug_label(), install_acp_test_buffer(), open_permission_request_reorders_queue_for_requested_picker(), pending_slash_completion_trigger_rejects_multiline_input() (+21 more)

### Community 75 - "editor-path/src/lib.rs"
Cohesion: 0.13
Nodes (19): contains_wildcards(), glob_literal_count(), glob_matches(), matcher_scores_filename_glob_and_extension_paths(), normalize_extension(), normalize_text(), PathMatcher, PathPattern (+11 more)

### Community 76 - "directory.rs"
Cohesion: 0.12
Nodes (47): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+39 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), environment_value(), explicit_windows_env_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (65): PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output() (+57 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.13
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.08
Nodes (37): exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+29 more)

### Community 82 - "AcpManager"
Cohesion: 0.15
Nodes (10): acp_connected(), acp_open_permission_request(), acp_permission_picker_closed(), AcpManager, AcpPendingPermissionUi, drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), open_permission_picker() (+2 more)

### Community 83 - "render.rs"
Cohesion: 0.06
Nodes (76): RenderColor, FontSet, acp_prefix_columns(), acp_spinner_frame(), adjusted_contextual_ligature_pixel_size(), alpha_bitmap_surface(), autocomplete_visible_start(), build_cached_text_layout() (+68 more)

### Community 84 - ".from"
Cohesion: 0.06
Nodes (39): main(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GitStashEntry, main(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers() (+31 more)

### Community 85 - "LanguageServerRegistry"
Cohesion: 0.14
Nodes (17): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LspError, path_is_solution(), resolve_single_solution_path() (+9 more)

### Community 86 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 87 - ".new"
Cohesion: 0.04
Nodes (101): cycle_hover_provider(), buffer_footer_layout(), render_buffer(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_plan_entries_normalize_completed_prefix_when_later_step_is_active(), acp_plan_entries_normalize_completed_prefix_without_active_step() (+93 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.14
Nodes (24): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+16 more)

### Community 89 - ".new"
Cohesion: 0.23
Nodes (18): diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), merge_git_preview(), open_git_cherry_buffer(), open_git_diff_buffer() (+10 more)

### Community 90 - "main"
Cohesion: 0.14
Nodes (15): command_palette_items(), main(), DebugAdapterSpec, Error, LanguageConfiguration, LanguageServerSpec, Theme, ExitCode (+7 more)

### Community 91 - "editor-plugin-host/src/lib.rs"
Cohesion: 0.13
Nodes (37): auto_loaded_packages(), auto_loaded_packages_filters_manual_packages_out(), bootstrap(), clear_package_registrations(), clear_package_registrations_removes_hook_bindings_and_declarations(), detail_filter_matches(), emitted_hook_actions_include_active_window_pane_and_buffer(), file_open_hook_filters_match_exact_basenames() (+29 more)

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

### Community 96 - "Option"
Cohesion: 0.11
Nodes (42): parse_log_oneline(), build_git_summary_snapshot(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), command_output_transcript(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail() (+34 more)

### Community 97 - "PluginBuffer"
Cohesion: 0.09
Nodes (6): PickerKeybindingContext, PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate, RVec

### Community 98 - "detect_markdown_table"
Cohesion: 0.19
Nodes (20): advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update(), detect_markdown_table(), format_markdown_table_at_cursor(), insert_markdown_table_row_at_cursor(), is_markdown_table_delimiter_row_candidate(), markdown_table_alignment() (+12 more)

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.15
Nodes (13): language_server_spec_exposes_workspace_configuration_builders(), AsRef, BTreeMap, From, I, Number, Self, T (+5 more)

### Community 100 - "shell/browser.rs"
Cohesion: 0.05
Nodes (81): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+73 more)

### Community 101 - "abi.rs"
Cohesion: 0.06
Nodes (43): AbiAcpClient, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiColor, AbiDebugAdapterSpec, AbiFiniteF64, AbiHoverProvider, AbiHoverProviderTopic (+35 more)

### Community 102 - "client.rs"
Cohesion: 0.04
Nodes (94): ClientCapabilities, char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), close_buffer_keeps_session_alive_for_next_file(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), completion_parser_handles_lists_and_docs() (+86 more)

### Community 103 - "common.rs"
Cohesion: 0.14
Nodes (18): binding_suffix(), GrammarSourceSpec, GrammarSourceSpec<'a>, CaptureThemeMapping, GrammarSource, LanguageConfiguration, Self, String (+10 more)

### Community 104 - "Option"
Cohesion: 0.13
Nodes (7): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), LanguageServerSession, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 105 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 106 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 107 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 108 - "sync_quickfix_popup_buffer"
Cohesion: 0.08
Nodes (24): buffer_is_quickfix(), ErrorSeverity, quickfix_clear_marks(), quickfix_entries_from_one_shot(), quickfix_entry_for_cursor(), quickfix_mark_all(), quickfix_open_current_list(), quickfix_open_entry() (+16 more)

### Community 109 - "PickerOverlay"
Cohesion: 0.09
Nodes (39): PickerOverlay, workspace_delete_picker_overlay(), workspace_switch_picker_overlay(), buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec() (+31 more)

### Community 110 - "Result"
Cohesion: 0.10
Nodes (68): active_runtime_popup(), active_and_secondary_buffer_ids(), add_linked_worktree(), browser_host_new_window_event_routes_into_browser_popup(), browser_popup_command_focuses_the_popup_surface(), configure_file_buffer(), dismissed_popup_toggle_restores_terminal_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes() (+60 more)

### Community 111 - "buffer_is_git_status"
Cohesion: 0.24
Nodes (11): git_status_command_name(), git_status_next_section_command(), git_status_previous_section_command(), handle_git_status_chord(), handle_git_status_tab(), move_git_section(), set_git_prefix(), take_git_prefix() (+3 more)

### Community 112 - "PathBuf"
Cohesion: 0.17
Nodes (13): language_server_session_in_workspace_scope(), live_sessions_for_workspace_includes_root_scoped_and_buffer_served(), LspClientState, LspInlineCompletionItem, LspLiveSession, normalize_path_for_compare(), parse_inline_completion_item(), parse_inline_completion_response() (+5 more)

### Community 113 - "resolve_picker_extra"
Cohesion: 0.14
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - "AbiSectionTree"
Cohesion: 0.09
Nodes (18): exported_git_status_sections(), exported_oil_defaults(), exported_oil_directory_sections(), DirectoryEntry, GitStatusSnapshot, OilDefaults, OilSortMode, Path (+10 more)

### Community 116 - ".spawn"
Cohesion: 0.10
Nodes (18): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+10 more)

### Community 117 - "PluginCommand"
Cohesion: 0.08
Nodes (24): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+16 more)

### Community 118 - "DbService"
Cohesion: 0.14
Nodes (17): db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbService, DbSession (+9 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - ".new"
Cohesion: 0.12
Nodes (26): ColumnData, Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, default_db_browser_items(), execute_postgres(), execute_sql_server() (+18 more)

### Community 121 - "DbEngine"
Cohesion: 0.15
Nodes (12): DbAutocompleteCandidate, DbEngine, DbHistoryEntry, DbQueryBufferMeta, DbSnippet, default_volt_state_dir(), PersistedDbState, QualifiedName (+4 more)

### Community 122 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 123 - "shell/git.rs"
Cohesion: 0.09
Nodes (57): apply_git_view(), find_paren_number_range(), format_section_line(), git_line_is_untracked(), git_log_args(), git_status_action_targets(), git_status_commit_item_spans(), git_status_commit_message_spans() (+49 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.10
Nodes (18): apply_git_fringe_hunk(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitPrefixState, GitSummarySnapshot, GitSummaryState (+10 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "PickerItem"
Cohesion: 0.19
Nodes (8): match_item(), PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 127 - "find_font_by_name"
Cohesion: 0.26
Nodes (15): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), pick_best_matching_font_path(), preferred_berkeley_mono_font(), preferred_berkeley_mono_font_candidates(), preferred_font_search_roots(), RenderError (+7 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "Option"
Cohesion: 0.11
Nodes (10): Option, Vec, terminal_render_snapshot_preserves_wide_character_widths(), terminal_render_snapshot_tracks_visible_cursor(), TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot (+2 more)

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "From"
Cohesion: 0.07
Nodes (28): AbiContextHelpEntry, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGhostTextLine, AbiIconFontCategory, AbiLanguageServerRootStrategy, AbiLigatureConfig, AbiOilKeyAction (+20 more)

### Community 134 - ".new_with_secret_store"
Cohesion: 0.27
Nodes (7): load_persisted_state(), Arc, Path, Self, Send, Sync, SecretStore

### Community 135 - "LspCodeAction"
Cohesion: 0.15
Nodes (4): LspCodeAction, LspDocumentTextEdits, Error, windows_should_retry_spawn_error()

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 138 - "Diagnostic"
Cohesion: 0.23
Nodes (3): Diagnostic, DiagnosticSeverity, LspWorkspaceDiagnostic

### Community 140 - "AcpEvent"
Cohesion: 0.08
Nodes (34): AvailableCommand, acp_pick_model(), AcpEvent, AcpSessionInfo, build_acp_input_hint(), command_input_hint(), config_option_is_mode(), config_option_is_model() (+26 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (9): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, BufferId, Into, Option, Self, String (+1 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "AbiStatuslineContext"
Cohesion: 0.24
Nodes (7): exported_statusline_render(), StatuslineContext, AbiLspDiagnosticsInfo, AbiStatuslineContext, LspDiagnosticsInfo, LspDiagnosticsInfo, StatuslineContext

### Community 144 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "treesittercontext_ghosttext.rs"
Cohesion: 0.06
Nodes (51): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+43 more)

### Community 148 - "String"
Cohesion: 0.12
Nodes (9): CommandPaletteState, CompilationState, format_micros_as_millis(), GitStatusPrefix, OilKeyAction, Option, StatuslineContext, String (+1 more)

### Community 149 - "TextRange"
Cohesion: 0.10
Nodes (21): CodeActionParams, TextRange, code_action_params(), code_action_params_use_flattened_lsp_shape(), diagnostic_matches_request_range(), formatting_parser_maps_text_edits(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+13 more)

### Community 150 - "Instant"
Cohesion: 0.05
Nodes (25): ActiveTypingFrameProfile, apply_lsp_notifications(), average_duration(), DirectoryPrefixState, FpsOverlayState, frame_pacing_remaining(), git_refresh_deferred_for_typing(), KeySequenceState (+17 more)

### Community 151 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 153 - "DbBrowserContext"
Cohesion: 0.14
Nodes (19): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), default_action(), hook_command(), package(), package_exports_required_commands() (+11 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 156 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 157 - "AcpPickerItemSpec"
Cohesion: 0.13
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - ".path"
Cohesion: 0.24
Nodes (10): db_query_buffer_receives_sql_highlighting_without_blocking(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test(), theme_source_fingerprint_from_dir_changes_when_global_toml_changes() (+2 more)

### Community 161 - "git_remote_worktree_branch_list"
Cohesion: 0.24
Nodes (10): begin_oil_worktree_request(), fetch_git_prune(), git_remote_list(), git_remote_worktree_branch_list(), oil_git_worktree_command(), open_git_remote_picker(), open_git_worktree_dashboard_create(), remote_and_branch_from_ref() (+2 more)

### Community 162 - "Vec"
Cohesion: 0.27
Nodes (10): autocomplete_items(), calculator_symbols(), CalculatorSymbol, hover_lines(), hover_topics(), initial_buffer_lines(), initial_buffer_lines_only_seed_input_examples(), AutocompleteProviderItem (+2 more)

### Community 163 - ".byte_slice_chunks"
Cohesion: 0.28
Nodes (6): Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

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
Cohesion: 0.21
Nodes (11): default_terminal_args(), default_terminal_program(), TerminalConfig, default_shell_args(), default_shell_args_fallback(), default_shell_program(), default_shell_program_fallback(), package() (+3 more)

### Community 168 - "build_output.rs"
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "GhostTextContext"
Cohesion: 0.13
Nodes (13): GhostTextLine, packages(), LanguageConfiguration, Vec, syntax_languages(), exported_ghost_text_lines(), GhostTextLine, AbiGhostTextContext (+5 more)

### Community 170 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 171 - "cmake.rs"
Cohesion: 0.39
Nodes (8): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "choose_permission_outcome"
Cohesion: 0.40
Nodes (6): choose_permission_outcome(), format_permission_option_kind(), PendingPermission, PermissionOption, PermissionOptionKind, RequestPermissionOutcome

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "acp_buffer_layout"
Cohesion: 0.16
Nodes (12): browser_buffer_layout(), BrowserBufferLayout, acp_buffer_layout(), acp_pane_body_visible_rows(), AcpBufferLayout, AcpPaneLayout, input_panel_chrome_height(), overlay_text_columns() (+4 more)

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "shell/acp.rs"
Cohesion: 0.10
Nodes (29): acp_complete_slash(), acp_permission_approve(), acp_permission_deny(), acp_pick_mode(), acp_picker_entries(), acp_picker_entry(), acp_slash_completion_query(), AcpUiAction (+21 more)

### Community 180 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

### Community 181 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 182 - "Quickfix List PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Open Design Decisions, Parallel Implementation Plan, Quickfix List PRD (+1 more)

### Community 183 - "User-Owned Extension Surfaces Migration PRD"
Cohesion: 0.20
Nodes (9): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements, 4. Technical Specifications, 5. Risks & Roadmap, Acceptance Checklist, Module Plans, Requirements (+1 more)

### Community 184 - "Building locally"
Cohesion: 0.20
Nodes (9): Build both at the same time, Build the packaged local distribution, Build the user shared library, Build the Volt application, Building locally, Current status, Developer commands, Linux native dependencies (+1 more)

### Community 185 - "Vec"
Cohesion: 0.29
Nodes (9): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), OilSection, Vec, UserConfig, WorkspaceRootConfig (+1 more)

### Community 186 - "AbiPdfOpenMode"
Cohesion: 0.32
Nodes (5): exported_pdf_open_mode(), PdfOpenMode, AbiPdfOpenMode, PdfOpenMode, PdfOpenMode

### Community 188 - "Option"
Cohesion: 0.03
Nodes (60): absolute_path_hint(), AcpBufferState, AcpPane, active_buffer_revision_key(), active_shell_workspace_id(), apply_sqls_workspace_settings_for_buffer(), autocomplete_query(), autocomplete_request_for_buffer() (+52 more)

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 194 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 195 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 196 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 197 - "setup_standalone_user_repository"
Cohesion: 0.33
Nodes (6): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git()

### Community 198 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 199 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 200 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 201 - "PluginPackage"
Cohesion: 0.07
Nodes (28): file_open_package(), package(), package(), package(), package_with_path_matchers(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 202 - "lua.rs"
Cohesion: 0.43
Nodes (7): lua_package_auto_attaches_all_extensions(), lua_package_metadata(), lua_package_registers_formatter(), lua_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 203 - "nix.rs"
Cohesion: 0.43
Nodes (7): nix_package_auto_attaches_all_extensions(), nix_package_metadata(), nix_package_registers_formatter(), nix_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 204 - "perl.rs"
Cohesion: 0.43
Nodes (7): package(), perl_package_auto_attaches_all_extensions(), perl_package_metadata(), perl_package_registers_formatter(), perl_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 205 - "php.rs"
Cohesion: 0.43
Nodes (7): package(), php_package_auto_attaches_all_extensions(), php_package_metadata(), php_package_registers_no_formatter(), php_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 206 - "proto.rs"
Cohesion: 0.43
Nodes (7): package(), proto_package_auto_attaches_all_extensions(), proto_package_metadata(), proto_package_registers_formatter(), proto_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 207 - "r.rs"
Cohesion: 0.43
Nodes (7): package(), r_package_auto_attaches_all_extensions(), r_package_has_no_formatter(), r_package_metadata(), r_syntax_language_metadata(), LanguageConfiguration, syntax_language()

### Community 208 - "ruby.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, ruby_package_auto_attaches_all_extensions(), ruby_package_has_no_formatter(), ruby_package_metadata(), ruby_syntax_language_metadata(), syntax_language()

### Community 209 - "scala.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, scala_package_auto_attaches_all_extensions(), scala_package_metadata(), scala_package_registers_formatter(), scala_syntax_language_metadata(), syntax_language()

### Community 210 - "solidity.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, solidity_package_auto_attaches_all_extensions(), solidity_package_metadata(), solidity_package_registers_formatter(), solidity_syntax_language_metadata(), syntax_language()

### Community 211 - "swift.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, swift_package_auto_attaches_all_extensions(), swift_package_metadata(), swift_package_registers_formatter(), swift_syntax_language_metadata(), syntax_language()

### Community 212 - "lang/vim.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), vim_package_auto_attaches_all_extensions(), vim_package_has_no_formatter(), vim_package_metadata(), vim_syntax_language_metadata()

### Community 213 - "xml.rs"
Cohesion: 0.43
Nodes (7): package(), LanguageConfiguration, syntax_language(), xml_package_auto_attaches_all_extensions(), xml_package_metadata(), xml_package_registers_formatter(), xml_syntax_language_metadata()

### Community 216 - "Language"
Cohesion: 0.33
Nodes (5): Issues, Language, Language servers, Volt, Workspace

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: GitHub"
Cohesion: 0.29
Nodes (6): Conventions, Issue tracker: GitHub, Pull requests as a triage surface, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 220 - "load"
Cohesion: 0.28
Nodes (5): load(), config(), KeymapConfig, config(), LigatureConfig

### Community 222 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 234 - "markdown.rs"
Cohesion: 0.48
Nodes (6): inline_syntax_language(), package(), package_auto_attaches_markdown_extensions_and_formatter(), LanguageConfiguration, syntax_language(), syntax_languages_register_markdown_grammars()

### Community 235 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_toml_and_registers_formatter(), LanguageConfiguration, syntax_language(), syntax_language_registers_toml_grammar()

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 238 - "index_syntax_lines"
Cohesion: 0.40
Nodes (5): index_syntax_lines(), relative_byte_column_to_char_column(), IndexedSyntaxLines, index_syntax_lines_converts_byte_columns_after_variation_selector(), index_syntax_lines_preserves_capture_names()

### Community 239 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 240 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 241 - "TextEdit"
Cohesion: 0.67
Nodes (4): TextEdit, apply_text_edits_to_span(), text_edit_to_input_edit(), InputEdit

### Community 242 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 244 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **137 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+132 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **30 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `shell/mod.rs`, `shell/tests.rs`, `ShellError`, `String`, `Result`, `AcpEvent`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `Instant`, `command_stream.rs`, `UserLibrary`, `.path`, `git_remote_worktree_branch_list`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `ShellUiState`, `.new`, `shell/acp.rs`, `shell_ui_mut`, `.len`, `temp_dir`, `Option`, `String`, `state_with_user_library`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `AcpManager`, `.new`, `.new`, `main`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `Option`, `shell/browser.rs`, `sync_quickfix_popup_buffer`, `PickerOverlay`, `Result`, `buffer_is_git_status`, `shell/git.rs`, `GitSummaryState`?**
  _High betweenness centrality (0.120) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `shell/mod.rs`, `Option`, `ShellError`, `AcpPaneState`, `.new`, `TextBuffer`, `Instant`, `state.rs`, `UserLibrary`, `shell/pdf.rs`, `ShellUiState`, `acp_buffer_layout`, `.new`, `shell/acp.rs`, `render_buffer_with_view_state`, `.len`, `Option`, `String`, `directory.rs`, `shell/terminal.rs`, `.new`, `draw_diagnostic_underlines_for_segment`, `.new`, `detect_markdown_table`, `shell/browser.rs`, `sync_quickfix_popup_buffer`, `PickerOverlay`, `panic_payload_message`, `shell/git.rs`, `GitSummaryState`?**
  _High betweenness centrality (0.079) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `oil.rs`, `user/lib.rs`, `syntax_language`, `sdk/src/lib.rs`, `calculator.rs`, `DbBrowserContext`, `lsp.rs`, `AcpPickerItemSpec`, `workspace.rs`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `user/terminal.rs`, `GhostTextContext`, `UserLibraryModule`, `cmake.rs`, `user/browser.rs`, `.new`, `HeaderlineTestUserLibrary`, `Self`, `bash.rs`, `clojure.rs`, `elixir.rs`, `.new`, `hcl.rs`, `java.rs`, `kotlin.rs`, `lua.rs`, `nix.rs`, `perl.rs`, `php.rs`, `proto.rs`, `r.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `xml.rs`, `PickerItemSpec`, `main`, `editor-plugin-host/src/lib.rs`, `package`, `PluginBuffer`, `common.rs`, `markdown.rs`, `syntax_language`, `debug_adapters`, `package`, `PluginKeyBinding`, `PluginCommand`?**
  _High betweenness centrality (0.071) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _137 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `shell/mod.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.02731143639517713 - nodes in this community are weakly interconnected._
- **Should `Path` be split into smaller, more focused modules?**
  _Cohesion score 0.0900483904902167 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.02562929061784897 - nodes in this community are weakly interconnected._