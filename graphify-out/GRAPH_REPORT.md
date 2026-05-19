# Graph Report - volt  (2026-05-19)

## Corpus Check
- 166 files · ~608,346 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 6416 nodes · 18048 edges · 60 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2480 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 88|Community 88]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 275 edges
2. `register_shell_hooks()` - 237 edges
3. `ShellBuffer` - 218 edges
4. `shell_ui()` - 169 edges
5. `shell_buffer()` - 153 edges
6. `shell_buffer_mut()` - 139 edges
7. `active_shell_buffer_id()` - 112 edges
8. `TextBuffer` - 103 edges
9. `ShellUiState` - 101 edges
10. `ShellState` - 94 edges

## Surprising Connections (you probably didn't know these)
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `register_clipboard_context()` --calls--> `run_demo_shell()`  [INFERRED]
  crates\editor-sdl\src\shell\clipboard.rs → crates\editor-sdl\src\shell\mod.rs
- `statusline_lsp_diagnostics()` --calls--> `render_buffer_with_view_state()`  [INFERRED]
  crates\editor-sdl\src\shell\diagnostics.rs → crates\editor-sdl\src\shell\render.rs
- `directory_entry_label()` --calls--> `directory_yank_for_range()`  [INFERRED]
  crates\editor-sdl\src\shell\directory.rs → crates\editor-sdl\src\shell\mod.rs

## Communities

### Community 0 - "Community 0"
Cohesion: 0.01
Nodes (477): init_acp_manager(), browser_state_for_kind(), default_vim_target(), acp_build_output_lines(), acp_build_plan_lines(), acp_decode_image(), acp_icon_segment(), acp_multiline_text_lines() (+469 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (689): acp_complete_slash(), acp_insert_slash_command(), acp_pick_mode(), acp_pick_model(), acp_pick_session(), acp_switch_pane(), maybe_open_slash_completion(), refresh_acp_input_hint() (+681 more)

### Community 2 - "Community 2"
Cohesion: 0.01
Nodes (341): package(), syntax_language(), capture_mappings(), jsx_syntax_language(), package(), syntax_language(), package(), syntax_language() (+333 more)

### Community 3 - "Community 3"
Cohesion: 0.01
Nodes (206): active_parameter_label(), apply_command_environment(), apply_windows_runtime_environment(), build_lsp_command(), char_to_byte_offset(), cleanup_unused_sessions(), client_capabilities(), client_capabilities_enable_window_work_done_progress_and_show_document() (+198 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (161): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), temp_dir(), styled_primary_font_path_prefers_real_style_files(), LanguageConfiguration, additional_highlight_languages_merge_spans() (+153 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (286): active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), apply_git_view(), begin_oil_worktree_request(), build_git_fringe_snapshot(), build_git_summary_snapshot() (+278 more)

### Community 6 - "Community 6"
Cohesion: 0.01
Nodes (177): packages(), syntax_languages(), seti_directory_icon(), seti_file_icon(), ancestor_contexts_for_cursor(), AncestorContextBufferKey, AncestorContextCache, AncestorContextQuery (+169 more)

### Community 7 - "Community 7"
Cohesion: 0.02
Nodes (74): close_acp_workspace_buffers(), clear_key_sequence(), set_key_sequence(), take_key_sequence(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_runtime_surface(), buffer_is_oil_preview() (+66 more)

### Community 8 - "Community 8"
Cohesion: 0.02
Nodes (147): acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+139 more)

### Community 9 - "Community 9"
Cohesion: 0.01
Nodes (146): main(), parse_symbol_line(), apply_directory_edit_actions(), copy_directory_recursive(), copy_directory_yank_entries(), copy_directory_yank_entries_copies_files_and_directories(), create_dir_action_creates_empty_directory(), diff_directory_lines() (+138 more)

### Community 10 - "Community 10"
Cohesion: 0.02
Nodes (79): workspace_project_picker_shows_repo_context_for_worktrees(), compact_project_path(), contains_wildcards(), csharp_language_server(), default_worktree_common_dir(), detect_project_kind(), dev_extension_server(), Diagnostic (+71 more)

### Community 11 - "Community 11"
Cohesion: 0.02
Nodes (164): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+156 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (60): ClipboardContext, register_clipboard_context(), with_clipboard_util(), write_system_clipboard(), yank_from_clipboard_text(), advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice() (+52 more)

### Community 13 - "Community 13"
Cohesion: 0.03
Nodes (77): build_tokio_runtime(), connect_sql_server(), connection_descriptor_detects_all_supported_engines(), ConnectionDescriptor, current_statement(), DbActionOutcome, DbAutocompleteCandidate, DbBrowserAction (+69 more)

### Community 14 - "Community 14"
Cohesion: 0.02
Nodes (65): apply_command_environment(), apply_windows_runtime_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), build_job_command_keeps_nvm_path_ahead_of_explicit_path(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult (+57 more)

### Community 15 - "Community 15"
Cohesion: 0.03
Nodes (110): acp_rendered_text_wrap_cols(), acp_prefix_columns(), acp_spinner_frame(), AcpBufferLayout, AcpPaneLayout, adjusted_contextual_ligature_pixel_size(), alpha_bitmap_surface(), ascii_ligature_byte_ranges_with_face() (+102 more)

### Community 16 - "Community 16"
Cohesion: 0.03
Nodes (42): CommandLineCompletionState, CommandLineOverlay, append_lines(), cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig (+34 more)

### Community 17 - "Community 17"
Cohesion: 0.03
Nodes (46): bootstrap(), builtin_user_library_validation_accepts_static_syntax_languages(), cargo(), catch_unwind_silently(), command_palette_items(), CommandPaletteState, CompilationState, DapState (+38 more)

### Community 18 - "Community 18"
Cohesion: 0.02
Nodes (38): AcpClient, AutocompleteProvider, AutocompleteProviderItem, GhostTextContext, GhostTextLine, GitStatusPrefix, HoverProvider, HoverProviderTopic (+30 more)

### Community 19 - "Community 19"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 20 - "Community 20"
Cohesion: 0.04
Nodes (25): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+17 more)

### Community 21 - "Community 21"
Cohesion: 0.05
Nodes (41): pdf_preview_page_from_url(), AbiStringPair, autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook() (+33 more)

### Community 22 - "Community 22"
Cohesion: 0.03
Nodes (11): comment_toggle_styles_cover_all_shipped_syntax_languages(), HeaderlineTestUserLibrary, statusline_icon_segments_split_acp_and_lsp_icons(), statusline_icon_segments_split_diagnostic_icons(), all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol (+3 more)

### Community 23 - "Community 23"
Cohesion: 0.06
Nodes (55): append_streamed_command_error(), append_streamed_command_header(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands(), run_streamed_command() (+47 more)

### Community 24 - "Community 24"
Cohesion: 0.1
Nodes (39): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+31 more)

### Community 25 - "Community 25"
Cohesion: 0.08
Nodes (21): best_contiguous_substring_bonus(), contiguous_substring_beats_split_path_match(), contiguous_substring_bonus(), empty_query_returns_all_items_in_sorted_order(), fringe_metadata_survives_matching(), fuzzy_query_prefers_prefix_and_contiguous_matches(), is_match_boundary(), is_match_end_boundary() (+13 more)

### Community 26 - "Community 26"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 27 - "Community 27"
Cohesion: 0.08
Nodes (16): BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode (+8 more)

### Community 28 - "Community 28"
Cohesion: 0.06
Nodes (18): hover_registry_includes_signature_help_provider(), AutocompleteEntry, AutocompleteProviderSpec, AutocompleteQuery, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind (+10 more)

### Community 29 - "Community 29"
Cohesion: 0.09
Nodes (10): amber(), registry_resolves_option_values(), registry_resolves_token_styles(), registry_resolves_tokens_from_active_theme(), Theme, ThemeError, ThemeRegistry, ThemeStyle (+2 more)

### Community 30 - "Community 30"
Cohesion: 0.05
Nodes (26): BlockInsertState, BlockSelection, DirectoryYankEntry, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch (+18 more)

### Community 31 - "Community 31"
Cohesion: 0.13
Nodes (24): box_drawing_segments(), BoxDrawingSegments, buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session(), refresh_pending_terminal() (+16 more)

### Community 32 - "Community 32"
Cohesion: 0.13
Nodes (1): InputField

### Community 33 - "Community 33"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 34 - "Community 34"
Cohesion: 0.17
Nodes (13): covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_line_spans_for_diagnostics(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+5 more)

### Community 35 - "Community 35"
Cohesion: 0.21
Nodes (7): AutocompleteProviderConfig, backends(), hook_command(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_have_unique_ids_and_keep_calculator_scoping()

### Community 36 - "Community 36"
Cohesion: 0.27
Nodes (9): compile_command_emits_run_command_hook(), compile_package_exports_compile_and_recompile_commands(), compile_package_exports_global_keybindings(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only(), parse_error_location_handles_rust_arrow_prefix() (+1 more)

### Community 37 - "Community 37"
Cohesion: 0.22
Nodes (1): ServiceRegistry

### Community 38 - "Community 38"
Cohesion: 1.0
Nodes (1): Color

### Community 39 - "Community 39"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 40 - "Community 40"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 41 - "Community 41"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 42 - "Community 42"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 43 - "Community 43"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 44 - "Community 44"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): PaneConfig

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 88 - "Community 88"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Knowledge Gaps
- **362 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+357 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 32`** (33 nodes): `InputField`, `.append_text()`, `.backspace()`, `.byte_index_for_char()`, `.cursor_char()`, `.cursor_line_col()`, `.cursor_line_col_with_starts()`, `.cursor_point()`, `.cursor_visual_row_col()`, `.delete_forward()`, `.delete_range()`, `.delete_selection()`, `.hint()`, `.insert_text()`, `.line_col_for_char()`, `.line_len_for()`, `.line_starts()`, `.move_down()`, `.move_left()`, `.move_line_end()`, `.move_line_start()`, `.move_right()`, `.move_up()`, `.new()`, `.placeholder()`, `.prompt()`, `.selected_char_range()`, `.selected_text()`, `.selection_visual_ranges()`, `.set_text()`, `.start_selection()`, `.text_line_count()`, `.visual_row_col_for_cursor()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 37`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 40`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 41`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 42`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 43`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 44`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `PaneConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 88`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `syntax_language()` connect `Community 11` to `Community 2`?**
  _High betweenness centrality (0.050) - this node is a cross-community bridge._
- **Are the 134 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 134 INFERRED edges - model-reasoned connections that need verification._
- **Are the 56 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 56 INFERRED edges - model-reasoned connections that need verification._
- **Are the 92 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 92 INFERRED edges - model-reasoned connections that need verification._
- **Are the 101 inferred relationships involving `shell_buffer()` (e.g. with `acp_complete_slash()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_buffer()` has 101 INFERRED edges - model-reasoned connections that need verification._
- **What connects `WordKind`, `BufferStats`, `TextEdit` to the rest of the system?**
  _362 weakly-connected nodes found - possible documentation gaps or missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.01 - nodes in this community are weakly interconnected._