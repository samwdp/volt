fn main() {
    if let Err(error) = copy_user_themes() {
        panic!("failed to copy user themes: {error}");
    }
    #[cfg(target_os = "windows")]
    {
        if let Err(error) = build_windows_icon() {
            panic!("failed to embed Windows icon: {error}");
        }
    }
}

fn copy_user_themes() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let workspace_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .ok_or("unable to locate workspace root")?;
    let themes_dir = workspace_root.join("user").join("themes");
    if !themes_dir.is_dir() {
        return Ok(());
    }
    println!("cargo:rerun-if-changed={}", themes_dir.display());

    let profile = env::var("PROFILE")?;
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root.join("target"))
        .join(profile);
    let destination = target_dir.join("user").join("themes");
    fs::create_dir_all(&destination)?;

    for entry in fs::read_dir(&themes_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("toml"))
        {
            let target = destination.join(entry.file_name());
            fs::copy(&path, &target)?;
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }

    Ok(())
}

#[cfg(target_os = "windows")]
fn build_windows_icon() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, fs, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let assets_dir = manifest_dir.join("assets");
    let icon_ico_source = assets_dir.join("logo.ico");
    let icon_png = assets_dir.join("logo_32x32.png");
    println!("cargo:rerun-if-changed={}", icon_ico_source.display());
    println!("cargo:rerun-if-changed={}", icon_png.display());

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let rc_path = out_dir.join("volt-icon.rc");
    let icon_path = if icon_ico_source.exists() {
        icon_ico_source
    } else {
        let icon_ico = out_dir.join("volt-icon.ico");
        write_ico_from_png(&icon_png, &icon_ico)?;
        icon_ico
    };
    let icon_path = icon_path.display().to_string().replace('\\', "/");
    fs::write(&rc_path, format!("32512 ICON \"{icon_path}\""))?;

    embed_resource::compile(&rc_path, embed_resource::NONE);
    Ok(())
}

#[cfg(target_os = "windows")]
fn write_ico_from_png(
    png_path: &std::path::Path,
    ico_path: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let png_bytes = std::fs::read(png_path)?;
    let (width, height) = png_dimensions(&png_bytes)?;
    if width == 0 || height == 0 || width > 256 || height > 256 {
        return Err(
            format!("icon size must be between 1 and 256 pixels (got {width}x{height})").into(),
        );
    }

    let width_byte = if width == 256 { 0 } else { width as u8 };
    let height_byte = if height == 256 { 0 } else { height as u8 };
    let image_len = png_bytes.len() as u32;
    let image_offset = 6u32 + 16u32;

    let mut ico = Vec::with_capacity(image_offset as usize + png_bytes.len());
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());

    ico.push(width_byte);
    ico.push(height_byte);
    ico.push(0);
    ico.push(0);
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&32u16.to_le_bytes());
    ico.extend_from_slice(&image_len.to_le_bytes());
    ico.extend_from_slice(&image_offset.to_le_bytes());
    ico.extend_from_slice(&png_bytes);

    std::fs::write(ico_path, ico)?;
    Ok(())
}

#[cfg(target_os = "windows")]
fn png_dimensions(png: &[u8]) -> Result<(u32, u32), Box<dyn std::error::Error>> {
    const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    if png.len() < 24 {
        return Err("PNG payload too small".into());
    }
    if png[0..8] != PNG_SIGNATURE {
        return Err("PNG signature mismatch".into());
    }
    if &png[12..16] != b"IHDR" {
        return Err("PNG IHDR chunk not found".into());
    }
    let width = u32::from_be_bytes([png[16], png[17], png[18], png[19]]);
    let height = u32::from_be_bytes([png[20], png[21], png[22], png[23]]);
    Ok((width, height))
}
