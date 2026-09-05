use super::InstallRecipe;

#[test]
fn github_release_builds_latest_download_url() {
    let recipe = InstallRecipe::github_release(
        "rust-lang/rust-analyzer",
        "rust-analyzer-x86_64-pc-windows-msvc.zip",
    );
    match recipe {
        InstallRecipe::Archive { url, binary } => {
            assert_eq!(
                url,
                "https://github.com/rust-lang/rust-analyzer/releases/latest/download/rust-analyzer-x86_64-pc-windows-msvc.zip"
            );
            assert!(binary.is_none());
        }
        other => panic!("expected archive, got {other:?}"),
    }
}
