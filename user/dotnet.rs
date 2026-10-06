//! NuGet package management via magit-style section-tree buffers.

use editor_plugin_api::{
    PickerActionSpec, PickerItemSpec, PickerProviderContext, PickerProviderSpec, PluginAction,
    PluginBuffer, PluginCommand, PluginKeyBinding, PluginKeymapScope, PluginPackage,
    PluginSectionContext, PluginSectionResult, PluginVimMode, Section, SectionAction, SectionItem,
    SectionTree, buffer_kinds, plugin_hooks, volt,
};
use serde_json::Value;

pub const PACKAGE_NAME: &str = "dotnet";
pub const PACKAGES_KIND: &str = buffer_kinds::DOTNET_PACKAGES;
pub const SEARCH_KIND: &str = buffer_kinds::DOTNET_SEARCH;
pub const PACKAGES_BUFFER: &str = "*dotnet-packages*";
pub const SEARCH_BUFFER: &str = "*dotnet-search*";
pub const PACKAGES_HANDLER: &str = "dotnet.packages";
pub const SEARCH_HANDLER: &str = "dotnet.search";
pub const CSPROJ_PICKER_ID: &str = "dotnet.csproj";
pub const SEARCH_PROMPT_ID: &str = "plugin.dotnet.search";

const ACTION_UPDATE: &str = "dotnet.update";
const ACTION_DELETE: &str = "dotnet.delete";
const ACTION_INSTALL: &str = "dotnet.install";
const ACTION_SELECT_PROJECT: &str = "dotnet.select-project";

/// Returns the NuGet packages plugin package.
pub fn package() -> PluginPackage {
    PluginPackage::new(
        PACKAGE_NAME,
        true,
        "NuGet package list, search, install, update, and delete via section-tree buffers.",
    )
    .with_commands(vec![
        PluginCommand::new(
            "dotnet.open",
            "Opens the NuGet packages section buffer for the workspace.",
            vec![
                PluginAction::open_buffer(PACKAGES_BUFFER, PACKAGES_KIND, None::<&str>),
                PluginAction::emit_hook(plugin_hooks::SECTION_REFRESH, None::<&str>),
            ],
        ),
        PluginCommand::new(
            "dotnet.refresh",
            "Refreshes the focused NuGet section buffer.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_REFRESH,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dotnet.search",
            "Opens NuGet search and prompts for a query.",
            vec![
                PluginAction::open_buffer(SEARCH_BUFFER, SEARCH_KIND, None::<&str>),
                PluginAction::emit_hook(
                    plugin_hooks::PROMPT,
                    Some(format!("{SEARCH_PROMPT_ID}|NuGet search: |")),
                ),
            ],
        ),
        PluginCommand::new(
            "dotnet.add",
            "Installs the package under the cursor (picker when multiple projects).",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_ACTIVATE,
                Some(ACTION_INSTALL),
            )],
        ),
        PluginCommand::new(
            "dotnet.update",
            "Updates the package under the cursor.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_ACTIVATE,
                Some(ACTION_UPDATE),
            )],
        ),
        PluginCommand::new(
            "dotnet.delete",
            "Removes the package under the cursor.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_ACTIVATE,
                Some(ACTION_DELETE),
            )],
        ),
        PluginCommand::new(
            "dotnet.prev-item",
            "Moves to the previous package row.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_PREV_ITEM,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dotnet.next-item",
            "Moves to the next package row.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_NEXT_ITEM,
                None::<&str>,
            )],
        ),
        PluginCommand::new(
            "dotnet.activate",
            "Activates the section action under the cursor.",
            vec![PluginAction::emit_hook(
                plugin_hooks::SECTION_ACTIVATE,
                None::<&str>,
            )],
        ),
    ])
    .with_buffers(vec![
        PluginBuffer::new(PACKAGES_KIND, vec!["Loading NuGet packages…".to_owned()])
            .with_section_handler(PACKAGES_HANDLER)
            .with_line_wrap(false)
            .with_key_bindings(packages_key_bindings()),
        PluginBuffer::new(SEARCH_KIND, vec!["Search NuGet packages…".to_owned()])
            .with_section_handler(SEARCH_HANDLER)
            .with_line_wrap(false)
            .with_key_bindings(search_key_bindings()),
    ])
}

fn packages_key_bindings() -> Vec<PluginKeyBinding> {
    vec![
        PluginKeyBinding::new("h", "dotnet.prev-item", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("j", "dotnet.next-item", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("s", "dotnet.search", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("a", "dotnet.add", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("u", "dotnet.update", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("d", "dotnet.delete", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("g r", "dotnet.refresh", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ]
}

fn search_key_bindings() -> Vec<PluginKeyBinding> {
    vec![
        PluginKeyBinding::new("h", "dotnet.prev-item", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("j", "dotnet.next-item", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("a", "dotnet.add", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Enter", "dotnet.activate", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
        PluginKeyBinding::new("Return", "dotnet.activate", PluginKeymapScope::Workspace)
            .with_vim_mode(PluginVimMode::Normal),
    ]
}

/// Picker provider that lists `.csproj` paths stored in the provider title detail.
pub fn csproj_picker_provider() -> PickerProviderSpec {
    PickerProviderSpec::user(CSPROJ_PICKER_ID, "Choose project")
}

/// Resolves picker rows for [`CSPROJ_PICKER_ID`] from workspace files ending in `.csproj`.
pub fn csproj_picker_items(context: &PickerProviderContext) -> Option<Vec<PickerItemSpec>> {
    if context.provider_id.as_str() != CSPROJ_PICKER_ID {
        return None;
    }
    let projects: Vec<String> = context
        .workspace_files
        .iter()
        .filter_map(|file| {
            let path = file.path.as_str();
            path.ends_with(".csproj").then(|| path.to_owned())
        })
        .collect();
    if projects.is_empty() {
        return Some(Vec::new());
    }
    Some(
        projects
            .into_iter()
            .map(|path| {
                PickerItemSpec::new(
                    path.clone(),
                    path.clone(),
                    "",
                    PickerActionSpec::emit_hook(
                        plugin_hooks::SECTION_ACTIVATE,
                        Some(format!("{ACTION_SELECT_PROJECT}|{path}")),
                    ),
                )
            })
            .collect(),
    )
}

/// Section-tree handler entry used by the User Library module.
pub fn handle_section(
    handler: &str,
    context: &PluginSectionContext,
) -> Option<PluginSectionResult> {
    match handler {
        PACKAGES_HANDLER => Some(handle_packages(context)),
        SEARCH_HANDLER => Some(handle_search(context)),
        _ => None,
    }
}

fn handle_packages(context: &PluginSectionContext) -> PluginSectionResult {
    let root = context
        .workspace_root()
        .map(str::to_owned)
        .or_else(volt::workspace_root);
    if let Some(action_id) = context.action_id() {
        match action_id {
            ACTION_UPDATE | ACTION_DELETE | ACTION_INSTALL => {
                return mutate_cursor_package(context, action_id, root.as_deref());
            }
            ACTION_SELECT_PROJECT => {
                if let Some(project) = context.action_detail() {
                    return install_pending_into_project(context, project, root.as_deref());
                }
            }
            id if id == SEARCH_PROMPT_ID => {
                return open_search_from_prompt(context.action_detail().unwrap_or(""));
            }
            _ => {}
        }
    }
    refresh_packages_tree(root.as_deref())
}

fn handle_search(context: &PluginSectionContext) -> PluginSectionResult {
    let root = context
        .workspace_root()
        .map(str::to_owned)
        .or_else(volt::workspace_root);
    if let Some(action_id) = context.action_id() {
        match action_id {
            ACTION_INSTALL | ACTION_SELECT_PROJECT => {
                return mutate_cursor_package(context, action_id, root.as_deref());
            }
            id if id == SEARCH_PROMPT_ID => {
                let query = context.action_detail().unwrap_or("").to_owned();
                return search_packages_tree(&query, root.as_deref());
            }
            _ => {}
        }
    }
    let query = context
        .opaque_state()
        .and_then(|opaque| {
            serde_json::from_str::<Value>(opaque)
                .ok()
                .and_then(|value| {
                    value
                        .get("query")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                })
        })
        .unwrap_or_default();
    search_packages_tree(&query, root.as_deref())
}

fn open_search_from_prompt(query: &str) -> PluginSectionResult {
    // Opening the search buffer is done by emitting OpenBuffer via a follow-up
    // host path when the packages handler returns a search-redirect opaque.
    // For lean wiring: rebuild packages tree and stash query so the search
    // command path can open the buffer separately. Here we just return search tree
    // so activate-from-prompt on packages buffer still shows results if host keeps focus.
    search_packages_tree(query, volt::workspace_root().as_deref())
}

fn refresh_packages_tree(root: Option<&str>) -> PluginSectionResult {
    let output = run_dotnet(&["package", "list", "--format", "json"], root);
    let tree = match parse_package_list_json(&output.stdout) {
        Ok(tree) => tree,
        Err(error) => error_tree("packages", &error, &output),
    };
    let projects = collect_projects_from_list(&output.stdout);
    let opaque = serde_json::json!({ "projects": projects }).to_string();
    PluginSectionResult::new(tree).with_opaque_state(opaque)
}

fn search_packages_tree(query: &str, root: Option<&str>) -> PluginSectionResult {
    if query.trim().is_empty() {
        let tree = SectionTree::new(vec![Section::new(
            "dotnet.search.empty",
            "Enter a search query (press s from packages)",
        )]);
        return PluginSectionResult::new(tree)
            .with_opaque_state(serde_json::json!({ "query": "" }).to_string());
    }
    let output = run_dotnet(&["package", "search", query, "--format", "json"], root);
    let tree = match parse_package_search_json(&output.stdout) {
        Ok(tree) => tree,
        Err(error) => error_tree("search", &error, &output),
    };
    PluginSectionResult::new(tree)
        .with_opaque_state(serde_json::json!({ "query": query }).to_string())
}

fn mutate_cursor_package(
    context: &PluginSectionContext,
    action_id: &str,
    root: Option<&str>,
) -> PluginSectionResult {
    let detail = context.action_detail().map(str::to_owned).or_else(|| {
        volt::buf::current()
            .and_then(volt::buf::section_action)
            .and_then(|(_, detail)| detail)
    });
    let Some(detail) = detail else {
        return refresh_for_kind(context, root);
    };
    if action_id == ACTION_SELECT_PROJECT {
        return install_pending_into_project(context, &detail, root);
    }
    let package = parse_package_ref(&detail);
    let project_from_detail = serde_json::from_str::<Value>(&detail)
        .ok()
        .and_then(|value| {
            value
                .get("project")
                .and_then(Value::as_str)
                .map(str::to_owned)
        });
    let projects = projects_from_opaque(context.opaque_state());
    if action_id == ACTION_INSTALL && project_from_detail.is_none() && projects.len() > 1 {
        return choose_project_tree(&package, &projects);
    }
    let project = project_from_detail
        .as_deref()
        .or_else(|| projects.first().map(String::as_str));
    let _ = match action_id {
        ACTION_UPDATE => run_package_mutation("update", &package, project, root),
        ACTION_DELETE => run_package_mutation("remove", &package, project, root),
        ACTION_INSTALL => run_package_mutation("add", &package, project, root),
        _ => SilentOk,
    };
    refresh_for_kind(context, root)
}

fn choose_project_tree(package: &str, projects: &[String]) -> PluginSectionResult {
    let items = projects
        .iter()
        .map(|path| {
            let detail = serde_json::json!({ "id": package, "project": path }).to_string();
            SectionItem::new(path.clone())
                .with_action(SectionAction::new(ACTION_SELECT_PROJECT).with_detail(detail))
        })
        .collect();
    let tree = SectionTree::new(vec![
        Section::new("dotnet.choose-project", format!("Install {package} into")).with_items(items),
    ]);
    let opaque = serde_json::json!({
        "projects": projects,
        "pending_package": package,
    })
    .to_string();
    PluginSectionResult::new(tree).with_opaque_state(opaque)
}

fn install_pending_into_project(
    context: &PluginSectionContext,
    detail: &str,
    root: Option<&str>,
) -> PluginSectionResult {
    let parsed = serde_json::from_str::<Value>(detail).ok();
    let package = parsed
        .as_ref()
        .and_then(|value| value.get("id").and_then(Value::as_str))
        .map(str::to_owned)
        .or_else(|| {
            context.opaque_state().and_then(|opaque| {
                serde_json::from_str::<Value>(opaque)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("pending_package")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
            })
        })
        .unwrap_or_else(|| parse_package_ref(detail));
    let project = parsed
        .as_ref()
        .and_then(|value| value.get("project").and_then(Value::as_str))
        .unwrap_or(detail);
    if !package.is_empty() {
        let _ = run_package_mutation("add", &package, Some(project), root);
    }
    refresh_packages_tree(root)
}

fn refresh_for_kind(context: &PluginSectionContext, root: Option<&str>) -> PluginSectionResult {
    if context.kind() == SEARCH_KIND {
        let query = context
            .opaque_state()
            .and_then(|opaque| {
                serde_json::from_str::<Value>(opaque)
                    .ok()
                    .and_then(|value| {
                        value
                            .get("query")
                            .and_then(Value::as_str)
                            .map(str::to_owned)
                    })
            })
            .unwrap_or_default();
        search_packages_tree(&query, root)
    } else {
        refresh_packages_tree(root)
    }
}

struct SilentOk;

fn run_package_mutation(
    verb: &str,
    package: &str,
    project: Option<&str>,
    root: Option<&str>,
) -> SilentOk {
    let mut args = vec!["package", verb];
    if let Some(project) = project {
        args.push("--project");
        args.push(project);
    }
    args.push(package);
    let _ = run_dotnet(&args, root);
    SilentOk
}

fn run_dotnet(args: &[&str], cwd: Option<&str>) -> volt::job::SilentCommandResult {
    volt::job::run_silent("dotnet", args, cwd).unwrap_or(volt::job::SilentCommandResult {
        code: 1,
        stdout: String::new(),
        stderr: "dotnet host call failed".to_owned(),
    })
}

fn error_tree(kind: &str, error: &str, output: &volt::job::SilentCommandResult) -> SectionTree {
    let mut items = vec![SectionItem::new(format!("error: {error}"))];
    if !output.stderr.trim().is_empty() {
        items.push(SectionItem::new(output.stderr.lines().next().unwrap_or("")));
    }
    if output.code != 0 {
        items.push(SectionItem::new(format!("exit code {}", output.code)));
    }
    SectionTree::new(vec![
        Section::new(format!("dotnet.{kind}.error"), "dotnet CLI").with_items(items),
    ])
}

/// Parses `dotnet package list --format json` into a section per project.
pub fn parse_package_list_json(stdout: &str) -> Result<SectionTree, String> {
    let value: Value = serde_json::from_str(stdout)
        .map_err(|error| format!("invalid package list JSON: {error}"))?;
    let projects = value
        .get("projects")
        .and_then(Value::as_array)
        .ok_or_else(|| "package list JSON missing projects[]".to_owned())?;
    let mut sections = Vec::new();
    for project in projects {
        let path = project
            .get("path")
            .and_then(Value::as_str)
            .unwrap_or("project")
            .to_owned();
        let frameworks = project
            .get("frameworks")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        let mut items = Vec::new();
        for framework in frameworks {
            let top_level = framework
                .get("topLevelPackages")
                .and_then(Value::as_array)
                .cloned()
                .unwrap_or_default();
            for package in top_level {
                let id = package
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown");
                let requested = package
                    .get("requestedVersion")
                    .and_then(Value::as_str)
                    .unwrap_or("?");
                let resolved = package
                    .get("resolvedVersion")
                    .and_then(Value::as_str)
                    .unwrap_or(requested);
                let detail = serde_json::json!({
                    "id": id,
                    "project": path,
                    "version": resolved,
                })
                .to_string();
                items.push(
                    SectionItem::new(format!("{id}  {resolved}"))
                        .with_action(SectionAction::new(ACTION_UPDATE).with_detail(detail)),
                );
            }
        }
        if items.is_empty() {
            items.push(SectionItem::new("(no top-level packages)".to_owned()));
        }
        sections.push(Section::new(format!("dotnet.project.{path}"), path).with_items(items));
    }
    if sections.is_empty() {
        sections.push(Section::new(
            "dotnet.packages.empty",
            "No projects with packages found",
        ));
    }
    Ok(SectionTree::new(sections))
}

/// Parses `dotnet package search --format json` into selectable package rows.
pub fn parse_package_search_json(stdout: &str) -> Result<SectionTree, String> {
    let value: Value =
        serde_json::from_str(stdout).map_err(|error| format!("invalid search JSON: {error}"))?;
    let search_results = value
        .get("searchResult")
        .and_then(Value::as_array)
        .ok_or_else(|| "search JSON missing searchResult[]".to_owned())?;
    let mut items = Vec::new();
    for source in search_results {
        let packages = source
            .get("packages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default();
        for package in packages {
            let id = package
                .get("id")
                .and_then(Value::as_str)
                .unwrap_or("unknown");
            let version = package
                .get("latestVersion")
                .or_else(|| package.get("version"))
                .and_then(Value::as_str)
                .unwrap_or("?");
            let detail = serde_json::json!({ "id": id, "version": version }).to_string();
            items.push(
                SectionItem::new(format!("{id}  {version}"))
                    .with_action(SectionAction::new(ACTION_INSTALL).with_detail(detail)),
            );
        }
    }
    if items.is_empty() {
        items.push(SectionItem::new("(no packages)".to_owned()));
    }
    Ok(SectionTree::new(vec![
        Section::new("dotnet.search.results", "Search results").with_items(items),
    ]))
}

fn collect_projects_from_list(stdout: &str) -> Vec<String> {
    let Ok(value) = serde_json::from_str::<Value>(stdout) else {
        return Vec::new();
    };
    value
        .get("projects")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|project| {
            project
                .get("path")
                .and_then(Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

fn projects_from_opaque(opaque: Option<&str>) -> Vec<String> {
    opaque
        .and_then(|opaque| serde_json::from_str::<Value>(opaque).ok())
        .and_then(|value| {
            value
                .get("projects")
                .and_then(Value::as_array)
                .map(|items| {
                    items
                        .iter()
                        .filter_map(|item| item.as_str().map(str::to_owned))
                        .collect()
                })
        })
        .unwrap_or_default()
}

fn parse_package_ref(detail: &str) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(detail)
        && let Some(id) = value.get("id").and_then(Value::as_str)
    {
        return id.to_owned();
    }
    detail
        .split_whitespace()
        .next()
        .unwrap_or(detail)
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::{parse_package_list_json, parse_package_search_json};

    #[test]
    fn package_list_json_builds_section_per_project() {
        let json = r#"{
          "version": 1,
          "projects": [
            {
              "path": "App.csproj",
              "frameworks": [
                {
                  "framework": "net8.0",
                  "topLevelPackages": [
                    {
                      "id": "Newtonsoft.Json",
                      "requestedVersion": "13.0.1",
                      "resolvedVersion": "13.0.3"
                    }
                  ]
                }
              ]
            }
          ]
        }"#;
        let tree = parse_package_list_json(json).expect("parse list");
        assert_eq!(tree.sections().len(), 1);
        assert_eq!(tree.sections()[0].title(), "App.csproj");
        assert_eq!(tree.sections()[0].items().len(), 1);
        assert!(
            tree.sections()[0].items()[0]
                .text()
                .contains("Newtonsoft.Json")
        );
        assert!(tree.sections()[0].items()[0].action().is_some());
    }

    #[test]
    fn package_search_json_builds_installable_rows() {
        let json = r#"{
          "searchResult": [
            {
              "sourceName": "nuget.org",
              "packages": [
                { "id": "Serilog", "latestVersion": "4.0.0" },
                { "id": "Serilog.Sinks.File", "version": "6.0.0" }
              ]
            }
          ]
        }"#;
        let tree = parse_package_search_json(json).expect("parse search");
        assert_eq!(tree.sections().len(), 1);
        assert_eq!(tree.sections()[0].items().len(), 2);
        assert_eq!(
            tree.sections()[0].items()[0]
                .action()
                .map(|action| action.id()),
            Some("dotnet.install")
        );
    }
}
