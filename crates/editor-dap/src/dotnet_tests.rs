use super::*;
use std::fs;

fn temp_dir(label: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!(
        "volt-dap-dotnet-{}-{}-{}",
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

fn write_project(dir: &Path, file_name: &str, output_type: &str) {
    fs::create_dir_all(dir).expect("project dir");
    fs::write(
        dir.join(file_name),
        format!(
            r#"<Project Sdk="Microsoft.NET.Sdk"><PropertyGroup><OutputType>{output_type}</OutputType><TargetFramework>net8.0</TargetFramework></PropertyGroup></Project>"#
        ),
    )
    .expect("csproj");
}

fn write_dll(project_dir: &Path, name: &str) -> PathBuf {
    let dir = project_dir.join("bin").join("Debug").join("net8.0");
    fs::create_dir_all(&dir).expect("bin");
    let path = dir.join(name);
    fs::write(&path, "").expect("dll");
    path
}

#[test]
fn nested_solution_lists_build_targets_and_entry_dlls_only() {
    let root = temp_dir("nested");
    let solution = root.join("src").join("dir1");
    let api = solution.join("dotnet_api");
    let tests = solution.join("dotnet_test_library");
    let shared = solution.join("dotnet_shared_library");
    write_project(&api, "api.csproj", "Exe");
    write_project(&tests, "test.csproj", "Library");
    write_project(&shared, "shared.csproj", "Library");
    fs::write(
        solution.join("a.sln"),
        r#"Microsoft Visual Studio Solution File, Format Version 12.00
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_api", "dotnet_api\api.csproj", "{11111111-1111-1111-1111-111111111111}"
EndProject
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_test_library", "dotnet_test_library\test.csproj", "{22222222-2222-2222-2222-222222222222}"
EndProject
Project("{FAE04EC0-301F-11D3-BF4B-00C04F79EFBC}") = "dotnet_shared_library", "dotnet_shared_library\shared.csproj", "{33333333-3333-3333-3333-333333333333}"
EndProject
"#,
    )
    .expect("sln");
    fs::write(api.join("bin").join("hidden.csproj"), "").ok();
    let _ = fs::create_dir_all(api.join("bin"));
    fs::write(api.join("bin").join("hidden.csproj"), "").expect("decoy project");

    let api_dll = write_dll(&api, "dotnet_api.dll");
    write_dll(&api, "Newtonsoft.Json.dll");
    write_dll(&tests, "dotnet_test_library.dll");
    write_dll(&shared, "dotnet_shared_library.dll");
    write_dll(&shared, "shared.dll");

    let active = api.join("Program.cs");
    fs::write(&active, "").expect("source");
    let targets = discover_dotnet_build_targets(&root, Some(&active));
    let labels = targets
        .iter()
        .map(|target| display_under_root(&root, target.path()))
        .collect::<Vec<_>>();
    assert!(
        labels.iter().all(|label| !label.contains("hidden.csproj")),
        "bin project leaked: {labels:?}"
    );
    assert_eq!(targets[0].kind(), DotnetBuildTargetKind::Project);
    assert!(targets[0].path().ends_with("api.csproj"));
    assert_eq!(targets.len(), 4);

    let command = dotnet_compile_command(&root, targets[0].path());
    assert!(command.contains("api.csproj"), "{command}");
    assert!(command.starts_with("dotnet build "), "{command}");

    let api_candidates = dotnet_program_candidates(&root, &api.join("api.csproj"));
    assert_eq!(api_candidates.len(), 1);
    assert_eq!(api_candidates[0].path(), api_dll.as_path());
    assert!(api_candidates[0].exists());
    assert!(api_candidates[0].picker_detail().contains("executable"));

    let solution_candidates = dotnet_program_candidates(&root, &solution.join("a.sln"));
    let names = solution_candidates
        .iter()
        .map(|candidate| {
            candidate
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert!(names.contains(&"dotnet_api.dll".to_owned()), "{names:?}");
    assert!(
        names.contains(&"dotnet_test_library.dll".to_owned()),
        "{names:?}"
    );
    assert!(
        names.contains(&"dotnet_shared_library.dll".to_owned()),
        "{names:?}"
    );
    assert!(names.contains(&"shared.dll".to_owned()), "{names:?}");
    assert!(
        !names.iter().any(|name| name == "Newtonsoft.Json.dll"),
        "{names:?}"
    );
    assert_eq!(solution_candidates[0].path(), api_dll.as_path());

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn unbuilt_project_offers_directory_name_and_project_stem() {
    let root = temp_dir("expected");
    let project = root.join("src").join("dotnet_api");
    write_project(&project, "api.csproj", "Exe");
    let candidates = dotnet_program_candidates(&root, &project.join("api.csproj"));
    let names = candidates
        .iter()
        .map(|candidate| {
            candidate
                .path()
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned()
        })
        .collect::<Vec<_>>();
    assert!(names.contains(&"dotnet_api.dll".to_owned()), "{names:?}");
    assert!(names.contains(&"api.dll".to_owned()), "{names:?}");
    assert!(candidates.iter().all(|candidate| !candidate.exists()));
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn spaced_project_path_is_quoted_and_parsed() {
    let root = temp_dir("spaces");
    let project = root.join("my app");
    write_project(&project, "app.csproj", "Exe");
    let command = dotnet_compile_command(&root, &project.join("app.csproj"));
    assert!(command.contains('"'), "{command}");
    let parsed = dotnet_build_target_from_command(&root, &command).expect("target");
    assert_eq!(parsed, project.join("app.csproj"));
    let _ = fs::remove_dir_all(&root);
}
