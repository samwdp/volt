use std::path::PathBuf;

use super::{
    DebugAdapterRegistry, DebugAdapterSpec, DebugAdapterTransport, DebugConfiguration,
    DebugRequestKind,
};

fn codelldb() -> DebugAdapterSpec {
    DebugAdapterSpec::new("codelldb", "rust", ["rs"], "codelldb", ["--port", "13000"])
        .with_transport(DebugAdapterTransport::Tcp {
            host: "127.0.0.1".to_owned(),
            port: 13000,
        })
        .with_preference(100)
        .with_root_markers(["Cargo.toml"])
}

fn gdb() -> DebugAdapterSpec {
    DebugAdapterSpec::new("gdb", "rust", ["rs"], "gdb", ["-i=dap"]).with_preference(50)
}

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

#[test]
fn registry_resolves_adapter_by_extension() {
    let mut registry = DebugAdapterRegistry::new();
    must(registry.register(codelldb()));

    let adapter = registry.adapter_for_extension("rs").expect("adapter");
    assert_eq!(adapter.id(), "codelldb");
    assert_eq!(adapter.program(), "codelldb");
    assert_eq!(adapter.preference(), 100);
}

#[test]
fn registry_prefers_higher_preference_when_multiple_match() {
    let mut registry = DebugAdapterRegistry::new();
    must(registry.register(gdb()));
    must(registry.register(codelldb()));

    let adapters = registry.adapters_for_extension("rs");
    assert_eq!(
        adapters
            .iter()
            .map(|adapter| adapter.id())
            .collect::<Vec<_>>(),
        ["codelldb", "gdb"]
    );
    assert_eq!(
        registry
            .resolve_adapter_for_extension("rs")
            .expect("preferred")
            .id(),
        "codelldb"
    );
}

#[test]
fn prepared_session_includes_configuration_and_launch_spec() {
    let mut registry = DebugAdapterRegistry::new();
    must(registry.register(codelldb()));

    let plan = must(
        registry.prepare_session(
            "codelldb",
            DebugConfiguration::new("Debug volt", DebugRequestKind::Launch)
                .with_target_program(PathBuf::from("target\\debug\\volt.exe"))
                .with_cwd(PathBuf::from("P:\\volt"))
                .with_args(["--shell-hidden"]),
        ),
    );

    assert_eq!(plan.adapter_id(), "codelldb");
    assert_eq!(plan.language_id(), "rust");
    assert_eq!(plan.adapter_launch().program(), "codelldb");
    assert_eq!(plan.configuration().name(), "Debug volt");
    assert_eq!(plan.configuration().args(), ["--shell-hidden"]);
    assert!(matches!(
        plan.transport(),
        DebugAdapterTransport::Tcp { port: 13000, .. }
    ));
}
