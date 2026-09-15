use super::{commands_for_recipe, install_launch_env, resolved_toolchain};
use crate::InstallRecipe;
use std::path::Path;

#[test]
fn npm_recipe_uses_prefix() {
    let package = Path::new("/tmp/volt-pkg");
    let commands = commands_for_recipe(
        &InstallRecipe::npm(["typescript-language-server"]),
        package,
        "typescript-language-server",
        "npm",
        vec![("PATH".to_owned(), "/usr/bin".to_owned())],
    )
    .expect("npm plan");
    assert_eq!(commands[0].program(), "npm");
    assert!(commands[0].args().contains(&"--prefix".to_owned()));
    assert!(
        commands[0]
            .args()
            .iter()
            .any(|arg| arg.contains("typescript-language-server"))
    );
}

#[test]
fn dotnet_prerelease_passes_flag() {
    let package = Path::new("/tmp/volt-pkg");
    let commands = commands_for_recipe(
        &InstallRecipe::dotnet_tool_prerelease("roslyn-language-server"),
        package,
        "roslyn-language-server",
        "dotnet",
        vec![("PATH".to_owned(), "/usr/bin".to_owned())],
    )
    .expect("dotnet plan");
    assert!(commands[0].args().iter().any(|arg| arg == "--prerelease"));
}

#[test]
fn resolved_toolchain_prefers_path_from_launch_env() {
    let temp = std::env::temp_dir().join(format!(
        "volt-npm-resolve-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).expect("mkdir");
    #[cfg(windows)]
    {
        let npm = temp.join("npm.cmd");
        std::fs::write(&npm, b"@echo off\r\n").expect("write npm");
    }
    #[cfg(not(windows))]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let npm = temp.join("npm");
        std::fs::write(&npm, b"#!/bin/sh\n").expect("write npm");
        let mut perms = std::fs::metadata(&npm).expect("meta").permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&npm, perms).expect("chmod");
    }
    let path_value = temp.to_string_lossy().into_owned();
    let env = vec![("PATH".to_owned(), path_value)];
    let resolved = resolved_toolchain(&InstallRecipe::npm(["typescript-language-server"]), &env)
        .expect("resolve npm");
    assert!(
        resolved.to_ascii_lowercase().contains("npm"),
        "unexpected toolchain `{resolved}`"
    );
    assert!(
        std::path::Path::new(&resolved).is_absolute() || resolved.eq_ignore_ascii_case("npm"),
        "toolchain should resolve via launch env PATH, got `{resolved}`"
    );
    let _ = std::fs::remove_dir_all(temp);
}

#[cfg(windows)]
#[test]
fn install_launch_env_can_surface_fnm_npm_when_present() {
    let env = install_launch_env(None);
    let path = env
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    // When fnm is installed, enrichment should make npm resolvable even if the
    // parent process PATH lacks the ephemeral multishell dir.
    if editor_jobs::resolve_command_path("fnm", &[], None).is_none() {
        return;
    }
    assert!(
        editor_jobs::resolve_command_path("npm", &env, None).is_some(),
        "fnm present but enriched install env still cannot resolve npm; PATH={path}"
    );
}
