# Graph Report - volt  (2026-05-12)

## Corpus Check
- 164 files · ~483,724 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 6058 nodes · 16841 edges · 62 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2370 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 90|Community 90]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 264 edges
2. `ShellBuffer` - 216 edges
3. `register_shell_hooks()` - 216 edges
4. `shell_ui()` - 162 edges
5. `shell_buffer()` - 142 edges
6. `shell_buffer_mut()` - 135 edges
7. `TextBuffer` - 103 edges
8. `ShellUiState` - 101 edges
9. `active_shell_buffer_id()` - 101 edges
10. `ShellState` - 90 edges

## Surprising Connections (you probably didn't know these)
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `directory_entry_label()` --calls--> `directory_yank_for_range()`  [INFERRED]
  crates\editor-sdl\src\shell\directory.rs → crates\editor-sdl\src\shell\mod.rs
- `active_shell_buffer_mut()` --calls--> `pdf_previous_page()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\pdf.rs
- `apply_motion_command()` --calls--> `terminal_scroll_for_motion()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\terminal.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.01
Nodes (532): acp_complete_slash(), acp_pick_session(), maybe_open_slash_completion(), ensure_browser_popup_buffer(), focus_active_browser_popup(), focus_browser_input_section(), open_browser_buffer_in_popup(), ClipboardContext (+524 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (432): acp_insert_slash_command(), acp_pick_model(), acp_switch_pane(), drain_events_shows_incremental_plan_progress_across_frames(), install_acp_test_buffer(), refresh_acp_input_hint(), session_finished_marks_plan_entries_completed(), submit_acp_prompt() (+424 more)

### Community 2 - "Community 2"
Cohesion: 0.01
Nodes (321): package(), syntax_language(), capture_mappings(), jsx_syntax_language(), package(), syntax_language(), package(), syntax_language() (+313 more)

### Community 3 - "Community 3"
Cohesion: 0.01
Nodes (103): browser_state_for_kind(), default_vim_target(), acp_pane_content_rows(), acp_rendered_line_row_count(), acp_tool_call_from_partial_update(), AcpBufferState, AcpPaneState, AcpRenderedLine (+95 more)

### Community 4 - "Community 4"
Cohesion: 0.01
Nodes (192): active_parameter_label(), apply_command_environment(), apply_windows_fnm_environment(), build_lsp_command(), char_to_byte_offset(), cleanup_unused_sessions(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document() (+184 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (281): oil_directory_line_spans(), active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot() (+273 more)

### Community 6 - "Community 6"
Cohesion: 0.02
Nodes (149): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), LanguageConfiguration, additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root() (+141 more)

### Community 7 - "Community 7"
Cohesion: 0.01
Nodes (172): packages(), syntax_languages(), all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, dynamic_user_library_can_wrap_exported_module(), ancestor_contexts_for_cursor() (+164 more)

### Community 8 - "Community 8"
Cohesion: 0.03
Nodes (198): browser_buffer_layout(), render_browser_buffer_body(), covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan (+190 more)

### Community 9 - "Community 9"
Cohesion: 0.01
Nodes (150): main(), parse_symbol_line(), apply_directory_edit_actions(), apply_directory_edit_queue(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory() (+142 more)

### Community 10 - "Community 10"
Cohesion: 0.02
Nodes (79): abi_language_server_spec_round_trips_workspace_configuration(), AbiWorkspaceConfigurationNumber, WorkspaceConfigurationValue, compact_project_path(), contains_wildcards(), csharp_language_server(), default_worktree_common_dir(), detect_project_kind() (+71 more)

### Community 11 - "Community 11"
Cohesion: 0.02
Nodes (146): acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+138 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (53): create_workspace_file_from_query(), advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner() (+45 more)

### Community 13 - "Community 13"
Cohesion: 0.02
Nodes (132): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+124 more)

### Community 14 - "Community 14"
Cohesion: 0.02
Nodes (48): CommandLineCompletionState, CommandLineOverlay, AutocompleteWorkerState, FileReloadWorkerState, InlineCompletionWorkerState, LspSyncWorkerState, sync_file_reload_watch(), VimSearchWorkerState (+40 more)

### Community 15 - "Community 15"
Cohesion: 0.02
Nodes (54): apply_command_environment(), apply_windows_fnm_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult, CompilationRunner (+46 more)

### Community 16 - "Community 16"
Cohesion: 0.03
Nodes (67): pdf_preview_page_from_url(), AbiStringPair, centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name() (+59 more)

### Community 17 - "Community 17"
Cohesion: 0.03
Nodes (38): bootstrap(), cargo(), command_palette_items(), CommandPaletteState, CompilationState, DapState, DynamicUserLibrary, EventLog (+30 more)

### Community 18 - "Community 18"
Cohesion: 0.02
Nodes (37): AcpClient, AutocompleteProvider, AutocompleteProviderItem, GhostTextContext, GhostTextLine, GitStatusPrefix, HoverProvider, HoverProviderTopic (+29 more)

### Community 19 - "Community 19"
Cohesion: 0.04
Nodes (25): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+17 more)

### Community 20 - "Community 20"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 21 - "Community 21"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 22 - "Community 22"
Cohesion: 0.1
Nodes (38): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+30 more)

### Community 23 - "Community 23"
Cohesion: 0.06
Nodes (11): CopilotDeviceCodePrompt, codelldb(), DapError, DebugAdapterRegistry, DebugAdapterSpec, DebugConfiguration, DebugRequestKind, DebugSessionPlan (+3 more)

### Community 24 - "Community 24"
Cohesion: 0.08
Nodes (16): BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode (+8 more)

### Community 25 - "Community 25"
Cohesion: 0.09
Nodes (15): empty_query_returns_all_items_in_sorted_order(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), match_item(), match_term(), PickerItem, PickerMatch, PickerResultOrder (+7 more)

### Community 26 - "Community 26"
Cohesion: 0.07
Nodes (16): AutocompleteEntry, AutocompleteProviderSpec, AutocompleteQuery, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind, HoverProviderSpec (+8 more)

### Community 27 - "Community 27"
Cohesion: 0.08
Nodes (31): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), syntax_language(), GrammarSourceSpec (+23 more)

### Community 28 - "Community 28"
Cohesion: 0.05
Nodes (26): BlockInsertState, BlockSelection, DirectoryYankEntry, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch (+18 more)

### Community 29 - "Community 29"
Cohesion: 0.11
Nodes (8): amber(), Color, registry_resolves_option_values(), registry_resolves_tokens_from_active_theme(), Theme, ThemeError, ThemeRegistry, volt_dark()

### Community 30 - "Community 30"
Cohesion: 0.13
Nodes (27): file_open_detail(), is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines(), pdf_delete_page(), pdf_header_lines() (+19 more)

### Community 31 - "Community 31"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 32 - "Community 32"
Cohesion: 0.14
Nodes (23): buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session(), focus_active_terminal_popup(), refresh_pending_terminal(), resize_active_terminal_session() (+15 more)

### Community 33 - "Community 33"
Cohesion: 0.14
Nodes (22): append_streamed_command_error(), continue_streamed_command_popup(), drain_completed_output_lines(), push_streamed_command_update(), refresh_pending_streamed_commands(), run_streamed_command(), stream_command_output(), streamed_command_notification() (+14 more)

### Community 34 - "Community 34"
Cohesion: 0.13
Nodes (12): compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register(), compose_joins_the_default_user_segments(), compose_skips_empty_optional_segments(), GitStatuslineInfo (+4 more)

### Community 35 - "Community 35"
Cohesion: 0.13
Nodes (2): FontSet<'ttf>, validate_bundled_icon_fonts()

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
Cohesion: 0.22
Nodes (4): ShellConfig, ShellError, ShellSummary, TypingProfileSummary

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (1): Color

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): PaneConfig

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 90 - "Community 90"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Knowledge Gaps
- **342 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+337 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 35`** (16 nodes): `FontSet<'ttf>`, `.emoji_font()`, `.emoji_font_has_char()`, `.emoji_pixel_size()`, `.emoji_raster_font()`, `.emoji_shape_face()`, `.icon_font()`, `.icon_font_index_for_char()`, `.icon_fonts()`, `.ligatures_enabled()`, `.new()`, `.prefers_icon_font()`, `.primary_pixel_size()`, `.primary_raster_font()`, `.primary_shape_face()`, `validate_bundled_icon_fonts()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `PaneConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 90`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `syntax_language()` connect `Community 27` to `Community 2`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 13` to `Community 27`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 27` to `Community 13`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Are the 131 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 131 INFERRED edges - model-reasoned connections that need verification._
- **Are the 56 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 56 INFERRED edges - model-reasoned connections that need verification._
- **Are the 88 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 88 INFERRED edges - model-reasoned connections that need verification._
- **Are the 95 inferred relationships involving `shell_buffer()` (e.g. with `acp_complete_slash()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_buffer()` has 95 INFERRED edges - model-reasoned connections that need verification._