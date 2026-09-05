
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
