use std::{collections::BTreeSet, fs, io, path::Path};

#[path = "src/standalone_user.rs"]
mod standalone_user;
#[path = "src/standalone_user_manifest.rs"]
mod standalone_user_manifest;

fn main() {
    if let Err(error) = copy_user_directory() {
        panic!("failed to copy user directory: {error}");
    }
    if let Err(error) = copy_assets_directory() {
        panic!("failed to copy assets directory: {error}");
    }
    #[cfg(target_os = "windows")]
    {
        if let Err(error) = build_windows_icon() {
            panic!("failed to embed Windows icon: {error}");
        }
    }
}

fn copy_user_directory() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let workspace_root = manifest_dir
        .parent()
        .and_then(|path| path.parent())
        .ok_or("unable to locate workspace root")?;
    let user_dir = workspace_root.join("user");
    if !user_dir.is_dir() {
        return Ok(());
    }
    let vendor_crates =
        standalone_user_manifest::standalone_user_vendor_crates(workspace_root, &user_dir)?;
    let target_profile_dir = target_profile_dir()?;
    let destination = target_profile_dir.join("user");
    remove_dir_all_if_exists(&destination)?;
    copy_dir_recursive(&user_dir, &destination)?;
    vendor_user_support_crates(workspace_root, &destination, &vendor_crates)?;
    rewrite_standalone_user_manifests(workspace_root, &user_dir, &destination, &vendor_crates)?;
    standalone_user::setup_standalone_user_repository(&destination)?;

    Ok(())
}

fn vendor_user_support_crates(
    workspace_root: &Path,
    user_destination: &Path,
    vendor_crates: &BTreeSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let vendor_dir = user_destination.join("vendor");
    remove_dir_all_if_exists(&vendor_dir)?;
    create_dir_all_with_retry(&vendor_dir)?;

    for crate_name in vendor_crates {
        let source = workspace_root.join("crates").join(crate_name);
        let destination = vendor_dir.join(crate_name);
        copy_dir_recursive(&source, &destination)?;
    }

    Ok(())
}

fn rewrite_standalone_user_manifests(
    workspace_root: &Path,
    user_source: &Path,
    user_destination: &Path,
    vendor_crates: &BTreeSet<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let user_replacements = standalone_user_manifest::standalone_user_path_replacements(
        &user_source.join("Cargo.toml"),
        workspace_root,
        "vendor",
    )?;
    rewrite_manifest(
        &user_destination.join("Cargo.toml"),
        &user_replacements,
        true,
    )?;
    let sdk_replacements = standalone_user_manifest::standalone_user_path_replacements(
        &user_source.join("sdk").join("Cargo.toml"),
        workspace_root,
        "../vendor",
    )?;
    rewrite_manifest(
        &user_destination.join("sdk").join("Cargo.toml"),
        &sdk_replacements,
        false,
    )?;
    for crate_name in vendor_crates {
        rewrite_manifest(
            &user_destination
                .join("vendor")
                .join(crate_name)
                .join("Cargo.toml"),
            &[],
            false,
        )?;
    }

    Ok(())
}

fn rewrite_manifest(
    manifest_path: &Path,
    path_replacements: &[standalone_user_manifest::ManifestPathReplacement],
    add_workspace_root: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut manifest = fs::read_to_string(manifest_path)?.replace("\r\n", "\n");
    manifest = inline_workspace_package_fields(manifest);
    for replacement in path_replacements {
        manifest = manifest.replace(&replacement.from, &replacement.to);
    }
    if add_workspace_root {
        manifest = add_standalone_workspace_root(manifest);
    }
    fs::write(manifest_path, manifest)?;
    Ok(())
}

fn inline_workspace_package_fields(mut manifest: String) -> String {
    manifest = manifest.replace("rust-version.workspace = true", "rust-version = \"1.91\"");
    manifest = manifest.replace("version.workspace = true", "version = \"0.1.0\"");
    manifest = manifest.replace("edition.workspace = true", "edition = \"2024\"");
    manifest = manifest.replace(
        "license.workspace = true",
        "license = \"MIT OR Apache-2.0\"",
    );
    manifest = manifest.replace(
        "[lints]\nworkspace = true\n",
        "[lints.rust]\nunsafe_code = \"forbid\"\nunused_crate_dependencies = \"warn\"\n\n[lints.clippy]\ndbg_macro = \"deny\"\ntodo = \"deny\"\nunwrap_used = \"deny\"\n",
    );
    manifest
}

fn add_standalone_workspace_root(mut manifest: String) -> String {
    if manifest.contains("\n[workspace]\n") || manifest.starts_with("[workspace]\n") {
        return manifest;
    }
    if !manifest.ends_with('\n') {
        manifest.push('\n');
    }
    manifest.push_str("\n[workspace]\nresolver = \"3\"\n");
    manifest
}

fn copy_assets_directory() -> Result<(), Box<dyn std::error::Error>> {
    use std::{env, path::PathBuf};

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let assets_dir = manifest_dir.join("assets");
    if !assets_dir.is_dir() {
        return Ok(());
    }
    let target_profile_dir = target_profile_dir()?;
    let destination = target_profile_dir.join("assets");
    remove_dir_all_if_exists(&destination)?;
    copy_dir_recursive(&assets_dir, &destination)?;

    Ok(())
}

fn target_profile_dir() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    use std::{env, ffi::OsStr, path::PathBuf};

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let build_dir = out_dir
        .ancestors()
        .find(|path| path.file_name() == Some(OsStr::new("build")))
        .ok_or("unable to locate build directory in OUT_DIR")?;
    let profile_dir = build_dir
        .parent()
        .ok_or("unable to locate target profile directory")?;
    Ok(profile_dir.to_path_buf())
}

fn copy_dir_recursive(source: &Path, destination: &Path) -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed={}", source.display());
    create_dir_all_with_retry(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let path = entry.path();
        let target = destination.join(entry.file_name());
        if path.is_dir() {
            if should_skip_dir(&path) {
                continue;
            }
            copy_dir_recursive(&path, &target)?;
        } else if path.is_file() {
            copy_file_with_retry(&path, &target)?;
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
    Ok(())
}

fn remove_dir_all_if_exists(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(_) => retry_windows_locked_fs(|| fs::remove_dir_all(path)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn create_dir_all_with_retry(path: &Path) -> io::Result<()> {
    retry_windows_locked_fs(|| fs::create_dir_all(path))
}

fn copy_file_with_retry(source: &Path, destination: &Path) -> io::Result<u64> {
    retry_windows_locked_fs(|| fs::copy(source, destination))
}

fn retry_windows_locked_fs<T, F>(mut operation: F) -> io::Result<T>
where
    F: FnMut() -> io::Result<T>,
{
    #[cfg(windows)]
    {
        const WINDOWS_LOCK_RETRY_ATTEMPTS: usize = 20;
        const WINDOWS_LOCK_RETRY_DELAY: std::time::Duration = std::time::Duration::from_millis(50);

        for attempt in 0..WINDOWS_LOCK_RETRY_ATTEMPTS {
            match operation() {
                Ok(value) => return Ok(value),
                Err(error)
                    if windows_should_retry_locked_fs(&error)
                        && attempt + 1 < WINDOWS_LOCK_RETRY_ATTEMPTS =>
                {
                    // Windows can transiently hold files in target\debug while concurrent builds
                    // or scanners touch freshly written artifacts.
                    std::thread::sleep(WINDOWS_LOCK_RETRY_DELAY);
                }
                Err(error) => return Err(error),
            }
        }
        unreachable!("retry loop always returns on success or final failure")
    }
    #[cfg(not(windows))]
    {
        operation()
    }
}

#[cfg(windows)]
fn windows_should_retry_locked_fs(error: &io::Error) -> bool {
    matches!(error.raw_os_error(), Some(5 | 32 | 33))
}

fn should_skip_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git" | "target")
    )
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
