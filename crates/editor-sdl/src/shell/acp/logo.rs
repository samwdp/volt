use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, OnceLock},
};

use sdl3::rect::Rect;

use super::super::*;

const BUNDLED_ASSETS_DIR_CANDIDATES: &[&[&str]] = &[&["crates", "volt", "assets"], &["assets"]];

type LogoCache = Mutex<HashMap<String, Option<DecodedImage>>>;

fn logo_cache() -> &'static LogoCache {
    static CACHE: OnceLock<LogoCache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub(crate) fn resolve_bundled_asset_path(relative: &str) -> Option<PathBuf> {
    let relative = relative.trim().trim_start_matches(['/', '\\']);
    if relative.is_empty() {
        return None;
    }
    let parts = relative
        .split(['/', '\\'])
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();
    if parts.is_empty() {
        return None;
    }
    let mut roots = Vec::new();
    if let Ok(exe_path) = env::current_exe()
        && let Some(exe_dir) = exe_path.parent()
    {
        roots.extend(
            exe_dir
                .ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    if let Ok(cwd) = env::current_dir() {
        roots.extend(
            cwd.ancestors()
                .take(BUNDLED_ICON_FONT_SEARCH_DEPTH)
                .map(Path::to_path_buf),
        );
    }
    for root in roots {
        for base_parts in BUNDLED_ASSETS_DIR_CANDIDATES {
            let mut candidate = asset_path_from_parts(&root, base_parts);
            for part in &parts {
                candidate = candidate.join(part);
            }
            if candidate.is_file() {
                return Some(candidate);
            }
        }
    }
    None
}

pub(crate) fn load_acp_logo(relative: &str) -> Option<DecodedImage> {
    let relative = relative.trim();
    if relative.is_empty() {
        return None;
    }
    if let Ok(cache) = logo_cache().lock()
        && let Some(cached) = cache.get(relative)
    {
        return cached.clone();
    }
    let loaded = resolve_bundled_asset_path(relative).and_then(|path| {
        let text = fs::read_to_string(&path).ok()?;
        rasterize_svg_text(&text, Some(path.as_path())).ok()
    });
    if let Ok(mut cache) = logo_cache().lock() {
        cache.insert(relative.to_owned(), loaded.clone());
    }
    loaded
}

pub(crate) fn draw_acp_logo(
    target: &mut DrawTarget<'_>,
    logo: &DecodedImage,
    dest: Rect,
) -> Result<(), ShellError> {
    draw_image(
        target,
        dest,
        logo.width,
        logo.height,
        Arc::clone(&logo.pixels),
        Some(dest),
    )
}

pub(crate) fn acp_logo_dest_rect(
    area_x: i32,
    area_y: i32,
    area_width: u32,
    area_height: u32,
    logo: &DecodedImage,
    inset: i32,
) -> Option<Rect> {
    let max_side = (area_height as i32)
        .saturating_sub(inset.saturating_mul(2))
        .max(1) as u32;
    if max_side == 0 || area_width == 0 {
        return None;
    }
    // Always fit to the slot so native SVG pixel size cannot dominate layout
    // (e.g. 24×24 Simple Icons vs 512×512 OpenCode).
    let scale = (max_side as f32 / logo.width.max(1) as f32)
        .min(max_side as f32 / logo.height.max(1) as f32);
    let width = ((logo.width as f32) * scale).round().max(1.0) as u32;
    let height = ((logo.height as f32) * scale).round().max(1.0) as u32;
    let x = area_x + area_width as i32 - inset - width as i32;
    let y = area_y + inset + ((area_height as i32 - inset * 2 - height as i32) / 2).max(0);
    if x < area_x {
        return None;
    }
    Some(Rect::new(x, y, width, height))
}

/// Matches headerless text-panel body origin (`rect.y + 10` in `render_text_panel`).
const TEXT_PANEL_BODY_TOP_PADDING: i32 = 10;

/// Leading logo for ACP footer text panels: sized to the text line and vertically
/// centred on that line (not the full panel chrome).
pub(crate) fn acp_logo_leading_rect(
    area_x: i32,
    area_y: i32,
    line_height: i32,
    logo: &DecodedImage,
    inset: i32,
) -> Rect {
    let line_height = line_height.max(1);
    let body_y = area_y + TEXT_PANEL_BODY_TOP_PADDING;
    let max_side = line_height as u32;
    let scale = (max_side as f32 / logo.width.max(1) as f32)
        .min(max_side as f32 / logo.height.max(1) as f32);
    let width = ((logo.width as f32) * scale).round().max(1.0) as u32;
    let height = ((logo.height as f32) * scale).round().max(1.0) as u32;
    let y = body_y + ((line_height - height as i32) / 2).max(0);
    Rect::new(area_x + inset, y, width, height)
}
