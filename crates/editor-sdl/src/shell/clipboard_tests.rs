
use super::*;

const TINY_PNG: &[u8] = &[
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4,
    0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
    0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE,
    0x42, 0x60, 0x82,
];

#[test]
fn sniff_image_mime_recognizes_png_magic() {
    assert_eq!(sniff_image_mime(TINY_PNG), Some("image/png"));
    assert_eq!(sniff_image_mime(b"not-an-image"), None);
}

#[test]
fn normalize_clipboard_image_prefers_sniffed_png_mime() {
    let image = normalize_clipboard_image(TINY_PNG.to_vec(), Some("image/jpeg"), "shot")
        .expect("png should normalize");
    assert_eq!(image.mime_type, "image/png");
    assert_eq!(image.name, "shot");
    assert_eq!(image.bytes, TINY_PNG);
}

#[test]
fn clipboard_image_from_path_loads_named_png() {
    let dir = std::env::temp_dir().join(format!(
        "volt-clipboard-image-{}",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("temp dir");
    let path = dir.join("screenshot.png");
    fs::write(&path, TINY_PNG).expect("write png");
    let image = clipboard_image_from_path(&path).expect("load png");
    assert_eq!(image.name, "screenshot.png");
    assert_eq!(image.mime_type, "image/png");
    let _ = fs::remove_dir_all(dir);
}

#[test]
fn clipboard_image_from_path_text_ignores_plain_text() {
    assert!(clipboard_image_from_path_text("hello world").is_none());
    assert!(clipboard_image_from_path_text("not/a/real/image.png").is_none());
}
