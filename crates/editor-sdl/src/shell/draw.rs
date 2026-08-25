//! Shared draw-argument groups so render helpers stay under clippy's arity cap.

use super::*;

#[derive(Clone, Copy, Debug)]
pub(super) struct CellMetrics {
    pub cell_width: i32,
    pub line_height: i32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct TextMetrics {
    pub cell_width: i32,
    pub line_height: i32,
    pub ascent: i32,
}

impl TextMetrics {
    pub const fn cells(self) -> CellMetrics {
        CellMetrics {
            cell_width: self.cell_width,
            line_height: self.line_height,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WindowSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FramePulse {
    pub now: Instant,
    pub typing_active: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ShellFrameView<'a> {
    pub size: WindowSize,
    pub fps_overlay: Option<&'a FpsOverlaySnapshot>,
    pub metrics: TextMetrics,
    pub pulse: FramePulse,
}

#[derive(Clone, Copy)]
pub(super) struct ShellChrome<'a> {
    pub user_library: &'a dyn UserLibrary,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub workspace_name: &'a str,
    pub lsp_server: Option<&'a str>,
    pub lsp_workspace_loaded: bool,
    pub acp_connected: bool,
}

#[derive(Clone, Copy)]
pub(super) struct BufferChrome<'a> {
    pub user_library: &'a dyn UserLibrary,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub workspace_name: &'a str,
    pub lsp_server: Option<&'a str>,
    pub lsp_workspace_loaded: bool,
    pub acp_connected: bool,
    pub git_summary: Option<&'a GitSummarySnapshot>,
}

impl<'a> BufferChrome<'a> {
    pub(super) fn from_shell(
        chrome: &ShellChrome<'a>,
        git_summary: Option<&'a GitSummarySnapshot>,
    ) -> Self {
        Self {
            user_library: chrome.user_library,
            theme_registry: chrome.theme_registry,
            workspace_name: chrome.workspace_name,
            lsp_server: chrome.lsp_server,
            lsp_workspace_loaded: chrome.lsp_workspace_loaded,
            acp_connected: chrome.acp_connected,
            git_summary,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OverlayCardStyle {
    pub radius: u32,
    pub border: Color,
    pub background: Color,
    pub window_effects: WindowEffects,
    pub accent: Option<Color>,
    pub shadow: bool,
}

#[derive(Clone, Copy)]
pub(super) struct OverlayAnchorContext<'a> {
    pub pane_rect: Rect,
    pub user_library: &'a dyn UserLibrary,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub metrics: CellMetrics,
    pub typing_active: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BufferDecorations<'a> {
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
    pub input_mode: InputMode,
    pub multicursor: Option<&'a MulticursorState>,
    pub vim_targets_input: bool,
    pub recording_macro: Option<char>,
    pub typing_active: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandLineSlot<'a> {
    pub input: Option<&'a InputField>,
    pub row_visible: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PaneSlot {
    pub rect: Rect,
    pub active: bool,
}

#[derive(Clone, Copy)]
pub(super) struct BufferDrawRequest<'a> {
    pub buffer: &'a ShellBuffer,
    pub view_state: BufferViewState,
    pub pane: PaneSlot,
    pub decorations: BufferDecorations<'a>,
    pub command_line: CommandLineSlot<'a>,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct WrapCollect {
    pub start_line: usize,
    pub max_rows: usize,
    pub wrap_cols: usize,
    pub indent_size: usize,
    pub scroll_col: usize,
    pub line_wrap: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct ScreenHit {
    pub x: i32,
    pub y: i32,
    pub clamp_body: bool,
    pub typing_active: bool,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MouseClick {
    pub x: i32,
    pub y: i32,
    pub clicks: u8,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct MarkdownPrettyPaintArgs {
    pub visible_start: usize,
    pub visible_end: usize,
    pub visual_selection: Option<VisualSelection>,
    pub input_mode: InputMode,
    pub pane_width_px: u32,
    pub line_height: i32,
}

#[derive(Clone, Copy)]
pub(super) struct PanelPalette<'a> {
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub panel_background: Color,
    pub header_background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub border_color: Color,
    pub active_border: Color,
    pub selection: Color,
    pub yank_flash_color: Color,
    pub cursor: Color,
    pub cursor_roundness: u32,
}

#[derive(Clone, Copy)]
pub(super) struct BufferBodyPalette<'a> {
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub base_background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub border_color: Color,
    pub selection: Color,
    pub yank_flash_color: Color,
    pub cursor: Color,
    pub cursor_roundness: u32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct CommandLinePaint {
    pub window_effects: WindowEffects,
    pub background: Color,
    pub foreground: Color,
    pub muted: Color,
    pub cursor: Color,
    pub cursor_roundness: u32,
    pub chip_radius: u32,
}

#[derive(Clone, Copy)]
pub(super) struct ModelineDraw<'a> {
    pub x: i32,
    pub y: i32,
    pub max_width: u32,
    pub default_color: Color,
    pub apply_tokens: bool,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub user_library: &'a dyn UserLibrary,
    pub acp_connected: bool,
    pub lsp_server_visible: bool,
    pub lsp_workspace_loaded: bool,
    pub connected_color: Color,
    pub cell_width: i32,
    pub line_height: Option<i32>,
}

#[derive(Clone, Copy)]
pub(super) struct ModelineSideDraw<'a> {
    pub x: i32,
    pub y: i32,
    pub max_width: u32,
    pub default_color: Color,
    pub apply_tokens: bool,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub icon_colors: &'a [(&'a str, Color)],
    pub highlighted_icons: &'a [&'a str],
    pub cell_width: i32,
    pub gap_width: u32,
    pub chip_height: u32,
    pub preserve_end: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TextPanelDraw<'a> {
    pub text: &'a TextBuffer,
    pub syntax_lines: Option<&'a IndexedSyntaxLines>,
    pub scroll_row: usize,
    pub cursor_point: Option<TextPoint>,
    pub pane_active: bool,
    pub pane_layout: TextPaneLayout,
    pub title: &'a str,
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
    pub input_mode: InputMode,
}

#[derive(Clone, Copy)]
pub(super) struct InputPanelDraw<'a> {
    pub input: &'a InputField,
    pub pane_active: bool,
    pub pane_layout: TextPaneLayout,
    pub input_mode: InputMode,
    pub window_effects: WindowEffects,
    pub corner_radius: u32,
}

#[derive(Clone, Copy)]
pub(super) struct AcpPaneDraw<'a> {
    pub pane: &'a AcpPaneState,
    pub pane_active: bool,
    pub pane_layout: AcpPaneLayout,
    pub title: &'a str,
    pub shell_active: bool,
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
    pub input_mode: InputMode,
}

#[derive(Clone, Copy)]
pub(super) struct PickerOverlayDraw<'a> {
    pub picker: &'a PickerOverlay,
    pub size: WindowSize,
    pub line_height: i32,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub picker_layout: editor_plugin_api::PickerLayout,
    pub truncate_strategy: editor_plugin_api::PickerTruncateStrategy,
}

#[derive(Clone, Copy)]
pub(super) struct TerminalBufferDraw<'a> {
    pub buffer: &'a ShellBuffer,
    pub terminal_render: &'a TerminalRenderSnapshot,
    pub rect: Rect,
    pub layout: BufferFooterLayout,
    pub active: bool,
    pub input_mode: InputMode,
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
}

pub(super) struct TerminalStatusline {
    pub text: String,
    pub active: Color,
    pub inactive: Color,
}

#[derive(Clone, Copy)]
pub(super) struct TerminalCursorDraw<'a> {
    pub text_x: i32,
    pub body_y: i32,
    pub cursor: &'a editor_terminal::TerminalCursorSnapshot,
    pub shape: editor_terminal::TerminalCursorShape,
    pub cursor_color: Color,
    pub text_override_color: Color,
    pub cursor_roundness: u32,
}

#[derive(Clone, Copy)]
pub(super) struct BrowserSyncView<'a> {
    pub runtime_popup: Option<&'a RuntimePopupSnapshot>,
    pub user_library: &'a dyn UserLibrary,
    pub size: WindowSize,
    pub metrics: CellMetrics,
    pub now: Instant,
}

#[derive(Clone, Copy)]
pub(super) struct BufferTextRun<'a> {
    pub x: i32,
    pub y: i32,
    pub line: &'a str,
    pub segment: LineWrapSegment,
    pub char_map: &'a LineCharMap,
    pub line_syntax_spans: Option<&'a [LineSyntaxSpan]>,
    pub default_color: Color,
    pub cell_width: i32,
}

#[derive(Clone, Copy)]
pub(super) struct CursorOverlayQuery<'a> {
    pub x: i32,
    pub line: &'a str,
    pub char_map: &'a LineCharMap,
    pub segment: LineWrapSegment,
    pub line_index: usize,
    pub cursor: TextPoint,
    pub color: Option<Color>,
    pub cell_width: i32,
}

#[derive(Clone, Copy)]
pub(super) struct DiagnosticUnderlineDraw<'a> {
    pub diagnostics: &'a [DiagnosticLineSpan],
    pub syntax_spans: Option<&'a [LineSyntaxSpan]>,
    pub char_map: &'a LineCharMap,
    pub segment_x: i32,
    pub y: i32,
    pub line_len: usize,
    pub segment: LineWrapSegment,
    pub metrics: CellMetrics,
    pub theme_registry: Option<&'a ThemeRegistry>,
}

pub(super) struct ScrollbarPaint {
    pub pane_rect: Rect,
    pub body_y: i32,
    pub visible_rows: usize,
    pub line_height: i32,
    pub scroll_row: usize,
    pub max_scroll: usize,
    pub color: Color,
    pub window_effects: WindowEffects,
}

#[derive(Clone, Copy)]
pub(super) struct AcpPrefixDraw<'a> {
    pub x: i32,
    pub y: i32,
    pub segments: &'a [AcpRenderedSegment],
    pub spinner_frame: &'a str,
    pub theme_registry: Option<&'a ThemeRegistry>,
    pub foreground: Color,
    pub muted: Color,
    pub accent: Color,
    pub cell_width: i32,
}

pub(super) struct ThemeRuntimeSlots<'a, 'ttf, 'texture> {
    pub theme_settings: &'a mut ThemeRuntimeSettings,
    pub fonts: &'a mut FontSet<'ttf>,
    pub font_path: &'a mut PathBuf,
    pub text_texture_cache: &'a mut TextTextureCache<'texture>,
    pub line_height: &'a mut usize,
    pub ascent: &'a mut i32,
    pub cell_width: &'a mut i32,
}

#[derive(Clone, Copy)]
pub(super) struct CommandLineOverlayDraw<'a> {
    pub input: Option<&'a InputField>,
    pub rect: Rect,
    pub layout: BufferFooterLayout,
    pub active: bool,
    pub input_mode: InputMode,
    pub paint: CommandLinePaint,
    pub metrics: CellMetrics,
}

#[derive(Clone, Copy)]
pub(super) struct PluginSectionDraw<'a> {
    pub buffer: &'a ShellBuffer,
    pub view_state: BufferViewState,
    pub pane: PaneSlot,
    pub layout: BufferFooterLayout,
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
    pub input_mode: InputMode,
}

#[derive(Clone, Copy)]
pub(super) struct BrowserBufferDraw<'a> {
    pub buffer: &'a ShellBuffer,
    pub rect: Rect,
    pub layout: BufferFooterLayout,
    pub active: bool,
    pub input_mode: InputMode,
}

#[derive(Clone, Copy)]
pub(super) struct AcpBufferDraw<'a> {
    pub buffer: &'a ShellBuffer,
    pub rect: Rect,
    pub layout: BufferFooterLayout,
    pub active: bool,
    pub visual_selection: Option<VisualSelection>,
    pub yank_flash: Option<VisualSelection>,
    pub input_mode: InputMode,
}
