//! Markdown Pretty host wiring: plan helpers, Forced Language, image loads.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;

use base64::Engine;
use base64::engine::general_purpose::STANDARD as BASE64;
use editor_markdown::{
    ImageDestination, MarkdownPrettyCacheKey, MarkdownPrettyConfig as PlanConfig,
    MarkdownPrettyPlan, MarkdownPrettyRequest, MarkdownPrettySourceStats, PrettyLinePlan,
};
use editor_plugin_api::UserLibrary;
use editor_syntax::SyntaxRegistry;

use super::{DecodedImage, ShellBuffer, decode_raster_image_bytes};

fn image_cache() -> &'static Mutex<HashMap<String, MarkdownImageCacheEntry>> {
    static CACHE: OnceLock<Mutex<HashMap<String, MarkdownImageCacheEntry>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Debug, Clone)]
pub(super) enum MarkdownImageCacheEntry {
    Loading,
    Ready(DecodedImage),
    Failed(String),
}

pub(super) fn plan_config_from_user(config: editor_plugin_api::MarkdownPrettyConfig) -> PlanConfig {
    let mut icons = BTreeMap::new();
    for entry in config.icons {
        icons.insert(entry.node_kind, entry.icon);
    }
    if icons.is_empty() {
        icons = editor_markdown::default_icon_map();
    }
    PlanConfig {
        enabled: config.enabled,
        kill_switch_enabled: config.kill_switch_enabled,
        kill_switch_max_lines: config.kill_switch_max_lines,
        kill_switch_max_bytes: config.kill_switch_max_bytes,
        image_max_bytes: config.image_max_bytes,
        image_max_rows: config.image_max_rows,
        icons,
    }
}

pub(super) fn build_markdown_pretty_plan(
    request: &MarkdownPrettyRequest<'_>,
    registry: Option<&mut SyntaxRegistry>,
) -> MarkdownPrettyPlan {
    editor_markdown::plan_markdown_pretty(request, registry)
}

pub(super) fn cached_plan_for_buffer(
    buffer: &ShellBuffer,
    config: &PlanConfig,
    enabled: bool,
    registry: Option<&mut SyntaxRegistry>,
) -> Arc<MarkdownPrettyPlan> {
    let key = MarkdownPrettyCacheKey::for_buffer(
        buffer.id().get(),
        buffer.text.revision(),
        enabled,
        config,
        buffer.language_id().map(str::to_owned),
    );
    let stats = MarkdownPrettySourceStats {
        line_count: buffer.text.line_count(),
        byte_count: buffer.text.byte_count(),
    };
    if let Ok(mut cache) = buffer.markdown_pretty_plan_cache.lock() {
        return cache
            .get_or_insert_with(key, stats, || {
                let text = buffer.text.text();
                let request = MarkdownPrettyRequest {
                    text: &text,
                    config,
                    buffer_enabled: Some(enabled),
                    buffer_path: buffer.text.path(),
                    workspace_root: None,
                    cursor_line: None,
                    visual_lines: None,
                    visible_lines: None,
                };
                (build_markdown_pretty_plan(&request, registry), Some(text))
            })
            .plan;
    }
    let text = buffer.text.text();
    let request = MarkdownPrettyRequest {
        text: &text,
        config,
        buffer_enabled: Some(enabled),
        buffer_path: buffer.text.path(),
        workspace_root: None,
        cursor_line: None,
        visual_lines: None,
        visible_lines: None,
    };
    Arc::new(build_markdown_pretty_plan(&request, registry))
}

#[cfg(test)]
pub(crate) fn last_cached_pretty_plan(buffer: &ShellBuffer) -> Option<Arc<MarkdownPrettyPlan>> {
    buffer
        .markdown_pretty_plan_cache
        .lock()
        .ok()
        .and_then(|cache| cache.last_plan())
}

pub(super) fn pretty_display_line(
    plan: &MarkdownPrettyPlan,
    anti_conceal: bool,
    line_index: usize,
    source_line: &str,
) -> String {
    editor_markdown::pretty_display_line(plan, anti_conceal, line_index, source_line)
}

pub(super) fn line_plan(plan: &MarkdownPrettyPlan, line_index: usize) -> Option<&PrettyLinePlan> {
    plan.line(line_index)
}

pub(super) fn ensure_image_loaded(
    destination: &ImageDestination,
    max_bytes: usize,
) -> MarkdownImageCacheEntry {
    let key = image_cache_key(destination);
    if let Ok(cache) = image_cache().lock()
        && let Some(entry) = cache.get(&key)
    {
        return entry.clone();
    }
    if let Ok(mut cache) = image_cache().lock() {
        cache.insert(key.clone(), MarkdownImageCacheEntry::Loading);
    }

    match destination {
        ImageDestination::Local(path) => match load_local_image(path, max_bytes) {
            Ok(image) => store_ready(&key, image),
            Err(error) => store_failed(&key, error),
        },
        ImageDestination::DataUrl(data) => match load_data_url_image(data, max_bytes) {
            Ok(image) => store_ready(&key, image),
            Err(error) => store_failed(&key, error),
        },
        ImageDestination::Https(url) => {
            spawn_https_fetch(key, url.clone(), max_bytes);
            MarkdownImageCacheEntry::Loading
        }
        ImageDestination::Unsupported(raw) => {
            store_failed(&key, format!("unsupported image source: {raw}"))
        }
    }
}

fn store_ready(key: &str, image: DecodedImage) -> MarkdownImageCacheEntry {
    let entry = MarkdownImageCacheEntry::Ready(image);
    if let Ok(mut cache) = image_cache().lock() {
        cache.insert(key.to_owned(), entry.clone());
    }
    entry
}

fn store_failed(key: &str, error: String) -> MarkdownImageCacheEntry {
    let entry = MarkdownImageCacheEntry::Failed(error);
    if let Ok(mut cache) = image_cache().lock() {
        cache.insert(key.to_owned(), entry.clone());
    }
    entry
}

fn spawn_https_fetch(key: String, url: String, max_bytes: usize) {
    thread::spawn(move || {
        let result = fetch_https_image(&url, max_bytes);
        if let Ok(mut guard) = image_cache().lock() {
            match result {
                Ok(image) => {
                    guard.insert(key, MarkdownImageCacheEntry::Ready(image));
                }
                Err(error) => {
                    guard.insert(key, MarkdownImageCacheEntry::Failed(error));
                }
            }
        }
        super::ping_shell_wakeup();
    });
}

fn fetch_https_image(url: &str, max_bytes: usize) -> Result<DecodedImage, String> {
    let disk_path = disk_cache_path(url)?;
    if disk_path.exists() {
        let bytes = fs::read(&disk_path).map_err(|error| error.to_string())?;
        if bytes.len() > max_bytes {
            return Err("cached image exceeds size limit".to_owned());
        }
        return decode_raster_image_bytes(&bytes);
    }
    let response = ureq::get(url).call().map_err(|error| error.to_string())?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(max_bytes.saturating_add(1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err("remote image exceeds size limit".to_owned());
    }
    if let Some(parent) = disk_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let _ = fs::write(&disk_path, &bytes);
    decode_raster_image_bytes(&bytes)
}

fn disk_cache_path(url: &str) -> Result<PathBuf, String> {
    let root = data_dir()?.join("volt").join("markdown-images");
    let hash = simple_hash(url);
    Ok(root.join(format!("{hash}.bin")))
}

fn data_dir() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("LOCALAPPDATA") {
        return Ok(PathBuf::from(dir));
    }
    if let Ok(dir) = std::env::var("XDG_DATA_HOME") {
        return Ok(PathBuf::from(dir));
    }
    std::env::var("HOME")
        .map(|home| PathBuf::from(home).join(".local").join("share"))
        .map_err(|_| "no data directory for markdown image cache".to_owned())
}

fn simple_hash(input: &str) -> String {
    let mut hash = 0u64;
    for byte in input.as_bytes() {
        hash = hash.wrapping_mul(131).wrapping_add(u64::from(*byte));
    }
    format!("{hash:016x}")
}

fn load_local_image(path: &Path, max_bytes: usize) -> Result<DecodedImage, String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err("local image exceeds size limit".to_owned());
    }
    decode_raster_image_bytes(&bytes)
}

fn load_data_url_image(data_url: &str, max_bytes: usize) -> Result<DecodedImage, String> {
    let (_, payload) = data_url
        .split_once(',')
        .ok_or_else(|| "invalid data URL".to_owned())?;
    let bytes = BASE64
        .decode(payload.trim())
        .map_err(|error| error.to_string())?;
    if bytes.len() > max_bytes {
        return Err("data URL image exceeds size limit".to_owned());
    }
    decode_raster_image_bytes(&bytes)
}

fn image_cache_key(destination: &ImageDestination) -> String {
    match destination {
        ImageDestination::Local(path) => format!("local:{}", path.display()),
        ImageDestination::Https(url) => format!("https:{url}"),
        ImageDestination::DataUrl(data) => format!("data:{}", simple_hash(data)),
        ImageDestination::Unsupported(raw) => format!("bad:{raw}"),
    }
}

pub(super) fn user_library_pretty_config(user_library: &dyn UserLibrary) -> PlanConfig {
    plan_config_from_user(user_library.markdown_pretty_config())
}
