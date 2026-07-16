# Graph Report - volt  (2026-07-16)

## Corpus Check
- 167 files · ~575,491 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 7040 nodes · 19665 edges · 73 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2702 edges (avg confidence: 0.8)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- [[_COMMUNITY_Community 0|Community 0]]
- [[_COMMUNITY_Community 1|Community 1]]
- [[_COMMUNITY_Community 2|Community 2]]
- [[_COMMUNITY_Community 3|Community 3]]
- [[_COMMUNITY_Community 4|Community 4]]
- [[_COMMUNITY_Community 5|Community 5]]
- [[_COMMUNITY_Community 6|Community 6]]
- [[_COMMUNITY_Community 7|Community 7]]
- [[_COMMUNITY_Community 8|Community 8]]
- [[_COMMUNITY_Community 9|Community 9]]
- [[_COMMUNITY_Community 10|Community 10]]
- [[_COMMUNITY_Community 11|Community 11]]
- [[_COMMUNITY_Community 12|Community 12]]
- [[_COMMUNITY_Community 13|Community 13]]
- [[_COMMUNITY_Community 14|Community 14]]
- [[_COMMUNITY_Community 15|Community 15]]
- [[_COMMUNITY_Community 16|Community 16]]
- [[_COMMUNITY_Community 17|Community 17]]
- [[_COMMUNITY_Community 18|Community 18]]
- [[_COMMUNITY_Community 19|Community 19]]
- [[_COMMUNITY_Community 20|Community 20]]
- [[_COMMUNITY_Community 21|Community 21]]
- [[_COMMUNITY_Community 22|Community 22]]
- [[_COMMUNITY_Community 23|Community 23]]
- [[_COMMUNITY_Community 24|Community 24]]
- [[_COMMUNITY_Community 25|Community 25]]
- [[_COMMUNITY_Community 26|Community 26]]
- [[_COMMUNITY_Community 27|Community 27]]
- [[_COMMUNITY_Community 28|Community 28]]
- [[_COMMUNITY_Community 29|Community 29]]
- [[_COMMUNITY_Community 30|Community 30]]
- [[_COMMUNITY_Community 31|Community 31]]
- [[_COMMUNITY_Community 32|Community 32]]
- [[_COMMUNITY_Community 33|Community 33]]
- [[_COMMUNITY_Community 34|Community 34]]
- [[_COMMUNITY_Community 35|Community 35]]
- [[_COMMUNITY_Community 36|Community 36]]
- [[_COMMUNITY_Community 37|Community 37]]
- [[_COMMUNITY_Community 38|Community 38]]
- [[_COMMUNITY_Community 39|Community 39]]
- [[_COMMUNITY_Community 42|Community 42]]
- [[_COMMUNITY_Community 43|Community 43]]
- [[_COMMUNITY_Community 44|Community 44]]
- [[_COMMUNITY_Community 45|Community 45]]
- [[_COMMUNITY_Community 46|Community 46]]
- [[_COMMUNITY_Community 47|Community 47]]
- [[_COMMUNITY_Community 48|Community 48]]
- [[_COMMUNITY_Community 49|Community 49]]
- [[_COMMUNITY_Community 50|Community 50]]
- [[_COMMUNITY_Community 51|Community 51]]
- [[_COMMUNITY_Community 52|Community 52]]
- [[_COMMUNITY_Community 53|Community 53]]
- [[_COMMUNITY_Community 54|Community 54]]
- [[_COMMUNITY_Community 55|Community 55]]
- [[_COMMUNITY_Community 56|Community 56]]
- [[_COMMUNITY_Community 57|Community 57]]
- [[_COMMUNITY_Community 58|Community 58]]
- [[_COMMUNITY_Community 59|Community 59]]
- [[_COMMUNITY_Community 60|Community 60]]
- [[_COMMUNITY_Community 61|Community 61]]
- [[_COMMUNITY_Community 62|Community 62]]
- [[_COMMUNITY_Community 63|Community 63]]
- [[_COMMUNITY_Community 64|Community 64]]
- [[_COMMUNITY_Community 65|Community 65]]
- [[_COMMUNITY_Community 66|Community 66]]
- [[_COMMUNITY_Community 67|Community 67]]
- [[_COMMUNITY_Community 68|Community 68]]
- [[_COMMUNITY_Community 69|Community 69]]
- [[_COMMUNITY_Community 70|Community 70]]
- [[_COMMUNITY_Community 71|Community 71]]
- [[_COMMUNITY_Community 72|Community 72]]
- [[_COMMUNITY_Community 73|Community 73]]
- [[_COMMUNITY_Community 102|Community 102]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 298 edges
2. `register_shell_hooks()` - 244 edges
3. `ShellBuffer` - 222 edges
4. `shell_ui()` - 188 edges
5. `shell_buffer()` - 165 edges
6. `shell_buffer_mut()` - 163 edges
7. `active_shell_buffer_id()` - 118 edges
8. `ShellUiState` - 105 edges
9. `TextBuffer` - 103 edges
10. `ShellState` - 97 edges

## Surprising Connections (you probably didn't know these)
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `directory_entry_label()` --calls--> `directory_yank_for_range()`  [INFERRED]
  crates\editor-sdl\src\shell\directory.rs → crates\editor-sdl\src\shell\mod.rs
- `acp_rendered_text_segments()` --calls--> `acp_wrapped_text_uses_full_width_on_continuation_rows()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\tests.rs
- `rebuild_theme_registry()` --calls--> `rebuild_theme_registry_preserves_active_theme_when_still_present()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\tests.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.01
Nodes (609): browser_state_for_kind(), write_system_clipboard(), yank_to_clipboard_text(), default_vim_target(), acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_icon_segment() (+601 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (556): acp_complete_slash(), acp_insert_slash_command(), acp_pick_mode(), acp_pick_model(), acp_picker_entries(), acp_switch_pane(), close_acp_workspace_buffers(), maybe_open_slash_completion() (+548 more)

### Community 2 - "Community 2"
Cohesion: 0.01
Nodes (350): package(), syntax_language(), diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), capture_mappings(), jsx_syntax_language() (+342 more)

### Community 3 - "Community 3"
Cohesion: 0.01
Nodes (211): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document(), close_buffer_keeps_session_alive_for_next_file() (+203 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (184): vim_search_entries_trim_whitespace_from_labels(), LanguageConfiguration, additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), append_query_source(), apply_text_edits_to_span() (+176 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (82): browser_sync_plan(), browser_viewport_rect_rect(), rects_intersect(), ClipboardContext, read_system_clipboard(), register_clipboard_context(), with_clipboard_util(), yank_from_clipboard_text() (+74 more)

### Community 6 - "Community 6"
Cohesion: 0.02
Nodes (270): main(), browser_buffer_layout(), browser_host_viewport_rect(), browser_viewport_rect(), render_browser_buffer_body(), covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line() (+262 more)

### Community 7 - "Community 7"
Cohesion: 0.02
Nodes (284): active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot(), cancel_git_commit_buffer() (+276 more)

### Community 8 - "Community 8"
Cohesion: 0.01
Nodes (189): packages(), syntax_languages(), AcpActionSpec, all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, seti_directory_icon() (+181 more)

### Community 9 - "Community 9"
Cohesion: 0.01
Nodes (98): abi_language_server_spec_round_trips_workspace_configuration(), AbiFiniteF64, AbiWorkspaceConfigurationNumber, WorkspaceConfigurationValue, CopilotDeviceCodePrompt, sanitize_transport_message(), transport_key_is_sensitive(), codelldb() (+90 more)

### Community 10 - "Community 10"
Cohesion: 0.02
Nodes (146): acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+138 more)

### Community 11 - "Community 11"
Cohesion: 0.02
Nodes (164): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+156 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (55): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending() (+47 more)

### Community 13 - "Community 13"
Cohesion: 0.01
Nodes (73): AcpClient, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, AcpPickerOption, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec (+65 more)

### Community 14 - "Community 14"
Cohesion: 0.03
Nodes (85): build_tokio_runtime(), connect_sql_server(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), db_browser_action_from_spec(), db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch() (+77 more)

### Community 15 - "Community 15"
Cohesion: 0.01
Nodes (76): abi_language_configuration_round_trips_path_matchers(), abi_language_server_spec_round_trips_activation_markers(), abi_language_server_spec_round_trips_default_enabled_flag(), abi_language_server_spec_round_trips_path_matchers(), AbiAcpClient, AbiAutocompleteProvider, AbiAutocompleteProviderItem, AbiBrowserFeatureSpec (+68 more)

### Community 16 - "Community 16"
Cohesion: 0.02
Nodes (64): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult (+56 more)

### Community 17 - "Community 17"
Cohesion: 0.02
Nodes (47): bootstrap(), builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), cargo(), catch_unwind_silently(), command_palette_items(), CommandPaletteState, CompilationState, DapState (+39 more)

### Community 18 - "Community 18"
Cohesion: 0.02
Nodes (44): CommandLineCompletionState, CommandLineOverlay, CommandLinePurpose, open_git_worktree_new_branch_prompt(), append_lines(), cube_color_component(), default_terminal_index_color(), default_terminal_named_color() (+36 more)

### Community 19 - "Community 19"
Cohesion: 0.03
Nodes (85): workspace_relative_path(), pdf_preview_page_from_url(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), collect_search_output(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+77 more)

### Community 20 - "Community 20"
Cohesion: 0.03
Nodes (85): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), pdf_buffer_lines(), pdf_header_lines(), pdf_inherited_page_value(), pdf_language_id(), pdf_latex_lines() (+77 more)

### Community 21 - "Community 21"
Cohesion: 0.04
Nodes (25): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+17 more)

### Community 22 - "Community 22"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 23 - "Community 23"
Cohesion: 0.07
Nodes (25): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+17 more)

### Community 24 - "Community 24"
Cohesion: 0.04
Nodes (2): DynamicUserLibrary, ShellTestUserLibrary

### Community 25 - "Community 25"
Cohesion: 0.1
Nodes (39): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+31 more)

### Community 26 - "Community 26"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 27 - "Community 27"
Cohesion: 0.08
Nodes (11): amber(), registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), Theme, ThemeError, ThemeOption, ThemeRegistry (+3 more)

### Community 28 - "Community 28"
Cohesion: 0.08
Nodes (16): BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode (+8 more)

### Community 29 - "Community 29"
Cohesion: 0.08
Nodes (34): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_match_sort_key(), font_match_sort_key_prefers_regular_faces_for_family_requests() (+26 more)

### Community 30 - "Community 30"
Cohesion: 0.11
Nodes (35): append_streamed_command_error(), append_streamed_command_lines(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands(), run_streamed_command() (+27 more)

### Community 31 - "Community 31"
Cohesion: 0.06
Nodes (18): hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteProviderSpec, AutocompleteQuery, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind (+10 more)

### Community 32 - "Community 32"
Cohesion: 0.05
Nodes (26): BlockInsertState, BlockSelection, DirectoryYankEntry, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch (+18 more)

### Community 33 - "Community 33"
Cohesion: 0.1
Nodes (32): apply_directory_edit_actions(), apply_directory_edit_queue(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines(), directory_cd_from_cursor() (+24 more)

### Community 34 - "Community 34"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 35 - "Community 35"
Cohesion: 0.21
Nodes (7): AutocompleteProviderConfig, backends(), hook_command(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping()

### Community 36 - "Community 36"
Cohesion: 0.26
Nodes (10): compile_command_emits_run_command_hook(), compile_package_exports_compile_and_recompile_commands(), compile_package_exports_global_keybindings(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only(), parse_error_location_handles_rust_arrow_prefix() (+2 more)

### Community 37 - "Community 37"
Cohesion: 0.22
Nodes (1): ServiceRegistry

### Community 38 - "Community 38"
Cohesion: 0.31
Nodes (6): hook_command(), HoverProviderConfig, package(), package_exports_hover_commands_and_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping()

### Community 39 - "Community 39"
Cohesion: 0.22
Nodes (4): ShellConfig, ShellError, ShellSummary, TypingProfileSummary

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (1): PickerTruncateStrategy

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (1): Color

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): PickerTruncateStrategy

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): ContextHelpEntry

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): ContextHelpSpec

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): GitPrefixBinding

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): GitCommandBinding

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): GitFeatureSpec

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): OilFeatureSpec

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): BrowserFeatureSpec

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): DbFeatureSpec

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): TerminalFeatureSpec

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 62 - "Community 62"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 63 - "Community 63"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 64 - "Community 64"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 66 - "Community 66"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 67 - "Community 67"
Cohesion: 1.0
Nodes (1): PaneConfig

### Community 68 - "Community 68"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 69 - "Community 69"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 70 - "Community 70"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 71 - "Community 71"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 72 - "Community 72"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 73 - "Community 73"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 102 - "Community 102"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Knowledge Gaps
- **401 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+396 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 24`** (54 nodes): `DynamicUserLibrary`, `.acp_client_by_id()`, `.acp_clients()`, `.acp_picker_items()`, `.autocomplete_providers()`, `.autocomplete_result_limit()`, `.autocomplete_token_icon()`, `.browser_buffer_lines()`, `.browser_feature_spec()`, `.browser_input_hint()`, `.browser_url_placeholder()`, `.browser_url_prompt()`, `.context_help_specs()`, `.db_feature_spec()`, `.debug_adapters()`, `.ghost_text_lines()`, `.git_command_for_chord()`, `.git_commit_template()`, `.git_feature_spec()`, `.git_prefix_for_chord()`, `.git_status_sections()`, `.gitfringe_symbol()`, `.gitfringe_token_added()`, `.gitfringe_token_modified()`, `.gitfringe_token_removed()`, `.headerline_lines()`, `.hover_line_limit()`, `.hover_providers()`, `.hover_signature_icon()`, `.hover_token_icon()`, `.ligature_config()`, `.lsp_show_buffer_diagnostics()`, `.oil_chord_action()`, `.oil_directory_sections()`, `.oil_feature_spec()`, `.oil_help_lines()`, `.oil_keybindings()`, `.oil_keydown_action()`, `.oil_strip_entry_icon_prefix()`, `.pane_config()`, `.pdf_open_mode()`, `.picker_provider_items()`, `.picker_providers()`, `.picker_truncate_strategy()`, `.run_plugin_buffer_evaluator()`, `.statusline_lsp_connected_icon()`, `.statusline_lsp_error_icon()`, `.statusline_lsp_warning_icon()`, `.statusline_render()`, `.terminal_feature_spec()`, `.workspace_roots()`, `ShellTestUserLibrary`, `.picker_provider_items()`, `.picker_providers()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `PickerTruncateStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `PickerTruncateStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `ContextHelpEntry`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `ContextHelpSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `GitPrefixBinding`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `GitCommandBinding`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `GitFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `OilFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `BrowserFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `DbFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `TerminalFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 67`** (2 nodes): `PaneConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 68`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 69`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 70`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 71`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 72`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 73`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 102`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `temp_dir()` connect `Community 7` to `Community 0`, `Community 33`, `Community 2`, `Community 3`, `Community 1`, `Community 4`, `Community 9`, `Community 10`, `Community 12`, `Community 14`, `Community 16`, `Community 19`, `Community 20`?**
  _High betweenness centrality (0.064) - this node is a cross-community bridge._
- **Why does `ShellBuffer` connect `Community 0` to `Community 1`, `Community 5`, `Community 6`, `Community 7`?**
  _High betweenness centrality (0.035) - this node is a cross-community bridge._
- **Why does `main()` connect `Community 17` to `Community 1`, `Community 2`, `Community 4`, `Community 5`, `Community 16`?**
  _High betweenness centrality (0.034) - this node is a cross-community bridge._
- **Are the 149 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 149 INFERRED edges - model-reasoned connections that need verification._
- **Are the 58 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 58 INFERRED edges - model-reasoned connections that need verification._
- **Are the 109 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 109 INFERRED edges - model-reasoned connections that need verification._
- **Are the 113 inferred relationships involving `shell_buffer()` (e.g. with `acp_complete_slash()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_buffer()` has 113 INFERRED edges - model-reasoned connections that need verification._