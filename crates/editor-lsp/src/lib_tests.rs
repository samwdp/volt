    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    use editor_buffer::{TextPoint, TextRange};
    use serde_json::json;

    use super::{
        Diagnostic, DiagnosticSeverity, LanguageServerRegistry, LanguageServerRootStrategy,
        LanguageServerSpec, LspError, WorkspaceConfigurationValue,
    };

    fn rust_analyzer() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "rust-analyzer",
            "rust",
            ["rs"],
            "rust-analyzer",
            ["--stdio"],
        )
        .with_root_markers(["Cargo.toml", "rust-project.json"])
    }

    fn typescript_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "typescript-language-server",
            "typescript",
            ["ts", "tsx", "js", "jsx"],
            "typescript-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([
            ("tsx", "typescriptreact"),
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
        ])
    }

    fn csharp_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "csharp-ls",
            "csharp",
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_root_markers(["*.sln", "*.csproj"])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
    }

    fn dockerfile_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "dockerfile-language-server",
            "dockerfile",
            [] as [&str; 0],
            "dockerfile-language-server",
            ["--stdio"],
        )
        .with_file_names(["Dockerfile"])
        .with_file_globs(["Dockerfile.*"])
        .with_document_language_ids([("Dockerfile", "dockerfile"), ("Dockerfile.*", "dockerfile")])
    }

    fn dev_extension_server() -> LanguageServerSpec {
        LanguageServerSpec::new("dev-server", "dev", ["dev"], "dev-server", ["--stdio"])
    }

    fn tailwind_language_server() -> LanguageServerSpec {
        LanguageServerSpec::new(
            "tailwindcss-language-server",
            "html",
            ["html", "js", "jsx", "ts", "tsx"],
            "tailwindcss-language-server",
            ["--stdio"],
        )
        .with_document_language_ids([
            ("js", "javascript"),
            ("jsx", "javascriptreact"),
            ("ts", "typescript"),
            ("tsx", "typescriptreact"),
        ])
        .with_root_strategy(LanguageServerRootStrategy::MarkersOrWorkspace)
        .with_root_markers([
            "tailwind.config.js",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
            "tailwind.config.ts",
            "tailwind.config.cts",
            "tailwind.config.mts",
            "node_modules/tailwindcss/package.json",
        ])
        .with_activation_markers([
            "tailwind.config.js",
            "tailwind.config.cjs",
            "tailwind.config.mjs",
            "tailwind.config.ts",
            "tailwind.config.cts",
            "tailwind.config.mts",
            "node_modules/tailwindcss/package.json",
        ])
    }

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("volt-editor-lsp-{unique}"))
    }

    #[test]
    fn registry_resolves_rust_server_by_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(rust_analyzer()));

        let server = registry.server_for_extension(".rs").expect("server");
        assert_eq!(server.id(), "rust-analyzer");
        assert_eq!(server.root_markers(), ["Cargo.toml", "rust-project.json"]);
    }

    #[test]
    fn registry_allows_multiple_servers_for_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(LanguageServerSpec::new(
            "harper",
            "markdown",
            ["md"],
            "harper-ls",
            ["--stdio"],
        )));
        must(registry.register(LanguageServerSpec::new(
            "marksman",
            "markdown",
            ["md"],
            "marksman",
            ["server"],
        )));

        let servers = registry.servers_for_extension(".md");
        assert_eq!(servers.len(), 2);
        assert_eq!(servers[0].id(), "harper");
        assert_eq!(servers[1].id(), "marksman");
    }

    #[test]
    fn registry_resolves_servers_by_filename_and_glob() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        assert_eq!(
            registry
                .server_for_path(Path::new("Dockerfile"))
                .map(|server| server.id()),
            Some("dockerfile-language-server")
        );
        assert_eq!(
            registry
                .server_for_path(&dev_dockerfile)
                .map(|server| server.id()),
            Some("dockerfile-language-server")
        );
    }

    #[test]
    fn registry_prefers_filename_globs_over_extension_matches() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dev_extension_server()));
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        let servers = registry.servers_for_path(&dev_dockerfile);
        assert_eq!(servers.len(), 1);
        assert_eq!(servers[0].id(), "dockerfile-language-server");
    }

    #[test]
    fn prepared_session_contains_launch_spec_and_diagnostics() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(rust_analyzer()));

        let session = must(
            registry
                .prepare_session("rust-analyzer", Some(PathBuf::from("P:\\volt")))
                .map(|session| {
                    session.with_diagnostics(vec![Diagnostic::new(
                        "rust-analyzer",
                        "Example diagnostic",
                        DiagnosticSeverity::Warning,
                        TextRange::new(TextPoint::new(0, 0), TextPoint::new(0, 5)),
                    )])
                }),
        );

        assert_eq!(session.server_id(), "rust-analyzer");
        assert_eq!(session.language_id(), "rust");
        assert_eq!(session.launch().program(), "rust-analyzer");
        assert_eq!(session.launch().args(), ["--stdio"]);
        assert_eq!(session.diagnostics().len(), 1);
    }

    #[test]
    fn workspace_configuration_value_round_trips_through_json() {
        let value = WorkspaceConfigurationValue::from(json!({
            "csharp": {
                "format.enable": true,
                "maxLineLength": 120.0,
                "inlayHints": ["types", null],
            }
        }));

        assert_eq!(
            value.to_json_value(),
            json!({
                "csharp": {
                    "format.enable": true,
                    "maxLineLength": 120.0,
                    "inlayHints": ["types", null],
                }
            })
        );

        let csharp = value
            .as_object()
            .and_then(|settings| settings.get("csharp"))
            .and_then(WorkspaceConfigurationValue::as_object)
            .expect("csharp object");
        assert_eq!(
            csharp
                .get("format.enable")
                .and_then(WorkspaceConfigurationValue::as_bool),
            Some(true)
        );
        assert_eq!(
            csharp
                .get("maxLineLength")
                .and_then(WorkspaceConfigurationValue::as_number)
                .and_then(serde_json::Number::as_f64),
            Some(120.0)
        );
        let hints = csharp
            .get("inlayHints")
            .and_then(WorkspaceConfigurationValue::as_array)
            .expect("hint array");
        assert_eq!(hints[0].as_str(), Some("types"));
        assert!(hints[1].is_null());
    }

    #[test]
    fn language_server_spec_exposes_workspace_configuration_builders() {
        let spec = LanguageServerSpec::new(
            "csharp-ls",
            "csharp",
            ["cs"],
            "csharp-ls",
            ["--features", "razor-support,metadata-uris"],
        )
        .with_workspace_configuration_section("csharp")
        .with_workspace_configuration_settings(
            LanguageServerSpec::workspace_settings_object([(
                "csharp",
                LanguageServerSpec::workspace_settings_object([
                    (
                        "enableAnalyzersSupport",
                        WorkspaceConfigurationValue::from(true),
                    ),
                    (
                        "inlayHints",
                        LanguageServerSpec::workspace_settings_array([
                            WorkspaceConfigurationValue::from("types"),
                            LanguageServerSpec::workspace_settings_null(),
                        ]),
                    ),
                    (
                        "maxLineLength",
                        LanguageServerSpec::workspace_settings_float(120.0).expect("finite float"),
                    ),
                ]),
            )]),
        );

        assert_eq!(spec.workspace_configuration().section(), Some("csharp"));
        assert_eq!(spec.workspace_configuration_section(), Some("csharp"));
        assert_eq!(
            spec.workspace_configuration_settings_json(),
            Some(json!({
                "csharp": {
                    "enableAnalyzersSupport": true,
                    "inlayHints": ["types", null],
                    "maxLineLength": 120.0,
                }
            }))
        );
    }

    #[test]
    fn prepared_session_carries_workspace_configuration_from_spec() {
        let mut registry = LanguageServerRegistry::new();
        must(
            registry.register(
                LanguageServerSpec::new(
                    "csharp-ls",
                    "csharp",
                    ["cs"],
                    "csharp-ls",
                    ["--features", "razor-support,metadata-uris"],
                )
                .with_workspace_configuration(
                    "csharp",
                    LanguageServerSpec::workspace_settings_object([(
                        "csharp",
                        LanguageServerSpec::workspace_settings_object([
                            (
                                "enableAnalyzersSupport",
                                WorkspaceConfigurationValue::from(true),
                            ),
                            ("sdk", WorkspaceConfigurationValue::from("dotnet")),
                        ]),
                    )]),
                ),
            ),
        );

        let session = must(registry.prepare_session("csharp-ls", Some(PathBuf::from("P:\\volt"))));

        assert_eq!(session.workspace_configuration().section(), Some("csharp"));
        assert_eq!(session.workspace_configuration_section(), Some("csharp"));
        assert_eq!(
            session.workspace_configuration_settings_json(),
            Some(json!({
                "csharp": {
                    "enableAnalyzersSupport": true,
                    "sdk": "dotnet",
                }
            }))
        );

        let overridden = session.with_workspace_configuration_settings(
            LanguageServerSpec::workspace_settings_object([("logging", true.into())]),
        );
        assert_eq!(
            overridden.workspace_configuration_settings_json(),
            Some(json!({
                "logging": true,
            }))
        );
    }

    #[test]
    fn prepared_session_resolves_document_language_ids_per_extension() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(typescript_language_server()));

        let session = must(registry.prepare_session(
            "typescript-language-server",
            Some(PathBuf::from("P:\\volt")),
        ));

        assert_eq!(session.language_id(), "typescript");
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.ts")),
            "typescript"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.tsx")),
            "typescriptreact"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.js")),
            "javascript"
        );
        assert_eq!(
            session.document_language_id_for_path(Path::new("app.jsx")),
            "javascriptreact"
        );
    }

    #[test]
    fn prepared_session_resolves_document_language_ids_by_filename_and_glob() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));
        let dev_dockerfile = Path::new("containers").join("Dockerfile.dev");

        let session = must(registry.prepare_session(
            "dockerfile-language-server",
            Some(PathBuf::from("P:\\volt")),
        ));

        assert_eq!(
            session.document_language_id_for_path(Path::new("Dockerfile")),
            "dockerfile"
        );
        assert_eq!(
            session.document_language_id_for_path(&dev_dockerfile),
            "dockerfile"
        );
    }

    #[test]
    fn prepared_session_for_path_prefers_solution_root_over_nested_csharp_project() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(root.join("App.sln"), "").expect("solution");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(session.launch().cwd(), Some(&root));
        assert_eq!(
            session.launch().args(),
            [
                "--features",
                "razor-support,metadata-uris",
                "--solution",
                "App.sln"
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_finds_solution_above_nested_workspace() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(root.join("App.sln"), "").expect("solution");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session = must(registry.prepare_session_for_path(
            "csharp-ls",
            &file_path,
            Some(project_dir.as_path()),
        ));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(session.launch().cwd(), Some(&root));
        assert_eq!(
            session.launch().args(),
            [
                "--features",
                "razor-support,metadata-uris",
                "--solution",
                "App.sln"
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_finds_solution_above_git_project_workspace() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(root.join("App.sln"), "").expect("solution");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        fs::write(project_dir.join(".git"), "gitdir: /tmp/fake").expect("git");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session = must(registry.prepare_session_for_path(
            "csharp-ls",
            &file_path,
            Some(project_dir.as_path()),
        ));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(
            session.launch().args(),
            [
                "--features",
                "razor-support,metadata-uris",
                "--solution",
                "App.sln"
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_finds_unique_solution_outside_file_ancestors() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        let solution_dir = root.join("solutions");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::create_dir_all(&solution_dir).expect("solution dir");
        fs::write(solution_dir.join("App.sln"), "").expect("solution");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&solution_dir));
        assert_eq!(session.launch().cwd(), Some(&solution_dir));
        assert_eq!(
            session.launch().args(),
            [
                "--features",
                "razor-support,metadata-uris",
                "--solution",
                "App.sln"
            ]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_falls_back_to_csproj_without_solution_arg() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&project_dir));
        assert_eq!(session.launch().cwd(), Some(&project_dir));
        assert_eq!(
            session.launch().args(),
            ["--features", "razor-support,metadata-uris"]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_skips_solution_arg_when_multiple_solutions_exist() {
        let root = temp_dir();
        let project_dir = root.join("src").join("Api");
        fs::create_dir_all(&project_dir).expect("project dir");
        fs::write(root.join("App.sln"), "").expect("solution");
        fs::write(root.join("App.Tests.sln"), "").expect("solution");
        fs::write(project_dir.join("Api.csproj"), "").expect("project");
        let file_path = project_dir.join("Program.cs");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(
            session.launch().args(),
            ["--features", "razor-support,metadata-uris"]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepared_session_for_path_falls_back_to_workspace_root_when_markers_do_not_match() {
        let root = temp_dir();
        let file_path = root.join("src").join("Program.cs");
        fs::create_dir_all(file_path.parent().expect("parent")).expect("dir");
        fs::write(&file_path, "class Program {}").expect("file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(csharp_language_server()));
        let session =
            must(registry.prepare_session_for_path("csharp-ls", &file_path, Some(root.as_path())));
        assert_eq!(session.root(), Some(&root));
        assert_eq!(session.launch().cwd(), Some(&root));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_sessions_for_extension_returns_all_matching_servers() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(LanguageServerSpec::new(
            "harper",
            "markdown",
            ["md"],
            "harper-ls",
            ["--stdio"],
        )));
        must(registry.register(LanguageServerSpec::new(
            "marksman",
            "markdown",
            ["md"],
            "marksman",
            ["server"],
        )));

        let sessions =
            must(registry.prepare_sessions_for_extension("md", Some(PathBuf::from("P:\\volt"))));
        assert_eq!(sessions.len(), 2);
        assert_eq!(sessions[0].server_id(), "harper");
        assert_eq!(sessions[1].server_id(), "marksman");
    }

    #[test]
    fn prepare_sessions_for_path_returns_filename_matches_without_extensions() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(dockerfile_language_server()));

        let sessions = must(
            registry.prepare_sessions_for_path(Path::new("Dockerfile"), Some(Path::new("P:\\"))),
        );
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].server_id(), "dockerfile-language-server");
    }

    #[test]
    fn prepare_sessions_for_path_skips_servers_disabled_by_default() {
        let mut registry = LanguageServerRegistry::new();
        must(registry.register(rust_analyzer()));
        must(
            registry.register(
                LanguageServerSpec::new(
                    "copilot-language-server",
                    "plaintext",
                    ["rs"],
                    "copilot",
                    std::iter::empty::<&str>(),
                )
                .with_enabled_by_default(false),
            ),
        );

        let matching = registry.servers_for_path(Path::new("lib.rs"));
        assert_eq!(matching.len(), 2);
        assert_eq!(matching[0].id(), "rust-analyzer");
        assert_eq!(matching[1].id(), "copilot-language-server");

        let sessions =
            must(registry.prepare_sessions_for_path(Path::new("lib.rs"), Some(Path::new("P:\\"))));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].server_id(), "rust-analyzer");
    }

    #[test]
    fn prepare_sessions_for_path_requires_activation_markers_when_declared() {
        let root = temp_dir();
        let src_dir = root.join("src");
        let file_path = src_dir.join("app.tsx");
        fs::create_dir_all(&src_dir).expect("src dir");
        fs::write(&file_path, "export default function App() {}").expect("tsx file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(tailwind_language_server()));

        let error = registry
            .prepare_sessions_for_path(&file_path, Some(root.as_path()))
            .expect_err("missing activation markers should block startup");
        assert_eq!(
            error,
            LspError::UnknownExtension(file_path.display().to_string())
        );

        fs::write(root.join("tailwind.config.ts"), "export default {}").expect("config");
        let sessions = must(registry.prepare_sessions_for_path(&file_path, Some(root.as_path())));
        assert_eq!(sessions.len(), 1);
        assert_eq!(sessions[0].server_id(), "tailwindcss-language-server");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn prepare_session_for_path_reports_missing_activation_markers_for_explicit_server() {
        let root = temp_dir();
        let src_dir = root.join("src");
        let file_path = src_dir.join("app.tsx");
        fs::create_dir_all(&src_dir).expect("src dir");
        fs::write(&file_path, "export default function App() {}").expect("tsx file");

        let mut registry = LanguageServerRegistry::new();
        must(registry.register(tailwind_language_server()));

        let error = registry
            .prepare_session_for_path(
                "tailwindcss-language-server",
                &file_path,
                Some(root.as_path()),
            )
            .expect_err("missing activation markers should block explicit startup");
        assert_eq!(
            error,
            LspError::ActivationMarkersNotFound {
                server_id: "tailwindcss-language-server".to_owned(),
                path: file_path.display().to_string(),
            }
        );

        let _ = fs::remove_dir_all(root);
    }
