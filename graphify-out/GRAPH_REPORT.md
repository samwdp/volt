# Graph Report - volt  (2026-07-02)

## Corpus Check
- 167 files · ~648,140 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 6905 nodes · 19316 edges · 74 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2654 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 40|Community 40]]
- [[_COMMUNITY_Community 41|Community 41]]
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
- [[_COMMUNITY_Community 102|Community 102]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 289 edges
2. `register_shell_hooks()` - 241 edges
3. `ShellBuffer` - 222 edges
4. `shell_ui()` - 184 edges
5. `shell_buffer()` - 163 edges
6. `shell_buffer_mut()` - 161 edges
7. `active_shell_buffer_id()` - 112 edges
8. `ShellUiState` - 104 edges
9. `TextBuffer` - 103 edges
10. `ShellState` - 95 edges

## Surprising Connections (you probably didn't know these)
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `register_clipboard_context()` --calls--> `run_demo_shell()`  [INFERRED]
  crates\editor-sdl\src\shell\clipboard.rs → crates\editor-sdl\src\shell\mod.rs
- `directory_entry_label()` --calls--> `directory_yank_for_range()`  [INFERRED]
  crates\editor-sdl\src\shell\directory.rs → crates\editor-sdl\src\shell\mod.rs
- `active_shell_buffer_mut()` --calls--> `pdf_previous_page()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\pdf.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.01
Nodes (622): init_acp_manager(), ClipboardContext, read_system_clipboard(), register_clipboard_context(), with_clipboard_util(), write_system_clipboard(), yank_from_clipboard_text(), yank_to_clipboard_text() (+614 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (542): acp_complete_slash(), acp_insert_slash_command(), acp_pick_mode(), acp_pick_model(), acp_picker_entries(), acp_switch_pane(), apply_acp_notification(), create_acp_buffer() (+534 more)

### Community 2 - "Community 2"
Cohesion: 0.01
Nodes (432): package(), syntax_language(), diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), capture_mappings(), jsx_syntax_language() (+424 more)

### Community 3 - "Community 3"
Cohesion: 0.01
Nodes (202): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), cleanup_unused_sessions(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document() (+194 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (182): LanguageConfiguration, additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), append_query_source(), apply_text_edits_to_span(), asset_path_from_parts() (+174 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (277): oil_directory_line_spans(), active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot() (+269 more)

### Community 6 - "Community 6"
Cohesion: 0.02
Nodes (72): clear_key_sequence(), accept_autocomplete(), active_buffer_event_context(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_runtime_surface(), active_shell_buffer_read_only(), alt_mod() (+64 more)

### Community 7 - "Community 7"
Cohesion: 0.02
Nodes (99): abi_language_server_spec_round_trips_workspace_configuration(), AbiFiniteF64, AbiWorkspaceConfigurationNumber, WorkspaceConfigurationValue, CopilotDeviceCodePrompt, parse_csharp_metadata_response(), sanitize_transport_message(), transport_key_is_sensitive() (+91 more)

### Community 8 - "Community 8"
Cohesion: 0.01
Nodes (176): main(), parse_symbol_line(), apply_directory_edit_actions(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+168 more)

### Community 9 - "Community 9"
Cohesion: 0.02
Nodes (215): browser_buffer_layout(), render_browser_buffer_body(), covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan (+207 more)

### Community 10 - "Community 10"
Cohesion: 0.02
Nodes (141): acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+133 more)

### Community 11 - "Community 11"
Cohesion: 0.02
Nodes (164): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+156 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (56): create_workspace_file_from_query(), advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner() (+48 more)

### Community 13 - "Community 13"
Cohesion: 0.01
Nodes (73): AcpClient, AcpPickerContext, AcpPickerItemSpec, AcpPickerKind, AcpPickerOption, AutocompleteProvider, AutocompleteProviderItem, BrowserFeatureSpec (+65 more)

### Community 14 - "Community 14"
Cohesion: 0.03
Nodes (84): build_tokio_runtime(), connect_sql_server(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), db_browser_action_from_spec(), db_browser_renderer_customizes_rows_and_preserves_actions(), db_browser_renderer_rejects_row_count_mismatch() (+76 more)

### Community 15 - "Community 15"
Cohesion: 0.02
Nodes (69): CommandLineCompletionState, CommandLineOverlay, InlineCompletionWorkerState, box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace() (+61 more)

### Community 16 - "Community 16"
Cohesion: 0.02
Nodes (63): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult (+55 more)

### Community 17 - "Community 17"
Cohesion: 0.02
Nodes (47): bootstrap(), builtin_user_library_validation_accepts_grammar_backed_syntax_languages(), cargo(), catch_unwind_silently(), command_palette_items(), CommandPaletteState, CompilationState, DapState (+39 more)

### Community 18 - "Community 18"
Cohesion: 0.03
Nodes (77): resolve_font_path(), pdf_preview_page_from_url(), lsp_location_uri_detail(), AbiStringPair, centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name() (+69 more)

### Community 19 - "Community 19"
Cohesion: 0.03
Nodes (71): AcpActionSpec, acp_picker_detail(), AcpClientConfig, client_by_id(), clients(), hook_command(), package(), picker_items() (+63 more)

### Community 20 - "Community 20"
Cohesion: 0.04
Nodes (26): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+18 more)

### Community 21 - "Community 21"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 22 - "Community 22"
Cohesion: 0.07
Nodes (24): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), custom_search_text_matches_hidden_path_segments(), divider_visible_with_empty_query_and_hidden_when_filtering(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches() (+16 more)

### Community 23 - "Community 23"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 24 - "Community 24"
Cohesion: 0.1
Nodes (38): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+30 more)

### Community 25 - "Community 25"
Cohesion: 0.08
Nodes (47): search_is_case_sensitive(), workspace_relative_path(), parse_grep_workspace_search_line_finds_case_insensitive_column(), parse_rg_workspace_search_line_extracts_location(), collect_search_output(), file_context_preview(), file_context_preview_marks_target_line(), lsp_code_action_explicit_kind_rank() (+39 more)

### Community 26 - "Community 26"
Cohesion: 0.1
Nodes (39): append_streamed_command_error(), append_streamed_command_header(), append_streamed_command_lines(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands() (+31 more)

### Community 27 - "Community 27"
Cohesion: 0.08
Nodes (16): BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode (+8 more)

### Community 28 - "Community 28"
Cohesion: 0.06
Nodes (18): hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteProviderSpec, AutocompleteQuery, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind (+10 more)

### Community 29 - "Community 29"
Cohesion: 0.09
Nodes (11): amber(), Color, registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), Theme, ThemeError, ThemeRegistry (+3 more)

### Community 30 - "Community 30"
Cohesion: 0.08
Nodes (31): ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery, buffer_line_text(), context_queries_enabled(), ensure_cached_buffer(), build_ghost_text_lines() (+23 more)

### Community 31 - "Community 31"
Cohesion: 0.05
Nodes (26): BlockInsertState, BlockSelection, DirectoryYankEntry, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch (+18 more)

### Community 32 - "Community 32"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 33 - "Community 33"
Cohesion: 0.14
Nodes (25): is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_header_lines(), pdf_inherited_page_value(), pdf_language_id() (+17 more)

### Community 34 - "Community 34"
Cohesion: 0.13
Nodes (12): compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register(), compose_joins_the_default_user_segments(), compose_skips_empty_optional_segments(), GitStatuslineInfo (+4 more)

### Community 35 - "Community 35"
Cohesion: 0.11
Nodes (2): FontSet<'ttf>, load_deferred_emoji_font()

### Community 36 - "Community 36"
Cohesion: 0.21
Nodes (7): AutocompleteProviderConfig, backends(), hook_command(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping()

### Community 37 - "Community 37"
Cohesion: 0.27
Nodes (9): compile_command_emits_run_command_hook(), compile_package_exports_compile_and_recompile_commands(), compile_package_exports_global_keybindings(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only(), parse_error_location_handles_rust_arrow_prefix() (+1 more)

### Community 38 - "Community 38"
Cohesion: 0.22
Nodes (1): ServiceRegistry

### Community 39 - "Community 39"
Cohesion: 0.31
Nodes (6): hook_command(), HoverProviderConfig, package(), package_exports_hover_commands_and_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping()

### Community 40 - "Community 40"
Cohesion: 0.22
Nodes (4): ShellConfig, ShellError, ShellSummary, TypingProfileSummary

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (1): PickerTruncateStrategy

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (1): Color

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): PickerTruncateStrategy

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): ContextHelpEntry

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): ContextHelpSpec

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): GitPrefixBinding

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): GitCommandBinding

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): GitFeatureSpec

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): OilFeatureSpec

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): BrowserFeatureSpec

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): DbFeatureSpec

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): TerminalFeatureSpec

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 62 - "Community 62"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 63 - "Community 63"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 64 - "Community 64"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 66 - "Community 66"
Cohesion: 1.0
Nodes (1): PaneConfig

### Community 67 - "Community 67"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 68 - "Community 68"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 69 - "Community 69"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 70 - "Community 70"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 71 - "Community 71"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 72 - "Community 72"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 102 - "Community 102"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Knowledge Gaps
- **399 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+394 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 35`** (18 nodes): `FontSet<'ttf>`, `.emoji_font()`, `.emoji_font_has_char()`, `.emoji_pixel_size()`, `.emoji_raster_font()`, `.emoji_shape_face()`, `.icon_font()`, `.ligatures_enabled()`, `.new()`, `.prefers_icon_font()`, `.primary_for_style()`, `.primary_pixel_size()`, `.primary_raster_font()`, `.primary_shape_face()`, `.primary_style_uses_synthetic_bold()`, `.push_icon_font()`, `.set_emoji_font()`, `load_deferred_emoji_font()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `PickerTruncateStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `PickerTruncateStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `ContextHelpEntry`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `ContextHelpSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `GitPrefixBinding`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `GitCommandBinding`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `GitFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `OilFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `BrowserFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `DbFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `TerminalFeatureSpec`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (2 nodes): `PaneConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 67`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 68`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 69`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 70`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 71`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 72`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 102`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `temp_dir()` connect `Community 5` to `Community 33`, `Community 2`, `Community 3`, `Community 1`, `Community 4`, `Community 7`, `Community 8`, `Community 10`, `Community 12`, `Community 14`, `Community 16`, `Community 19`, `Community 25`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Why does `register_shell_hooks()` connect `Community 0` to `Community 1`, `Community 2`, `Community 5`, `Community 6`, `Community 8`, `Community 10`, `Community 12`, `Community 13`, `Community 15`, `Community 26`?**
  _High betweenness centrality (0.047) - this node is a cross-community bridge._
- **Why does `syntax_language()` connect `Community 11` to `Community 2`?**
  _High betweenness centrality (0.042) - this node is a cross-community bridge._
- **Are the 146 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 146 INFERRED edges - model-reasoned connections that need verification._
- **Are the 56 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 56 INFERRED edges - model-reasoned connections that need verification._
- **Are the 106 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 106 INFERRED edges - model-reasoned connections that need verification._
- **Are the 111 inferred relationships involving `shell_buffer()` (e.g. with `acp_complete_slash()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_buffer()` has 111 INFERRED edges - model-reasoned connections that need verification._