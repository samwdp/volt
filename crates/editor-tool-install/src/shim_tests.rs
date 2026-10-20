use super::{shim_contents, shim_path};
use std::path::Path;

#[test]
fn windows_shim_invokes_quoted_target() {
    let contents = shim_contents(Path::new(r"C:\volt\lsp\packages\x\tool.exe"));
    if cfg!(windows) {
        assert!(contents.contains("@echo off"));
        assert!(contents.contains(r"C:\volt\lsp\packages\x\tool.exe"));
    } else {
        assert!(contents.starts_with("#!/bin/sh"));
    }
}

#[test]
fn powershell_script_shim_wraps_pwsh_file() {
    let contents = shim_contents(Path::new(
        r"C:\volt\lsp\packages\powershell-editor-services\PowerShellEditorServices\Start-EditorServices.ps1",
    ));
    assert!(
        contents.contains("pwsh -NoLogo -NoProfile -File"),
        "shim should launch PowerShell scripts through pwsh: {contents}"
    );
    assert!(contents.contains("Start-EditorServices.ps1"));
}

#[test]
fn shim_path_adds_cmd_on_windows() {
    let path = shim_path(Path::new("/bin"), "typescript-language-server");
    if cfg!(windows) {
        assert!(path.ends_with("typescript-language-server.cmd"));
    } else {
        assert!(path.ends_with("typescript-language-server"));
    }
}
