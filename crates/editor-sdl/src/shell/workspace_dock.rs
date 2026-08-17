use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use editor_core::WorkspaceId;
use editor_plugin_api::{UserLibrary, WorkspaceDockConfig, WorkspaceDockSide};
use editor_render::PixelRect;
use editor_theme::ThemeRegistry;
use sdl3::pixels::Color;

use super::*;

const WORKSPACE_DOCK_MIN_CELLS: u32 = 28;
const WORKSPACE_DOCK_MAX_CELLS: u32 = 36;
const WORKSPACE_DOCK_CARD_LINE_COUNT: u32 = 3;
const WORKSPACE_DOCK_CARD_GAP_LINES: u32 = 1;
const WORKSPACE_DOCK_BRANCH_REFRESH_INTERVAL: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkspaceDockEntry {
    pub(super) workspace_id: WorkspaceId,
    pub(super) name: String,
    pub(super) buffer_count: usize,
    pub(super) branch: Option<String>,
    pub(super) active: bool,
    pub(super) unread: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct WorkspaceDockLayout {
    pub(super) visible: bool,
    pub(super) side: WorkspaceDockSide,
    pub(super) dock_width: u32,
    pub(super) dock_rect: PixelRect,
    pub(super) content_x: i32,
    pub(super) content_width: u32,
    pub(super) content_height: u32,
}

impl WorkspaceDockLayout {
    pub(super) fn hidden(width: u32, height: u32) -> Self {
        Self {
            visible: false,
            side: WorkspaceDockSide::Left,
            dock_width: 0,
            dock_rect: PixelRect::new(0, 0, 0, 0),
            content_x: 0,
            content_width: width,
            content_height: height,
        }
    }
}

#[derive(Debug, Clone)]
pub(super) struct WorkspaceDockBranchCache {
    branches: Arc<Mutex<HashMap<PathBuf, Option<String>>>>,
    inflight: Arc<Mutex<HashMap<PathBuf, ()>>>,
    last_refresh_at: Option<Instant>,
}

impl WorkspaceDockBranchCache {
    pub(super) fn new() -> Self {
        Self {
            branches: Arc::new(Mutex::new(HashMap::new())),
            inflight: Arc::new(Mutex::new(HashMap::new())),
            last_refresh_at: None,
        }
    }

    pub(super) fn branch_for_root(&self, root: &Path) -> Option<String> {
        let guard = self.branches.lock().ok()?;
        guard.get(root).cloned().flatten()
    }

    fn refresh_due(&self, now: Instant) -> bool {
        self.last_refresh_at
            .map(|last| now.duration_since(last) >= WORKSPACE_DOCK_BRANCH_REFRESH_INTERVAL)
            .unwrap_or(true)
    }

    fn mark_refreshed(&mut self, now: Instant) {
        self.last_refresh_at = Some(now);
    }

    fn queue_roots(&self, roots: &[PathBuf]) {
        let Ok(mut inflight) = self.inflight.lock() else {
            return;
        };
        let Ok(branches) = self.branches.lock() else {
            return;
        };
        for root in roots {
            if branches.contains_key(root) || inflight.contains_key(root) {
                continue;
            }
            inflight.insert(root.clone(), ());
            let root = root.clone();
            let branches = Arc::clone(&self.branches);
            let inflight_map = Arc::clone(&self.inflight);
            thread::spawn(move || {
                let branch = git_command_output_background(
                    &root,
                    &["rev-parse", "--abbrev-ref", "HEAD"],
                    &[0],
                )
                .map(|output| output.trim().to_owned())
                .filter(|branch| !branch.is_empty() && branch != "HEAD");
                if let Ok(mut guard) = branches.lock() {
                    guard.insert(root.clone(), branch);
                }
                if let Ok(mut guard) = inflight_map.lock() {
                    guard.remove(&root);
                }
            });
        }
    }
}

pub(super) fn workspace_dock_visible(user_library: &dyn UserLibrary, ui: &ShellUiState) -> bool {
    let config = user_library.workspace_dock_config();
    config.docked || ui.workspace_dock_open()
}

pub(super) fn workspace_dock_config(user_library: &dyn UserLibrary) -> WorkspaceDockConfig {
    user_library.workspace_dock_config()
}

pub(super) fn workspace_dock_width(content_width: u32, cell_width: i32) -> u32 {
    let cell = cell_width.max(1) as u32;
    if content_width <= cell {
        return content_width;
    }
    let desired = (content_width / 5)
        .max(cell.saturating_mul(WORKSPACE_DOCK_MIN_CELLS))
        .min(cell.saturating_mul(WORKSPACE_DOCK_MAX_CELLS));
    let max_width = content_width.saturating_sub(cell).max(cell);
    desired.min(max_width)
}

pub(super) fn workspace_dock_layout(
    user_library: &dyn UserLibrary,
    ui: &ShellUiState,
    width: u32,
    height: u32,
    cell_width: i32,
) -> WorkspaceDockLayout {
    if !workspace_dock_visible(user_library, ui) {
        return WorkspaceDockLayout::hidden(width, height);
    }
    let config = workspace_dock_config(user_library);
    let dock_width = workspace_dock_width(width, cell_width);
    let content_width = width.saturating_sub(dock_width);
    let (dock_x, content_x) = match config.side {
        WorkspaceDockSide::Left => (0, dock_width as i32),
        WorkspaceDockSide::Right => (content_width as i32, 0),
    };
    WorkspaceDockLayout {
        visible: true,
        side: config.side,
        dock_width,
        dock_rect: PixelRect::new(dock_x, 0, dock_width, height),
        content_x,
        content_width,
        content_height: height,
    }
}

pub(super) fn workspace_dock_card_height(line_height: i32) -> u32 {
    let row = line_height.max(1) as u32;
    row.saturating_mul(WORKSPACE_DOCK_CARD_LINE_COUNT + WORKSPACE_DOCK_CARD_GAP_LINES)
}

pub(super) fn workspace_dock_entry_at_point(
    layout: &WorkspaceDockLayout,
    entries: &[WorkspaceDockEntry],
    line_height: i32,
    x: i32,
    y: i32,
) -> Option<WorkspaceId> {
    if !layout.visible {
        return None;
    }
    let rect = layout.dock_rect;
    let right = rect.x.saturating_add(rect.width as i32);
    let bottom = rect.y.saturating_add(rect.height as i32);
    if x < rect.x || x >= right || y < rect.y || y >= bottom {
        return None;
    }
    let card_height = workspace_dock_card_height(line_height) as i32;
    if card_height <= 0 {
        return None;
    }
    let index = ((y - rect.y) / card_height) as usize;
    entries.get(index).map(|entry| entry.workspace_id)
}

pub(super) fn refresh_workspace_dock_branches(
    cache: &mut WorkspaceDockBranchCache,
    roots: &[PathBuf],
    now: Instant,
) {
    if roots.is_empty() {
        return;
    }
    cache.queue_roots(roots);
    if cache.refresh_due(now) {
        cache.mark_refreshed(now);
        cache.queue_roots(roots);
    }
}

pub(super) fn render_workspace_dock(
    target: &mut DrawTarget<'_>,
    layout: &WorkspaceDockLayout,
    entries: &[WorkspaceDockEntry],
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
        "ui.workspace-dock.background",
        adjust_color(base_background, if is_dark { 10 } else { -10 }),
    );
    let foreground = theme_color(
        theme_registry,
        "ui.workspace-dock.foreground",
        theme_color(
            theme_registry,
            "ui.foreground",
            Color::RGBA(215, 221, 232, 255),
        ),
    );
    let muted = theme_color(
        theme_registry,
        "ui.workspace-dock.muted",
        blend_color(foreground, dock_background, 0.45),
    );
    let selection = theme_color(
        theme_registry,
        "ui.workspace-dock.selection",
        theme_color(
            theme_registry,
            "ui.selection",
            adjust_color(dock_background, if is_dark { 36 } else { -36 }),
        ),
    );
    let accent = theme_color(
        theme_registry,
        "ui.workspace-dock.accent",
        theme_color(theme_registry, "ui.cursor", Color::RGB(80, 140, 220)),
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
    let card_height = workspace_dock_card_height(line_height) as i32;
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
        let name = truncate_dock_text(&entry.name, max_chars);
        let buffers = truncate_dock_text(
            &format!(
                "{} buffer{}",
                entry.buffer_count,
                if entry.buffer_count == 1 { "" } else { "s" }
            ),
            max_chars,
        );
        let branch = truncate_dock_text(entry.branch.as_deref().unwrap_or("—"), max_chars);
        let baseline = card_y + ascent.max(0);
        draw_text(target, text_x, baseline, &name, foreground)?;
        draw_text(target, text_x, baseline + line_height, &buffers, muted)?;
        draw_text(target, text_x, baseline + line_height * 2, &branch, muted)?;
        if entry.unread > 0 {
            let badge = entry.unread.min(9);
            let label = if entry.unread > 9 {
                "9+".to_owned()
            } else {
                badge.to_string()
            };
            let badge_size = (line_height.max(12) as u32).saturating_sub(2);
            let badge_x =
                layout.dock_rect.x + layout.dock_rect.width as i32 - cell_width.max(1) * 2 - 4;
            let badge_y = card_y + 4;
            fill_overlay_surface_rounded_rect(
                target,
                PixelRectToRect::rect(badge_x, badge_y, badge_size, badge_size),
                badge_size / 2,
                accent,
                window_effects,
            )?;
            draw_text(target, badge_x + 3, badge_y + 1, &label, dock_background)?;
        }
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

fn truncate_dock_text(text: &str, max_chars: usize) -> String {
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
