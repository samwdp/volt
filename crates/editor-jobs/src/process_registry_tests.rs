use std::{
    fs,
    path::PathBuf,
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{
    ProcessLaunchSpec, ProcessRegistry, ShareKey, WorkspaceId, owned_process_pid_alive,
    process_registry::{discover_volt_supervisor_exe, synthetic_sleep_command},
};

fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("unexpected error: {error:?}"),
    }
}

fn wait_until(timeout: Duration, mut predicate: impl FnMut() -> bool) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        if predicate() {
            return true;
        }
        thread::sleep(Duration::from_millis(25));
    }
    predicate()
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}-{nanos}"))
}

fn with_supervisor(mut spec: ProcessLaunchSpec) -> ProcessLaunchSpec {
    if let Some(supervisor) = discover_volt_supervisor_exe() {
        spec = spec.with_process_supervisor_exe(supervisor);
    }
    spec
}

fn sleep_spec(workspace: WorkspaceId) -> ProcessLaunchSpec {
    let (program, args) = synthetic_sleep_command(60);
    with_supervisor(ProcessLaunchSpec::new(program, args).with_workspace(workspace))
}

fn descendant_tree_spec(workspace: WorkspaceId, child_pid_file: &std::path::Path) -> ProcessLaunchSpec {
    let pid_file = child_pid_file.display().to_string();
    #[cfg(windows)]
    let (program, args) = (
        "powershell".to_owned(),
        vec![
            "-NoProfile".to_owned(),
            "-Command".to_owned(),
            format!(
                "$ErrorActionPreference='Stop'; \
                 $p = Start-Process -FilePath ping -ArgumentList '-n','60','127.0.0.1' \
                    -WindowStyle Hidden -PassThru; \
                 Set-Content -LiteralPath '{pid_file}' -Value $p.Id -NoNewline; \
                 Wait-Process -Id $p.Id"
            ),
        ],
    );
    #[cfg(unix)]
    let (program, args) = (
        "sh".to_owned(),
        vec![
            "-c".to_owned(),
            format!("sleep 60 & echo $! > '{pid_file}'; wait"),
        ],
    );
    with_supervisor(ProcessLaunchSpec::new(program, args).with_workspace(workspace))
}

fn read_child_pid(path: &std::path::Path) -> u32 {
    assert!(
        wait_until(Duration::from_secs(5), || path.is_file()),
        "descendant should publish its pid file"
    );
    let contents = fs::read_to_string(path).expect("read child pid file");
    contents
        .trim()
        .parse::<u32>()
        .expect("child pid file should contain a pid")
}

fn process_image_name(pid: u32) -> String {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        use std::process::{Command, Stdio};
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let output = Command::new("tasklist")
            .args(["/FI", &format!("PID eq {pid}"), "/FO", "CSV", "/NH"])
            .stdin(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .output()
            .expect("tasklist");
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout
            .split(',')
            .next()
            .unwrap_or("")
            .trim_matches('"')
            .to_ascii_lowercase()
    }
    #[cfg(unix)]
    {
        let _ = pid;
        String::new()
    }
}

#[test]
fn process_launch_registers_alive_owned_process_tree() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(1);
    let id = must(registry.launch(sleep_spec(workspace)));

    assert!(registry.is_alive(id), "launched owned process should be alive");
    let pid = registry
        .root_pid(id)
        .expect("registered owned process should expose a root pid");
    assert!(owned_process_pid_alive(pid));

    must(registry.application_quit(Duration::from_millis(200)));
    assert!(!registry.is_alive(id));
    assert!(
        wait_until(Duration::from_secs(2), || !owned_process_pid_alive(pid)),
        "root pid {pid} should be gone after quit"
    );
}

#[test]
fn process_launch_wraps_with_supervisor_when_volt_is_available() {
    let Some(supervisor) = discover_volt_supervisor_exe() else {
        // ADR: supervisor wrapping applies when a Volt supervisor exe is available.
        return;
    };
    let mut registry = ProcessRegistry::new();
    let id = must(registry.launch(
        ProcessLaunchSpec::new(
            synthetic_sleep_command(60).0,
            synthetic_sleep_command(60).1,
        )
        .with_workspace(WorkspaceId::from_raw(2))
        .with_process_supervisor_exe(supervisor),
    ));
    let pid = registry.root_pid(id).expect("root pid");
    #[cfg(windows)]
    {
        let image = process_image_name(pid);
        assert!(
            image.contains("volt"),
            "hybrid Launch root should be the process supervisor (got `{image}`)"
        );
    }
    must(registry.application_quit(Duration::from_millis(200)));
}

#[test]
fn workspace_close_kills_descendant_tree_not_just_root() {
    let child_pid_file = unique_temp_path("volt-owned-child-pid");
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(3);
    let id = must(registry.launch(descendant_tree_spec(workspace, &child_pid_file)));
    let root_pid = registry.root_pid(id).expect("root pid");
    let child_pid = read_child_pid(&child_pid_file);
    assert_ne!(root_pid, child_pid, "synthetic tree must include a distinct descendant");
    assert!(owned_process_pid_alive(child_pid), "descendant should be alive before close");

    must(registry.workspace_close(workspace, Duration::from_millis(200)));

    assert!(!registry.is_alive(id));
    assert!(
        wait_until(Duration::from_secs(3), || {
            !owned_process_pid_alive(root_pid) && !owned_process_pid_alive(child_pid)
        }),
        "workspace close must reap root {root_pid} and descendant {child_pid}"
    );
    let _ = fs::remove_file(child_pid_file);
}

#[test]
fn workspace_close_kills_processes_tagged_for_that_workspace_only() {
    let mut registry = ProcessRegistry::new();
    let keep = WorkspaceId::from_raw(10);
    let close = WorkspaceId::from_raw(11);

    let keep_id = must(registry.launch(sleep_spec(keep)));
    let close_id = must(registry.launch(sleep_spec(close)));
    let close_pid = registry.root_pid(close_id).expect("close pid");

    must(registry.workspace_close(close, Duration::from_millis(200)));

    assert!(
        registry.is_alive(keep_id),
        "other workspace process must survive"
    );
    assert!(!registry.is_alive(close_id), "closed workspace process must die");
    assert!(
        wait_until(Duration::from_secs(2), || !owned_process_pid_alive(close_pid)),
        "closed workspace root must not remain as an orphan"
    );

    must(registry.application_quit(Duration::from_millis(200)));
}

#[test]
fn share_key_keeps_process_until_last_workspace_tag_drops() {
    let mut registry = ProcessRegistry::new();
    let first = WorkspaceId::from_raw(21);
    let second = WorkspaceId::from_raw(22);
    let share = ShareKey::new("lsp:demo-root");

    let (program, args) = synthetic_sleep_command(60);
    let first_spec = with_supervisor(
        ProcessLaunchSpec::new(program.clone(), args.clone())
            .with_workspace(first)
            .with_share_key(share.clone()),
    );
    let second_spec = with_supervisor(
        ProcessLaunchSpec::new(program, args)
            .with_workspace(second)
            .with_share_key(share),
    );

    let first_id = must(registry.launch(first_spec));
    let second_id = must(registry.launch(second_spec));
    assert_eq!(first_id, second_id, "share key should reuse the owned process");

    let pid = registry.root_pid(first_id).expect("shared pid");
    must(registry.workspace_close(first, Duration::from_millis(200)));
    assert!(
        registry.is_alive(first_id),
        "shared process must stay alive while another workspace tag remains"
    );
    assert!(owned_process_pid_alive(pid));

    must(registry.workspace_close(second, Duration::from_millis(200)));
    assert!(!registry.is_alive(first_id));
    assert!(
        wait_until(Duration::from_secs(2), || !owned_process_pid_alive(pid)),
        "shared process must die when the last workspace tag drops"
    );
}

#[test]
fn application_quit_clears_entire_registry() {
    let mut registry = ProcessRegistry::new();
    let a = must(registry.launch(sleep_spec(WorkspaceId::from_raw(31))));
    let b = must(registry.launch(sleep_spec(WorkspaceId::from_raw(32))));
    let pid_a = registry.root_pid(a).expect("pid a");
    let pid_b = registry.root_pid(b).expect("pid b");

    must(registry.application_quit(Duration::from_millis(200)));

    assert!(!registry.is_alive(a));
    assert!(!registry.is_alive(b));
    assert!(wait_until(Duration::from_secs(2), || {
        !owned_process_pid_alive(pid_a) && !owned_process_pid_alive(pid_b)
    }));
}

#[test]
fn graceful_then_force_deadline_always_reaps_tree() {
    let mut registry = ProcessRegistry::new();
    let id = must(registry.launch(sleep_spec(WorkspaceId::from_raw(41))));
    let pid = registry.root_pid(id).expect("pid");

    // Zero grace forces the hard-kill path immediately after the soft signal.
    must(registry.workspace_close(WorkspaceId::from_raw(41), Duration::ZERO));
    assert!(!registry.is_alive(id));
    assert!(
        wait_until(Duration::from_secs(2), || !owned_process_pid_alive(pid)),
        "force path must leave no survivors after the deadline"
    );
}

#[test]
fn owner_crash_leaves_no_orphans_for_launch_created_tree() {
    let child_pid_file = unique_temp_path("volt-owned-crash-child-pid");
    let mut registry = ProcessRegistry::new();
    let id = must(registry.launch(descendant_tree_spec(
        WorkspaceId::from_raw(51),
        &child_pid_file,
    )));
    let root_pid = registry.root_pid(id).expect("pid");
    let child_pid = read_child_pid(&child_pid_file);
    assert!(owned_process_pid_alive(root_pid));
    assert!(owned_process_pid_alive(child_pid));

    must(registry.simulate_owner_crash(id));
    assert!(!registry.is_alive(id));
    assert!(
        wait_until(Duration::from_secs(3), || {
            !owned_process_pid_alive(root_pid) && !owned_process_pid_alive(child_pid)
        }),
        "owner-crash kill-tree close must leave no orphan root {root_pid} or descendant {child_pid}"
    );
    let _ = fs::remove_file(child_pid_file);
}
