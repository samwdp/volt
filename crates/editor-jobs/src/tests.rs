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
    fn build_job_command_keeps_fnm_path_ahead_of_explicit_path() {
        let spec = JobSpec::command("node-version", "node", ["--version"])
            .with_env("PATH", "C:\\custom")
            .with_env("NODE_OPTIONS", "--trace-warnings");
        let command = super::build_job_command(
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
        let vars = command
            .get_envs()
            .filter_map(|(key, value)| {
                Some((
                    key.to_string_lossy().into_owned(),
                    value?.to_string_lossy().into_owned(),
                ))
            })
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
    fn build_job_command_keeps_nvm_path_ahead_of_explicit_path() {
        let spec = JobSpec::command("node-version", "node", ["--version"])
            .with_env("PATH", "C:\\custom")
            .with_env("NODE_OPTIONS", "--trace-warnings");
        let command = super::build_job_command(
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
        let vars = command
            .get_envs()
            .filter_map(|(key, value)| {
                Some((
                    key.to_string_lossy().into_owned(),
                    value?.to_string_lossy().into_owned(),
                ))
            })
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
        let version = super::parse_windows_nvm_current_version("v22.1.0\r\n")
            .expect("nvm current should parse");
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
