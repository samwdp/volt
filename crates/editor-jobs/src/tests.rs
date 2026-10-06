use super::{CompilationRunner, JobKind, JobManager, JobSpec};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

#[cfg(windows)]
fn temp_dir(prefix: &str) -> std::path::PathBuf {
    use std::time::{SystemTime, UNIX_EPOCH};
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{unique}"))
}

#[cfg(windows)]
#[test]
fn windows_fnm_npm_sibling_node_is_visible_after_double_enrich() {
    let root = std::env::temp_dir().join(format!(
        "volt-fnm-npm-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&root).expect("mkdir");
    let env = super::enrich_env_with_node_manager(
        Some(&root),
        vec![("PATH".to_owned(), r"C:\Windows\system32".to_owned())],
    );
    let Some(npm) = super::resolve_command_path("npm", &env, None) else {
        let _ = std::fs::remove_dir_all(&root);
        return;
    };
    let env = super::enrich_env_with_node_manager(Some(&root), env);
    let npm = super::resolve_command_path(&npm, &env, None).unwrap_or(npm);
    let node = std::path::Path::new(&npm)
        .parent()
        .expect("parent")
        .join("node.exe");
    assert!(
        node.is_file(),
        "resolved npm `{npm}` has no sibling node.exe (is_file={})",
        node.is_file()
    );
    let env = super::ensure_sibling_node_on_path(&npm, env);
    let path_keys = env
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .count();
    let path = env
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    let first = path.split(';').next().unwrap_or_default();
    assert_eq!(path_keys, 1, "duplicate PATH keys in launch env");
    assert!(
        std::path::Path::new(first).join("node.exe").is_file(),
        "first PATH entry `{first}` has no node.exe; npm=`{npm}`"
    );
    let _ = std::fs::remove_dir_all(root);
}

#[cfg(windows)]
#[test]
fn ensure_sibling_node_on_path_keeps_npm_path_under_cmd_limit() {
    let temp = temp_dir("volt-npm-path-budget");
    std::fs::create_dir_all(&temp).expect("mkdir");
    std::fs::write(temp.join("node.exe"), b"").expect("node");
    std::fs::write(temp.join("npm.cmd"), b"").expect("npm");
    let long_entry = "C:\\tools\\segment";
    let long_path = std::iter::repeat_n(long_entry, 400)
        .collect::<Vec<_>>()
        .join(";");
    let git = r"C:\Program Files\Git\cmd";
    let updated = super::ensure_sibling_node_on_path(
        &temp.join("npm.cmd").to_string_lossy(),
        vec![
            ("Path".to_owned(), format!("{git};{long_path}")),
            ("PATH".to_owned(), long_path.clone()),
        ],
    );
    let path_keys = updated
        .iter()
        .filter(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .count();
    let path = updated
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    let first = path.split(';').next().unwrap_or_default();
    assert_eq!(path_keys, 1);
    assert!(
        first.eq_ignore_ascii_case(&temp.to_string_lossy()),
        "node dir should lead PATH, got `{path}`"
    );
    assert!(path.len() <= 2_500, "npm PATH len {}", path.len());
    assert!(
        path.to_ascii_lowercase().contains(r"\system32"),
        "system32 missing from `{path}`"
    );
    assert!(
        path.to_ascii_lowercase().contains(r"\git\cmd"),
        "git missing from `{path}`"
    );
    let _ = std::fs::remove_dir_all(temp);
}

#[test]
fn ensure_sibling_node_on_path_prepends_npm_directory() {
    let temp = std::env::temp_dir().join(format!(
        "volt-sibling-node-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("time")
            .as_nanos()
    ));
    std::fs::create_dir_all(&temp).expect("mkdir");
    let node_name = if cfg!(windows) { "node.exe" } else { "node" };
    let npm_name = if cfg!(windows) { "npm.cmd" } else { "npm" };
    std::fs::write(temp.join(node_name), b"").expect("node");
    std::fs::write(temp.join(npm_name), b"").expect("npm");
    let npm = temp.join(npm_name);
    let updated = super::ensure_sibling_node_on_path(
        &npm.to_string_lossy(),
        vec![("PATH".to_owned(), "/usr/bin".to_owned())],
    );
    let path = updated
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    let separator = if cfg!(windows) { ';' } else { ':' };
    let first = path.split(separator).next().unwrap_or_default();
    assert!(
        first.eq_ignore_ascii_case(&temp.to_string_lossy()),
        "node dir should lead PATH, got `{path}`"
    );
    let _ = std::fs::remove_dir_all(temp);
}

#[test]
fn job_manager_runs_commands_and_collects_output() {
    let mut jobs = JobManager::new();
    let handle = must(jobs.spawn(JobSpec::command("rustc-version", "rustc", ["--version"])));
    let result = must(handle.wait());

    assert_eq!(result.spec().kind(), JobKind::Command);
    assert!(result.succeeded());
    assert!(result.stdout().contains("rustc"));
    assert!(result.duration().as_nanos() > 0);
}

#[test]
fn compilation_runner_marks_jobs_as_compilation() {
    let mut jobs = JobManager::new();
    let compilation = must(CompilationRunner::new().run(
        &mut jobs,
        JobSpec::compilation("rustc-version", "rustc", ["--version"]),
    ));

    assert_eq!(compilation.job().spec().kind(), JobKind::Compilation);
    assert!(compilation.succeeded());
    assert!(compilation.transcript().contains("rustc"));
}

#[cfg(windows)]
#[test]
fn windows_parse_cmd_environment_extracts_variables() {
    let env = super::parse_windows_cmd_environment(
        "SET PATH=C:\\fnm;C:\\tools\r\nSET FNM_DIR=C:\\Users\\sam\\AppData\\Roaming\\fnm\r\n",
    )
    .expect("fnm env should parse");
    assert_eq!(
        env,
        vec![
            ("PATH".to_owned(), "C:\\fnm;C:\\tools".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned()
            ),
        ]
    );
}

#[cfg(windows)]
#[test]
fn merge_windows_explicit_and_runtime_env_keeps_runtime_path_first() {
    let merged = super::merge_windows_explicit_and_runtime_env(
        &[
            ("PATH".to_owned(), "C:\\custom".to_owned()),
            ("NODE_OPTIONS".to_owned(), "--trace-warnings".to_owned()),
        ],
        &[
            ("PATH".to_owned(), "C:\\fnm".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned(),
            ),
        ],
    );
    let vars = merged
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        vars.get("PATH").map(String::as_str),
        Some("C:\\fnm;C:\\custom")
    );
    assert_eq!(
        vars.get("FNM_DIR").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\fnm")
    );
    assert_eq!(
        vars.get("NODE_OPTIONS").map(String::as_str),
        Some("--trace-warnings")
    );
}

#[test]
fn enrich_env_with_node_manager_preserves_explicit_vars_when_manager_missing() {
    let env = vec![("VOLT_TEST_MARKER".to_owned(), "1".to_owned())];
    let enriched = super::enrich_env_with_node_manager(None, env.clone());
    // When fnm/nvm cannot be resolved, explicit env is returned as-is. When they
    // can, the marker must still survive the merge.
    assert!(
        enriched
            .iter()
            .any(|(key, value)| key == "VOLT_TEST_MARKER" && value == "1")
    );
}

#[cfg(windows)]
#[test]
fn resolve_windows_fnm_program_uses_well_known_when_env_path_lacks_fnm() {
    use std::path::PathBuf;

    let well_known = std::env::var_os("LOCALAPPDATA")
        .map(|local| PathBuf::from(local).join(r"Microsoft\WinGet\Links\fnm.exe"));
    let Some(well_known) = well_known.filter(|path| path.is_file()) else {
        return;
    };

    // Explicit PATH wins over process PATH in resolve_command_path, so system32-only
    // forces the well-known fallback (CreateProcess bare-name lookup is separate).
    let env = vec![("PATH".to_owned(), r"C:\Windows\system32".to_owned())];
    let resolved = super::resolve_windows_fnm_program(&env).expect("fnm via well-known");
    assert!(
        PathBuf::from(&resolved).is_file(),
        "resolved fnm is missing: {resolved}"
    );
    assert!(
        resolved.eq_ignore_ascii_case(&well_known.to_string_lossy())
            || PathBuf::from(&resolved).file_name() == well_known.file_name(),
        "expected well-known fnm, got {resolved}"
    );
}

#[cfg(windows)]
#[test]
fn enrich_env_surfaces_npm_via_persistent_or_well_known_fnm() {
    use std::path::PathBuf;

    let fnm_installed = std::env::var_os("LOCALAPPDATA")
        .map(|local| PathBuf::from(local).join(r"Microsoft\WinGet\Links\fnm.exe"))
        .is_some_and(|path| path.is_file())
        || super::resolve_windows_fnm_program(&[(
            "PATH".to_owned(),
            r"C:\Windows\system32".to_owned(),
        )])
        .is_some();
    if !fnm_installed {
        return;
    }

    // GUI-like: env PATH has no node shims. Enrichment must still surface npm via
    // Machine/User Path and/or well-known fnm, then `fnm env`.
    let enriched = super::enrich_env_with_node_manager(
        None,
        vec![("PATH".to_owned(), r"C:\Windows\system32".to_owned())],
    );
    let path = enriched
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("PATH"))
        .map(|(_, value)| value.as_str())
        .unwrap_or_default();
    assert!(
        super::resolve_command_path("npm", &enriched, None).is_some(),
        "fnm installed but enrich still cannot resolve npm from GUI-like PATH; PATH={path}"
    );
}

#[cfg(windows)]
#[test]
fn build_job_launch_spec_keeps_fnm_path_ahead_of_explicit_path() {
    let spec = JobSpec::command("node-version", "node", ["--version"])
        .with_env("PATH", "C:\\custom")
        .with_env("NODE_OPTIONS", "--trace-warnings");
    let launch = super::build_job_launch_spec(
        &spec,
        "node",
        Some(&[
            ("PATH".to_owned(), "C:\\fnm".to_owned()),
            (
                "FNM_DIR".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\fnm".to_owned(),
            ),
        ]),
    );
    let vars = launch
        .env
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        vars.get("PATH").map(String::as_str),
        Some("C:\\fnm;C:\\custom")
    );
    assert_eq!(
        vars.get("FNM_DIR").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\fnm")
    );
    assert_eq!(
        vars.get("NODE_OPTIONS").map(String::as_str),
        Some("--trace-warnings")
    );
}

#[cfg(windows)]
#[test]
fn build_job_launch_spec_keeps_nvm_path_ahead_of_explicit_path() {
    let spec = JobSpec::command("node-version", "node", ["--version"])
        .with_env("PATH", "C:\\custom")
        .with_env("NODE_OPTIONS", "--trace-warnings");
    let launch = super::build_job_launch_spec(
        &spec,
        "node",
        Some(&[
            (
                "PATH".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\nvm\\v22.1.0".to_owned(),
            ),
            (
                "NVM_HOME".to_owned(),
                "C:\\Users\\sam\\AppData\\Roaming\\nvm".to_owned(),
            ),
        ]),
    );
    let vars = launch
        .env
        .into_iter()
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(
        vars.get("PATH").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\nvm\\v22.1.0;C:\\custom")
    );
    assert_eq!(
        vars.get("NVM_HOME").map(String::as_str),
        Some("C:\\Users\\sam\\AppData\\Roaming\\nvm")
    );
    assert_eq!(
        vars.get("NODE_OPTIONS").map(String::as_str),
        Some("--trace-warnings")
    );
}

#[cfg(windows)]
#[test]
fn windows_should_retry_invalid_exe_format() {
    let error = std::io::Error::from_raw_os_error(193);
    assert!(super::windows_should_retry_spawn_error(&error));
}

#[cfg(windows)]
#[test]
fn windows_fnm_launch_program_candidates_resolve_absolute_command_shims() {
    let temp_dir = temp_dir("volt-fnm-jobs");
    std::fs::create_dir_all(&temp_dir).expect("temp dir");
    let candidate_path = temp_dir.join("prettier.cmd");
    std::fs::write(&candidate_path, "@echo off\r\n").expect("candidate");

    let candidates = super::windows_fnm_launch_program_candidates(
        "prettier",
        &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
    );
    assert!(candidates.contains(&candidate_path.to_string_lossy().into_owned()));

    let _ = std::fs::remove_file(candidate_path);
    let _ = std::fs::remove_dir(temp_dir);
}

#[cfg(windows)]
#[test]
fn windows_fnm_launch_program_candidates_prefer_windows_shims_over_extensionless_scripts() {
    let temp_dir = temp_dir("volt-fnm-jobs");
    std::fs::create_dir_all(&temp_dir).expect("temp dir");
    let script_path = temp_dir.join("prettier");
    let shim_path = temp_dir.join("prettier.cmd");
    std::fs::write(&script_path, "#!/bin/sh\n").expect("script");
    std::fs::write(&shim_path, "@echo off\r\n").expect("shim");

    let candidates = super::windows_fnm_launch_program_candidates(
        "prettier",
        &[("PATH".to_owned(), temp_dir.to_string_lossy().into_owned())],
    );
    assert_eq!(
        candidates.first().map(String::as_str),
        Some(shim_path.to_string_lossy().as_ref())
    );
    assert!(candidates.contains(&script_path.to_string_lossy().into_owned()));

    let _ = std::fs::remove_file(script_path);
    let _ = std::fs::remove_file(shim_path);
    let _ = std::fs::remove_dir(temp_dir);
}

#[cfg(windows)]
#[test]
fn parse_windows_nvm_current_version_extracts_active_version() {
    let version =
        super::parse_windows_nvm_current_version("v22.1.0\r\n").expect("nvm current should parse");
    assert_eq!(version, "v22.1.0");
}

#[cfg(windows)]
#[test]
fn windows_nvm_node_dir_accepts_version_with_or_without_v_prefix() {
    let temp_dir = temp_dir("volt-nvm-jobs");
    let version_dir = temp_dir.join("v22.1.0");
    std::fs::create_dir_all(&version_dir).expect("version dir");
    std::fs::write(version_dir.join("node.exe"), []).expect("node exe");

    let resolved = super::windows_nvm_node_dir(&temp_dir, "22.1.0").expect("node dir");
    assert_eq!(resolved, version_dir);

    let _ = std::fs::remove_file(version_dir.join("node.exe"));
    let _ = std::fs::remove_dir(version_dir);
    let _ = std::fs::remove_dir(temp_dir);
}
