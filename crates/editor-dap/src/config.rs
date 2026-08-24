//! Hybrid Debug Configuration resolution: project files, inference, and history.

use std::{
    collections::VecDeque,
    error::Error,
    fmt, fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::{DebugConfiguration, DebugRequestKind};

/// Relative path for optional project Debug Configurations.
pub const PROJECT_DEBUG_CONFIG_PATH: &str = ".volt/debug.json";

/// How many recent starts `dap.start-recent` keeps.
pub const DEBUG_START_HISTORY_LIMIT: usize = 16;

/// Where a Debug Configuration candidate came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugConfigurationSource {
    /// Template derived from compiled adapter defaults.
    CompiledDefault,
    /// Loaded from the project `.volt/debug.json`.
    Project,
    /// Inferred from the open file / project markers.
    Inferred,
    /// Replayed from start history.
    History,
}

impl DebugConfigurationSource {
    /// Short label for picker detail text.
    pub const fn label(self) -> &'static str {
        match self {
            Self::CompiledDefault => "compiled default",
            Self::Project => "project",
            Self::Inferred => "inferred",
            Self::History => "recent",
        }
    }
}

/// A selectable Debug Configuration plus its origin and optional adapter pin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugConfigurationCandidate {
    configuration: DebugConfiguration,
    source: DebugConfigurationSource,
    adapter_id: Option<String>,
}

impl DebugConfigurationCandidate {
    /// Creates a candidate.
    pub fn new(
        configuration: DebugConfiguration,
        source: DebugConfigurationSource,
        adapter_id: Option<String>,
    ) -> Self {
        Self {
            configuration,
            source,
            adapter_id,
        }
    }

    /// Returns the configuration.
    pub fn configuration(&self) -> &DebugConfiguration {
        &self.configuration
    }

    /// Consumes the candidate and returns the configuration.
    pub fn into_configuration(self) -> DebugConfiguration {
        self.configuration
    }

    /// Returns the source.
    pub const fn source(&self) -> DebugConfigurationSource {
        self.source
    }

    /// Returns a pinned adapter id when the candidate names one.
    pub fn adapter_id(&self) -> Option<&str> {
        self.adapter_id.as_deref()
    }

    /// Picker label.
    pub fn picker_label(&self) -> String {
        let request = match self.configuration.request() {
            DebugRequestKind::Launch => "launch",
            DebugRequestKind::Attach => "attach",
        };
        match self.adapter_id() {
            Some(adapter) => format!("{} ({request} · {adapter})", self.configuration.name()),
            None => format!("{} ({request})", self.configuration.name()),
        }
    }

    /// Picker detail line.
    pub fn picker_detail(&self) -> String {
        let mut parts = vec![self.source.label().to_owned()];
        if let Some(program) = self.configuration.target_program() {
            parts.push(program.display().to_string());
        }
        if let Some(compile) = self.configuration.compile_command() {
            parts.push(format!("compile: {compile}"));
        }
        parts.join(" · ")
    }
}

/// One remembered successful Debug Session start.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DebugStartRecord {
    adapter_id: String,
    configuration: DebugConfiguration,
}

impl DebugStartRecord {
    /// Creates a history record.
    pub fn new(adapter_id: impl Into<String>, configuration: DebugConfiguration) -> Self {
        Self {
            adapter_id: adapter_id.into(),
            configuration,
        }
    }

    /// Returns the adapter id used for the start.
    pub fn adapter_id(&self) -> &str {
        &self.adapter_id
    }

    /// Returns the configuration used for the start.
    pub fn configuration(&self) -> &DebugConfiguration {
        &self.configuration
    }

    /// Converts this record into a selectable candidate.
    pub fn to_candidate(&self) -> DebugConfigurationCandidate {
        DebugConfigurationCandidate::new(
            self.configuration.clone(),
            DebugConfigurationSource::History,
            Some(self.adapter_id.clone()),
        )
    }
}

/// In-memory last/recent Debug Configuration history for the process lifetime.
#[derive(Debug, Default, Clone)]
pub struct DebugStartHistory {
    last: Option<DebugStartRecord>,
    recent: VecDeque<DebugStartRecord>,
}

impl DebugStartHistory {
    /// Creates an empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Records a successful start (dedupes identical adapter+name at the front).
    pub fn record(&mut self, adapter_id: impl Into<String>, configuration: DebugConfiguration) {
        let record = DebugStartRecord::new(adapter_id, configuration);
        self.recent.retain(|existing| {
            !(existing.adapter_id() == record.adapter_id()
                && existing.configuration().name() == record.configuration().name())
        });
        self.recent.push_front(record.clone());
        while self.recent.len() > DEBUG_START_HISTORY_LIMIT {
            self.recent.pop_back();
        }
        self.last = Some(record);
    }

    /// Returns the most recent start, if any.
    pub fn last(&self) -> Option<&DebugStartRecord> {
        self.last.as_ref()
    }

    /// Returns recent starts, newest first.
    pub fn recent(&self) -> impl Iterator<Item = &DebugStartRecord> {
        self.recent.iter()
    }

    /// Returns whether any history exists.
    pub fn is_empty(&self) -> bool {
        self.recent.is_empty()
    }
}

/// Errors from loading or resolving Debug Configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DapConfigError {
    /// Project config file could not be read.
    Io(String),
    /// Project config file was not valid JSON / schema.
    Parse(String),
}

impl fmt::Display for DapConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(message) => write!(formatter, "debug config io error: {message}"),
            Self::Parse(message) => write!(formatter, "debug config parse error: {message}"),
        }
    }
}

impl Error for DapConfigError {}

#[derive(Debug, Deserialize)]
struct ProjectDebugFile {
    #[serde(default)]
    configurations: Vec<ProjectDebugConfiguration>,
}

#[derive(Debug, Deserialize)]
struct ProjectDebugConfiguration {
    name: String,
    #[serde(default)]
    adapter: Option<String>,
    #[serde(default = "default_request")]
    request: String,
    #[serde(default)]
    program: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default)]
    compile: Option<String>,
}

fn default_request() -> String {
    "launch".to_owned()
}

/// Loads optional project Debug Configurations from `.volt/debug.json`.
pub fn load_project_configurations(
    workspace_root: &Path,
) -> Result<Vec<DebugConfigurationCandidate>, DapConfigError> {
    let path = workspace_root.join(PROJECT_DEBUG_CONFIG_PATH);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents =
        fs::read_to_string(&path).map_err(|error| DapConfigError::Io(error.to_string()))?;
    let file: ProjectDebugFile = serde_json::from_str(&contents)
        .map_err(|error| DapConfigError::Parse(error.to_string()))?;
    let mut candidates = Vec::with_capacity(file.configurations.len());
    for entry in file.configurations {
        let request = match entry.request.trim().to_ascii_lowercase().as_str() {
            "attach" => DebugRequestKind::Attach,
            "launch" => DebugRequestKind::Launch,
            other => {
                return Err(DapConfigError::Parse(format!(
                    "unknown request `{other}` in {}",
                    path.display()
                )));
            }
        };
        let mut configuration = DebugConfiguration::new(entry.name, request);
        if let Some(program) = entry.program {
            configuration =
                configuration.with_target_program(resolve_path(workspace_root, &program));
        }
        if let Some(cwd) = entry.cwd {
            configuration = configuration.with_cwd(resolve_path(workspace_root, &cwd));
        } else {
            configuration = configuration.with_cwd(workspace_root.to_path_buf());
        }
        if !entry.args.is_empty() {
            configuration = configuration.with_args(entry.args);
        }
        if let Some(compile) = entry.compile {
            configuration = configuration.with_compile_command(compile);
        }
        if let Some(adapter) = &entry.adapter {
            configuration = configuration.with_adapter_id(adapter.clone());
        }
        candidates.push(DebugConfigurationCandidate::new(
            configuration,
            DebugConfigurationSource::Project,
            entry.adapter,
        ));
    }
    Ok(candidates)
}

/// Context for inferring Debug Configurations.
#[derive(Debug, Clone)]
pub struct DebugInferContext<'a> {
    /// Project Workspace root, when present.
    pub workspace_root: Option<&'a Path>,
    /// Active buffer path, when present.
    pub active_file: Option<&'a Path>,
    /// Preferred adapter id when the user already chose one.
    pub preferred_adapter_id: Option<&'a str>,
    /// When false, skip deep project inference (Default Workspace).
    pub allow_deep_inference: bool,
}

/// Builds hybrid candidates: project configs, then inferred / compiled defaults.
pub fn collect_configuration_candidates(
    ctx: &DebugInferContext<'_>,
) -> Result<Vec<DebugConfigurationCandidate>, DapConfigError> {
    let mut candidates = Vec::new();
    if let Some(root) = ctx.workspace_root {
        candidates.extend(load_project_configurations(root)?);
    }

    if let Some(preferred) = ctx.preferred_adapter_id {
        candidates.retain(|candidate| {
            candidate
                .adapter_id()
                .is_none_or(|adapter_id| adapter_id == preferred)
        });
    }

    candidates.extend(infer_configurations(ctx));
    Ok(candidates)
}

/// Infers launch configurations from the open file / project markers.
pub fn infer_configurations(ctx: &DebugInferContext<'_>) -> Vec<DebugConfigurationCandidate> {
    let Some(active_file) = ctx.active_file else {
        return Vec::new();
    };
    let cwd = ctx
        .workspace_root
        .map(Path::to_path_buf)
        .or_else(|| active_file.parent().map(Path::to_path_buf));

    let mut candidates = Vec::new();

    if ctx.allow_deep_inference
        && let Some(root) = ctx.workspace_root
        && let Some(inferred) = infer_rust_cargo_binary(root, active_file)
    {
        let mut configuration = DebugConfiguration::new("Debug (Cargo)", DebugRequestKind::Launch)
            .with_target_program(inferred)
            .with_compile_command("cargo build");
        if let Some(cwd) = &cwd {
            configuration = configuration.with_cwd(cwd.clone());
        }
        if let Some(adapter) = ctx.preferred_adapter_id {
            configuration = configuration.with_adapter_id(adapter);
        }
        candidates.push(DebugConfigurationCandidate::new(
            configuration,
            DebugConfigurationSource::Inferred,
            ctx.preferred_adapter_id.map(str::to_owned),
        ));
    }

    if ctx.allow_deep_inference
        && let Some(root) = ctx.workspace_root
        && let Some(dll) = infer_dotnet_dll(root, active_file)
    {
        let mut configuration = DebugConfiguration::new("Debug (dotnet)", DebugRequestKind::Launch)
            .with_target_program(dll)
            .with_compile_command("dotnet build");
        if let Some(cwd) = &cwd {
            configuration = configuration.with_cwd(cwd.clone());
        }
        if let Some(adapter) = ctx.preferred_adapter_id {
            configuration = configuration.with_adapter_id(adapter);
        }
        candidates.push(DebugConfigurationCandidate::new(
            configuration,
            DebugConfigurationSource::Inferred,
            ctx.preferred_adapter_id.map(str::to_owned),
        ));
    }

    // Shallow default: launch the open file path (useful for scripts / Default Workspace).
    // Skip for C# when we already inferred a project — launching the .cs source is useless.
    let skip_current_file = candidates.iter().any(|candidate| {
        candidate.configuration().name() == "Debug (dotnet)"
            || candidate.configuration().name() == "Debug (Cargo)"
    });
    if !skip_current_file {
        let mut configuration =
            DebugConfiguration::new("Debug (current file)", DebugRequestKind::Launch)
                .with_target_program(active_file.to_path_buf());
        if let Some(cwd) = cwd {
            configuration = configuration.with_cwd(cwd);
        }
        if let Some(adapter) = ctx.preferred_adapter_id {
            configuration = configuration.with_adapter_id(adapter);
        }
        candidates.push(DebugConfigurationCandidate::new(
            configuration,
            if ctx.allow_deep_inference {
                DebugConfigurationSource::CompiledDefault
            } else {
                DebugConfigurationSource::Inferred
            },
            ctx.preferred_adapter_id.map(str::to_owned),
        ));
    }

    candidates
}

/// Suggests a heuristic compile-before-debug command when the config has none.
pub fn infer_compile_heuristic(
    workspace_root: Option<&Path>,
    active_file: Option<&Path>,
) -> Option<String> {
    let root = workspace_root?;
    let extension = active_file
        .and_then(|path| path.extension())
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase());
    match extension.as_deref() {
        Some("rs") if root.join("Cargo.toml").is_file() => Some("cargo build".to_owned()),
        Some("cs") if active_file.is_some_and(|file| find_csproj(root, file).is_some()) => {
            Some("dotnet build".to_owned())
        }
        Some("c" | "cpp" | "cc" | "cxx" | "h" | "hpp") if root.join("Makefile").is_file() => {
            Some("make".to_owned())
        }
        Some("c" | "cpp" | "cc" | "cxx" | "h" | "hpp") if root.join("CMakeLists.txt").is_file() => {
            Some("cmake --build build".to_owned())
        }
        _ => None,
    }
}

/// Returns whether a configuration is missing fields that block start.
pub fn configuration_holes(configuration: &DebugConfiguration) -> Vec<&'static str> {
    let mut holes = Vec::new();
    match configuration.request() {
        DebugRequestKind::Launch => {
            if configuration.target_program().is_none() {
                holes.push("program");
            }
        }
        DebugRequestKind::Attach => {
            if configuration.target_program().is_none() && configuration.process_id().is_none() {
                holes.push("program or process id");
            }
        }
    }
    holes
}

fn resolve_path(root: &Path, value: &str) -> PathBuf {
    let path = PathBuf::from(value);
    if path.is_absolute() {
        path
    } else {
        root.join(path)
    }
}

fn first_project_file(dir: &Path, extensions: &[&str]) -> Option<PathBuf> {
    let mut entries = fs::read_dir(dir)
        .ok()?
        .filter_map(Result::ok)
        .collect::<Vec<_>>();
    entries.sort_by_key(|entry| entry.file_name());
    entries.into_iter().find_map(|entry| {
        let path = entry.path();
        let ext = path.extension()?.to_str()?;
        extensions
            .iter()
            .any(|wanted| ext.eq_ignore_ascii_case(wanted))
            .then_some(path)
    })
}

fn find_csproj(root: &Path, active_file: &Path) -> Option<PathBuf> {
    let is_csharp = active_file
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("cs"));
    if !is_csharp {
        return None;
    }
    let mut current = active_file.parent()?;
    loop {
        if let Some(project) = first_project_file(current, &["csproj"]) {
            return Some(project);
        }
        if current == root {
            break;
        }
        current = current.parent()?;
        if !current.starts_with(root) && current != root {
            break;
        }
    }
    first_project_file(root, &["csproj"])
}

/// Resolves the Debug-build DLL path for a nearby .csproj (SharpDbg needs `program`).
fn infer_dotnet_dll(root: &Path, active_file: &Path) -> Option<PathBuf> {
    let csproj = find_csproj(root, active_file)?;
    let contents = fs::read_to_string(&csproj).ok()?;
    let assembly = parse_csproj_property(&contents, "AssemblyName").unwrap_or_else(|| {
        csproj
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| "App".to_owned())
    });
    let tfm = parse_csproj_property(&contents, "TargetFramework")
        .or_else(|| {
            parse_csproj_property(&contents, "TargetFrameworks").and_then(|frameworks| {
                frameworks
                    .split([';', ' '])
                    .map(str::trim)
                    .find(|part| !part.is_empty())
                    .map(str::to_owned)
            })
        })
        .unwrap_or_else(|| "net8.0".to_owned());
    let project_dir = csproj.parent()?;
    let candidate = project_dir
        .join("bin")
        .join("Debug")
        .join(&tfm)
        .join(format!("{assembly}.dll"));
    if candidate.exists() {
        return Some(candidate);
    }
    if let Some(existing) = find_existing_debug_dll(project_dir, &assembly) {
        return Some(existing);
    }
    // Expected path after `dotnet build` (mirrors cargo inference before first build).
    Some(candidate)
}

fn find_existing_debug_dll(project_dir: &Path, assembly: &str) -> Option<PathBuf> {
    let debug_root = project_dir.join("bin").join("Debug");
    let entries = fs::read_dir(&debug_root).ok()?;
    let mut matches = Vec::new();
    for entry in entries.filter_map(Result::ok) {
        let tfm_dir = entry.path();
        if !tfm_dir.is_dir() {
            continue;
        }
        let dll = tfm_dir.join(format!("{assembly}.dll"));
        if dll.is_file() {
            matches.push(dll);
        }
    }
    matches.sort();
    matches.pop()
}

fn parse_csproj_property(contents: &str, name: &str) -> Option<String> {
    let open = format!("<{name}>");
    let close = format!("</{name}>");
    let start = contents.find(&open)? + open.len();
    let rest = &contents[start..];
    let end = rest.find(&close)?;
    let value = rest[..end].trim();
    (!value.is_empty()).then(|| value.to_owned())
}

fn infer_rust_cargo_binary(root: &Path, active_file: &Path) -> Option<PathBuf> {
    let cargo_toml = root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        return None;
    }
    let contents = fs::read_to_string(cargo_toml).ok()?;
    let package_name = parse_cargo_package_name(&contents)?;
    let binary_name = package_name.replace('-', "_");
    #[cfg(windows)]
    let binary = format!("{binary_name}.exe");
    #[cfg(not(windows))]
    let binary = binary_name;
    let candidate = root.join("target").join("debug").join(binary);
    if candidate.exists() || active_file.extension().and_then(|ext| ext.to_str()) == Some("rs") {
        Some(candidate)
    } else {
        None
    }
}

fn parse_cargo_package_name(contents: &str) -> Option<String> {
    let mut in_package = false;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != "name" {
            continue;
        }
        let value = value.trim().trim_matches('"').trim_matches('\'');
        if !value.is_empty() {
            return Some(value.to_owned());
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_dir(label: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!(
            "volt-dap-config-{}-{}-{}",
            label,
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|duration| duration.as_nanos())
                .unwrap_or(0)
        ));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("temp dir");
        path
    }

    #[test]
    fn loads_project_debug_configurations() {
        let root = temp_dir("project");
        let volt_dir = root.join(".volt");
        fs::create_dir_all(&volt_dir).expect("volt dir");
        fs::write(
            volt_dir.join("debug.json"),
            r#"{
              "configurations": [
                {
                  "name": "Debug volt",
                  "adapter": "codelldb",
                  "request": "launch",
                  "program": "target/debug/volt",
                  "compile": "cargo build",
                  "args": ["--shell-hidden"]
                },
                {
                  "name": "Attach demo",
                  "adapter": "gdb",
                  "request": "attach",
                  "program": "target/debug/volt"
                }
              ]
            }"#,
        )
        .expect("write debug.json");

        let candidates = load_project_configurations(&root).expect("load");
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates[0].configuration().name(), "Debug volt");
        assert_eq!(candidates[0].adapter_id(), Some("codelldb"));
        assert_eq!(
            candidates[0].configuration().compile_command(),
            Some("cargo build")
        );
        assert_eq!(candidates[0].source(), DebugConfigurationSource::Project);
        assert_eq!(
            candidates[1].configuration().request(),
            DebugRequestKind::Attach
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn deep_inference_finds_cargo_binary_and_heuristic() {
        let root = temp_dir("cargo");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo-app\"\nversion = \"0.1.0\"\n",
        )
        .expect("cargo");
        let main = root.join("src");
        fs::create_dir_all(&main).expect("src");
        let file = main.join("main.rs");
        fs::write(&file, "fn main() {}\n").expect("main");

        let ctx = DebugInferContext {
            workspace_root: Some(root.as_path()),
            active_file: Some(file.as_path()),
            preferred_adapter_id: Some("codelldb"),
            allow_deep_inference: true,
        };
        let inferred = infer_configurations(&ctx);
        assert!(
            inferred
                .iter()
                .any(|candidate| candidate.configuration().name() == "Debug (Cargo)")
        );
        assert_eq!(
            infer_compile_heuristic(Some(root.as_path()), Some(file.as_path())).as_deref(),
            Some("cargo build")
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn deep_inference_finds_dotnet_dll() {
        let root = temp_dir("dotnet");
        fs::write(
            root.join("App.csproj"),
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>Exe</OutputType><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#,
        )
        .expect("csproj");
        let file = root.join("Program.cs");
        fs::write(&file, "Console.WriteLine(\"hi\");\n").expect("cs");

        let ctx = DebugInferContext {
            workspace_root: Some(root.as_path()),
            active_file: Some(file.as_path()),
            preferred_adapter_id: Some("sharpdbg"),
            allow_deep_inference: true,
        };
        let inferred = infer_configurations(&ctx);
        assert_eq!(inferred.len(), 1);
        assert_eq!(inferred[0].configuration().name(), "Debug (dotnet)");
        assert_eq!(
            inferred[0].configuration().target_program(),
            Some(
                &root
                    .join("bin")
                    .join("Debug")
                    .join("net8.0")
                    .join("App.dll")
            )
        );
        assert_eq!(
            inferred[0].configuration().compile_command(),
            Some("dotnet build")
        );
        assert_eq!(
            infer_compile_heuristic(Some(root.as_path()), Some(file.as_path())).as_deref(),
            Some("dotnet build")
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn default_workspace_skips_deep_inference() {
        let root = temp_dir("default-ws");
        fs::write(
            root.join("Cargo.toml"),
            "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
        )
        .expect("cargo");
        let file = root.join("main.rs");
        fs::write(&file, "fn main() {}\n").expect("main");

        let ctx = DebugInferContext {
            workspace_root: None,
            active_file: Some(file.as_path()),
            preferred_adapter_id: None,
            allow_deep_inference: false,
        };
        let inferred = infer_configurations(&ctx);
        assert_eq!(inferred.len(), 1);
        assert_eq!(inferred[0].configuration().name(), "Debug (current file)");
        assert!(
            infer_compile_heuristic(None, Some(file.as_path())).is_none(),
            "no root → no heuristic compile"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn history_records_last_and_recent() {
        let mut history = DebugStartHistory::new();
        history.record(
            "codelldb",
            DebugConfiguration::new("one", DebugRequestKind::Launch),
        );
        history.record(
            "gdb",
            DebugConfiguration::new("two", DebugRequestKind::Launch),
        );
        assert_eq!(history.last().expect("last").adapter_id(), "gdb");
        let recent: Vec<_> = history.recent().map(|record| record.adapter_id()).collect();
        assert_eq!(recent, ["gdb", "codelldb"]);
    }

    #[test]
    fn configuration_holes_detect_missing_launch_program() {
        let config = DebugConfiguration::new("empty", DebugRequestKind::Launch);
        assert_eq!(configuration_holes(&config), ["program"]);
    }
}
