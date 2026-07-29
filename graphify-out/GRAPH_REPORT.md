# Graph Report - volt  (2026-07-29)

## Corpus Check
- 223 files · ~562,851 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 8945 nodes · 36607 edges · 300 communities (274 shown, 26 thin omitted)
- Extraction: 92% EXTRACTED · 8% INFERRED · 0% AMBIGUOUS · INFERRED: 3027 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Graph Freshness
- Built from commit: `86ecb4a5`
- Run `git rev-parse HEAD` and compare to check if the graph is stale.
- Run `graphify update .` after code changes (no API cost).

## Community Hubs (Navigation)
- String
- LspClientError
- shell/tests.rs
- src/tests.rs
- Result
- user/lib.rs
- editor-syntax/src/lib.rs
- String
- Self
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
- editor-db/src/lib.rs
- .new
- window_effects.rs
- command_stream.rs
- editor-render/src/lib.rs
- HoverOverlay
- Theme
- load_font_set_with_mode
- EditorRuntime
- Path
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
- register_shell_hooks
- draw_primary_ligature_texture_if_available
- AbiGitFeatureSpec
- shell/mod.rs
- AbiContextHelpSpec
- TextPoint
- HeaderlineTestUserLibrary
- shell_ui_mut
- Vec
- Section
- render_buffer_with_view_state
- .is_empty
- Result
- client.rs
- RVec
- Option
- Result
- Self
- spawn_reader_thread
- AbiKeymapConfig
- String
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
- FontSet
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
- active_git_status_command_context
- PluginBuffer
- submit_input_buffer
- WorkspaceConfigurationValue
- shell/browser.rs
- RString
- LspInlineCompletionItem
- common.rs
- Option
- editor-lsp/src/lib.rs
- PickerSession
- editor-picker/src/lib.rs
- execute_oil_action
- PickerOverlay
- BufferId
- shell/git.rs
- browser_host.rs
- resolve_picker_extra
- PluginKeyBinding
- AbiSectionTree
- String
- PluginCommand
- DbService
- process_supervisor.rs
- Vec
- DbEngine
- String
- Vec
- GitSummaryState
- statusline.rs
- PickerItem
- AbiGitStatusSnapshot
- JobError
- editor-terminal/src/lib.rs
- user/config.rs
- oil.rs
- key_sequence.rs
- Self
- sqlite_query_execution_and_schema_cache_work
- treesittercontext_ghosttext.rs
- oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration
- TerminalCursorSnapshot
- AbiPickerTruncateStrategy
- DynamicUserLibrary
- AcpEvent
- CommandLineOverlay
- corpus_inventory.rs
- .refresh_pdf_view
- JobSpec
- ShellConfig
- standalone_user_manifest.rs
- editor-icons/src/lib.rs
- Option
- .new
- Instant
- load_user_library
- VimActionContext
- DbBrowserContext
- lsp.rs
- .oil_directory_sections
- LspLogEntry
- picker_items
- config_root_dir_from_exe_dir
- Copilot instructions for `volt`
- .path
- .new
- LspLocation
- treesittercontext_shared.rs
- ServiceRegistry
- aligned_indent_column
- String
- user/terminal.rs
- build_output.rs
- build_headerline_lines
- OilDefaultsSection
- ancestor_contexts_for_cursor
- .next_token
- JobResult
- choose_permission_outcome
- user/browser.rs
- UserLibrary
- .oil_keybindings
- `user`
- shell/acp.rs
- spawn_terminal_reader
- .oil_directory_sections
- Quickfix List PRD
- User-Owned Extension Surfaces Migration PRD
- Building locally
- Vec
- syntax_languages
- .acp_client_by_id
- Option
- .git_command_for_chord
- package
- .autocomplete_providers
- Database Explorer PRD
- .browser_feature_spec
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
- debug_adapters
- package
- Agent skills
- 0001-csharp-ls-one-session-per-solution.md
- triage-labels.md

## God Nodes (most connected - your core abstractions)
1. `EditorRuntime` - 741 edges
2. `ShellBuffer` - 362 edges
3. `shell_ui_mut()` - 330 edges
4. `register_shell_hooks()` - 256 edges
5. `shell_ui()` - 218 edges
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

## Communities (300 total, 26 thin omitted)

### Community 0 - "String"
Cohesion: 0.03
Nodes (135): acp_decode_image(), active_lsp_buffer_context(), active_project_workspace_root(), active_shell_buffer_path(), active_shell_workspace_id(), active_workspace_root(), apply_copilot_auth_notification(), apply_pending_lsp_state() (+127 more)

### Community 1 - "LspClientError"
Cohesion: 0.07
Nodes (38): BufRead, ClientCapabilities, client_capabilities(), inline_completion_params(), is_copilot_server(), is_csharp_metadata_uri(), lsp_formatting_options(), LspClientError (+30 more)

### Community 2 - "shell/tests.rs"
Cohesion: 0.03
Nodes (68): cycle_hover_provider(), ascii_ligature_byte_ranges_isolate_inline_operator_in_mixed_text(), berkeley_mono_font(), berkeley_mono_ligature_test_assets(), browser_buffer_submit_tracks_requested_navigation(), browser_host_focus_parent_event_returns_to_normal_mode(), browser_host_open_devtools_event_is_ignored_without_a_live_webview(), browser_location_updates_rename_buffer_with_current_url() (+60 more)

### Community 3 - "src/tests.rs"
Cohesion: 0.14
Nodes (63): autocomplete_closes_when_no_results_remain(), autocomplete_opens_while_typing_buffer_tokens(), autocomplete_trigger_updates_and_accepts_buffer_tokens(), ctrl_n_and_ctrl_p_cycle_autocomplete_without_inserting_text(), ctrl_space_triggers_autocomplete_without_inserting_space(), file_buffer_reload_refreshes_clean_open_buffers_after_disk_change(), file_buffer_reload_waits_for_dirty_buffers_to_become_clean(), flush_picker_searches() (+55 more)

### Community 4 - "Result"
Cohesion: 0.04
Nodes (76): Display, Error, From, ShellError, browser_sync_plan(), Instant, clear_key_sequence(), active_lsp_workspace_loaded() (+68 more)

### Community 5 - "user/lib.rs"
Cohesion: 0.03
Nodes (95): bundled_highlight_query(), capture_requires_theme_token(), debug_adapters(), every_installed_grammar_highlight_query_compiles(), exported_acp_client_by_id(), exported_acp_clients(), exported_acp_picker_items(), exported_autocomplete_providers() (+87 more)

### Community 6 - "editor-syntax/src/lib.rs"
Cohesion: 0.09
Nodes (73): additional_highlight_languages_merge_spans(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), bundled_folds_query_compiles_for_rust(), bundled_html_highlights_query_compiles(), bundled_injections_query_compiles_for_rust(), bundled_locals_query_compiles_for_rust(), bundled_optional_query_asset_ignores_stale_installed_query() (+65 more)

### Community 7 - "String"
Cohesion: 0.09
Nodes (51): ActiveBufferEventContext, begin_oil_worktree_request(), ensure_no_rebase_in_progress(), fetch_git_prune(), fetch_git_pushremote(), fetch_git_upstream(), finish_oil_worktree_branch_selection(), git_branch_list() (+43 more)

### Community 8 - "Self"
Cohesion: 0.05
Nodes (30): apply_vim_search_result(), build_keydown_chord(), ChordModifiers, format_undo_snapshot_diff(), keycode_name_token(), keydown_chord(), keydown_chord_token(), KeydownChordToken (+22 more)

### Community 9 - "render.rs"
Cohesion: 0.05
Nodes (98): advance_point_by_text(), is_zero_width_display_character(), multicursor_selection_offsets(), acp_slice_chars(), adjusted_contextual_ligature_pixel_size(), ascii_ligature_byte_ranges_with_face(), autocomplete_preview_lines(), autocomplete_visible_start() (+90 more)

### Community 10 - "Result"
Cohesion: 0.14
Nodes (26): AcpClientConfig, acp_cycle_mode(), acp_disconnect(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_permission_picker_submitted(), acp_pick_session() (+18 more)

### Community 11 - "PluginPackage"
Cohesion: 0.06
Nodes (43): file_open_package(), package(), package(), package(), LanguageConfiguration, syntax_language(), package(), LanguageConfiguration (+35 more)

### Community 12 - "sdk/src/lib.rs"
Cohesion: 0.04
Nodes (53): AcpClient, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec, ContextHelpEntry, ContextHelpSpec, DbFeatureSpec, GhostTextLine (+45 more)

### Community 13 - "TextBuffer"
Cohesion: 0.03
Nodes (78): around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending(), EditRecord (+70 more)

### Community 14 - "LanguageServerSpec"
Cohesion: 0.12
Nodes (13): document_language_id_for_extension(), document_language_id_for_glob(), document_language_id_for_path(), language_server_spec_exposes_workspace_configuration_builders(), LanguageServerRootStrategy, LanguageServerSpec, BTreeMap, Into (+5 more)

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
Nodes (32): autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics(), calculator_package_binds_ctrl_c_ctrl_c() (+24 more)

### Community 24 - "editor-db/src/lib.rs"
Cohesion: 0.08
Nodes (39): connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch(), default_db_browser_items(), default_db_browser_line(), escape_bracket() (+31 more)

### Community 25 - ".new"
Cohesion: 0.16
Nodes (22): browser_host_event_for_ipc(), browser_navigation_retry_required(), BrowserHostEvent, BrowserHostService, BrowserInstance, BrowserLocationUpdate, DesktopBrowserHostService, optional_non_empty_text() (+14 more)

### Community 26 - "window_effects.rs"
Cohesion: 0.11
Nodes (46): apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur(), clear_window_blur() (+38 more)

### Community 27 - "command_stream.rs"
Cohesion: 0.08
Nodes (69): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), cargo_toml_wins_over_other_markers(), continue_streamed_command_popup(), detect_build_command(), detects_cargo_toml(), detects_csproj() (+61 more)

### Community 28 - "editor-render/src/lib.rs"
Cohesion: 0.10
Nodes (45): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+37 more)

### Community 29 - "HoverOverlay"
Cohesion: 0.07
Nodes (39): autocomplete_entries(), autocomplete_score(), AutocompleteProviderKind, buffer_autocomplete_entries(), db_autocomplete_entries(), lsp_autocomplete_entries(), lsp_kind_icon(), manual_autocomplete_entries() (+31 more)

### Community 30 - "Theme"
Cohesion: 0.09
Nodes (24): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), BTreeMap, Display, Error (+16 more)

### Community 31 - "load_font_set_with_mode"
Cohesion: 0.08
Nodes (30): EmojiFont, FontSet<'ttf>, FontSetInit, IconFont, load_deferred_emoji_font(), load_emoji_font(), load_font_set_with_mode(), load_icon_font() (+22 more)

### Community 32 - "EditorRuntime"
Cohesion: 0.09
Nodes (81): EditorRuntime, Default, cancel_git_commit_buffer(), checkout_git_branch(), cherry_pick_git_commit(), cherry_pick_git_commit_no_commit(), commit_git_buffer(), create_git_worktree() (+73 more)

### Community 33 - "Path"
Cohesion: 0.09
Nodes (19): asset_path_from_parts(), command_failure_message(), ensure_cloned_grammar_dir_exists(), GrammarSource, InstallCommandSpec, io_error(), LanguageInstallPlan, remove_compiler_sidecar_artifacts() (+11 more)

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
Nodes (128): Cow, write_system_clipboard(), yank_from_clipboard_text(), yank_to_clipboard_text(), acp_tool_call_from_partial_update(), active_shell_buffer_id(), active_shell_buffer_is_terminal(), active_shell_buffer_mut() (+120 more)

### Community 42 - "UserLibraryModule"
Cohesion: 0.11
Nodes (16): exported_pdf_open_mode(), PdfOpenMode, AbiIconFontSymbol, AbiOilKeybindings, AbiPdfOpenMode, AbiWorkspaceRoot, IconFontSymbol, OilKeybindings (+8 more)

### Community 43 - "clipboard.rs"
Cohesion: 0.19
Nodes (12): ClipboardUtil, ClipboardContext, configure_background_command(), read_system_clipboard(), register_clipboard_context(), Command, FnOnce, Option (+4 more)

### Community 44 - "String"
Cohesion: 0.06
Nodes (43): apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), configure_lsp_command(), explicit_windows_env_value(), hover_marked_string(), hover_marked_string_markdown_text(), hover_parser_formats_marked_string_language_blocks_as_markdown() (+35 more)

### Community 45 - "register_shell_hooks"
Cohesion: 0.03
Nodes (96): BufferKind, browser_state_for_kind(), default_vim_target(), accept_autocomplete(), active_buffer_event_context(), active_lsp_code_action_range(), active_runtime_buffer(), active_shell_buffer_has_input() (+88 more)

### Community 46 - "draw_primary_ligature_texture_if_available"
Cohesion: 0.12
Nodes (20): Canvas, cached_primary_text_runs(), column_to_relative_byte_offset(), draw_primary_ligature_texture_if_available(), draw_text_texture_with_cache(), draw_undercurl_canvas(), fill_rounded_rect_canvas(), LigatureShapeCacheEntry (+12 more)

### Community 47 - "AbiGitFeatureSpec"
Cohesion: 0.15
Nodes (12): GitCommandBinding, GitPrefixBinding, AbiGitCommandBinding, AbiGitFeatureSpec, AbiGitPrefixBinding, AbiGitStatusPrefix, GitCommandBinding, GitFeatureSpec (+4 more)

### Community 48 - "shell/mod.rs"
Cohesion: 0.02
Nodes (207): DbAutocompleteCandidate, ActiveLspBufferContext, WorkspaceId, acp_build_output_lines(), acp_build_plan_lines(), acp_icon_segment(), acp_multiline_text_lines(), acp_padding_prefix() (+199 more)

### Community 49 - "AbiContextHelpSpec"
Cohesion: 0.06
Nodes (31): exported_browser_feature_spec(), exported_context_help_specs(), exported_db_feature_spec(), exported_git_feature_spec(), exported_oil_feature_spec(), exported_oil_keybindings(), exported_terminal_feature_spec(), BrowserFeatureSpec (+23 more)

### Community 50 - "TextPoint"
Cohesion: 0.11
Nodes (7): advance_point_by_text(), is_inline_whitespace(), is_sentence_closer(), Fn, Selection, TextPoint, TextRange

### Community 51 - "HeaderlineTestUserLibrary"
Cohesion: 0.03
Nodes (57): AtomicUsize, load_font_set(), active_input_prompt_text(), CommandLog, comment_toggle_styles_cover_all_shipped_syntax_languages(), compose_emoji_surface_rasterizes_simple_emoji(), compose_ligature_surface_uses_grayscale_glyph_coverage(), contextual_ligature_raster_size_never_upscales_smaller_substitute_glyphs() (+49 more)

### Community 52 - "shell_ui_mut"
Cohesion: 0.11
Nodes (33): cycle_runtime_pane(), queue_workspace_readme_open(), queue_workspace_syntax_prewarm(), shell_ui_mut(), split_runtime_pane(), accept_autocomplete_avoids_double_dot_when_lsp_insert_includes_trigger(), accept_autocomplete_uses_lsp_text_edit_range_covering_trigger(), browser_open_buffer_command_uses_existing_split_pane() (+25 more)

### Community 53 - "Vec"
Cohesion: 0.10
Nodes (11): EventLog, format_micros_as_millis(), LspState, AutocompleteProvider, ContextHelpSpec, GitStatusSnapshot, HoverProvider, StatuslineContext (+3 more)

### Community 54 - "Section"
Cohesion: 0.14
Nodes (15): render_lines_respects_collapsed_state(), render_section(), BTreeSet, Into, Option, Self, String, Vec (+7 more)

### Community 55 - "render_buffer_with_view_state"
Cohesion: 0.12
Nodes (81): render_browser_buffer_body(), Color, adjust_color(), blend_color(), DrawTarget, FpsOverlaySnapshot, is_dark_color(), Color (+73 more)

### Community 56 - ".is_empty"
Cohesion: 0.04
Nodes (42): acp_pane_content_rows(), acp_pane_cursor_visual_row(), acp_pane_line_index_for_visual_row(), acp_pane_max_scroll_visual_row(), acp_pane_total_visual_rows(), acp_rendered_line_row_count(), AcpPaneState, byte_index_for_char_column() (+34 more)

### Community 57 - "Result"
Cohesion: 0.14
Nodes (17): Compat, build_tokio_runtime(), connect_sql_server(), DbExecutionOutput, execute_postgres(), execute_sql_server(), execute_sqlite(), initialize_native_keyring() (+9 more)

### Community 58 - "client.rs"
Cohesion: 0.04
Nodes (98): active_parameter_label(), char_to_byte_offset(), client_capabilities_enable_window_work_done_progress_and_show_document(), completion_documentation(), completion_level_for_message(), completion_parser_handles_lists_and_docs(), completion_parser_prefers_text_edit_over_insert_text_and_keeps_range(), completion_parser_reads_insert_replace_edit_replace_range() (+90 more)

### Community 59 - "RVec"
Cohesion: 0.14
Nodes (13): AbiDebugAdapterSpec, AbiHoverProvider, AbiHoverProviderTopic, AbiTerminalConfig, DebugAdapterSpec, HoverProvider, HoverProviderTopic, DebugAdapterSpec (+5 more)

### Community 60 - "Option"
Cohesion: 0.07
Nodes (59): parse_log_oneline(), build_git_fringe_snapshot(), build_git_summary_snapshot(), command_output_transcript(), git_branch_merge(), git_branch_push_remote(), git_branch_remote(), git_command_output_background() (+51 more)

### Community 61 - "Result"
Cohesion: 0.08
Nodes (91): default_error_log_path(), format_current_line_indent(), shell_buffer(), shell_buffer_mut(), syntax_registry_mut(), acp_escape_from_insert_keeps_input_cursor_position(), acp_input_field_cw_enters_insert_mode(), acp_input_field_dd_deletes_current_line() (+83 more)

### Community 62 - "Self"
Cohesion: 0.07
Nodes (12): AcpActionSpec, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, AcpPickerOption, DbActionSpec, DbBrowserItemContext, DbBrowserItemKind (+4 more)

### Community 63 - "spawn_reader_thread"
Cohesion: 0.14
Nodes (21): ChildStdin, diagnostic_matches_request_range(), launch_summary(), record_notification(), record_transport_entry(), record_transport_event(), record_transport_message(), AtomicBool (+13 more)

### Community 64 - "AbiKeymapConfig"
Cohesion: 0.10
Nodes (17): exported_keymap_config(), exported_ligature_config(), exported_pane_config(), KeymapConfig, LigatureConfig, PaneConfig, config(), PaneConfig (+9 more)

### Community 65 - "String"
Cohesion: 0.06
Nodes (113): active_runtime_popup(), ctrl_mod(), install_mark_list_state_for_test(), open_workspace_from_project(), shell_ui(), acp_paste_code_with_inline_double_slash_comments_closes_slash_picker(), acp_slash_picker_backspace_can_delete_leading_slash(), acp_slash_picker_text_input_updates_acp_input() (+105 more)

### Community 66 - "SyntaxRegistry"
Cohesion: 0.08
Nodes (51): TextEdit, apply_text_edits_to_span(), buffer_text_for_byte_range(), capture_requires_theme_token(), changed_range_windows(), collect_injection_regions(), compile_query_source(), create_parser() (+43 more)

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
Nodes (67): Box, Error, Path, Result, setup_standalone_user_repository(), setup_standalone_user_repository_writes_gitignore_and_initializes_git(), feature_spec(), DbFeatureSpec (+59 more)

### Community 71 - "Option"
Cohesion: 0.06
Nodes (30): append_query_source(), CaptureThemeMapping, default_query_asset_root(), DeferredQuery, GrammarRecompileFailure, GrammarRecompileReport, LanguageConfiguration, LanguageLoader (+22 more)

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
Cohesion: 0.06
Nodes (71): apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+63 more)

### Community 77 - "editor-jobs/src/lib.rs"
Cohesion: 0.14
Nodes (40): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), command_candidate_names(), configure_background_command(), default_process_supervisor_executable(), environment_value(), explicit_windows_env_value() (+32 more)

### Community 78 - "workspace_search.rs"
Cohesion: 0.08
Nodes (66): LspWorkspaceDiagnostic, PickerEntry, search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), apply_lsp_code_action(), apply_lsp_document_edit() (+58 more)

### Community 79 - "shell/terminal.rs"
Cohesion: 0.12
Nodes (41): active_terminal_dimensions(), box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, draw_box_drawing_cell() (+33 more)

### Community 80 - "User Packages"
Cohesion: 0.04
Nodes (46): Actions, Adding a Build Command for a New Language, Adding a Command to an Existing Plugin, Adding Language Support, Adding YAML config to your own plugin, Adding Your Own Provider, Architecture Overview, Autocomplete Providers (+38 more)

### Community 81 - "PickerItemSpec"
Cohesion: 0.07
Nodes (58): workspace_picker_item(), acp_client_picker_items(), buffer_close_picker_items(), buffer_picker_detail(), buffer_picker_items(), buffer_picker_label(), buffer_picker_shows_file_name_first_and_keeps_path_search(), command_picker_items() (+50 more)

### Community 82 - "AcpManager"
Cohesion: 0.15
Nodes (10): acp_connected(), acp_open_permission_request(), acp_permission_picker_closed(), AcpManager, AcpPendingPermissionUi, drain_acp_event_batch(), drain_acp_event_batch_limits_per_frame_work(), open_permission_picker() (+2 more)

### Community 83 - "FontSet"
Cohesion: 0.12
Nodes (33): RenderColor, Self, TextStyle, FontSet, alpha_bitmap_surface(), cached_emoji_layout(), CachedLigatureGlyphPlacement, CachedLigatureLayout (+25 more)

### Community 84 - ".from"
Cohesion: 0.07
Nodes (33): main(), lsp_diagnostic_scope_keeps_active_workspace_buffers_without_root(), GhostTextLine, GhostTextLine, main(), exported_ghost_text_lines(), exported_language_servers(), GhostTextLine (+25 more)

### Community 85 - "LanguageServerRegistry"
Cohesion: 0.13
Nodes (18): directory_contains_extension(), directory_matches_root_marker(), find_root_for_path(), find_root_for_path_matching_marker(), LanguageServerRegistry, LspError, normalize_optional_string(), path_is_solution() (+10 more)

### Community 86 - "volt/src/main.rs"
Cohesion: 0.10
Nodes (30): builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), catch_unwind_silently(), dynamic_user_library_can_wrap_exported_module(), LaunchMode, LaunchOptions, panic_payload_message(), parse_launch_options(), parse_launch_options_accepts_fps_overlay() (+22 more)

### Community 87 - ".new"
Cohesion: 0.05
Nodes (94): buffer_footer_layout(), acp_multiline_text_lines_strip_carriage_returns(), acp_nonleading_double_slash_does_not_open_slash_picker(), acp_output_scroll_reaches_wrapped_tail(), acp_plan_entries_normalize_completed_prefix_when_later_step_is_active(), acp_plan_entries_normalize_completed_prefix_without_active_step(), acp_plan_entries_populate_static_plan_pane(), acp_plan_height_caps_wrapped_content_at_ten_rows() (+86 more)

### Community 88 - "draw_diagnostic_underlines_for_segment"
Cohesion: 0.11
Nodes (28): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+20 more)

### Community 89 - ".new"
Cohesion: 0.11
Nodes (36): apply_git_view(), diff_git_dwim(), git_args_with_no_pager(), git_commit_message(), git_log_args(), git_status_diff_staged_command(), git_status_diff_unstaged_command(), git_status_log_all_branches_command() (+28 more)

### Community 90 - "main"
Cohesion: 0.13
Nodes (17): bootstrap(), HostBootstrap, command_palette_items(), main(), DebugAdapterSpec, Error, LanguageConfiguration, LanguageServerSpec (+9 more)

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

### Community 96 - "active_git_status_command_context"
Cohesion: 0.11
Nodes (33): active_git_status_command_context(), cherry_pick_apply_at_point_or_picker(), cherry_pick_commit_at_point_or_picker(), diff_git_commit_at_point(), diff_git_stash_at_point(), git_action_detail(), git_commit_at_point(), git_commit_list() (+25 more)

### Community 97 - "PluginBuffer"
Cohesion: 0.08
Nodes (6): PickerKeybindingContext, PluginBuffer, PluginBufferSection, PluginBufferSections, PluginBufferSectionUpdate, RVec

### Community 98 - "submit_input_buffer"
Cohesion: 0.12
Nodes (26): activate_db_browser_line(), apply_db_browser_view(), buffer_is_command_output(), buffer_is_db_browser(), buffer_is_db_connect(), buffer_is_db_query(), db_service(), db_service_mut() (+18 more)

### Community 99 - "WorkspaceConfigurationValue"
Cohesion: 0.14
Nodes (9): normalize_unique_entries(), From, I, Number, T, WorkspaceConfigurationValue, K, abi_language_server_spec_round_trips_workspace_configuration() (+1 more)

### Community 100 - "shell/browser.rs"
Cohesion: 0.11
Nodes (41): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len() (+33 more)

### Community 101 - "RString"
Cohesion: 0.11
Nodes (17): AbiAcpClient, AbiColor, AbiStringPair, AbiTheme, AbiThemeOption, AbiThemeOptionEntry, AbiThemeToken, AcpClient (+9 more)

### Community 102 - "LspInlineCompletionItem"
Cohesion: 0.10
Nodes (28): close_buffer_keeps_session_alive_for_next_file(), diagnostics_parser_maps_lsp_fields(), file_uri_roundtrip_handles_windows_paths(), file_uri_to_path(), LspClientState, LspInlineCompletionItem, parse_inline_completion_item(), parse_inline_completion_response() (+20 more)

### Community 103 - "common.rs"
Cohesion: 0.10
Nodes (28): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), LanguageConfiguration, syntax_language() (+20 more)

### Community 104 - "Option"
Cohesion: 0.13
Nodes (5): LanguageServerSession, AsRef, Option, WorkspaceConfiguration, WorkspaceConfigurationValue

### Community 105 - "editor-lsp/src/lib.rs"
Cohesion: 0.21
Nodes (28): Client, csharp_language_server(), dev_extension_server(), dockerfile_language_server(), must(), prepare_session_for_path_reports_missing_activation_markers_for_explicit_server(), prepare_sessions_for_extension_returns_all_matching_servers(), prepare_sessions_for_path_requires_activation_markers_when_declared() (+20 more)

### Community 106 - "PickerSession"
Cohesion: 0.14
Nodes (6): PickerResultOrder, PickerSession, Vec, selection_skips_divider_rows(), selection_wraps_across_match_list(), source_order_preserves_input_order()

### Community 107 - "editor-picker/src/lib.rs"
Cohesion: 0.18
Nodes (17): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+9 more)

### Community 108 - "execute_oil_action"
Cohesion: 0.15
Nodes (18): active_directory_root(), buffer_is_directory(), directory_yank_for_range(), ensure_directory_buffer(), execute_oil_action(), handle_directory_chord(), handle_directory_keydown_chord(), oil_default_root() (+10 more)

### Community 109 - "PickerOverlay"
Cohesion: 0.09
Nodes (38): PickerOverlay, ShellTestUserLibrary, buffer_close_confirm_overlay(), buffer_picker_preview(), ensure_picker_keybindings(), message_picker_overlay(), picker_action_from_spec(), picker_overlay() (+30 more)

### Community 110 - "BufferId"
Cohesion: 0.13
Nodes (41): active_and_secondary_buffer_ids(), configure_file_buffer(), file_reload_notifications_reload_hidden_buffers_without_focus_changes(), file_reload_notifications_target_only_matching_buffers(), file_reload_notifications_wait_for_dirty_buffers_to_become_clean(), git_push_upstream_streams_into_popup_buffer_and_refreshes_status(), git_status_buffer_supports_first_commit_on_fresh_repo(), git_status_ctrl_v_visual_s_stages_selected_items() (+33 more)

### Community 111 - "shell/git.rs"
Cohesion: 0.10
Nodes (36): apply_git_status_snapshot(), git_line_is_untracked(), git_status_action_targets(), git_status_checkout_file_command(), git_status_command_name(), git_status_delete_target_for_line(), git_status_delete_targets(), git_status_diff_paths_command() (+28 more)

### Community 112 - "browser_host.rs"
Cohesion: 0.11
Nodes (18): allow_browser_drag_drop(), browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+10 more)

### Community 113 - "resolve_picker_extra"
Cohesion: 0.12
Nodes (16): create_row_selection_still_yields_defined_context_for_command_noop(), empty_selection_still_yields_defined_context_for_command_noop(), matching_extra_resolves_command_close_and_selected_context(), matching_extra_snapshots_exportable_quickfix_rows(), non_extra_chord_falls_through_for_shared_popup_bindings(), PickerExportableRow, PickerExtraDispatch, PickerExtraKeybind (+8 more)

### Community 114 - "PluginKeyBinding"
Cohesion: 0.12
Nodes (23): plugin_vim_mode_matches(), plugin_key_binding_can_target_multiple_commands(), PluginKeyBinding, PluginKeymapScope, PluginVimMode, I, leader_binding(), normal_binding() (+15 more)

### Community 115 - "AbiSectionTree"
Cohesion: 0.10
Nodes (17): exported_git_status_sections(), exported_oil_defaults(), exported_oil_directory_sections(), DirectoryEntry, OilDefaults, OilSortMode, Path, SectionTree (+9 more)

### Community 116 - "String"
Cohesion: 0.10
Nodes (18): append_lines(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig, Into, IntoIterator, Item, PathBuf, Self (+10 more)

### Community 117 - "PluginCommand"
Cohesion: 0.16
Nodes (17): action_aliases(), command_line_commands_have_unique_names(), command_line_exports_commands_when_enabled(), commands(), enabled(), hook_aliases(), picker_aliases(), Option (+9 more)

### Community 118 - "DbService"
Cohesion: 0.16
Nodes (12): DbActionOutcome, DbBrowserAction, DbBrowserBufferKind, DbBrowserBufferState, DbQueryBufferMeta, DbService, DbSession, DbSessionId (+4 more)

### Community 119 - "process_supervisor.rs"
Cohesion: 0.18
Nodes (25): ProcessSupervisionMode, configure_supervised_command(), exit_with_status(), maybe_run(), ParentProcess, parse_request(), parse_request_accepts_background_targets(), ProcessSupervisorRequest (+17 more)

### Community 120 - "Vec"
Cohesion: 0.17
Nodes (19): ColumnData, DbColumn, DbSchemaCache, DbTable, load_postgres_schema(), load_sql_server_schema(), load_sqlite_columns(), load_sqlite_schema() (+11 more)

### Community 121 - "DbEngine"
Cohesion: 0.21
Nodes (9): db_browser_action_from_spec(), DbEngine, DbHistoryEntry, DbIndex, DbSnippet, PersistedDbState, qualified_name_from_spec(), QualifiedName (+1 more)

### Community 122 - "String"
Cohesion: 0.27
Nodes (5): DbBrowserBufferView, DisabledSecretStore, String, summarize_sql(), DbBrowserItemRenderer

### Community 123 - "Vec"
Cohesion: 0.20
Nodes (27): find_paren_number_range(), format_section_line(), git_status_commit_item_spans(), git_status_commit_message_spans(), git_status_entry_item_spans(), git_status_entry_token(), git_status_entry_token_from_icon(), git_status_head_spans() (+19 more)

### Community 124 - "GitSummaryState"
Cohesion: 0.10
Nodes (18): apply_git_fringe_hunk(), git_summary_changed_tracks_head_updates(), GitFringeKind, GitFringeSnapshot, GitFringeState, GitPrefixState, GitSummarySnapshot, GitSummaryState (+10 more)

### Community 125 - "statusline.rs"
Cohesion: 0.19
Nodes (25): StatuslineSegment, acp_segment(), buffer_segment(), compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register() (+17 more)

### Community 126 - "PickerItem"
Cohesion: 0.19
Nodes (8): match_item(), PickerItem, PickerMatch, Into, Option, Self, String, picker_fringe_width_chars()

### Community 127 - "AbiGitStatusSnapshot"
Cohesion: 0.14
Nodes (12): GitStashEntry, AbiGitLogEntry, AbiGitStashEntry, AbiGitStatusSnapshot, AbiStatusEntry, GitLogEntry, GitStashEntry, GitStatusSnapshot (+4 more)

### Community 128 - "JobError"
Cohesion: 0.15
Nodes (17): compilation_runner_marks_jobs_as_compilation(), CompilationRunner, job_manager_runs_commands_and_collects_output(), JobError, JobHandle, JobManager, must(), Display (+9 more)

### Community 129 - "editor-terminal/src/lib.rs"
Cohesion: 0.08
Nodes (34): cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), map_terminal_cursor_shape(), must(), push_snapshot_line(), push_terminal_render_run(), resolve_terminal_background() (+26 more)

### Community 130 - "user/config.rs"
Cohesion: 0.21
Nodes (22): default_oil_close(), default_oil_create_git_worktree(), default_oil_cycle_sort(), default_oil_open_entry(), default_oil_open_external(), default_oil_open_horizontal_split(), default_oil_open_new_pane(), default_oil_open_parent() (+14 more)

### Community 131 - "oil.rs"
Cohesion: 0.09
Nodes (38): seti_directory_icon(), chord_action(), default_oil_keybindings_map_to_actions(), defaults(), directory_entry_display_label(), directory_entry_display_label_from_parts(), directory_sections(), feature_spec() (+30 more)

### Community 132 - "key_sequence.rs"
Cohesion: 0.23
Nodes (21): ambiguous_prefix_timeout_is_configurable(), ambiguous_short_waits_then_fires_on_timeout(), exact_chord_without_longer_prefix_fires_immediately(), incompatible_input_clears_pending_short_without_firing(), KeySequenceOptions, KeySequencePush, KeySequenceTick, longer_chord_within_window_cancels_short() (+13 more)

### Community 133 - "Self"
Cohesion: 0.06
Nodes (47): AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiCaptureThemeMapping, AbiContextHelpEntry, AbiDirectoryEntry, AbiDirectoryEntryKind, AbiGrammarSource, AbiIconFontCategory (+39 more)

### Community 134 - "sqlite_query_execution_and_schema_cache_work"
Cohesion: 0.13
Nodes (15): default_volt_state_dir(), InMemorySecretStore, load_persisted_state(), remembered_connections_store_metadata_separately_from_secret(), Arc, HashMap, Mutex, Path (+7 more)

### Community 135 - "treesittercontext_ghosttext.rs"
Cohesion: 0.20
Nodes (12): build_ghost_text_lines(), build_ghost_text_lines_includes_loop_contexts(), build_ghost_text_lines_keeps_current_line_for_block_end_contexts(), build_ghost_text_lines_prefers_inner_context_on_shared_closing_line(), build_ghost_text_lines_skips_current_line_for_non_block_multiline_contexts(), build_ghost_text_lines_skips_current_line_for_single_line_contexts(), ghost_text_lines(), is_block_closing_line() (+4 more)

### Community 136 - "oh-my-githubcopilot (OMG) - Intelligent Multi-Agent Orchestration"
Cohesion: 0.09
Nodes (21): Agent Catalog, Analysis Skills, Cancellation, Commit Protocol, Completion Rules, Delegation Rules, Execution Protocols, Global Rules (+13 more)

### Community 137 - "TerminalCursorSnapshot"
Cohesion: 0.32
Nodes (3): terminal_cursor_shape_for_input_mode(), TerminalCursorShape, TerminalCursorSnapshot

### Community 138 - "AbiPickerTruncateStrategy"
Cohesion: 0.32
Nodes (5): exported_picker_truncate_strategy(), PickerTruncateStrategy, AbiPickerTruncateStrategy, PickerTruncateStrategy, PickerTruncateStrategy

### Community 140 - "AcpEvent"
Cohesion: 0.08
Nodes (34): AvailableCommand, acp_pick_model(), AcpEvent, AcpSessionInfo, build_acp_input_hint(), command_input_hint(), config_option_is_mode(), config_option_is_model() (+26 more)

### Community 141 - "CommandLineOverlay"
Cohesion: 0.11
Nodes (10): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, InputPromptOverlay, BufferId, Into, Option, Self (+2 more)

### Community 142 - "corpus_inventory.rs"
Cohesion: 0.22
Nodes (19): collect_invalid_capture_valued_set_directives(), collect_operators(), corpus_predicate_operator_inventory_is_stable(), corpus_query_asset_root_contains_expected_languages(), corpus_set_directives_do_not_use_capture_values(), corpus_set_key_inventory_is_stable(), find_matching_paren(), query_asset_root() (+11 more)

### Community 143 - ".refresh_pdf_view"
Cohesion: 0.20
Nodes (5): reset_active_image_buffer_zoom(), Error, zoom_active_image_buffer_in(), zoom_active_image_buffer_out(), pdf_zoom_percent_from_scale()

### Community 144 - "JobSpec"
Cohesion: 0.25
Nodes (8): build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), JobKind, JobSpec, Into, IntoIterator, Item, Self

### Community 145 - "ShellConfig"
Cohesion: 0.16
Nodes (13): RenderBackend, Arc, Debug, Default, Formatter, Option, Result, Self (+5 more)

### Community 146 - "standalone_user_manifest.rs"
Cohesion: 0.33
Nodes (18): canonicalize_path(), collect_dependency_section(), collect_manifest_dependencies(), manifest_path_dependencies(), ManifestPathDependency, ManifestPathReplacement, BTreeSet, Path (+10 more)

### Community 147 - "editor-icons/src/lib.rs"
Cohesion: 0.12
Nodes (14): all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, IconFontCategory, Option, Path, String (+6 more)

### Community 148 - "Option"
Cohesion: 0.15
Nodes (7): CommandPaletteState, CompilationState, AcpClient, GitStatusPrefix, OilKeyAction, Option, TerminalState

### Community 149 - ".new"
Cohesion: 0.06
Nodes (32): CodeActionParams, code_action_params(), code_action_params_use_flattened_lsp_shape(), code_action_parser_collects_active_file_edits(), code_action_parser_tracks_command_and_resource_operations(), full_document_range(), lsp_code_action_diagnostic(), lsp_diagnostic_severity() (+24 more)

### Community 150 - "Instant"
Cohesion: 0.06
Nodes (22): ActiveTypingFrameProfile, average_duration(), DirectoryPrefixState, format_duration_ms(), frame_pacing_remaining(), git_refresh_deferred_for_typing(), KeySequenceState, pace_frame_to_120fps() (+14 more)

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
Cohesion: 0.33
Nodes (4): DirectoryEntry, OilSortMode, Path, SectionTree

### Community 156 - "LspLogEntry"
Cohesion: 0.09
Nodes (10): LspLogDirection, LspLogEntry, LspLogSnapshot, LspNotificationEntry, LspNotificationLog, LspNotificationSnapshot, LspTransportLog, notification_log_snapshot_is_bounded_and_tracks_revision() (+2 more)

### Community 157 - "picker_items"
Cohesion: 0.28
Nodes (14): acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items(), picker_items_mark_current_models() (+6 more)

### Community 158 - "config_root_dir_from_exe_dir"
Cohesion: 0.23
Nodes (15): config_root_dir(), config_root_dir_from_exe_dir(), config_root_prefers_workspace_user_directory(), config_source_files(), config_source_files_from_root(), config_source_files_include_master_and_children(), load_reads_referenced_child_files(), load_uses_defaults_when_files_are_missing() (+7 more)

### Community 159 - "Copilot instructions for `volt`"
Cohesion: 0.13
Nodes (14): Agent skills, Architecture, Build, test, and lint, caveman, Copilot instructions for `volt`, Cursor Cloud specific instructions, Domain docs, graphify (+6 more)

### Community 160 - ".path"
Cohesion: 0.23
Nodes (11): db_query_buffer_receives_sql_highlighting_without_blocking(), opened_file_receives_tree_sitter_highlighting(), opened_sql_file_survives_layout_and_syntax_refresh(), opened_toml_file_survives_layout_and_receives_tree_sitter_highlighting(), recompile_installed_tree_sitter_languages_notifies_when_no_grammars_are_installed(), resolve_default_workspace_root_falls_back_to_executable_user_dir(), resolve_default_workspace_root_prefers_existing_executable_relative_user_dir(), sync_active_buffer_layout_for_test() (+3 more)

### Community 161 - ".new"
Cohesion: 0.20
Nodes (7): Env, eval_line(), EvalResult, is_valid_ident(), Option, Self, split_assignment()

### Community 162 - "LspLocation"
Cohesion: 0.22
Nodes (3): location_from_link(), LspLocation, LocationLink

### Community 163 - "treesittercontext_shared.rs"
Cohesion: 0.36
Nodes (14): collapse_whitespace(), context_icon(), extract_control_flow_header(), extract_named_keyword(), extract_signature(), format_context_label_from_header(), ignored_context_kind(), is_conditional_kind() (+6 more)

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
Cohesion: 0.27
Nodes (11): create_symlink(), distributed_user_library_paths(), distributed_user_library_paths_points_from_out_dir_to_root_library(), install_root_library_link(), install_root_library_link_creates_or_updates_symlink(), Option, Path, PathBuf (+3 more)

### Community 169 - "build_headerline_lines"
Cohesion: 0.36
Nodes (4): build_headerline_lines(), headerline_lines(), String, Vec

### Community 170 - "OilDefaultsSection"
Cohesion: 0.28
Nodes (6): ConfigOilSortMode, default_oil_sort_mode(), OilDefaultsSection, OilSection, OilSortMode, OilDefaults

### Community 171 - "ancestor_contexts_for_cursor"
Cohesion: 0.29
Nodes (11): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), LanguageConfiguration (+3 more)

### Community 173 - "JobResult"
Cohesion: 0.20
Nodes (3): CompilationResult, JobResult, Duration

### Community 174 - "choose_permission_outcome"
Cohesion: 0.40
Nodes (6): choose_permission_outcome(), format_permission_option_kind(), PendingPermission, PermissionOption, PermissionOptionKind, RequestPermissionOutcome

### Community 175 - "user/browser.rs"
Cohesion: 0.23
Nodes (10): buffer_lines(), buffer_lines_include_current_url_when_present(), feature_spec(), input_hint(), package(), package_exports_browser_open_command(), BrowserFeatureSpec, Option (+2 more)

### Community 176 - "UserLibrary"
Cohesion: 0.11
Nodes (27): browser_buffer_layout(), browser_host_viewport_rect(), browser_viewport_rect(), BrowserBufferLayout, Rect, buffer_context_overlay_snapshot(), wrap_columns_for_width(), acp_buffer_layout() (+19 more)

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
Cohesion: 0.36
Nodes (7): AcpClientConfig, AcpSection, default_acp_clients(), default_project_search_roots(), Vec, WorkspaceRootConfig, WorkspaceSection

### Community 186 - "syntax_languages"
Cohesion: 0.60
Nodes (4): packages(), LanguageConfiguration, Vec, syntax_languages()

### Community 188 - "Option"
Cohesion: 0.03
Nodes (50): absolute_path_hint(), active_buffer_revision_key(), adjust_tag_child_indent(), apply_markdown_code_fence_syntax(), block_comment_toggle_removal_lens(), char_immediately_before(), chars_immediately_before(), closing_tag_name_after_cursor() (+42 more)

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
Cohesion: 0.24
Nodes (7): load(), load_from_root(), UserConfig, config(), KeymapConfig, config(), LigatureConfig

### Community 222 - "package"
Cohesion: 0.47
Nodes (5): open_mode(), package(), package_exports_pdf_buffer_keybindings(), package_exports_pdf_commands(), PdfOpenMode

### Community 236 - "debug_adapters"
Cohesion: 0.40
Nodes (4): debug_adapters(), package(), DebugAdapterSpec, Vec

### Community 240 - "package"
Cohesion: 0.83
Nodes (3): package(), package_exports_image_commands(), package_exports_image_keybindings()

### Community 249 - "Agent skills"
Cohesion: 0.33
Nodes (5): Agent skills, Domain docs, graphify, Issue tracker, Triage labels

## Knowledge Gaps
- **136 isolated node(s):** `StartupProfile`, `topbar`, `navToggle`, `pageSidebar`, `navLinks` (+131 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **26 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `EditorRuntime` connect `EditorRuntime` to `String`, `shell/tests.rs`, `Result`, `String`, `Self`, `Result`, `AcpEvent`, `.refresh_pdf_view`, `editor-issues/src/lib.rs`, `HookBus`, `EditorModel`, `KeymapScope`, `Instant`, `command_stream.rs`, `.path`, `shell/pdf.rs`, `ServiceRegistry`, `ShellBuffer`, `register_shell_hooks`, `shell/mod.rs`, `shell/acp.rs`, `shell_ui_mut`, `.is_empty`, `Option`, `Option`, `Result`, `String`, `String`, `directory.rs`, `workspace_search.rs`, `shell/terminal.rs`, `AcpManager`, `.new`, `main`, `editor-plugin-host/src/lib.rs`, `CommandSource`, `editor-core/src/lib.rs`, `active_git_status_command_context`, `submit_input_buffer`, `shell/browser.rs`, `execute_oil_action`, `PickerOverlay`, `BufferId`, `shell/git.rs`, `GitSummaryState`?**
  _High betweenness centrality (0.131) - this node is a cross-community bridge._
- **Why does `PluginPackage` connect `PluginPackage` to `oil.rs`, `user/lib.rs`, `sdk/src/lib.rs`, `calculator.rs`, `DbBrowserContext`, `lsp.rs`, `picker_items`, `AutocompleteProviderConfig`, `compile.rs`, `HoverProviderConfig`, `user/terminal.rs`, `UserLibraryModule`, `user/browser.rs`, `shell/mod.rs`, `HeaderlineTestUserLibrary`, `syntax_languages`, `package`, `Self`, `bash.rs`, `clojure.rs`, `elixir.rs`, `graphql.rs`, `.new`, `hcl.rs`, `java.rs`, `kotlin.rs`, `latex.rs`, `lua.rs`, `nix.rs`, `perl.rs`, `php.rs`, `proto.rs`, `r.rs`, `ruby.rs`, `scala.rs`, `solidity.rs`, `swift.rs`, `lang/vim.rs`, `xml.rs`, `PickerItemSpec`, `main`, `editor-plugin-host/src/lib.rs`, `package`, `PluginBuffer`, `common.rs`, `debug_adapters`, `package`, `PluginKeyBinding`, `PluginCommand`?**
  _High betweenness centrality (0.074) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `ShellBuffer` to `String`, `editor-terminal/src/lib.rs`, `Result`, `Self`, `TextBuffer`, `.refresh_pdf_view`, `Instant`, `shell/pdf.rs`, `register_shell_hooks`, `UserLibrary`, `shell/mod.rs`, `shell/acp.rs`, `render_buffer_with_view_state`, `.is_empty`, `Option`, `Result`, `directory.rs`, `shell/terminal.rs`, `.new`, `draw_diagnostic_underlines_for_segment`, `.new`, `submit_input_buffer`, `shell/browser.rs`, `LspInlineCompletionItem`, `execute_oil_action`, `PickerOverlay`, `shell/git.rs`, `Vec`, `GitSummaryState`?**
  _High betweenness centrality (0.070) - this node is a cross-community bridge._
- **What connects `StartupProfile`, `topbar`, `navToggle` to the rest of the system?**
  _136 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `String` be split into smaller, more focused modules?**
  _Cohesion score 0.03371173030615755 - nodes in this community are weakly interconnected._
- **Should `LspClientError` be split into smaller, more focused modules?**
  _Cohesion score 0.06786651668541432 - nodes in this community are weakly interconnected._
- **Should `shell/tests.rs` be split into smaller, more focused modules?**
  _Cohesion score 0.0254841997961264 - nodes in this community are weakly interconnected._