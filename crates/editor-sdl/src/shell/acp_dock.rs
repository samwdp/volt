use editor_core::BufferId;
use editor_plugin_api::{UserLibrary, WorkspaceDockSide};
use editor_render::PixelRect;
use editor_theme::ThemeRegistry;
use sdl3::pixels::Color;

use super::*;

const ACP_DOCK_CARD_LINE_COUNT: u32 = 3;
const ACP_DOCK_CARD_GAP_LINES: u32 = 1;

/// ACP dock always opens on the right so it can sit opposite the workspace dock.
const ACP_DOCK_SIDE: WorkspaceDockSide = WorkspaceDockSide::Right;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct AcpDockEntry {
    pub(super) buffer_id: BufferId,
    pub(super) name: String,
    pub(super) session: String,
    pub(super) client: String,
    pub(super) active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct AcpDockLayout {
    pub(super) visible: bool,
    pub(super) side: WorkspaceDockSide,
    pub(super) dock_width: u32,
    pub(super) dock_rect: PixelRect,
}

impl AcpDockLayout {
    pub(super) fn hidden() -> Self {
        Self {
            visible: false,
            side: ACP_DOCK_SIDE,
            dock_width: 0,
            dock_rect: PixelRect::new(0, 0, 0, 0),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ShellDockEntries<'a> {
    pub(super) workspace: &'a [WorkspaceDockEntry],
    pub(super) acp: &'a [AcpDockEntry],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ShellDocksLayout {
    pub(super) workspace: WorkspaceDockLayout,
    pub(super) acp: AcpDockLayout,
    pub(super) content_x: i32,
    pub(super) content_width: u32,
    pub(super) content_height: u32,
}

pub(super) fn acp_dock_visible(ui: &ShellUiState) -> bool {
    ui.acp_dock_open()
}

pub(super) fn acp_dock_width(content_width: u32, cell_width: i32) -> u32 {
    workspace_dock_width(content_width, cell_width).min(
        content_width
            .saturating_sub(cell_width.max(1) as u32)
            .max(cell_width.max(1) as u32),
    )
}

pub(super) fn shell_docks_layout(
    user_library: &dyn UserLibrary,
    ui: &ShellUiState,
    width: u32,
    height: u32,
    cell_width: i32,
) -> ShellDocksLayout {
    let mut workspace = workspace_dock_layout(user_library, ui, width, height, cell_width);
    let mut left = if workspace.visible && workspace.side == WorkspaceDockSide::Left {
        workspace.dock_width
    } else {
        0
    };
    let mut right = if workspace.visible && workspace.side == WorkspaceDockSide::Right {
        workspace.dock_width
    } else {
        0
    };
    let mut acp = AcpDockLayout::hidden();

    if acp_dock_visible(ui) {
        let remaining = width.saturating_sub(left).saturating_sub(right);
        let dock_width = acp_dock_width(remaining.max(width / 5).max(1), cell_width).min(remaining);
        if dock_width > 0 {
            match ACP_DOCK_SIDE {
                WorkspaceDockSide::Left => {
                    acp = AcpDockLayout {
                        visible: true,
                        side: ACP_DOCK_SIDE,
                        dock_width,
                        dock_rect: PixelRect::new(left as i32, 0, dock_width, height),
                    };
                    left = left.saturating_add(dock_width);
                }
                WorkspaceDockSide::Right => {
                    acp = AcpDockLayout {
                        visible: true,
                        side: ACP_DOCK_SIDE,
                        dock_width,
                        dock_rect: PixelRect::new(
                            width.saturating_sub(right).saturating_sub(dock_width) as i32,
                            0,
                            dock_width,
                            height,
                        ),
                    };
                    right = right.saturating_add(dock_width);
                }
            }
        }
    }

    let content_width = width.saturating_sub(left).saturating_sub(right);
    workspace.content_x = left as i32;
    workspace.content_width = content_width;
    workspace.content_height = height;
    ShellDocksLayout {
        workspace,
        acp,
        content_x: left as i32,
        content_width,
        content_height: height,
    }
}

pub(super) fn acp_dock_card_height(line_height: i32) -> u32 {
    let row = line_height.max(1) as u32;
    row.saturating_mul(ACP_DOCK_CARD_LINE_COUNT + ACP_DOCK_CARD_GAP_LINES)
}

pub(super) fn acp_dock_entry_at_point(
    layout: &AcpDockLayout,
    entries: &[AcpDockEntry],
    line_height: i32,
    x: i32,
    y: i32,
) -> Option<BufferId> {
    if !layout.visible {
        return None;
    }
    let rect = layout.dock_rect;
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    if x < rect.x || x >= right || y < rect.y || y >= bottom {
        return None;
    }
    let card_height = acp_dock_card_height(line_height) as i32;
    if card_height <= 0 {
        return None;
    }
    let index = ((y - rect.y) / card_height) as usize;
    entries.get(index).map(|entry| entry.buffer_id)
}

pub(super) fn render_acp_dock(
    target: &mut DrawTarget<'_>,
    layout: &AcpDockLayout,
    entries: &[AcpDockEntry],
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
        "ui.acp-dock.background",
        theme_color(
            theme_registry,
            "ui.workspace-dock.background",
            adjust_color(base_background, if is_dark { 10 } else { -10 }),
        ),
    );
    let foreground = theme_color(
        theme_registry,
        "ui.acp-dock.foreground",
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
        "ui.acp-dock.muted",
        theme_color(
            theme_registry,
            "ui.workspace-dock.muted",
            blend_color(foreground, dock_background, 0.45),
        ),
    );
    let selection = theme_color(
        theme_registry,
        "ui.acp-dock.selection",
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
        "ui.acp-dock.accent",
        theme_color(
            theme_registry,
            "ui.workspace-dock.accent",
            theme_color(theme_registry, "ui.cursor", Color::RGB(80, 140, 220)),
        ),
    );
    let border = adjust_color(dock_background, if is_dark { 28 } else { -28 });
    let dock_rect = PixelRectToRect::rect(
        layout.dock_rect.x,
        layout.dock_rect.y,
        layout.dock_rect.width,
        layout.dock_rect.height,
    );
    fill_window_surface_rect(target, dock_rect, dock_background, window_effects)?;

    let card_inset = 6i32;
    let card_height = acp_dock_card_height(line_height) as i32;
    let text_x = layout.dock_rect.x + cell_width.max(1) + card_inset;
    let max_chars = ((layout.dock_rect.width as i32)
        .saturating_sub(cell_width.max(1) * 2 + card_inset * 2)
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
        let name = truncate_acp_dock_text(&entry.name, max_chars);
        let session = truncate_acp_dock_text(&entry.session, max_chars);
        let client = truncate_acp_dock_text(&entry.client, max_chars);
        let baseline = card_y + ascent.max(0);
        draw_text(target, text_x, baseline, &name, foreground)?;
        draw_text(target, text_x, baseline + line_height, &session, muted)?;
        draw_text(target, text_x, baseline + line_height * 2, &client, muted)?;
        let separator_y = card_y + card_height - 1;
        if separator_y < layout.dock_rect.y + layout.dock_rect.height as i32 {
            fill_overlay_surface_rect(
                target,
                PixelRectToRect::rect(
                    layout.dock_rect.x + 4,
                    separator_y,
                    layout.dock_rect.width.saturating_sub(8),
                    1,
                ),
                border,
                window_effects,
            )?;
        }
    }
    Ok(())
}

fn truncate_acp_dock_text(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    if max_chars <= 1 {
        return text.chars().take(1).collect();
    }
    let keep = max_chars.saturating_sub(1);
    let mut truncated: String = text.chars().take(keep).collect();
    truncated.push('…');
    truncated
}
