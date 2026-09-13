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

/// Inline inherited workspace package fields so a staged User Source Tree builds without
/// `[workspace.package]`. Handles dotted-key and table-form TOML.
pub fn inline_workspace_package_fields(mut manifest: String) -> String {
    manifest = manifest.replace("rust-version.workspace = true", "rust-version = \"1.91\"");
    manifest = manifest.replace("version.workspace = true", "version = \"0.1.0\"");
    manifest = manifest.replace("edition.workspace = true", "edition = \"2024\"");
    manifest = manifest.replace(
        "license.workspace = true",
        "license = \"MIT OR Apache-2.0\"",
    );
    manifest = manifest.replace("[package.edition]\nworkspace = true", "edition = \"2024\"");
    manifest = manifest.replace("[package.version]\nworkspace = true", "version = \"0.1.0\"");
    manifest = manifest.replace(
        "[package.license]\nworkspace = true",
        "license = \"MIT OR Apache-2.0\"",
    );
    manifest = manifest.replace(
        "[package.rust-version]\nworkspace = true",
        "rust-version = \"1.91\"",
    );
    manifest = manifest.replace(
        "lints.workspace = true",
        "[lints.rust]\nunsafe_code = \"forbid\"\nunused_crate_dependencies = \"warn\"\n\n[lints.clippy]\ndbg_macro = \"deny\"\ntodo = \"deny\"\nunwrap_used = \"deny\"",
    );
    manifest = manifest.replace(
        "[lints]\nworkspace = true",
        "[lints.rust]\nunsafe_code = \"forbid\"\nunused_crate_dependencies = \"warn\"\n\n[lints.clippy]\ndbg_macro = \"deny\"\ntodo = \"deny\"\nunwrap_used = \"deny\"",
    );
    manifest
}

#[cfg(test)]
mod tests {
    use super::{
        inline_workspace_package_fields, manifest_path_dependencies,
        standalone_user_path_replacements, standalone_user_vendor_crates,
    };
    use std::{
        fs,
        path::{Path, PathBuf},
    };

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .expect("workspace root")
            .to_path_buf()
    }

    #[test]
    fn standalone_user_vendor_crates_is_empty_when_plugin_sdk_is_a_leaf() {
        let workspace_root = workspace_root();
        let vendor_crates =
            standalone_user_vendor_crates(&workspace_root, &workspace_root.join("user"))
                .expect("collect standalone user vendor crates");
        assert!(
            vendor_crates.is_empty(),
            "Plugin SDK must not path-depend on Volt crates, so staging must not vendor them: {vendor_crates:?}"
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

        assert!(
            user_manifest.is_empty(),
            "User Library must not path-depend on crates/: {user_manifest:?}"
        );
        assert!(
            sdk_manifest.is_empty(),
            "Plugin SDK must not path-depend on crates/: {sdk_manifest:?}"
        );
    }

    fn user_library_source_files(root: &Path) -> Vec<PathBuf> {
        let mut files = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let entries = fs::read_dir(&dir).expect("read user library directory");
            for entry in entries {
                let entry = entry.expect("read user library entry");
                let path = entry.path();
                let file_type = entry.file_type().expect("user library file type");
                if file_type.is_dir() {
                    if path.file_name().and_then(|name| name.to_str()) == Some("sdk") {
                        continue;
                    }
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                    files.push(path);
                }
            }
        }
        files
    }

    #[test]
    fn user_library_modules_outside_sdk_import_only_plugin_sdk() {
        let user_root = workspace_root().join("user");
        let mut violations = Vec::new();
        for path in user_library_source_files(&user_root) {
            let source = fs::read_to_string(&path).expect("read user library source");
            for (index, line) in source.lines().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("//") {
                    continue;
                }
                let Some(import) = trimmed.strip_prefix("use ") else {
                    continue;
                };
                if import.starts_with("editor_") && !import.starts_with("editor_plugin_api") {
                    violations.push(format!("{}:{}: {trimmed}", path.display(), index + 1));
                }
            }
        }
        assert!(
            violations.is_empty(),
            "User Library modules must import Volt types only via editor_plugin_api:\n{}",
            violations.join("\n")
        );
    }

    #[test]
    fn user_library_manifest_path_depends_only_on_plugin_sdk() {
        let workspace_root = workspace_root();
        let manifest_path = workspace_root.join("user").join("Cargo.toml");
        let replacements =
            standalone_user_path_replacements(&manifest_path, &workspace_root, "vendor")
                .expect("user manifest replacements");
        assert!(
            replacements.is_empty(),
            "User Library must not path-depend on Volt crates under crates/: {replacements:?}"
        );

        let dependencies =
            manifest_path_dependencies(&manifest_path).expect("user path dependencies");
        let unique_paths = dependencies
            .iter()
            .map(|dependency| dependency.raw_path.as_str())
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(
            unique_paths,
            ["sdk"].into_iter().collect(),
            "User Library must path-depend only on the Plugin SDK"
        );
    }

    #[test]
    fn inline_workspace_package_fields_inlines_table_form_inheritance() {
        let manifest = "\
[package]
name = \"demo\"

[package.edition]
workspace = true

[package.version]
workspace = true

[package.license]
workspace = true

[package.rust-version]
workspace = true

[lints]
workspace = true
";
        let inlined = inline_workspace_package_fields(manifest.to_owned());

        assert!(inlined.contains("edition = \"2024\""));
        assert!(inlined.contains("version = \"0.1.0\""));
        assert!(inlined.contains("license = \"MIT OR Apache-2.0\""));
        assert!(inlined.contains("rust-version = \"1.91\""));
        assert!(inlined.contains("[lints.rust]"));
        assert!(inlined.contains("unsafe_code = \"forbid\""));
        assert!(!inlined.contains("[package.edition]"));
        assert!(!inlined.contains("[package.version]"));
        assert!(!inlined.contains("[package.license]"));
        assert!(!inlined.contains("[package.rust-version]"));
        assert!(!inlined.contains("[lints]\nworkspace = true"));
        assert!(!inlined.contains("workspace = true"));
    }

    #[test]
    fn inline_workspace_package_fields_keeps_dotted_key_inheritance_working() {
        let manifest = "\
[package]
name = \"demo\"
edition.workspace = true
version.workspace = true
license.workspace = true
rust-version.workspace = true
lints.workspace = true
";
        let inlined = inline_workspace_package_fields(manifest.to_owned());

        assert!(inlined.contains("edition = \"2024\""));
        assert!(inlined.contains("version = \"0.1.0\""));
        assert!(inlined.contains("license = \"MIT OR Apache-2.0\""));
        assert!(inlined.contains("rust-version = \"1.91\""));
        assert!(inlined.contains("[lints.rust]"));
        assert!(!inlined.contains("edition.workspace = true"));
        assert!(!inlined.contains("version.workspace = true"));
        assert!(!inlined.contains("license.workspace = true"));
        assert!(!inlined.contains("rust-version.workspace = true"));
        assert!(!inlined.contains("lints.workspace = true"));
        assert!(!inlined.contains("workspace = true"));
    }
}
