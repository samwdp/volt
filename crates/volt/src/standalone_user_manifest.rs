use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    fs,
    path::{Path, PathBuf},
};

use toml::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestPathReplacement {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ManifestPathDependency {
    raw_path: String,
    resolved_dir: PathBuf,
}

pub fn standalone_user_vendor_crates(
    workspace_root: &Path,
    user_source_dir: &Path,
) -> Result<BTreeSet<String>, String> {
    let workspace_root = canonicalize_path(workspace_root)?;
    let crates_root = canonicalize_path(&workspace_root.join("crates"))?;
    let mut manifests = VecDeque::from([canonicalize_path(&user_source_dir.join("Cargo.toml"))?]);
    let mut visited_manifests = BTreeSet::new();
    let mut vendor_crates = BTreeSet::new();

    while let Some(manifest_path) = manifests.pop_front() {
        if !visited_manifests.insert(manifest_path.clone()) {
            continue;
        }

        for dependency in manifest_path_dependencies(&manifest_path)? {
            if !dependency.resolved_dir.starts_with(&workspace_root) {
                continue;
            }

            let dependency_manifest = dependency.resolved_dir.join("Cargo.toml");
            if dependency_manifest.is_file() {
                manifests.push_back(canonicalize_path(&dependency_manifest)?);
            }

            if dependency.resolved_dir.starts_with(&crates_root) {
                let crate_name = dependency
                    .resolved_dir
                    .file_name()
                    .and_then(|name| name.to_str())
                    .ok_or_else(|| {
                        format!(
                            "failed to determine crate name for `{}`",
                            dependency.resolved_dir.display()
                        )
                    })?;
                vendor_crates.insert(crate_name.to_owned());
            }
        }
    }

    Ok(vendor_crates)
}

pub fn standalone_user_path_replacements(
    manifest_path: &Path,
    workspace_root: &Path,
    vendor_prefix: &str,
) -> Result<Vec<ManifestPathReplacement>, String> {
    let crates_root = canonicalize_path(&canonicalize_path(workspace_root)?.join("crates"))?;
    let mut replacements = BTreeMap::new();

    for dependency in manifest_path_dependencies(manifest_path)? {
        if !dependency.resolved_dir.starts_with(&crates_root) {
            continue;
        }

        let crate_name = dependency
            .resolved_dir
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| {
                format!(
                    "failed to determine crate name for `{}`",
                    dependency.resolved_dir.display()
                )
            })?;
        replacements.insert(dependency.raw_path, format!("{vendor_prefix}/{crate_name}"));
    }

    Ok(replacements
        .into_iter()
        .map(|(from, to)| ManifestPathReplacement { from, to })
        .collect())
}

fn manifest_path_dependencies(manifest_path: &Path) -> Result<Vec<ManifestPathDependency>, String> {
    let manifest_dir = manifest_path.parent().ok_or_else(|| {
        format!(
            "failed to determine manifest directory for `{}`",
            manifest_path.display()
        )
    })?;
    let manifest = fs::read_to_string(manifest_path)
        .map_err(|error| format!("failed to read `{}`: {error}", manifest_path.display()))?;
    let manifest = manifest.parse::<Value>().map_err(|error| {
        format!(
            "failed to parse Cargo manifest `{}`: {error}",
            manifest_path.display()
        )
    })?;
    let manifest = manifest.as_table().ok_or_else(|| {
        format!(
            "Cargo manifest `{}` did not parse to a table",
            manifest_path.display()
        )
    })?;
    let mut dependencies = Vec::new();
    collect_manifest_dependencies(manifest, manifest_dir, &mut dependencies)?;
    Ok(dependencies)
}

fn collect_manifest_dependencies(
    manifest: &toml::value::Table,
    manifest_dir: &Path,
    dependencies: &mut Vec<ManifestPathDependency>,
) -> Result<(), String> {
    for section in ["dependencies", "dev-dependencies", "build-dependencies"] {
        let Some(section) = manifest.get(section).and_then(Value::as_table) else {
            continue;
        };
        collect_dependency_section(section, manifest_dir, dependencies)?;
    }

    if let Some(targets) = manifest.get("target").and_then(Value::as_table) {
        for target in targets.values() {
            let Some(target_manifest) = target.as_table() else {
                continue;
            };
            collect_manifest_dependencies(target_manifest, manifest_dir, dependencies)?;
        }
    }

    Ok(())
}

fn collect_dependency_section(
    dependencies_table: &toml::value::Table,
    manifest_dir: &Path,
    dependencies: &mut Vec<ManifestPathDependency>,
) -> Result<(), String> {
    for dependency in dependencies_table.values() {
        let Some(dependency) = dependency.as_table() else {
            continue;
        };
        let Some(raw_path) = dependency.get("path").and_then(Value::as_str) else {
            continue;
        };
        dependencies.push(ManifestPathDependency {
            raw_path: raw_path.to_owned(),
            resolved_dir: canonicalize_path(&manifest_dir.join(raw_path))?,
        });
    }

    Ok(())
}

fn canonicalize_path(path: &Path) -> Result<PathBuf, String> {
    fs::canonicalize(path).map_err(|error| {
        format!(
            "failed to canonicalize `{}` while staging standalone user manifests: {error}",
            path.display()
        )
    })
}

#[cfg(test)]
mod tests {
    use super::{
        ManifestPathReplacement, standalone_user_path_replacements, standalone_user_vendor_crates,
    };
    use std::path::PathBuf;

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn standalone_user_vendor_crates_include_transitive_workspace_dependencies() {
        let workspace_root = workspace_root();
        let vendor_crates =
            standalone_user_vendor_crates(&workspace_root, &workspace_root.join("user"))
                .expect("collect standalone user vendor crates");

        assert!(vendor_crates.contains("editor-core"));
        assert!(vendor_crates.contains("editor-lsp"));
        assert!(vendor_crates.contains("editor-syntax"));
        assert!(
            vendor_crates.contains("editor-path"),
            "transitive workspace crates should be vendored for standalone user builds"
        );
    }

    #[test]
    fn standalone_user_path_replacements_target_vendor_siblings() {
        let workspace_root = workspace_root();
        let user_manifest = standalone_user_path_replacements(
            &workspace_root.join("user").join("Cargo.toml"),
            &workspace_root,
            "vendor",
        )
        .expect("user manifest replacements");
        let sdk_manifest = standalone_user_path_replacements(
            &workspace_root.join("user").join("sdk").join("Cargo.toml"),
            &workspace_root,
            "../vendor",
        )
        .expect("sdk manifest replacements");

        assert!(user_manifest.contains(&ManifestPathReplacement {
            from: "../crates/editor-core".to_owned(),
            to: "vendor/editor-core".to_owned(),
        }));
        assert!(user_manifest.contains(&ManifestPathReplacement {
            from: "../crates/editor-syntax".to_owned(),
            to: "vendor/editor-syntax".to_owned(),
        }));
        assert!(sdk_manifest.contains(&ManifestPathReplacement {
            from: "../../crates/editor-lsp".to_owned(),
            to: "../vendor/editor-lsp".to_owned(),
        }));
        assert!(sdk_manifest.contains(&ManifestPathReplacement {
            from: "../../crates/editor-theme".to_owned(),
            to: "../vendor/editor-theme".to_owned(),
        }));
    }
}
