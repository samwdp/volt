mod build_output;

use std::{
    env,
    error::Error,
    path::{Path, PathBuf},
};

use build_output::{distributed_user_library_paths, install_root_library_link};

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-changed=build.rs");
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    link_root_user_library(&manifest_dir, &out_dir)?;
    Ok(())
}

fn link_root_user_library(manifest_dir: &Path, out_dir: &Path) -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed=CARGO_TARGET_DIR");
    let target_os = env::var("CARGO_CFG_TARGET_OS")?;
    let Some(paths) = distributed_user_library_paths(manifest_dir, out_dir, &target_os) else {
        return Ok(());
    };

    if let Err(error) = install_root_library_link(&paths) {
        println!(
            "cargo:warning=failed to expose `{}` at repository root `{}`: {error}",
            paths.built_library_path.display(),
            paths.root_library_path.display()
        );
    }

    Ok(())
}
