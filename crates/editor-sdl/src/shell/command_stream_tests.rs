use super::super::shell_command_eval_args;
use super::detect_build_command;
use std::{
    fs,
    time::{SystemTime, UNIX_EPOCH},
};

#[test]
fn nushell_command_eval_uses_login_flag() {
    assert_eq!(shell_command_eval_args("nu"), vec!["-l", "-c"]);
    assert_eq!(
        shell_command_eval_args(r"C:\Program Files\nu\bin\nu.exe"),
        vec!["-l", "-c"]
    );
}

#[test]
fn windows_shell_command_eval_flags_unchanged() {
    if !cfg!(windows) {
        return;
    }
    assert_eq!(shell_command_eval_args("cmd"), vec!["/C"]);
    assert_eq!(shell_command_eval_args("cmd.exe"), vec!["/C"]);
    assert_eq!(shell_command_eval_args("powershell"), vec!["-Command"]);
    assert_eq!(shell_command_eval_args("pwsh.exe"), vec!["-Command"]);
}

struct TempDir {
    path: std::path::PathBuf,
}

impl TempDir {
    fn new(tag: &str) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("volt-detect-build-{tag}-{unique}"));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn detects_cargo_toml() {
    let dir = TempDir::new("cargo");
    fs::write(dir.path.join("Cargo.toml"), "").expect("write");
    assert_eq!(detect_build_command(&dir.path), "cargo build");
}

#[test]
fn detects_sln() {
    let dir = TempDir::new("sln");
    fs::write(dir.path.join("MyApp.sln"), "").expect("write");
    assert_eq!(detect_build_command(&dir.path), "dotnet build");
}

#[test]
fn detects_csproj() {
    let dir = TempDir::new("csproj");
    fs::write(dir.path.join("MyApp.csproj"), "").expect("write");
    assert_eq!(detect_build_command(&dir.path), "dotnet build");
}

#[test]
fn detects_package_json() {
    let dir = TempDir::new("npm");
    fs::write(dir.path.join("package.json"), "{}").expect("write");
    assert_eq!(detect_build_command(&dir.path), "npm run build");
}

#[test]
fn detects_makefile() {
    let dir = TempDir::new("make");
    fs::write(dir.path.join("Makefile"), "").expect("write");
    assert_eq!(detect_build_command(&dir.path), "make");
}

#[test]
fn empty_dir_returns_empty_string() {
    let dir = TempDir::new("empty");
    assert_eq!(detect_build_command(&dir.path), "");
}

#[test]
fn cargo_toml_wins_over_other_markers() {
    let dir = TempDir::new("priority");
    fs::write(dir.path.join("Cargo.toml"), "").expect("write");
    fs::write(dir.path.join("package.json"), "{}").expect("write");
    fs::write(dir.path.join("Makefile"), "").expect("write");
    assert_eq!(detect_build_command(&dir.path), "cargo build");
}

#[test]
fn missing_dir_returns_empty_string() {
    let path = std::path::Path::new("/nonexistent/volt/test/dir");
    assert_eq!(detect_build_command(path), "");
}
