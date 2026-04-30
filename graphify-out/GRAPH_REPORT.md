# Graph Report - volt  (2026-04-30)

## Corpus Check
- 164 files · ~468,721 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 5876 nodes · 16031 edges · 67 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2248 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 95|Community 95]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 245 edges
2. `register_shell_hooks()` - 208 edges
3. `ShellBuffer` - 207 edges
4. `shell_ui()` - 154 edges
5. `shell_buffer()` - 128 edges
6. `shell_buffer_mut()` - 123 edges
7. `TextBuffer` - 103 edges
8. `ShellUiState` - 100 edges
9. `active_shell_buffer_id()` - 94 edges
10. `ShellState` - 87 edges

## Surprising Connections (you probably didn't know these)
- `Declarative Package Metadata` --semantically_similar_to--> `PluginPackage`  [INFERRED] [semantically similar]
  CLAUDE.md → docs\user-packages.md
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `register_clipboard_context()` --calls--> `run_demo_shell()`  [INFERRED]
  crates\editor-sdl\src\shell\clipboard.rs → crates\editor-sdl\src\shell\mod.rs
- `shell_ui()` --calls--> `hover_scroll_offset()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\tests.rs

## Hyperedges (group relationships)
- **Volt Logo Composition** — logo_volt_app_icon, logo_lightning_bolt_monogram, logo_volt_brand [EXTRACTED 1.00]
- **Volt Logo Composition** — logo_lightning_bolt_glyph, logo_volt_wordmark, logo_rounded_app_icon_background [EXTRACTED 1.00]
- **Volt Banner Composition** — banner_volt_brand_banner, banner_volt_logo, banner_code_editor_backdrop [EXTRACTED 1.00]
- **Volt logo components** — logo_rounded_square_badge, logo_lightning_bolt_glyph, logo_stylized_v_accent, logo_volt_wordmark [EXTRACTED 1.00]
- **Volt Brand Mark** — logo_32x32_volt_logo, logo_32x32_v_shaped_bolt, logo_32x32_wordmark [EXTRACTED 1.00]
- **Architecture Stack** — architecture_diagram_user_interface_layer, architecture_diagram_plugin_packages_layer, architecture_diagram_editor_core_layer, architecture_diagram_runtime_systems_layer, architecture_diagram_graphics_platform_layer [EXTRACTED 1.00]
- **Plugin Metadata Model** — user_packages_plugin_package, user_packages_plugin_command, user_packages_plugin_action, user_packages_plugin_hook, user_packages_plugin_keybinding, user_packages_plugin_buffer [EXTRACTED 1.00]
- **Language Support Pipeline** — language_module_contract, language_common_helper, language_lsp_server_spec, language_theme_tokens [EXTRACTED 1.00]

## Communities

### Community 0 - "Community 0"
Cohesion: 0.01
Nodes (551): acp_complete_slash(), acp_disconnect(), acp_pick_mode(), acp_pick_model(), acp_pick_session(), acp_switch_pane(), create_acp_buffer(), resolve_permission() (+543 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (313): package(), syntax_language(), capture_mappings(), jsx_syntax_language(), package(), syntax_language(), package(), syntax_language() (+305 more)

### Community 2 - "Community 2"
Cohesion: 0.02
Nodes (299): append_streamed_command_header(), diagnostic_line_spans_for_diagnostics(), apply_git_view(), handle_git_view_chord(), active_theme_state_path(), cycle_runtime_pane(), default_error_log_path(), default_typing_profile_log_path() (+291 more)

### Community 3 - "Community 3"
Cohesion: 0.02
Nodes (108): close_acp_workspace_buffers(), maybe_open_slash_completion(), clear_key_sequence(), active_buffer_event_context(), active_lsp_workspace_loaded(), active_runtime_buffer(), active_runtime_surface(), active_window_id() (+100 more)

### Community 4 - "Community 4"
Cohesion: 0.01
Nodes (170): active_parameter_label(), apply_command_environment(), apply_windows_fnm_environment(), build_lsp_command(), char_to_byte_offset(), cleanup_unused_sessions(), client_capabilities(), client_capabilities_enable_window_work_done_progress() (+162 more)

### Community 5 - "Community 5"
Cohesion: 0.02
Nodes (242): browser_buffer_layout(), render_browser_buffer_body(), covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_severity_rank(), diagnostic_underlines_for_segment(), DiagnosticLineSpan (+234 more)

### Community 6 - "Community 6"
Cohesion: 0.02
Nodes (258): oil_directory_line_spans(), active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), apply_git_status_snapshot(), build_git_fringe_snapshot(), build_git_summary_snapshot() (+250 more)

### Community 7 - "Community 7"
Cohesion: 0.02
Nodes (149): diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), temp_dir(), LanguageConfiguration, additional_highlight_languages_merge_spans(), aligned_indent_column() (+141 more)

### Community 8 - "Community 8"
Cohesion: 0.01
Nodes (173): packages(), syntax_languages(), all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, ancestor_contexts_for_cursor(), AncestorContextBufferKey (+165 more)

### Community 9 - "Community 9"
Cohesion: 0.01
Nodes (134): main(), parse_symbol_line(), apply_directory_edit_actions(), apply_directory_edit_queue(), apply_directory_state(), diff_directory_lines(), directory_cd_from_cursor(), directory_edit_actions() (+126 more)

### Community 10 - "Community 10"
Cohesion: 0.03
Nodes (58): ClipboardContext, read_system_clipboard(), register_clipboard_context(), with_clipboard_util(), yank_from_clipboard_text(), create_workspace_file_from_query(), advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline() (+50 more)

### Community 11 - "Community 11"
Cohesion: 0.03
Nodes (111): acp_connected(), acp_cycle_mode(), acp_insert_slash_command(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+103 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (50): abi_language_server_spec_round_trips_workspace_configuration(), AbiWorkspaceConfigurationNumber, WorkspaceConfigurationValue, contains_wildcards(), csharp_language_server(), dev_extension_server(), Diagnostic, DiagnosticSeverity (+42 more)

### Community 13 - "Community 13"
Cohesion: 0.02
Nodes (132): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+124 more)

### Community 14 - "Community 14"
Cohesion: 0.03
Nodes (89): file_open_detail(), resolve_font_path(), resolve_font_request(), is_pdf_path(), latex_escape_text(), load_pdf_buffer_state(), open_pdf_workspace_file(), pdf_buffer_lines() (+81 more)

### Community 15 - "Community 15"
Cohesion: 0.02
Nodes (44): CommandLineCompletionState, CommandLineOverlay, AutocompleteWorkerState, VimSearchWorkerState, append_lines(), cube_color_component(), default_terminal_index_color(), default_terminal_named_color() (+36 more)

### Community 16 - "Community 16"
Cohesion: 0.03
Nodes (44): apply_command_environment(), apply_windows_fnm_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), codelldb(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult (+36 more)

### Community 17 - "Community 17"
Cohesion: 0.02
Nodes (34): AcpClient, AutocompleteProvider, AutocompleteProviderItem, GhostTextContext, GhostTextLine, GitStatusPrefix, HoverProvider, HoverProviderTopic (+26 more)

### Community 18 - "Community 18"
Cohesion: 0.03
Nodes (39): bootstrap(), cargo(), command_palette_items(), CommandPaletteState, CompilationState, DapState, dynamic_user_library_can_wrap_exported_module(), DynamicUserLibrary (+31 more)

### Community 19 - "Community 19"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 20 - "Community 20"
Cohesion: 0.04
Nodes (24): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+16 more)

### Community 21 - "Community 21"
Cohesion: 0.04
Nodes (21): detect_in_progress(), git_available(), GitLogEntry, GitStashEntry, GitStatusError, GitStatusSnapshot, list_repository_files(), parse_header() (+13 more)

### Community 22 - "Community 22"
Cohesion: 0.06
Nodes (11): AbiThemeOption, ThemeOption, amber(), registry_resolves_option_values(), registry_resolves_tokens_from_active_theme(), TerminalRenderRun, Theme, ThemeError (+3 more)

### Community 23 - "Community 23"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 24 - "Community 24"
Cohesion: 0.1
Nodes (38): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+30 more)

### Community 25 - "Community 25"
Cohesion: 0.08
Nodes (26): workspace_project_picker_shows_repo_context_for_worktrees(), compact_project_path(), default_worktree_common_dir(), detect_project_kind(), directory_buffer_reads_and_renames_entries(), DirectoryBuffer, DirectoryEntry, DirectoryEntryKind (+18 more)

### Community 26 - "Community 26"
Cohesion: 0.08
Nodes (16): BindingKey, ChordModifier, duplicate_detection_uses_canonical_chords(), KeyBinding, KeymapError, KeymapRegistry, KeymapScope, KeymapVimMode (+8 more)

### Community 27 - "Community 27"
Cohesion: 0.09
Nodes (15): empty_query_returns_all_items_in_sorted_order(), fuzzy_query_prefers_prefix_and_contiguous_matches(), item(), match_item(), match_term(), PickerItem, PickerMatch, PickerResultOrder (+7 more)

### Community 28 - "Community 28"
Cohesion: 0.07
Nodes (16): AutocompleteEntry, AutocompleteProviderSpec, AutocompleteQuery, AutocompleteRegistry, HoverOverlay, HoverProviderContent, HoverProviderKind, HoverProviderSpec (+8 more)

### Community 29 - "Community 29"
Cohesion: 0.08
Nodes (31): cmake_package_auto_attaches_cmakelists(), cmake_package_auto_attaches_extension(), cmake_package_metadata(), cmake_package_no_formatter(), cmake_syntax_language_metadata(), package(), syntax_language(), GrammarSourceSpec (+23 more)

### Community 30 - "Community 30"
Cohesion: 0.06
Nodes (25): BlockInsertState, BlockSelection, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch, MulticursorState (+17 more)

### Community 31 - "Community 31"
Cohesion: 0.1
Nodes (31): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_host_viewport_rect(), browser_surface_buffer_at_point(), browser_sync_plan() (+23 more)

### Community 32 - "Community 32"
Cohesion: 0.12
Nodes (1): InputField

### Community 33 - "Community 33"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 34 - "Community 34"
Cohesion: 0.13
Nodes (24): append_streamed_command_error(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands(), run_streamed_command(), stream_command_output() (+16 more)

### Community 35 - "Community 35"
Cohesion: 0.09
Nodes (31): Debug Adapters, Editor Core Crates Layer, Graphics and Platform Layer, Language Servers, Plugin Packages Layer, Runtime and Systems Layer, User Interface Layer, Version Control (+23 more)

### Community 36 - "Community 36"
Cohesion: 0.13
Nodes (12): compose(), compose_includes_filetype_and_modified_icon(), compose_includes_git_segment(), compose_includes_lsp_diagnostic_counts(), compose_includes_macro_recording_register(), compose_joins_the_default_user_segments(), compose_skips_empty_optional_segments(), GitStatuslineInfo (+4 more)

### Community 37 - "Community 37"
Cohesion: 0.2
Nodes (9): compile_command_emits_run_command_hook(), compile_package_binds_f5_keybinding(), compile_package_exports_compile_and_recompile_commands(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only(), parse_error_location_handles_rust_arrow_prefix() (+1 more)

### Community 38 - "Community 38"
Cohesion: 0.2
Nodes (7): AutocompleteProviderConfig, backends(), hook_command(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_prioritize_lsp_over_calculator_over_buffer()

### Community 39 - "Community 39"
Cohesion: 0.22
Nodes (1): ServiceRegistry

### Community 40 - "Community 40"
Cohesion: 0.22
Nodes (4): ShellConfig, ShellError, ShellSummary, TypingProfileSummary

### Community 41 - "Community 41"
Cohesion: 0.42
Nodes (9): Lightning Bolt Glyph, Lightning Bolt Monogram, Rounded App Icon Background, Rounded-square badge, Stylized V accent, Volt App Icon, Volt, Volt Logo (+1 more)

### Community 42 - "Community 42"
Cohesion: 0.29
Nodes (7): Validation Workflow, Operator Validation Workflows, Shell and Bootstrap Entry Points, User Library Build and Smoke Test, Compiled Customization Layer, Project Search Roots, Global Theme Configuration

### Community 43 - "Community 43"
Cohesion: 0.47
Nodes (6): Lightning Motif, Volt Product Identity, V-Shaped Lightning Bolt Icon, V-Shaped Mark, Volt Logo, Volt Wordmark

### Community 44 - "Community 44"
Cohesion: 0.53
Nodes (6): Blurred Code Editor Backdrop, Volt Banner Graphic, Volt Brand Banner, Volt Lightning Bolt Logo, Volt Logo, Volt Wordmark

### Community 45 - "Community 45"
Cohesion: 1.0
Nodes (1): Color

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 62 - "Community 62"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 63 - "Community 63"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 64 - "Community 64"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (2): Layered Programmable Core, Volt Editor Project

### Community 95 - "Community 95"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Ambiguous Edges - Review These
- `Volt` → `Stylized V accent`  [AMBIGUOUS]
  docs\assets\logo.svg · relation: references

## Knowledge Gaps
- **349 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+344 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 32`** (34 nodes): `InputField`, `.backspace()`, `.byte_index_for_char()`, `.cursor_char()`, `.cursor_line_col()`, `.cursor_line_col_with_starts()`, `.cursor_point()`, `.cursor_visual_row_col()`, `.delete_forward()`, `.delete_range()`, `.delete_selection()`, `.hint()`, `.insert_text()`, `.line_col_for_char()`, `.line_len_for()`, `.line_starts()`, `.move_down()`, `.move_left()`, `.move_line_end()`, `.move_line_start()`, `.move_right()`, `.move_up()`, `.new()`, `.placeholder()`, `.prompt()`, `.selected_char_range()`, `.selected_text()`, `.selection_visual_ranges()`, `.set_text()`, `.start_selection()`, `.text_line_count()`, `.visual_line_count()`, `.visual_row_col_for_cursor()`, `.wrapped_visual_rows()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 45`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (2 nodes): `Layered Programmable Core`, `Volt Editor Project`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 95`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Volt` and `Stylized V accent`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `syntax_language()` connect `Community 29` to `Community 1`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 13` to `Community 29`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 29` to `Community 13`?**
  _High betweenness centrality (0.052) - this node is a cross-community bridge._
- **Are the 115 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 115 INFERRED edges - model-reasoned connections that need verification._
- **Are the 52 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 52 INFERRED edges - model-reasoned connections that need verification._
- **Are the 82 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 82 INFERRED edges - model-reasoned connections that need verification._