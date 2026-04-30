# Graph Report - volt  (2026-04-30)

## Corpus Check
- 164 files · ~465,760 words
- Verdict: corpus is large enough that graph structure adds value.

## Summary
- 5865 nodes · 15994 edges · 68 communities detected
- Extraction: 86% EXTRACTED · 14% INFERRED · 0% AMBIGUOUS · INFERRED: 2238 edges (avg confidence: 0.8)
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
- [[_COMMUNITY_Community 96|Community 96]]

## God Nodes (most connected - your core abstractions)
1. `shell_ui_mut()` - 245 edges
2. `register_shell_hooks()` - 207 edges
3. `ShellBuffer` - 205 edges
4. `shell_ui()` - 154 edges
5. `shell_buffer()` - 127 edges
6. `shell_buffer_mut()` - 122 edges
7. `TextBuffer` - 103 edges
8. `ShellUiState` - 100 edges
9. `active_shell_buffer_id()` - 93 edges
10. `ShellState` - 87 edges

## Surprising Connections (you probably didn't know these)
- `Declarative Package Metadata` --semantically_similar_to--> `PluginPackage`  [INFERRED] [semantically similar]
  CLAUDE.md → docs\user-packages.md
- `overlay_window_surface_opacity()` --calls--> `overlay_window_surface_color()`  [INFERRED]
  crates\editor-sdl\src\window_effects.rs → crates\editor-sdl\src\shell\render.rs
- `acp_permission_picker_submitted()` --calls--> `register_shell_hooks()`  [INFERRED]
  crates\editor-sdl\src\shell\acp.rs → crates\editor-sdl\src\shell\mod.rs
- `shell_ui()` --calls--> `hover_scroll_offset()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\tests.rs
- `active_shell_buffer_mut()` --calls--> `pdf_previous_page()`  [INFERRED]
  crates\editor-sdl\src\shell\mod.rs → crates\editor-sdl\src\shell\pdf.rs

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
Nodes (616): acp_complete_slash(), acp_insert_slash_command(), acp_pick_model(), acp_pick_session(), acp_switch_pane(), create_acp_buffer(), maybe_open_slash_completion(), refresh_acp_input_hint() (+608 more)

### Community 1 - "Community 1"
Cohesion: 0.01
Nodes (354): browser_host_viewport_rect(), browser_sync_plan(), browser_viewport_rect(), browser_viewport_rect_rect(), append_streamed_command_header(), diagnostic_line_spans_for_diagnostics(), apply_directory_state(), update_directory_state() (+346 more)

### Community 2 - "Community 2"
Cohesion: 0.01
Nodes (294): package(), syntax_language(), diff_syntax_language_metadata(), diff_syntax_language_preserves_diff_capture_theme_tokens(), syntax_language(), syntax_language(), capture_mappings(), jsx_syntax_language() (+286 more)

### Community 3 - "Community 3"
Cohesion: 0.01
Nodes (173): AbiWorkspaceConfigurationValue, WorkspaceConfigurationValue, active_parameter_label(), apply_command_environment(), apply_windows_fnm_environment(), build_lsp_command(), char_to_byte_offset(), cleanup_unused_sessions() (+165 more)

### Community 4 - "Community 4"
Cohesion: 0.02
Nodes (164): temp_dir(), vim_search_entries_trim_whitespace_from_labels(), LanguageConfiguration, additional_highlight_languages_merge_spans(), aligned_indent_column(), ancestor_contexts_include_named_nodes_up_to_the_root(), ancestor_contexts_parse_session_matches_cold_query_after_edits(), append_query_source() (+156 more)

### Community 5 - "Community 5"
Cohesion: 0.01
Nodes (187): packages(), syntax_languages(), all_symbols(), find_symbol(), IconFontCategory, IconFontSymbol, dynamic_user_library_can_wrap_exported_module(), ancestor_contexts_for_cursor() (+179 more)

### Community 6 - "Community 6"
Cohesion: 0.02
Nodes (251): oil_directory_line_spans(), active_git_status_command_context(), ActiveBufferEventContext, ActiveLspBufferContext, apply_git_fringe_hunk(), apply_git_status_snapshot(), apply_git_view(), build_git_fringe_snapshot() (+243 more)

### Community 7 - "Community 7"
Cohesion: 0.02
Nodes (54): diagnostic_severity_rank(), acp_pane_content_rows(), acp_rendered_line_row_count(), acp_rendered_text_segments(), AcpPaneState, AutocompleteWorkerState, buffer_kind_label(), evaluate_active_plugin_buffer() (+46 more)

### Community 8 - "Community 8"
Cohesion: 0.03
Nodes (188): browser_buffer_layout(), render_browser_buffer_body(), covering_syntax_span_for_range(), diagnostic_color(), diagnostic_columns_for_line(), diagnostic_underlines_for_segment(), DiagnosticLineSpan, DiagnosticUnderlineSpan (+180 more)

### Community 9 - "Community 9"
Cohesion: 0.01
Nodes (131): main(), parse_symbol_line(), apply_directory_edit_actions(), apply_directory_edit_queue(), diff_directory_lines(), directory_cd_from_cursor(), directory_edit_actions(), directory_edit_lines() (+123 more)

### Community 10 - "Community 10"
Cohesion: 0.02
Nodes (72): workspace_project_picker_shows_repo_context_for_worktrees(), abi_language_server_spec_round_trips_workspace_configuration(), compact_project_path(), contains_wildcards(), csharp_language_server(), default_worktree_common_dir(), detect_project_kind(), dev_extension_server() (+64 more)

### Community 11 - "Community 11"
Cohesion: 0.03
Nodes (56): advance_point_by_text(), around_word_ranges_at_line_end_exclude_newline(), big_word_backward_end_and_match_pair_cover_quickref_motion_slice(), BufferStats, delimited_and_tag_ranges_cover_quickref_objects(), delimited_ranges_cover_quotes_and_brackets(), delimiter_partner(), detect_preferred_line_ending() (+48 more)

### Community 12 - "Community 12"
Cohesion: 0.03
Nodes (113): acp_connected(), acp_cycle_mode(), acp_disconnect(), acp_load_session(), acp_new_session(), acp_open_permission_request(), acp_permission_approve(), acp_permission_deny() (+105 more)

### Community 13 - "Community 13"
Cohesion: 0.02
Nodes (66): apply_command_environment(), apply_windows_fnm_environment(), build_job_command(), build_job_command_keeps_fnm_path_ahead_of_explicit_path(), codelldb(), command_candidate_names(), compilation_runner_marks_jobs_as_compilation(), CompilationResult (+58 more)

### Community 14 - "Community 14"
Cohesion: 0.02
Nodes (132): bash_package_auto_attaches_all_extensions(), bash_package_metadata(), bash_package_registers_formatter(), bash_syntax_language_metadata(), package(), syntax_language(), package(), syntax_language() (+124 more)

### Community 15 - "Community 15"
Cohesion: 0.03
Nodes (42): CommandLineCompletionState, CommandLineOverlay, append_lines(), cube_color_component(), default_terminal_index_color(), default_terminal_named_color(), live_terminal_session_spawns_and_terminates(), LiveTerminalConfig (+34 more)

### Community 16 - "Community 16"
Cohesion: 0.02
Nodes (34): AcpClient, AutocompleteProvider, AutocompleteProviderItem, GhostTextContext, GhostTextLine, GitStatusPrefix, HoverProvider, HoverProviderTopic (+26 more)

### Community 17 - "Community 17"
Cohesion: 0.03
Nodes (38): bootstrap(), cargo(), command_palette_items(), CommandPaletteState, CompilationState, DapState, DynamicUserLibrary, EventLog (+30 more)

### Community 18 - "Community 18"
Cohesion: 0.04
Nodes (8): Buffer, BufferKind, EditorModel, ModelError, Pane, Popup, Window, Workspace

### Community 19 - "Community 19"
Cohesion: 0.04
Nodes (24): CommandDefinition, CommandError, CommandRegistry, CommandSource, RegisteredCommand, HookBus, HookDefinition, HookError (+16 more)

### Community 20 - "Community 20"
Cohesion: 0.05
Nodes (40): AbiStringPair, autocomplete_items(), autocomplete_provider(), buffer_sections(), calculator_autocomplete_provider_scopes_manual_items_to_calculator_buffers(), calculator_buffer_sections_start_with_single_output_row(), calculator_evaluate_command_emits_generic_plugin_evaluate_hook(), calculator_hover_provider_exports_function_and_constant_topics() (+32 more)

### Community 21 - "Community 21"
Cohesion: 0.06
Nodes (11): AbiThemeOption, ThemeOption, amber(), registry_resolves_option_values(), registry_resolves_tokens_from_active_theme(), TerminalRenderRun, Theme, ThemeError (+3 more)

### Community 22 - "Community 22"
Cohesion: 0.07
Nodes (22): browser_additional_args(), browser_additional_args_from_env(), browser_additional_args_from_env_appends_custom_args(), browser_additional_args_from_env_appends_web_security_bypass(), browser_host_event_for_ipc(), browser_host_ipc_event_ignores_unknown_messages(), browser_host_ipc_event_routes_focus_parent_requests(), browser_host_ipc_event_routes_open_devtools_requests() (+14 more)

### Community 23 - "Community 23"
Cohesion: 0.1
Nodes (38): hidden_window_startup_smoke_supports_window_effects(), apply_blur(), apply_window_blur(), apply_window_effects(), apply_window_effects_ignores_native_window_opacity_to_keep_text_opaque(), apply_window_effects_still_calls_native_blur_backend_when_requested(), apply_window_effects_to_target(), clear_blur() (+30 more)

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
Cohesion: 0.06
Nodes (25): BlockInsertState, BlockSelection, FormatterRegistry, FormatterSpec, InputMode, LastFind, LastSearch, MulticursorState (+17 more)

### Community 29 - "Community 29"
Cohesion: 0.11
Nodes (2): AcpBufferState, InputField

### Community 30 - "Community 30"
Cohesion: 0.09
Nodes (9): render_lines_respects_collapsed_state(), render_section(), Section, SectionAction, SectionCollapseState, SectionItem, SectionRenderLine, SectionRenderLineKind (+1 more)

### Community 31 - "Community 31"
Cohesion: 0.11
Nodes (27): apply_browser_location_updates(), apply_browser_page_load_state(), browser_buffer_display_name(), browser_display_url(), browser_display_url_prefers_requested_navigation(), browser_surface_buffer_at_point(), browser_url_candidates(), browser_url_prefix_len() (+19 more)

### Community 32 - "Community 32"
Cohesion: 0.14
Nodes (23): append_streamed_command_error(), continue_streamed_command_popup(), drain_completed_output_lines(), open_streamed_command_popup(), push_streamed_command_update(), refresh_pending_streamed_commands(), run_streamed_command(), stream_command_output() (+15 more)

### Community 33 - "Community 33"
Cohesion: 0.15
Nodes (22): buffer_is_terminal(), close_terminal_buffer(), close_terminal_buffers_for_workspace(), CursorTextOverlay, ensure_terminal_session(), refresh_pending_terminal(), resize_active_terminal_session(), scroll_active_terminal_view() (+14 more)

### Community 34 - "Community 34"
Cohesion: 0.09
Nodes (31): Debug Adapters, Editor Core Crates Layer, Graphics and Platform Layer, Language Servers, Plugin Packages Layer, Runtime and Systems Layer, User Interface Layer, Version Control (+23 more)

### Community 35 - "Community 35"
Cohesion: 0.11
Nodes (19): centered_rect(), default_font_candidates(), DrawCommand, find_font_by_name(), find_system_monospace_font(), font_data_matches_name(), font_metadata_matching_accepts_family_names(), font_name_matches() (+11 more)

### Community 36 - "Community 36"
Cohesion: 0.2
Nodes (9): compile_command_emits_run_command_hook(), compile_package_binds_f5_keybinding(), compile_package_exports_compile_and_recompile_commands(), package(), parse_error_location(), parse_error_location_handles_path_line_col(), parse_error_location_handles_path_line_only(), parse_error_location_handles_rust_arrow_prefix() (+1 more)

### Community 37 - "Community 37"
Cohesion: 0.2
Nodes (7): AutocompleteProviderConfig, backends(), hook_command(), package(), package_exports_commands_and_insert_keybindings(), providers(), providers_prioritize_lsp_over_calculator_over_buffer()

### Community 38 - "Community 38"
Cohesion: 0.2
Nodes (2): FontSet<'ttf>, validate_bundled_icon_fonts()

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
Cohesion: 0.32
Nodes (5): keycode_name_token(), keydown_chord_token(), KeydownChordToken, normalize_named_key_token(), shifted_printable_character()

### Community 43 - "Community 43"
Cohesion: 0.29
Nodes (7): Validation Workflow, Operator Validation Workflows, Shell and Bootstrap Entry Points, User Library Build and Smoke Test, Compiled Customization Layer, Project Search Roots, Global Theme Configuration

### Community 44 - "Community 44"
Cohesion: 0.53
Nodes (6): Blurred Code Editor Backdrop, Volt Banner Graphic, Volt Brand Banner, Volt Lightning Bolt Logo, Volt Logo, Volt Wordmark

### Community 45 - "Community 45"
Cohesion: 0.47
Nodes (6): Lightning Motif, Volt Product Identity, V-Shaped Lightning Bolt Icon, V-Shaped Mark, Volt Logo, Volt Wordmark

### Community 46 - "Community 46"
Cohesion: 1.0
Nodes (1): Color

### Community 47 - "Community 47"
Cohesion: 1.0
Nodes (1): LanguageServerRootStrategy

### Community 48 - "Community 48"
Cohesion: 1.0
Nodes (1): OilSortMode

### Community 49 - "Community 49"
Cohesion: 1.0
Nodes (1): PdfOpenMode

### Community 50 - "Community 50"
Cohesion: 1.0
Nodes (1): OilKeyAction

### Community 51 - "Community 51"
Cohesion: 1.0
Nodes (1): GitStatusPrefix

### Community 52 - "Community 52"
Cohesion: 1.0
Nodes (1): AutocompleteProviderItem

### Community 53 - "Community 53"
Cohesion: 1.0
Nodes (1): AutocompleteProvider

### Community 54 - "Community 54"
Cohesion: 1.0
Nodes (1): HoverProviderTopic

### Community 55 - "Community 55"
Cohesion: 1.0
Nodes (1): HoverProvider

### Community 56 - "Community 56"
Cohesion: 1.0
Nodes (1): AcpClient

### Community 57 - "Community 57"
Cohesion: 1.0
Nodes (1): WorkspaceRoot

### Community 58 - "Community 58"
Cohesion: 1.0
Nodes (1): TerminalConfig

### Community 59 - "Community 59"
Cohesion: 1.0
Nodes (1): LigatureConfig

### Community 60 - "Community 60"
Cohesion: 1.0
Nodes (1): LspDiagnosticsInfo

### Community 61 - "Community 61"
Cohesion: 1.0
Nodes (1): OilDefaults

### Community 62 - "Community 62"
Cohesion: 1.0
Nodes (1): OilKeybindings

### Community 63 - "Community 63"
Cohesion: 1.0
Nodes (1): DirectoryEntryKind

### Community 64 - "Community 64"
Cohesion: 1.0
Nodes (1): IconFontCategory

### Community 65 - "Community 65"
Cohesion: 1.0
Nodes (1): IconFontSymbol

### Community 66 - "Community 66"
Cohesion: 1.0
Nodes (2): Layered Programmable Core, Volt Editor Project

### Community 96 - "Community 96"
Cohesion: 1.0
Nodes (1): UserLibraryModuleRef

## Ambiguous Edges - Review These
- `Volt` → `Stylized V accent`  [AMBIGUOUS]
  docs\assets\logo.svg · relation: references

## Knowledge Gaps
- **349 isolated node(s):** `WordKind`, `BufferStats`, `TextEdit`, `TextByteChunkSource`, `TextByteChunks` (+344 more)
  These have ≤1 connection - possible missing edges or undocumented components.
- **Thin community `Community 29`** (36 nodes): `AcpBufferState`, `.new()`, `InputField`, `.backspace()`, `.byte_index_for_char()`, `.cursor_char()`, `.cursor_line_col()`, `.cursor_line_col_with_starts()`, `.cursor_point()`, `.cursor_visual_row_col()`, `.delete_forward()`, `.delete_range()`, `.delete_selection()`, `.hint()`, `.insert_text()`, `.line_col_for_char()`, `.line_len_for()`, `.line_starts()`, `.move_down()`, `.move_left()`, `.move_line_end()`, `.move_line_start()`, `.move_right()`, `.move_up()`, `.new()`, `.placeholder()`, `.prompt()`, `.selected_char_range()`, `.selected_text()`, `.selection_visual_ranges()`, `.set_placeholder()`, `.set_text()`, `.start_selection()`, `.text_line_count()`, `.visual_row_col_for_cursor()`, `.delete_range()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 38`** (11 nodes): `FontSet<'ttf>`, `.icon_font()`, `.icon_font_index_for_char()`, `.icon_fonts()`, `.ligatures_enabled()`, `.new()`, `.prefers_icon_font()`, `.primary_pixel_size()`, `.primary_raster_font()`, `.primary_shape_face()`, `validate_bundled_icon_fonts()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 39`** (10 nodes): `services.rs`, `ServiceRegistry`, `.contains()`, `.get()`, `.get_mut()`, `.insert()`, `.is_empty()`, `.len()`, `.new()`, `.remove()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 46`** (2 nodes): `Color`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 47`** (2 nodes): `LanguageServerRootStrategy`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 48`** (2 nodes): `OilSortMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 49`** (2 nodes): `PdfOpenMode`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 50`** (2 nodes): `OilKeyAction`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 51`** (2 nodes): `GitStatusPrefix`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 52`** (2 nodes): `AutocompleteProviderItem`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 53`** (2 nodes): `AutocompleteProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 54`** (2 nodes): `HoverProviderTopic`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 55`** (2 nodes): `HoverProvider`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 56`** (2 nodes): `AcpClient`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 57`** (2 nodes): `WorkspaceRoot`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 58`** (2 nodes): `TerminalConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 59`** (2 nodes): `LigatureConfig`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 60`** (2 nodes): `LspDiagnosticsInfo`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 61`** (2 nodes): `OilDefaults`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 62`** (2 nodes): `OilKeybindings`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 63`** (2 nodes): `DirectoryEntryKind`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 64`** (2 nodes): `IconFontCategory`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 65`** (2 nodes): `IconFontSymbol`, `.from()`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 66`** (2 nodes): `Layered Programmable Core`, `Volt Editor Project`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.
- **Thin community `Community 96`** (1 nodes): `UserLibraryModuleRef`
  Too small to be a meaningful cluster - may be noise or needs more connections extracted.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **What is the exact relationship between `Volt` and `Stylized V accent`?**
  _Edge tagged AMBIGUOUS (relation: references) - confidence is low._
- **Why does `syntax_language()` connect `Community 27` to `Community 2`?**
  _High betweenness centrality (0.063) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 14` to `Community 27`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._
- **Why does `package()` connect `Community 27` to `Community 14`?**
  _High betweenness centrality (0.061) - this node is a cross-community bridge._
- **Are the 115 inferred relationships involving `shell_ui_mut()` (e.g. with `create_acp_buffer()` and `focus_acp_buffer()`) actually correct?**
  _`shell_ui_mut()` has 115 INFERRED edges - model-reasoned connections that need verification._
- **Are the 51 inferred relationships involving `register_shell_hooks()` (e.g. with `terminal_buffer_cursor_point_for_normal_mode()` and `apply_directory_edit_queue()`) actually correct?**
  _`register_shell_hooks()` has 51 INFERRED edges - model-reasoned connections that need verification._
- **Are the 82 inferred relationships involving `shell_ui()` (e.g. with `open_acp_client_with_config()` and `maybe_open_slash_completion()`) actually correct?**
  _`shell_ui()` has 82 INFERRED edges - model-reasoned connections that need verification._