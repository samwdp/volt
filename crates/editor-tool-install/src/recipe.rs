/// Typed method on a Language Server Spec or Debug Adapter Spec.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallRecipe {
    /// `npm install --prefix <package> <packages…>`
    Npm { packages: Vec<String> },
    /// `dotnet tool install --tool-path <package> <id>`
    DotnetTool { package: String, prerelease: bool },
    /// `GOBIN=<package> go install <module>`
    Go { module: String },
    /// Create a venv in the package dir, then pip-install packages.
    Pip { packages: Vec<String> },
    /// `cargo install --root <package> <crate>`
    Cargo { crate_name: String },
    /// Download an archive (zip, vsix, tar.gz, or gz) and unpack it.
    Archive { url: String, binary: Option<String> },
}

impl InstallRecipe {
    /// npm packages installed into the Spec's package directory.
    pub fn npm(packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Npm {
            packages: packages.into_iter().map(Into::into).collect(),
        }
    }

    /// A .NET global-style tool installed with `--tool-path`.
    pub fn dotnet_tool(package: impl Into<String>) -> Self {
        Self::DotnetTool {
            package: package.into(),
            prerelease: false,
        }
    }

    /// A prerelease .NET tool (`--prerelease`).
    pub fn dotnet_tool_prerelease(package: impl Into<String>) -> Self {
        Self::DotnetTool {
            package: package.into(),
            prerelease: true,
        }
    }

    /// A Go module installed with `go install`.
    pub fn go(module: impl Into<String>) -> Self {
        Self::Go {
            module: module.into(),
        }
    }

    /// PyPI packages installed into a package-local venv.
    pub fn pip(packages: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self::Pip {
            packages: packages.into_iter().map(Into::into).collect(),
        }
    }

    /// A crates.io crate installed with `cargo install --root`.
    pub fn cargo(crate_name: impl Into<String>) -> Self {
        Self::Cargo {
            crate_name: crate_name.into(),
        }
    }

    /// GitHub `releases/latest/download/<asset>`.
    pub fn github_release(repository: impl AsRef<str>, asset: impl AsRef<str>) -> Self {
        Self::Archive {
            url: format!(
                "https://github.com/{}/releases/latest/download/{}",
                repository.as_ref(),
                asset.as_ref()
            ),
            binary: None,
        }
    }

    /// GitHub latest download plus a relative binary path inside the extract.
    pub fn github_release_binary(
        repository: impl AsRef<str>,
        asset: impl AsRef<str>,
        binary: impl Into<String>,
    ) -> Self {
        Self::Archive {
            url: format!(
                "https://github.com/{}/releases/latest/download/{}",
                repository.as_ref(),
                asset.as_ref()
            ),
            binary: Some(binary.into()),
        }
    }

    /// Host tool required on PATH before this recipe can run.
    pub fn toolchain_program(&self) -> &'static str {
        match self {
            Self::Npm { .. } => "npm",
            Self::DotnetTool { .. } => "dotnet",
            Self::Go { .. } => "go",
            Self::Pip { .. } => "python",
            Self::Cargo { .. } => "cargo",
            Self::Archive { .. } => "curl",
        }
    }

    /// Short label for errors and Command Stream titles.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Npm { .. } => "npm",
            Self::DotnetTool { .. } => "dotnet tool",
            Self::Go { .. } => "go install",
            Self::Pip { .. } => "pip",
            Self::Cargo { .. } => "cargo",
            Self::Archive { .. } => "archive",
        }
    }
}

#[cfg(test)]
mod tests {
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
}
