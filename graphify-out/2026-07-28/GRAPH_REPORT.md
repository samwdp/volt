# Graph Report - volt  (2026-07-28)

## Corpus Check
- 245 files · ~613,080 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8976 nodes · 36398 edges · 309 communities (298 shown, 11 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 2987 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `7daf2a4a`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- register_shell_hooks
- LspClientError
- shell/tests.rs
- .from_text
- Result
- user/lib.rs
- .new
- EditorRuntime
- .from
- render.rs
- Result
- PluginPackage
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
- editor-picker/src/lib.rs
- browser_host.rs
- window_effects.rs
- treesitter_install.rs
- find_font_by_name
- HoverOverlay
- Theme
- load_font_set_with_mode
- Option
- Section
- shell/pdf.rs
- AutocompleteProviderConfig
- compile.rs
- HoverProviderConfig
- script.js
- Self
- ShellBuffer
- UserLibraryModule
- editor-db/src/lib.rs
- String
- shell/mod.rs
- ShellUiState
- AbiGitFeatureSpec
- .new
- AbiContextHelpSpec
- client.rs
- HeaderlineTestUserLibrary
- String
- PluginBuffer
- sync_quickfix_popup_buffer
- render_buffer_with_view_state
- .len
- ROption
- Option
- RVec
- temp_dir
- Result
- Self
- LspNotification
- AbiKeymapConfig
- state_with_user_library
- SyntaxRegistry
- .default
- String
- DebugConfiguration
- .new
- String
- theme.rs
- .send
- .new
- workspace.rs
- directory.rs
- editor-jobs/src/lib.rs
- workspace_search.rs
- shell/terminal.rs
- User Packages
- PickerItemSpec
- AcpManager
- render_text_with_fonts
- Diagnostic
- LanguageServerRegistry
- volt/src/main.rs
- .new
- LineCharMap
- shell/git.rs
- Path
- editor-plugin-host/src/lib.rs
- CommandSource
- editor-core/src/lib.rs
- registered_queries.rs
- workspace_nav.rs
- Option
- spawn_reader_thread
- PickerSession
- WorkspaceConfigurationValue
- UserLibrary
- main
- TextPoint
- common.rs
- PickerItem
- editor-lsp/src/lib.rs
- .move_object_end_forward
- Option
- git_root
- shell/picker.rs
- BufferId
- .new
- Option
- resolve_picker_extra
- PluginKeyBinding
- .load_from_path
- .spawn
- PluginCommand
- DbService
- process_supervisor.rs
- .new
- DbEngine
- BufferId
- Vec
- GitSummaryState
- statusline.rs
- Option
- String
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- oil.rs
- key_sequence.rs
- String
- .new_with_secret_store
- treesittercontext_ghosttext.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- .path
- PixelRect
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- editor-buffer/src/lib.rs
- JobSpec
- ShellConfig
- standalone_user_manifest.rs
- editor-render/src/lib.rs
- .get
- OilDefaultsSection
- LspCodeAction
- LspLocation
- Vec
- DbBrowserContext
- lsp.rs
- TextRange
- LspLogEntry
- AcpPickerItemSpec
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- clipboard.rs
- AbiPickerTruncateStrategy
- .line
- AbiLanguageConfiguration
- ServiceRegistry
- editor-syntax/src/lib.rs
- String
- load
- build_output.rs
- GhostTextContext
- AbiSectionTree
- ancestor_contexts_for_cursor
- normalize_inline_text
- JobResult
- GrammarRecompileReport
- user/browser.rs
- TerminalDimensions
- .oil_keybindings
- `user`
- shell/acp.rs
- spawn_terminal_reader
- I
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- panic_payload_message
- Workspace Issues Plugin
- Workspace Switching and Marks
- Workspace Dashboard Worktree Remove + Picker Extra Keybinds
- load_user_library
- .oil_directory_sections
- Database Explorer PRD
- MulticursorState
- bash.rs
- clojure.rs
- elixir.rs
- graphql.rs
- hcl.rs
- java.rs
- kotlin.rs
- latex.rs
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
- .oil_directory_sections
- AbiOilKeyAction
- Language
- choose_permission_outcome
- Domain Docs
- Issue tracker: Local Markdown
- syntax_language
- package
- Issue Store and Create
- Status commands
- Capture focused file
- Async Capture on save
- Issue Board
- Place and jump to code
- Open Issue from Code Reference
- Issue Scan
- 01 — Ambiguous Prefix Timeout
- 03 — Cycle Project Workspaces
- 04 — Mark List Management
- 05 — Marked Workspace Slot Jumps
- debug_adapters
- syntax_language
- 02 — Minor Mode Keybinding Precedence
- package
- VimActionContext
- 01 — Picker Extra Keybinds (dispatch + provider API)
- 02 — Migrate QuickFix Ctrl+q onto provider extras
- CLAUDE.md
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md
- 03 — Worktree Remove command + streamed popup
- 04-dashboard-ctrl-d.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 741 edges
2. `ShellBuffer` - 362 edges
3. `shell_ui_mut()` - 326 edges
4. `register_shell_hooks()` - 255 edges
5. `shell_ui()` - 211 edges
6. `ShellError` - 181 edges
7. `shell_buffer()` - 178 edges
8. `shell_buffer_mut()` - 174 edges
9. `TextBuffer` - 166 edges
10. `ShellUiState` - 155 edges

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

## Communities (309 total, 11 thin omitted)

### Community 0 - "register_shell_hooks"
Cohesion: 0.06
Nodes (121): buffer_uses_browser_host_surface(), Cow, write_system_clipboard(), yank_from_clipboard_text(), yank_to_clipboard_text(), active_shell_buffer_has_input(), active_shell_buffer_id(), active_shell_buffer_is_terminal() (+113 more)

### Community 1 - "LspClientError"
Cohesion: 0.08
Nodes (26): CodeActionParams, code_action_params(), inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), LspClientError, LspClientManager, LspSessionHandle (+18 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (63): load_font_set(), acp_wrapped_text_uses_full_width_on_continuation_rows(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_sync_plan_avoids_notification_overlays(), codicon_glyphs_fit_inside_one_editor_cell(), compose_emoji_surface_rasterizes_simple_emoji() (+55 more)

### Community 3 - ".from_text"
Cohesion: 0.13
Nodes (67): line_ranges_and_char_searches_resolve_expected_points(), move_word_forward_advances_to_the_next_word(), word_motions_treat_punctuation_runs_as_words(), vim_search_entries_trim_whitespace_from_labels(), autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text() (+59 more)

### Community 4 - "Result"
Cohesion: 0.05
Nodes (52): Display, Error, From, ShellError, clear_key_sequence(), accept_autocomplete(), active_lsp_workspace_loaded(), active_runtime_surface() (+44 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (91): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), exported_acp_client_by_id(), exported_acp_clients(), exported_autocomplete_providers(), exported_autocomplete_result_limit(), exported_autocomplete_token_icon() (+83 more)

### Community 6 - ".new"
Cohesion: 0.12
Nodes (63): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+55 more)

### Community 7 - "EditorRuntime"
Cohesion: 0.07
Nodes (101): EditorRuntime, Default, active_git_status_command_context(), cancel_git_commit_buffer(), ensure_no_rebase_in_progress(), ensure_rebase_in_progress(), git_branch_list(), git_command_output_owned() (+93 more)

### Community 8 - ".from"
Cohesion: 0.04
Nodes (75): main(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GitStashEntry, main(), abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers() (+67 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (115): FontSet, wrap_columns_for_width(), acp_buffer_layout(), acp_pane_body_visible_rows(), acp_prefix_columns(), acp_spinner_frame(), AcpBufferLayout, adjusted_contextual_ligature_pixel_size() (+107 more)

### Community 10 - "Result"
Cohesion: 0.14
Nodes (26): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_submitted(), acp_pick_session() (+18 more)

### Community 11 - "PluginPackage"
Cohesion: 0.05
Nodes (45): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+37 more)

### Community 12 - "sdk/src/lib.rs"
Cohesion: 0.05
Nodes (42): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec, default_db_browser_line() (+34 more)

### Community 13 - "TextBuffer"
Cohesion: 0.11
Nodes (5): BufferStats, delimiter_partner(), Default, Option, TextBuffer

### Community 14 - "LanguageServerSpec"
Cohesion: 0.12
Nodes (12): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), LanguageServerSpec, BTreeMap, Into, IntoIterator, Item (+4 more)

### Community 15 - "LiveTerminalSession"
Cohesion: 0.09
Nodes (23): AlacrittyEvent, Keycode, Mod, terminal_key_for_event(), LiveTerminalError, LiveTerminalSession, QueuedEventListener, Arc (+15 more)

### Community 16 - "editor-fs/src/lib.rs"
Cohesion: 0.06
Nodes (60): compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind, discover_projects() (+52 more)

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
Cohesion: 0.07
Nodes (39): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+31 more)

### Community 24 - "editor-picker/src/lib.rs"
Cohesion: 0.17
Nodes (18): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+10 more)

### Community 25 - "browser_host.rs"
Cohesion: 0.09
Nodes (39): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests() (+31 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "treesitter_install.rs"
Cohesion: 0.12
Nodes (55): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands() (+47 more)

### Community 28 - "find_font_by_name"
Cohesion: 0.26
Nodes (15): default_font_candidates(), find_font_by_name(), find_system_monospace_font(), pick_best_matching_font_path(), preferred_berkeley_mono_font(), preferred_berkeley_mono_font_candidates(), preferred_font_search_roots(), RenderError (+7 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (28): RankedAutocompleteEntry, hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteOverlay, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind (+20 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "load_font_set_with_mode"
Cohesion: 0.07
Nodes (33): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode(), load_icon_font() (+25 more)

### Community 32 - "Option"
Cohesion: 0.02
Nodes (97): ActiveTypingFrameProfile, adjust_tag_child_indent(), advance_markdown_table_insert_tab(), advance_markdown_table_normal_tab(), apply_markdown_table_update(), average_duration(), block_comment_toggle_removal_lens(), buffer_context_overlay_snapshot() (+89 more)

### Community 33 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

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
Cohesion: 0.02
Nodes (47): acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), acp_tool_call_from_partial_update(), AcpPane (+39 more)

### Community 42 - "UserLibraryModule"
Cohesion: 0.07
Nodes (26): exported_oil_defaults(), exported_oil_keybindings(), exported_pdf_open_mode(), OilDefaults, OilKeybindings, PdfOpenMode, AbiIconFontCategory, AbiIconFontSymbol (+18 more)

### Community 43 - "editor-db/src/lib.rs"
Cohesion: 0.09
Nodes (33): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbColumn, DbIndex, DbSchemaCache, DbTable, default_db_browser_line() (+25 more)

### Community 44 - "String"
Cohesion: 0.08
Nodes (29): active_parameter_label(), CopilotDeviceCodePrompt, documentation_lines(), hover_marked_string(), hover_marked_string_markdown_text(), hover_parser_formats_marked_string_language_blocks_as_markdown(), hover_parser_keeps_plaintext_markup_plain(), hover_parser_preserves_markdown_content() (+21 more)

### Community 45 - "shell/mod.rs"
Cohesion: 0.02
Nodes (275): acp_decode_image(), activate_db_browser_line(), active_buffer_revision_key(), active_lsp_buffer_context(), active_lsp_code_action_range(), active_project_workspace_root(), active_shell_buffer_path(), active_shell_workspace_id() (+267 more)

### Community 46 - "ShellUiState"
Cohesion: 0.06
Nodes (26): active_buffer_event_context(), active_runtime_buffer(), buffer_is_git_commit(), buffer_is_oil_preview(), BufferViewState, close_runtime_pane(), execute_vim_command_line(), handle_terminal_vim_edit() (+18 more)

### Community 47 - "AbiGitFeatureSpec"
Cohesion: 0.19
Nodes (9): GitPrefixBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitFeatureSpec, GitPrefixBinding, GitStatusPrefix, GitFeatureSpec (+1 more)

### Community 48 - ".new"
Cohesion: 0.02
Nodes (164): BufferKind, ActiveLspBufferContext, default_vim_target(), WorkspaceId, absolute_path_hint(), acp_build_output_lines(), acp_build_plan_lines(), acp_icon_segment() (+156 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.07
Nodes (26): exported_browser_feature_spec(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_oil_feature_spec(), exported_terminal_feature_spec(), BrowserFeatureSpec, ContextHelpSpec (+18 more)

### Community 50 - "client.rs"
Cohesion: 0.05
Nodes (58): ClientCapabilities, apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), code_action_parser_collects_active_file_edits() (+50 more)

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (33): AtomicUsize, CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), directory_view_state_uses_user_oil_defaults(), HeaderlineTestUserLibrary, AcpClient, Arc, AutocompleteProvider (+25 more)

### Community 52 - "String"
Cohesion: 0.05
Nodes (104): cycle_hover_provider(), cycle_runtime_pane(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), shell_ui(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger() (+96 more)

### Community 53 - "PluginBuffer"
Cohesion: 0.08
Nodes (4): PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate

### Community 54 - "sync_quickfix_popup_buffer"
Cohesion: 0.09
Nodes (24): active_directory_root(), buffer_is_quickfix(), ensure_directory_buffer(), oil_default_root(), oil_parent_root(), quickfix_clear_marks(), quickfix_entry_for_cursor(), quickfix_mark_all() (+16 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.11
Nodes (86): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+78 more)

### Community 56 - ".len"
Cohesion: 0.06
Nodes (31): apply_input_operator_motion(), ascii_control_caret_notation(), char_at_index(), display_columns_for_character(), exact_match_positions_in_chars(), find_char_forward(), fuzzy_match_end(), fuzzy_match_end_in_chars() (+23 more)

### Community 57 - "ROption"
Cohesion: 0.12
Nodes (15): GitCommandBinding, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiGitCommandBinding, AbiLspDiagnosticsInfo, AbiStatuslineContext, AutocompleteProvider, AutocompleteProviderItem (+7 more)

### Community 58 - "Option"
Cohesion: 0.07
Nodes (61): BufRead, code_action_params_use_flattened_lsp_shape(), completion_documentation(), configuration_item_section(), copilot_status_notifications_offer_sign_in_action(), effective_workspace_configuration_settings(), execute_command_params(), execute_command_params_from_inline_item() (+53 more)

### Community 59 - "RVec"
Cohesion: 0.13
Nodes (14): exported_terminal_config(), AbiAcpClient, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, AcpClient, HoverProvider, HoverProviderTopic (+6 more)

### Community 60 - "temp_dir"
Cohesion: 0.08
Nodes (49): parse_log_oneline(), begin_oil_worktree_request(), build_git_fringe_snapshot(), create_git_worktree_from_query(), fetch_git_prune(), git_branch_merge(), git_branch_push_remote(), git_branch_remote() (+41 more)

### Community 61 - "Result"
Cohesion: 0.08
Nodes (87): default_error_log_path(), format_current_line_indent(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line(), acp_input_field_dw_deletes_motion_range() (+79 more)

### Community 62 - "Self"
Cohesion: 0.07
Nodes (27): AcpActionSpec, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind, PickerAcpClientContext, PickerActionSpec, PickerBufferContext (+19 more)

### Community 63 - "LspNotification"
Cohesion: 0.10
Nodes (10): completion_level_for_message(), LspNotification, LspNotificationAction, LspNotificationEntry, LspNotificationLevel, LspNotificationLog, LspNotificationProgress, LspNotificationSnapshot (+2 more)

### Community 64 - "AbiKeymapConfig"
Cohesion: 0.10
Nodes (17): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, PaneConfig, config(), PaneConfig (+9 more)

### Community 65 - "state_with_user_library"
Cohesion: 0.08
Nodes (70): active_runtime_popup(), ctrl_mod(), install_mark_list_state_for_test(), open_workspace_file(), open_workspace_from_project(), add_linked_worktree(), browser_popup_command_focuses_the_popup_surface(), browser_sync_plan_excludes_pdf_buffers() (+62 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.10
Nodes (44): buffer_text_for_byte_range(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser(), desired_indent_for_loaded_language(), general_predicates_match(), highlight_inline_language_per_line() (+36 more)

### Community 67 - ".default"
Cohesion: 0.11
Nodes (47): Self, commit_buffer_template(), commit_buffer_template_matches_git_commit_message_format(), commit_buffer_template_shows_initial_commit_without_head(), commit_section(), feature_spec(), flatten_section_ids(), git_section_title() (+39 more)

### Community 68 - "String"
Cohesion: 0.14
Nodes (46): active_command_input_hint(), apply_acp_notification(), apply_background_pipes(), apply_command_environment(), apply_launch_environment(), apply_output_limit(), background_command_candidates(), background_command_names() (+38 more)

### Community 69 - "DebugConfiguration"
Cohesion: 0.08
Nodes (28): codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan, must() (+20 more)

### Community 70 - ".new"
Cohesion: 0.04
Nodes (66): zig_flat_grammar_uses_bundled_queries(), configure_background_command(), Box, Command, Error, Path, Result, setup_standalone_user_repository() (+58 more)

### Community 71 - "String"
Cohesion: 0.09
Nodes (19): CaptureThemeMapping, cmake_configuration(), DeferredQuery, dockerfile_configuration(), LanguageConfiguration, LanguageLoader, load_language(), normalize_extension() (+11 more)

### Community 72 - "theme.rs"
Cohesion: 0.12
Nodes (51): apply_language_options_table(), apply_options_table(), assert_bundled_theme_omits_shared_sections(), assert_bundled_theme_uses_pallet_colors(), bundled_shared_theme_config(), bundled_shared_theme_config_includes_window_effect_defaults(), bundled_theme_sources(), bundled_themes_define_defaults_for_all_compiled_languages() (+43 more)

### Community 73 - ".send"
Cohesion: 0.10
Nodes (44): ChildStderr, ClientSideConnection, acp_runtime_loop(), AcpClient, AcpCommand, AcpRuntime, AcpRuntimeState, AcpSession (+36 more)

### Community 74 - ".new"
Cohesion: 0.10
Nodes (29): buffer_lookup_is_scoped_to_workspace(), close_buffer_disconnects_sessions_and_clears_reuse_state(), connected_event_for_closed_buffer_disconnects_orphaned_session(), drain_events_shows_incremental_plan_progress_across_frames(), humanize_debug_label(), install_acp_test_buffer(), open_permission_request_reorders_queue_for_requested_picker(), pending_slash_completion_trigger_rejects_multiline_input() (+21 more)

### Community 75 - "workspace.rs"
Cohesion: 0.14
Nodes (25): existing_workspace_for_project(), file_picker_preview(), message_item(), package(), package_exports_cycle_project_workspace_commands(), package_exports_format_command(), package_exports_mark_list_commands(), package_exports_marked_workspace_slot_jump_commands() (+17 more)

### Community 76 - "directory.rs"
Cohesion: 0.06
Nodes (69): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+61 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), environment_value(), explicit_windows_env_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.09
Nodes (47): LspWorkspaceDiagnostic, PickerEntry, workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+39 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.14
Nodes (38): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session() (+30 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.11
Nodes (26): ShellTestUserLibrary, exported_picker_provider_items(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search() (+18 more)

### Community 82 - "AcpManager"
Cohesion: 0.15
Nodes (10): acp_connected(), acp_open_permission_request(), acp_permission_picker_closed(), AcpManager, AcpPendingPermissionUi, drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), open_permission_picker() (+2 more)

### Community 83 - "render_text_with_fonts"
Cohesion: 0.06
Nodes (54): Canvas, DrawCommand, RenderColor, Arc, Self, TextStyle, acp_slice_chars(), alpha_bitmap_surface() (+46 more)

### Community 84 - "Diagnostic"
Cohesion: 0.08
Nodes (31): close_buffer_keeps_session_alive_for_next_file(), diagnostics_parser_maps_lsp_fields(), file_uri_roundtrip_handles_windows_paths(), file_uri_to_path(), lsp_code_action_diagnostic(), lsp_diagnostic_severity(), LspClientState, LspInlineCompletionItem (+23 more)

### Community 85 - "LanguageServerRegistry"
Cohesion: 0.14
Nodes (15): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LspError, path_is_solution(), resolve_single_solution_path() (+7 more)

### Community 86 - "volt/src/main.rs"
Cohesion: 0.13
Nodes (26): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, parse_launch_options(), parse_launch_options_accepts_fps_overlay(), parse_launch_options_accepts_profile_alias() (+18 more)

### Community 87 - ".new"
Cohesion: 0.06
Nodes (77): buffer_footer_layout(), acp_multiline_text_lines_strip_carriage_returns(), acp_section_layout_orders_output_input_footer_and_statusline(), autocomplete_entries_are_not_limited_by_visible_result_limit(), autocomplete_or_group_uses_first_provider_with_results(), autocomplete_query_allows_empty_member_access_after_dot_and_arrow(), block_cursor_text_overlay_positions_multibyte_cursor_text(), block_cursor_text_overlay_uses_visible_glyph_for_variation_selector() (+69 more)

### Community 88 - "LineCharMap"
Cohesion: 0.10
Nodes (28): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+20 more)

### Community 89 - "shell/git.rs"
Cohesion: 0.12
Nodes (34): apply_git_view(), build_git_summary_snapshot(), command_output_transcript(), git_command_output_background(), git_log_args(), git_read_command_output_allow_exit_codes(), git_repository_present(), git_status_command_name() (+26 more)

### Community 90 - "Path"
Cohesion: 0.07
Nodes (24): append_query_source(), command_failure_message(), default_install_root(), ensure_cloned_grammar_dir_exists(), extra_highlight_query_can_supply_missing_bundled_query(), GrammarSource, InstallCommandSpec, io_error() (+16 more)

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
Nodes (34): csharp_config(), csharp_flat_grammar_uses_bundled_queries(), csharp_grammar_available(), default_grammars_root(), markdown_and_inline_merged_highlight_compiles(), markdown_config(), markdown_fenced_code_blocks_use_injected_language_highlighting(), markdown_grammar_available() (+26 more)

### Community 95 - "workspace_nav.rs"
Cohesion: 0.09
Nodes (27): cycle_project_workspace(), CycleDirection, mark_and_jump_use_normalized_path_identity(), mark_appends_absent_root_and_duplicate_is_no_op(), mark_list_parse_and_serialize_strip_blank_lines_and_preserve_order(), marked_workspace_jump(), marked_workspace_jump_switches_open_root_opens_closed_and_notifies_missing(), MarkedWorkspaceJump (+19 more)

### Community 96 - "Option"
Cohesion: 0.15
Nodes (29): cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), git_line_is_untracked(), git_status_delete_target_for_line() (+21 more)

### Community 97 - "spawn_reader_thread"
Cohesion: 0.14
Nodes (21): ChildStdin, diagnostic_matches_request_range(), launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), AtomicBool (+13 more)

### Community 98 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.16
Nodes (9): language_server_spec_exposes_workspace_configuration_builders(), From, Number, T, workspace_configuration_value_round_trips_through_json(), WorkspaceConfigurationValue, K, abi_language_server_spec_round_trips_workspace_configuration() (+1 more)

### Community 100 - "UserLibrary"
Cohesion: 0.09
Nodes (52): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_buffer_layout(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_host_viewport_rect(), browser_state_for_kind() (+44 more)

### Community 101 - "main"
Cohesion: 0.13
Nodes (17): bootstrap(), HostBootstrap, command_palette_items(), main(), DebugAdapterSpec, Error, LanguageConfiguration, LanguageServerSpec (+9 more)

### Community 102 - "TextPoint"
Cohesion: 0.10
Nodes (10): Self, Selection, TextPoint, TextSnapshot, char_immediately_before(), chars_immediately_before(), normalize_completion_replacement(), text_point_to_tree_sitter_point() (+2 more)

### Community 103 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 104 - "PickerItem"
Cohesion: 0.20
Nodes (7): PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 105 - "editor-lsp/src/lib.rs"
Cohesion: 0.19
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), LanguageServerRootStrategy, must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers() (+20 more)

### Community 106 - ".move_object_end_forward"
Cohesion: 0.13
Nodes (8): is_object_separator(), is_punctuation_char(), is_word_char(), matches_word_kind(), word_kind_ranges_cover_big_word_objects(), word_motion_class(), WordKind, WordMotionClass

### Community 107 - "Option"
Cohesion: 0.14
Nodes (6): LanguageServerSession, normalize_optional_string(), AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 108 - "git_root"
Cohesion: 0.14
Nodes (38): checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer(), create_git_worktree(), delete_git_status_targets(), fetch_git_all(), fetch_git_pushremote() (+30 more)

### Community 109 - "shell/picker.rs"
Cohesion: 0.10
Nodes (37): UserLibraryService, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay(), picker_overlay_from_spec() (+29 more)

### Community 110 - "BufferId"
Cohesion: 0.14
Nodes (37): active_and_secondary_buffer_ids(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+29 more)

### Community 111 - ".new"
Cohesion: 0.25
Nodes (17): diff_git_dwim(), git_args_with_no_pager(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), merge_git_preview(), open_git_diff_buffer(), open_git_diff_commit(), open_git_diff_staged() (+9 more)

### Community 112 - "Option"
Cohesion: 0.14
Nodes (7): CommandPaletteState, CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 113 - "resolve_picker_extra"
Cohesion: 0.13
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.10
Nodes (26): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, hook_command(), leader_binding() (+18 more)

### Community 115 - ".load_from_path"
Cohesion: 0.12
Nodes (16): from_reader_normalizes_crlf_and_tracks_line_endings(), must(), reload_from_path_requires_a_backing_file(), reload_from_path_returns_false_when_disk_state_is_unchanged(), reload_from_path_updates_content_preserves_cursor_and_marks_clean(), AsRef, Drop, E (+8 more)

### Community 116 - ".spawn"
Cohesion: 0.11
Nodes (15): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+7 more)

### Community 117 - "PluginCommand"
Cohesion: 0.10
Nodes (17): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+9 more)

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

### Community 122 - "BufferId"
Cohesion: 0.15
Nodes (23): ActiveBufferEventContext, apply_git_status_snapshot(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_snapshot_for_buffer(), handle_git_status_tab(), open_git_cherry_buffer(), open_git_status_popup() (+15 more)

### Community 123 - "Vec"
Cohesion: 0.20
Nodes (28): oil_directory_line_spans(), find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token(), git_status_entry_token_from_icon() (+20 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.10
Nodes (17): apply_git_fringe_hunk(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitSummarySnapshot, GitSummaryState, parse_git_fringe_diff() (+9 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "Option"
Cohesion: 0.12
Nodes (7): Option, Vec, TerminalCursorSnapshot, TerminalRenderLine, TerminalRenderRun, TerminalRenderSnapshot, TerminalSnapshot

### Community 127 - "String"
Cohesion: 0.14
Nodes (17): db_browser_action_from_spec(), DisabledSecretStore, initialize_native_keyring(), InMemorySecretStore, load_postgres_schema(), OsSecretStore, qualified_name_from_spec(), redact_error() (+9 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.14
Nodes (29): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+21 more)

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "String"
Cohesion: 0.28
Nodes (19): search_is_case_sensitive(), apply_lsp_code_action(), apply_lsp_document_edit(), collect_search_output(), lsp_code_action_picker_entry(), lsp_code_action_picker_preview(), lsp_code_action_supported_edits(), lsp_code_actions_picker_overlay() (+11 more)

### Community 134 - ".new_with_secret_store"
Cohesion: 0.27
Nodes (7): load_persisted_state(), Arc, Path, Self, Send, Sync, SecretStore

### Community 135 - "treesittercontext_ghosttext.rs"
Cohesion: 0.07
Nodes (40): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+32 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - ".path"
Cohesion: 0.19
Nodes (12): db_query_buffer_receives_sql_highlighting_without_blocking(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), Drop, sync_active_buffer_layout_for_test() (+4 more)

### Community 138 - "PixelRect"
Cohesion: 0.14
Nodes (17): centered_rect(), golden_split_size(), horizontal_golden_ratio_grows_the_first_active_pane(), horizontal_golden_ratio_grows_the_second_active_pane(), horizontal_pane_rects(), horizontal_pane_rects_for_active(), horizontal_split_returns_two_stacked_rects(), PixelRect (+9 more)

### Community 139 - "DynamicUserLibrary"
Cohesion: 0.04
Nodes (20): DynamicUserLibrary, AcpClient, AutocompleteProvider, BrowserFeatureSpec, ContextHelpSpec, DbFeatureSpec, DebugAdapterSpec, GitFeatureSpec (+12 more)

### Community 140 - "AcpEvent"
Cohesion: 0.08
Nodes (34): AvailableCommand, acp_pick_model(), AcpEvent, AcpSessionInfo, build_acp_input_hint(), command_input_hint(), config_option_is_mode(), config_option_is_model() (+26 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.16
Nodes (8): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, BufferId, Option, Self, String, Vec

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - "editor-buffer/src/lib.rs"
Cohesion: 0.12
Nodes (22): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), delimited_ranges_cover_quotes_and_brackets(), detect_preferred_line_ending(), EditRecord, edits_since_returns_contiguous_forward_edits(), find_matching_close_tag() (+14 more)

### Community 144 - "JobSpec"
Cohesion: 0.23
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "editor-render/src/lib.rs"
Cohesion: 0.23
Nodes (13): font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests(), font_metadata_matching_accepts_family_names(), font_name_matches(), font_style_rank(), is_font_file(), normalize_font_name() (+5 more)

### Community 148 - ".get"
Cohesion: 0.31
Nodes (4): DbBrowserBufferView, snippets_and_history_persist(), summarize_sql(), DbBrowserItemRenderer

### Community 149 - "OilDefaultsSection"
Cohesion: 0.32
Nodes (5): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSortMode, OilDefaults

### Community 150 - "LspCodeAction"
Cohesion: 0.08
Nodes (20): formatting_parser_maps_text_edits(), full_document_range(), lsp_formatting_options(), lsp_position_from_text_point(), lsp_range_from_text_range(), lsp_text_edit_from_lsp(), LspCodeAction, LspDocumentTextEdits (+12 more)

### Community 151 - "LspLocation"
Cohesion: 0.16
Nodes (11): definition_parser_preserves_uri_backed_locations(), definition_parser_supports_location_links(), location_from_link(), location_from_lsp(), location_sorting_deduplicates_reference_results(), LspLocation, parse_definition_response(), parse_reference_response() (+3 more)

### Community 152 - "Vec"
Cohesion: 0.10
Nodes (11): EventLog, format_micros_as_millis(), LspState, AutocompleteProvider, ContextHelpSpec, GhostTextLine, HoverProvider, StatuslineContext (+3 more)

### Community 153 - "DbBrowserContext"
Cohesion: 0.14
Nodes (19): browser_item(), browser_items(), browser_items_shape_table_rows_from_user_config(), connect_buffer_lines(), default_action(), hook_command(), package(), package_exports_required_commands() (+11 more)

### Community 154 - "lsp.rs"
Cohesion: 0.21
Nodes (16): auto_start_binding_details(), auto_start_bindings_match_registered_server_path_matchers(), copilot_language_server(), csharp_workspace_configuration_remains_well_formed_when_present(), has_command(), language_servers(), language_servers_have_unique_ids_and_nonempty_programs(), package() (+8 more)

### Community 155 - "TextRange"
Cohesion: 0.14
Nodes (6): delimited_and_tag_ranges_cover_quickref_objects(), is_inline_whitespace(), is_sentence_closer(), Fn, sentence_ranges_cover_inner_and_around_text_objects(), TextRange

### Community 156 - "LspLogEntry"
Cohesion: 0.17
Nodes (5): LspLogDirection, LspLogEntry, LspLogSnapshot, LspTransportLog, SystemTime

### Community 157 - "AcpPickerItemSpec"
Cohesion: 0.12
Nodes (18): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+10 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (16): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_from_root(), load_reads_referenced_child_files() (+8 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - "clipboard.rs"
Cohesion: 0.19
Nodes (12): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+4 more)

### Community 161 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 162 - ".line"
Cohesion: 0.29
Nodes (6): large_buffers_expose_line_windows_without_full_materialization(), String, Vec, trimmed_line(), visible_line_len(), RopeSlice

### Community 163 - "AbiLanguageConfiguration"
Cohesion: 0.25
Nodes (7): exported_syntax_languages(), AbiGrammarSource, AbiLanguageConfiguration, GrammarSource, LanguageConfiguration, GrammarSource, LanguageConfiguration

### Community 164 - "ServiceRegistry"
Cohesion: 0.21
Nodes (6): BoxedService, HashMap, Option, T, ServiceRegistry, TypeId

### Community 165 - "editor-syntax/src/lib.rs"
Cohesion: 0.06
Nodes (45): aligned_indent_column(), apply_text_edits_to_span(), asset_path_from_parts(), capture_requires_theme_token(), current_line_starts_with_token(), default_query_asset_root(), delimiter_column(), evaluate_general_predicate() (+37 more)

### Community 166 - "String"
Cohesion: 0.29
Nodes (6): call_function(), Lexer<'a>, Parser<'a, 'b>, Result, String, Token

### Community 167 - "load"
Cohesion: 0.12
Nodes (16): default_terminal_args(), default_terminal_program(), load(), config(), KeymapConfig, TerminalConfig, config(), LigatureConfig (+8 more)

### Community 168 - "build_output.rs"
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "GhostTextContext"
Cohesion: 0.15
Nodes (12): packages(), LanguageConfiguration, Vec, syntax_languages(), exported_ghost_text_lines(), GhostTextLine, AbiGhostTextContext, GhostTextContext (+4 more)

### Community 170 - "AbiSectionTree"
Cohesion: 0.10
Nodes (18): exported_git_status_sections(), exported_oil_directory_sections(), DirectoryEntry, OilSortMode, Path, SectionTree, AbiDirectoryEntry, AbiDirectoryEntryKind (+10 more)

### Community 171 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 172 - "normalize_inline_text"
Cohesion: 0.20
Nodes (8): normalize_inline_text(), Item, Iterator, Range, TextByteChunks, TextByteChunks<'a>, TextByteChunkSource, RopeChunks

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 175 - "user/browser.rs"
Cohesion: 0.29
Nodes (8): buffer_lines(), buffer_lines_include_current_url_when_present(), input_hint(), package(), package_exports_browser_open_command(), Option, String, Vec

### Community 176 - "TerminalDimensions"
Cohesion: 0.25
Nodes (4): From, TerminalDimensions, WindowSize, Dimensions

### Community 178 - "`user`"
Cohesion: 0.17
Nodes (11): Building the user package, Change shared UI options, font, and language defaults, Change the default theme, Change theme colors, Changing the theme and font, Making configuration changes, Other runtime-backed config files, Project discovery: `workspace.rs` (+3 more)

### Community 179 - "shell/acp.rs"
Cohesion: 0.10
Nodes (29): acp_complete_slash(), acp_permission_approve(), acp_permission_deny(), acp_pick_mode(), acp_picker_entries(), acp_picker_entry(), acp_slash_completion_query(), AcpUiAction (+21 more)

### Community 180 - "spawn_terminal_reader"
Cohesion: 0.33
Nodes (5): AsyncRead, spawn_terminal_reader(), CreateTerminalRequest, CreateTerminalResponse, Unpin

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

### Community 186 - "panic_payload_message"
Cohesion: 0.50
Nodes (4): panic_payload_message(), Any, Box, Send

### Community 187 - "Workspace Issues Plugin"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Issues Plugin

### Community 188 - "Workspace Switching and Marks"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Switching and Marks

### Community 189 - "Workspace Dashboard Worktree Remove + Picker Extra Keybinds"
Cohesion: 0.22
Nodes (8): Further Notes, Implementation Decisions, Out of Scope, Problem Statement, Solution, Testing Decisions, User Stories, Workspace Dashboard Worktree Remove + Picker Extra Keybinds

### Community 190 - "load_user_library"
Cohesion: 0.32
Nodes (5): load_user_library(), Arc, Instant, Self, StartupTrace

### Community 191 - ".oil_directory_sections"
Cohesion: 0.25
Nodes (5): DirectoryEntry, GitStatusSnapshot, OilSortMode, Path, SectionTree

### Community 192 - "Database Explorer PRD"
Cohesion: 0.25
Nodes (7): 1. Executive Summary, 2. User Experience & Functionality, 3. AI System Requirements (If Applicable), 4. Technical Specifications, 5. Risks & Roadmap, Database Explorer PRD, Open Design Decisions

### Community 193 - "MulticursorState"
Cohesion: 0.60
Nodes (5): advance_point_by_text(), multicursor_selection_offsets(), multicursor_cursor_points(), multicursor_ranges_for_line(), MulticursorState

### Community 194 - "bash.rs"
Cohesion: 0.43
Nodes (7): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 195 - "clojure.rs"
Cohesion: 0.43
Nodes (7): clojure_package_auto_attaches_all_extensions(), clojure_package_metadata(), clojure_package_no_formatter(), clojure_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 196 - "elixir.rs"
Cohesion: 0.43
Nodes (7): elixir_package_auto_attaches_all_extensions(), elixir_package_metadata(), elixir_package_registers_formatter(), elixir_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 197 - "graphql.rs"
Cohesion: 0.43
Nodes (7): graphql_package_auto_attaches_all_extensions(), graphql_package_metadata(), graphql_package_registers_formatter(), graphql_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 198 - "hcl.rs"
Cohesion: 0.43
Nodes (7): hcl_package_auto_attaches_all_extensions(), hcl_package_metadata(), hcl_package_no_formatter(), hcl_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 199 - "java.rs"
Cohesion: 0.43
Nodes (7): java_package_auto_attaches_all_extensions(), java_package_metadata(), java_package_registers_formatter(), java_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 200 - "kotlin.rs"
Cohesion: 0.43
Nodes (7): kotlin_package_auto_attaches_all_extensions(), kotlin_package_metadata(), kotlin_package_registers_formatter(), kotlin_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

### Community 201 - "latex.rs"
Cohesion: 0.43
Nodes (7): latex_package_auto_attaches_all_extensions(), latex_package_metadata(), latex_package_registers_formatter(), latex_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language()

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

### Community 214 - ".oil_directory_sections"
Cohesion: 0.40
Nodes (3): DirectoryEntry, OilSortMode, SectionTree

### Community 215 - "AbiOilKeyAction"
Cohesion: 0.60
Nodes (3): AbiOilKeyAction, OilKeyAction, OilKeyAction

### Community 216 - "Language"
Cohesion: 0.33
Nodes (5): Issues, Language, Language servers, Volt, Workspace

### Community 217 - "choose_permission_outcome"
Cohesion: 0.40
Nodes (6): choose_permission_outcome(), format_permission_option_kind(), PendingPermission, PermissionOption, PermissionOptionKind, RequestPermissionOutcome

### Community 218 - "Domain Docs"
Cohesion: 0.33
Nodes (5): Before exploring, read these, Domain Docs, File structure, Flag ADR conflicts, Use the glossary's vocabulary

### Community 219 - "Issue tracker: Local Markdown"
Cohesion: 0.33
Nodes (5): Conventions, Issue tracker: Local Markdown, Wayfinding operations, When a skill says "fetch the relevant ticket", When a skill says "publish to the issue tracker"

### Community 221 - "syntax_language"
Cohesion: 0.47
Nodes (5): package(), package_auto_attaches_for_yaml_extensions(), LanguageConfiguration, syntax_language(), syntax_language_registers_yaml_grammar()

### Community 222 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 224 - "Issue Store and Create"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Store and Create, Parent, What to build

### Community 225 - "Status commands"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Parent, Status commands, What to build

### Community 226 - "Capture focused file"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Capture focused file, Parent, What to build

### Community 227 - "Async Capture on save"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Async Capture on save, Parent, What to build

### Community 228 - "Issue Board"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Board, Parent, What to build

### Community 229 - "Place and jump to code"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Parent, Place and jump to code, What to build

### Community 230 - "Open Issue from Code Reference"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Open Issue from Code Reference, Parent, What to build

### Community 231 - "Issue Scan"
Cohesion: 0.40
Nodes (4): Acceptance criteria, Issue Scan, Parent, What to build

### Community 232 - "01 — Ambiguous Prefix Timeout"
Cohesion: 0.40
Nodes (4): 01 — Ambiguous Prefix Timeout, Acceptance criteria, Answer, Parent

### Community 233 - "03 — Cycle Project Workspaces"
Cohesion: 0.40
Nodes (4): 03 — Cycle Project Workspaces, Acceptance criteria, Answer, Parent

### Community 234 - "04 — Mark List Management"
Cohesion: 0.40
Nodes (4): 04 — Mark List Management, Acceptance criteria, Answer, Parent

### Community 235 - "05 — Marked Workspace Slot Jumps"
Cohesion: 0.40
Nodes (4): 05 — Marked Workspace Slot Jumps, Acceptance criteria, Answer, Parent

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 237 - "syntax_language"
Cohesion: 0.60
Nodes (4): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), LanguageConfiguration, syntax_language()

### Community 239 - "02 — Minor Mode Keybinding Precedence"
Cohesion: 0.50
Nodes (3): 02 — Minor Mode Keybinding Precedence, Acceptance criteria, Parent

### Community 240 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

## Knowledge Gaps
- **195 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+190 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **11 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `register_shell_hooks`, `shell/tests.rs`, `Result`, `String`, `.path`, `Result`, `AcpEvent`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `treesitter_install.rs`, `Option`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `shell/mod.rs`, `ShellUiState`, `.new`, `shell/acp.rs`, `String`, `sync_quickfix_popup_buffer`, `.len`, `temp_dir`, `Result`, `state_with_user_library`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `AcpManager`, `shell/git.rs`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `Option`, `UserLibrary`, `main`, `git_root`, `shell/picker.rs`, `BufferId`, `.new`, `BufferId`, `GitSummaryState`?**
  _High betweenness centrality (0.133) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `register_shell_hooks`, `Result`, `render.rs`, `TextBuffer`, `Option`, `shell/pdf.rs`, `shell/mod.rs`, `ShellUiState`, `.new`, `shell/acp.rs`, `sync_quickfix_popup_buffer`, `render_buffer_with_view_state`, `.len`, `Result`, `directory.rs`, `shell/terminal.rs`, `.new`, `LineCharMap`, `shell/git.rs`, `Option`, `UserLibrary`, `git_root`, `shell/picker.rs`, `Vec`, `GitSummaryState`, `Option`?**
  _High betweenness centrality (0.087) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `oil.rs`, `user/lib.rs`, `sdk/src/lib.rs`, `calculator.rs`, `DbBrowserContext`, `lsp.rs`, `AcpPickerItemSpec`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `load`, `GhostTextContext`, `UserLibraryModule`, `user/browser.rs`, `.new`, `HeaderlineTestUserLibrary`, `PluginBuffer`, `Self`, `bash.rs`, `clojure.rs`, `elixir.rs`, `graphql.rs`, `.new`, `hcl.rs`, `java.rs`, `kotlin.rs`, `latex.rs`, `lua.rs`, `nix.rs`, `perl.rs`, `php.rs`, `proto.rs`, `r.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `xml.rs`, `PickerItemSpec`, `editor-plugin-host/src/lib.rs`, `syntax_language`, `package`, `main`, `common.rs`, `debug_adapters`, `package`, `PluginKeyBinding`, `PluginCommand`, `workspace.rs`?**
  _High betweenness centrality (0.080) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _195 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `register_shell_hooks` be split into smaller, more focused modules?**
  _Cohesion score 0.05602267359470949 - nodes in this community are weakly interconnected._
- **Should `LspClientError` be split into smaller, more focused modules?**
  _Cohesion score 0.08151609553478713 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.02857142857142857 - nodes in this community are weakly interconnected._