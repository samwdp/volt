use super::commands_for_recipe;
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
    )
    .expect("dotnet plan");
    assert!(commands[0].args().iter().any(|arg| arg == "--prerelease"));
}
