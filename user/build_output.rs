use std::{
    fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserLibraryPaths {
    pub built_library_path: PathBuf,
    pub root_library_path: PathBuf,
}

pub fn distributed_user_library_paths(
    manifest_dir: &Path,
    out_dir: &Path,
    target_os: &str,
) -> Option<UserLibraryPaths> {
    let workspace_root = manifest_dir.parent()?;
    let profile_dir = out_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)?;
    let file_name = user_library_filename(target_os);
    Some(UserLibraryPaths {
        built_library_path: profile_dir.join(file_name),
        root_library_path: workspace_root.join(file_name),
    })
}

pub fn user_library_filename(target_os: &str) -> &'static str {
    match target_os {
        "windows" => "user.dll",
        "macos" => "libuser.dylib",
        _ => "libuser.so",
    }
}

pub fn install_root_library_link(paths: &UserLibraryPaths) -> io::Result<()> {
    if let Ok(current_target) = fs::read_link(&paths.root_library_path)
        && current_target == paths.built_library_path
    {
        return Ok(());
    }

    match fs::symlink_metadata(&paths.root_library_path) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "root library path `{}` is a directory",
                    paths.root_library_path.display()
                ),
            ));
        }
        Ok(_) => fs::remove_file(&paths.root_library_path)?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(error),
    }

    create_symlink(&paths.built_library_path, &paths.root_library_path)
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::unix::fs::symlink(target, link)
}

#[cfg(windows)]
fn create_symlink(target: &Path, link: &Path) -> io::Result<()> {
    std::os::windows::fs::symlink_file(target, link)
}

#[cfg(test)]
mod tests {
    use super::{distributed_user_library_paths, install_root_library_link, user_library_filename};
    use std::{
        env, fs,
        path::Path,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[test]
    fn distributed_user_library_paths_points_from_out_dir_to_root_library() {
        let manifest_dir = Path::new("/workspace/volt/user");
        let out_dir = Path::new("/workspace/volt/target/debug/build/volt-user-hash/out");
        let paths = distributed_user_library_paths(manifest_dir, out_dir, "linux")
            .expect("user library paths");

        assert_eq!(
            paths.built_library_path,
            Path::new("/workspace/volt/target/debug/libuser.so")
        );
        assert_eq!(
            paths.root_library_path,
            Path::new("/workspace/volt/libuser.so")
        );
    }

    #[test]
    fn user_library_filename_matches_platform_conventions() {
        assert_eq!(user_library_filename("linux"), "libuser.so");
        assert_eq!(user_library_filename("macos"), "libuser.dylib");
        assert_eq!(user_library_filename("windows"), "user.dll");
    }

    #[cfg(unix)]
    #[test]
    fn install_root_library_link_creates_or_updates_symlink() {
        let temp_root = env::temp_dir().join(format!(
            "volt-user-build-output-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time")
                .as_millis()
        ));
        let profile_dir = temp_root.join("target/debug");
        fs::create_dir_all(&profile_dir).expect("create profile dir");

        let first_target = profile_dir.join("libuser.so");
        let root_link = temp_root.join("libuser.so.root-link");
        let second_target = temp_root.join("alternate/libuser.so");
        fs::write(&first_target, "first").expect("write first target");
        fs::create_dir_all(second_target.parent().expect("alternate dir")).expect("create alt dir");
        fs::write(&second_target, "second").expect("write second target");

        install_root_library_link(&super::UserLibraryPaths {
            built_library_path: first_target.clone(),
            root_library_path: root_link.clone(),
        })
        .expect("create root link");
        assert_eq!(
            fs::read_link(&root_link).expect("read first link"),
            first_target
        );

        install_root_library_link(&super::UserLibraryPaths {
            built_library_path: second_target.clone(),
            root_library_path: root_link.clone(),
        })
        .expect("update root link");
        assert_eq!(
            fs::read_link(&root_link).expect("read second link"),
            second_target
        );

        fs::remove_dir_all(&temp_root).expect("cleanup temp root");
    }
}
