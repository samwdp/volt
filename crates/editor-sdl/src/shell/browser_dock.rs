use editor_core::BufferId;
use editor_plugin_api::WorkspaceDockSide;
use editor_render::PixelRect;
use editor_theme::ThemeRegistry;
use sdl3::pixels::Color;

use super::*;

const BROWSER_DOCK_CARD_LINE_COUNT: u32 = 2;
const BROWSER_DOCK_CARD_GAP_LINES: u32 = 1;
const BROWSER_DOCK_SIDE: WorkspaceDockSide = WorkspaceDockSide::Left;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum BrowserDockEntryKind {
    Buffer,
    Tab,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BrowserDockEntry {
    pub(super) buffer_id: BufferId,
    pub(super) tab_id: Option<u64>,
    pub(super) kind: BrowserDockEntryKind,
    pub(super) title: String,
    pub(super) detail: String,
    pub(super) logo: &'static str,
    pub(super) active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct BrowserDockLayout {
    pub(super) visible: bool,
    pub(super) side: WorkspaceDockSide,
    pub(super) dock_width: u32,
    pub(super) dock_rect: PixelRect,
}

impl BrowserDockLayout {
    pub(super) fn hidden() -> Self {
        Self {
            visible: false,
            side: BROWSER_DOCK_SIDE,
            dock_width: 0,
            dock_rect: PixelRect::new(0, 0, 0, 0),
        }
    }
}

pub(super) fn browser_dock_visible(ui: &ShellUiState) -> bool {
    ui.browser_dock_open()
}

pub(super) fn browser_dock_width(content_width: u32, cell_width: i32) -> u32 {
    workspace_dock_width(content_width, cell_width).min(
        content_width
            .saturating_sub(cell_width.max(1) as u32)
            .max(cell_width.max(1) as u32),
    )
}

pub(super) fn browser_dock_card_height(line_height: i32) -> u32 {
    let row = line_height.max(1) as u32;
    row.saturating_mul(BROWSER_DOCK_CARD_LINE_COUNT + BROWSER_DOCK_CARD_GAP_LINES)
}

pub(super) fn collect_browser_dock_entries(
    runtime: &EditorRuntime,
) -> Result<Vec<BrowserDockEntry>, String> {
    let ui = shell_ui(runtime)?;
    let workspace_id = runtime
        .model()
        .active_workspace_id()
        .map_err(|error| error.to_string())?;
    let workspace = runtime
        .model()
        .workspace(workspace_id)
        .map_err(|error| error.to_string())?;
    let active_buffer = ui.active_buffer_id();
    let mut entries = Vec::new();
    for runtime_buffer in workspace.buffers() {
        if !matches!(runtime_buffer.kind(), BufferKind::Plugin(kind) if kind == BROWSER_KIND) {
            continue;
        }
        let Some(buffer) = ui.buffer(runtime_buffer.id()) else {
            continue;
        };
        let Some(state) = buffer.browser_state.as_ref() else {
            continue;
        };
        let buffer_active = active_buffer == Some(buffer.id());
        let buffer_detail = browser_display_url(state).unwrap_or("no url").to_owned();
        entries.push(BrowserDockEntry {
            buffer_id: buffer.id(),
            tab_id: None,
            kind: BrowserDockEntryKind::Buffer,
            title: buffer.display_name().to_owned(),
            detail: format!("{} tab(s) · {buffer_detail}", state.tabs.len()),
            logo: BROWSER_BUFFER_LOGO,
            active: buffer_active,
        });
        for tab in &state.tabs {
            let detail = tab
                .requested_url
                .as_deref()
                .or(tab.current_url.as_deref())
                .unwrap_or("about:blank")
                .to_owned();
            entries.push(BrowserDockEntry {
                buffer_id: buffer.id(),
                tab_id: Some(tab.id.0),
                kind: BrowserDockEntryKind::Tab,
                title: format!("  {}", tab.label()),
                detail,
                logo: BROWSER_TAB_LOGO,
                active: buffer_active && tab.id == state.active_tab_id,
            });
        }
    }
    Ok(entries)
}

pub(super) fn browser_dock_entries_for_render(
    runtime: &EditorRuntime,
) -> Result<Vec<BrowserDockEntry>, String> {
    let mut entries = collect_browser_dock_entries(runtime)?;
    let cursor = shell_ui(runtime)?.browser_dock_cursor();
    if let Some(cursor) = cursor.filter(|index| *index < entries.len()) {
        for (index, entry) in entries.iter_mut().enumerate() {
            entry.active = index == cursor;
        }
    }
    Ok(entries)
}

pub(super) fn browser_dock_entry_at_point(
    layout: &BrowserDockLayout,
    entries: &[BrowserDockEntry],
    line_height: i32,
    x: i32,
    y: i32,
) -> Option<usize> {
    if !layout.visible {
        return None;
    }
    let rect = layout.dock_rect;
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    if x < rect.x || x >= right || y < rect.y || y >= bottom {
        return None;
    }
    let card_height = browser_dock_card_height(line_height) as i32;
    if card_height <= 0 {
        return None;
    }
    let index = ((y - rect.y) / card_height) as usize;
    (index < entries.len()).then_some(index)
}

pub(super) fn activate_browser_dock_entry(
    runtime: &mut EditorRuntime,
    entry: &BrowserDockEntry,
) -> Result<(), String> {
    match entry.kind {
        BrowserDockEntryKind::Buffer => focus_shell_buffer(runtime, entry.buffer_id),
        BrowserDockEntryKind::Tab => {
            let tab_id = entry
                .tab_id
                .map(BrowserTabId)
                .ok_or_else(|| "browser dock tab entry is missing tab id".to_owned())?;
            activate_browser_tab(runtime, entry.buffer_id, tab_id)
        }
    }
}

pub(super) fn render_browser_dock(
    target: &mut DrawTarget<'_>,
    layout: &BrowserDockLayout,
    entries: &[BrowserDockEntry],
    theme_registry: Option<&ThemeRegistry>,
    cell_width: i32,
    line_height: i32,
    ascent: i32,
) -> Result<(), ShellError> {
    if !layout.visible {
        return Ok(());
    }
    let window_effects = current_window_effect_settings(theme_registry);
    let base_background = theme_color(theme_registry, "ui.background", Color::RGB(15, 16, 20));
    let is_dark = is_dark_color(base_background);
    let dock_background = theme_color(
        theme_registry,
        "ui.browser-dock.background",
        theme_color(
            theme_registry,
            "ui.workspace-dock.background",
            adjust_color(base_background, if is_dark { 10 } else { -10 }),
        ),
    );
    let foreground = theme_color(
        theme_registry,
        "ui.browser-dock.foreground",
        theme_color(
            theme_registry,
            "ui.workspace-dock.foreground",
            theme_color(
                theme_registry,
                "ui.foreground",
                Color::RGBA(215, 221, 232, 255),
            ),
        ),
    );
    let muted = theme_color(
        theme_registry,
        "ui.browser-dock.muted",
        theme_color(
            theme_registry,
            "ui.workspace-dock.muted",
            blend_color(foreground, dock_background, 0.45),
        ),
    );
    let selection = theme_color(
        theme_registry,
        "ui.browser-dock.selection",
        theme_color(
            theme_registry,
            "ui.workspace-dock.selection",
            theme_color(
                theme_registry,
                "ui.selection",
                adjust_color(dock_background, if is_dark { 36 } else { -36 }),
            ),
        ),
    );
    let accent = theme_color(
        theme_registry,
        "ui.browser-dock.accent",
        theme_color(
            theme_registry,
            "ui.workspace-dock.accent",
            theme_color(theme_registry, "ui.cursor", Color::RGB(80, 140, 220)),
        ),
    );
    let dock_rect = PixelRectToRect::rect(
        layout.dock_rect.x,
        layout.dock_rect.y,
        layout.dock_rect.width,
        layout.dock_rect.height,
    );
    fill_window_surface_rect(target, dock_rect, dock_background, window_effects)?;

    let card_inset = 6i32;
    let card_height = browser_dock_card_height(line_height) as i32;
    let text_x = layout.dock_rect.x + cell_width.max(1) + card_inset;
    let logo_reserve = line_height.max(1) + cell_width.max(1) * 2 + card_inset;
    let max_chars = ((layout.dock_rect.width as i32)
        .saturating_sub(cell_width.max(1) * 2 + card_inset * 2 + logo_reserve)
        / cell_width.max(1))
    .max(4) as usize;
    for (index, entry) in entries.iter().enumerate() {
        let card_y = layout.dock_rect.y + index as i32 * card_height;
        if card_y >= layout.dock_rect.y + layout.dock_rect.height as i32 {
            break;
        }
        let card_rect = PixelRectToRect::rect(
            layout.dock_rect.x + card_inset,
            card_y + 2,
            layout
                .dock_rect
                .width
                .saturating_sub((card_inset * 2) as u32),
            (card_height - 4).max(0) as u32,
        );
        if entry.active {
            fill_rounded_rect_with_left_accent(
                target,
                card_rect,
                shared_corner_radius(theme_registry).min(10),
                selection,
                accent,
                window_effects,
            )?;
        }
        let title = truncate_browser_dock_text(&entry.title, max_chars);
        let detail = truncate_browser_dock_text(&entry.detail, max_chars);
        let baseline = card_y + ascent.max(0);
        draw_text(target, text_x, baseline, &title, foreground)?;
        draw_text(target, text_x, baseline + line_height, &detail, muted)?;
        let logo_side = line_height.max(1) as u32;
        let logo_area_y =
            card_rect.y() + ((card_rect.height() as i32 - logo_side as i32) / 2).max(0);
        if let Some(logo) = super::acp::load_acp_logo(entry.logo)
            && let Some(dest) = super::acp::acp_logo_dest_rect(
                card_rect.x(),
                logo_area_y,
                card_rect.width().saturating_sub(card_inset as u32),
                logo_side,
                &logo,
                0,
            )
        {
            super::acp::draw_acp_logo(target, &logo, dest)?;
        }
    }
    Ok(())
}

fn truncate_browser_dock_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let mut truncated = text
        .chars()
        .take(max_chars.saturating_sub(1))
        .collect::<String>();
    truncated.push('…');
    truncated
}
