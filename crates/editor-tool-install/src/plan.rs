use std::{fs, path::PathBuf};

use editor_jobs::resolve_command_path;

use crate::{
    InstallRecipe, ToolInstallError, ToolKind,
    locate::program_is_available,
    paths::{effective_path_env, package_dir},
};

/// One External Command in an Install Command Stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallCommand {
    label: String,
    program: String,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: PathBuf,
}

impl InstallCommand {
    /// Command Stream / job label.
    pub fn label(&self) -> &str {
        &self.label
    }

    /// Program to spawn.
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Arguments.
    pub fn args(&self) -> &[String] {
        &self.args
    }

    /// Extra environment (includes PATH with Volt bins).
    pub fn env(&self) -> &[(String, String)] {
        &self.env
    }

    /// Working directory.
    pub fn cwd(&self) -> &std::path::Path {
        &self.cwd
    }
}

/// Prepared Install: dirs created, toolchain checked, commands queued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallPlan {
    kind: ToolKind,
    spec_id: String,
    program: String,
    recipe: InstallRecipe,
    commands: Vec<InstallCommand>,
}

impl InstallPlan {
    /// Language Server vs Debug Adapter.
    pub const fn kind(&self) -> ToolKind {
        self.kind
    }

    /// Spec id (package folder name).
    pub fn spec_id(&self) -> &str {
        &self.spec_id
    }

    /// Spec program name (shim name).
    pub fn program(&self) -> &str {
        &self.program
    }

    /// Recipe being executed.
    pub fn recipe(&self) -> &InstallRecipe {
        &self.recipe
    }

    /// Sequential External Commands.
    pub fn commands(&self) -> &[InstallCommand] {
        &self.commands
    }

    /// Remaining commands after the first has been started.
    pub fn remaining_after_first(&self) -> Vec<InstallCommand> {
        self.commands.iter().skip(1).cloned().collect()
    }

    /// First command, if any.
    pub fn first_command(&self) -> Option<&InstallCommand> {
        self.commands.first()
    }
}

/// Builds an InstallPlan after checking the recipe toolchain is on PATH.
pub fn prepare_install(
    kind: ToolKind,
    spec_id: &str,
    program: &str,
    recipe: &InstallRecipe,
) -> Result<InstallPlan, ToolInstallError> {
    let package = package_dir(kind, spec_id);
    if package.exists() {
        fs::remove_dir_all(&package)?;
    }
    fs::create_dir_all(&package)?;
    crate::paths::ensure_install_layout()?;
    let toolchain = resolved_toolchain(recipe)?;
    let commands = commands_for_recipe(recipe, &package, program, &toolchain)?;
    Ok(InstallPlan {
        kind,
        spec_id: spec_id.to_owned(),
        program: program.to_owned(),
        recipe: recipe.clone(),
        commands,
    })
}

fn resolved_toolchain(recipe: &InstallRecipe) -> Result<String, ToolInstallError> {
    let requested = recipe.toolchain_program();
    if requested == "python" {
        return python_program().ok_or_else(|| ToolInstallError::MissingToolchain {
            program: "python".to_owned(),
            recipe: recipe.label().to_owned(),
        });
    }
    if program_is_available(requested) {
        return Ok(requested.to_owned());
    }
    // Windows npm/dotnet often resolve as npm.cmd via PATHEXT through resolve_command_path.
    let path_env = effective_path_env();
    resolve_command_path(requested, std::slice::from_ref(&path_env), None).ok_or(
        ToolInstallError::MissingToolchain {
            program: requested.to_owned(),
            recipe: recipe.label().to_owned(),
        },
    )
}

fn python_program() -> Option<String> {
    for candidate in ["python3", "python", "py"] {
        if program_is_available(candidate) {
            return Some(candidate.to_owned());
        }
    }
    None
}

fn commands_for_recipe(
    recipe: &InstallRecipe,
    package: &std::path::Path,
    program: &str,
    toolchain: &str,
) -> Result<Vec<InstallCommand>, ToolInstallError> {
    let mut env = vec![effective_path_env()];
    let cwd = package.to_path_buf();
    let commands = match recipe {
        InstallRecipe::Npm { packages } => {
            let mut args = vec!["install".to_owned(), "--prefix".to_owned()];
            args.push(package.to_string_lossy().into_owned());
            args.extend(packages.iter().cloned());
            vec![command(
                format!("npm install {}", packages.join(" ")),
                toolchain,
                args,
                env,
                cwd,
            )]
        }
        InstallRecipe::DotnetTool {
            package: tool,
            prerelease,
        } => {
            let mut args = vec![
                "tool".to_owned(),
                "install".to_owned(),
                tool.clone(),
                "--tool-path".to_owned(),
                package.to_string_lossy().into_owned(),
            ];
            if *prerelease {
                args.push("--prerelease".to_owned());
            }
            vec![command(
                format!("dotnet tool install {tool}"),
                toolchain,
                args,
                env,
                cwd,
            )]
        }
        InstallRecipe::Go { module } => {
            env.push(("GOBIN".to_owned(), package.to_string_lossy().into_owned()));
            vec![command(
                format!("go install {module}"),
                toolchain,
                vec!["install".to_owned(), module.clone()],
                env,
                cwd,
            )]
        }
        InstallRecipe::Pip { packages } => {
            let venv = package.join(".venv");
            let pip = venv_pip(&venv);
            vec![
                command(
                    "python -m venv".to_owned(),
                    toolchain,
                    vec![
                        "-m".to_owned(),
                        "venv".to_owned(),
                        venv.to_string_lossy().into_owned(),
                    ],
                    env.clone(),
                    cwd.clone(),
                ),
                command(
                    format!("pip install {}", packages.join(" ")),
                    pip.to_string_lossy().into_owned(),
                    {
                        let mut args = vec!["install".to_owned()];
                        args.extend(packages.iter().cloned());
                        args
                    },
                    env,
                    cwd,
                ),
            ]
        }
        InstallRecipe::Cargo { crate_name } => vec![command(
            format!("cargo install {crate_name}"),
            toolchain,
            vec![
                "install".to_owned(),
                "--root".to_owned(),
                package.to_string_lossy().into_owned(),
                crate_name.clone(),
            ],
            env,
            cwd,
        )],
        InstallRecipe::Archive { url, .. } => archive_commands(url, package, program, env)?,
    };
    Ok(commands)
}

fn archive_commands(
    url: &str,
    package: &std::path::Path,
    program: &str,
    env: Vec<(String, String)>,
) -> Result<Vec<InstallCommand>, ToolInstallError> {
    if !program_is_available("curl") {
        return Err(ToolInstallError::MissingToolchain {
            program: "curl".to_owned(),
            recipe: "archive".to_owned(),
        });
    }
    let file_name = url.rsplit('/').next().unwrap_or("download.bin");
    let archive_path = package.join(file_name);
    let download = command(
        format!("curl {file_name}"),
        "curl",
        vec![
            "-fsSL".to_owned(),
            "-o".to_owned(),
            archive_path.to_string_lossy().into_owned(),
            url.to_owned(),
        ],
        env.clone(),
        package.to_path_buf(),
    );
    let lower = file_name.to_ascii_lowercase();
    if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        return Ok(vec![
            download,
            command(
                "tar extract".to_owned(),
                "tar",
                vec![
                    "-xzf".to_owned(),
                    archive_path.to_string_lossy().into_owned(),
                    "-C".to_owned(),
                    package.to_string_lossy().into_owned(),
                ],
                env,
                package.to_path_buf(),
            ),
        ]);
    }
    if lower.ends_with(".zip") || lower.ends_with(".vsix") {
        return Ok(vec![
            download,
            command(
                "tar extract".to_owned(),
                "tar",
                vec![
                    "-xf".to_owned(),
                    archive_path.to_string_lossy().into_owned(),
                    "-C".to_owned(),
                    package.to_string_lossy().into_owned(),
                ],
                env,
                package.to_path_buf(),
            ),
        ]);
    }
    if lower.ends_with(".gz") {
        let dest = package.join(program);
        let extract = if cfg!(windows) {
            command(
                format!("gzip {program}"),
                "cmd",
                vec![
                    "/C".to_owned(),
                    format!(
                        "gzip -dc \"{}\" > \"{}\"",
                        archive_path.display(),
                        dest.display()
                    ),
                ],
                env,
                package.to_path_buf(),
            )
        } else {
            command(
                format!("gzip {program}"),
                "sh",
                vec![
                    "-c".to_owned(),
                    format!(
                        "gzip -dc \"{}\" > \"{}\" && chmod +x \"{}\"",
                        archive_path.display(),
                        dest.display(),
                        dest.display()
                    ),
                ],
                env,
                package.to_path_buf(),
            )
        };
        return Ok(vec![download, extract]);
    }
    Ok(vec![download])
}

fn venv_pip(venv: &std::path::Path) -> PathBuf {
    if cfg!(windows) {
        venv.join("Scripts").join("pip.exe")
    } else {
        venv.join("bin").join("pip")
    }
}

fn command(
    label: String,
    program: impl Into<String>,
    args: Vec<String>,
    env: Vec<(String, String)>,
    cwd: PathBuf,
) -> InstallCommand {
    InstallCommand {
        label,
        program: program.into(),
        args,
        env,
        cwd,
    }
}

#[cfg(test)]
#[path = "plan_tests.rs"]
mod tests;
