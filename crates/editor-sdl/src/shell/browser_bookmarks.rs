use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use editor_plugin_api::volt_data_dir;
use serde_json::{Value, json};

const BOOKMARKS_FILE_NAME: &str = "browser-bookmarks.json";
const BOOKMARKS_FILE_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct BrowserBookmark {
    pub(super) name: String,
    pub(super) url: String,
}

fn persist_path_override() -> &'static Mutex<Option<PathBuf>> {
    static OVERRIDE: OnceLock<Mutex<Option<PathBuf>>> = OnceLock::new();
    OVERRIDE.get_or_init(|| Mutex::new(None))
}

pub(super) fn browser_bookmarks_path() -> PathBuf {
    persist_path_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone()
        .unwrap_or_else(|| volt_data_dir().join(BOOKMARKS_FILE_NAME))
}

#[cfg(test)]
pub(super) fn set_browser_bookmarks_path_for_test(path: Option<PathBuf>) {
    *persist_path_override()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner()) = path;
}

pub(super) fn load_browser_bookmarks() -> Vec<BrowserBookmark> {
    load_browser_bookmarks_from_path(&browser_bookmarks_path())
}

fn load_browser_bookmarks_from_path(path: &Path) -> Vec<BrowserBookmark> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };
    let Ok(value) = serde_json::from_str::<Value>(&contents) else {
        return Vec::new();
    };
    let version = value.get("version").and_then(Value::as_u64).unwrap_or(0);
    if version != u64::from(BOOKMARKS_FILE_VERSION) {
        return Vec::new();
    }
    value
        .get("bookmarks")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?.trim();
            let url = entry.get("url")?.as_str()?.trim();
            if name.is_empty() || url.is_empty() {
                return None;
            }
            Some(BrowserBookmark {
                name: name.to_owned(),
                url: url.to_owned(),
            })
        })
        .collect()
}

pub(super) fn save_browser_bookmark(name: &str, url: &str) -> Result<(), String> {
    let name = name.trim();
    let url = url.trim();
    if name.is_empty() {
        return Err("bookmark name is required".to_owned());
    }
    if url.is_empty() {
        return Err("bookmark url is required".to_owned());
    }
    let path = browser_bookmarks_path();
    let mut bookmarks = load_browser_bookmarks_from_path(&path);
    if let Some(existing) = bookmarks.iter_mut().find(|bookmark| bookmark.url == url) {
        existing.name = name.to_owned();
    } else {
        bookmarks.push(BrowserBookmark {
            name: name.to_owned(),
            url: url.to_owned(),
        });
    }
    persist_browser_bookmarks(&path, &bookmarks)
}

fn persist_browser_bookmarks(path: &Path, bookmarks: &[BrowserBookmark]) -> Result<(), String> {
    let Some(parent) = path.parent() else {
        return Err("bookmark path is missing a parent directory".to_owned());
    };
    fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let body = json!({
        "version": BOOKMARKS_FILE_VERSION,
        "bookmarks": bookmarks.iter().map(|bookmark| json!({
            "name": bookmark.name,
            "url": bookmark.url,
        })).collect::<Vec<_>>(),
    });
    let encoded =
        serde_json::to_vec_pretty(&body).map_err(|error| format!("encode bookmarks: {error}"))?;
    let tmp = path.with_extension("json.tmp");
    let write_tmp = (|| -> io::Result<()> {
        let mut handle = fs::File::create(&tmp)?;
        handle.write_all(&encoded)?;
        handle.sync_all()?;
        Ok(())
    })();
    if let Err(error) = write_tmp {
        let _ = fs::remove_file(&tmp);
        return Err(error.to_string());
    }
    if let Err(error) = fs::rename(&tmp, path) {
        let _ = fs::remove_file(&tmp);
        return Err(error.to_string());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn save_and_load_round_trip() -> Result<(), String> {
        let dir =
            std::env::temp_dir().join(format!("volt-browser-bookmarks-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
        let path = dir.join(BOOKMARKS_FILE_NAME);
        set_browser_bookmarks_path_for_test(Some(path.clone()));
        save_browser_bookmark("Example", "https://example.com")?;
        save_browser_bookmark("Example Renamed", "https://example.com")?;
        let loaded = load_browser_bookmarks();
        set_browser_bookmarks_path_for_test(None);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            loaded,
            vec![BrowserBookmark {
                name: "Example Renamed".to_owned(),
                url: "https://example.com".to_owned(),
            }]
        );
        Ok(())
    }
}
