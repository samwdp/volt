use std::{
    fs,
    io::Read,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use super::{
    JobManager, JobSpec, OwnedProcessId, ProcessLaunchSpec, ProcessLaunchStdio, ProcessRegistry,
    ProcessSupervisionMode, ShareKey, WorkspaceId, language_server_share_key,
    owned_process_pid_alive,
    process_registry::{discover_volt_supervisor_exe, synthetic_sleep_command},
};
use std::path::Path;

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

fn launch_id(registry: &mut ProcessRegistry, spec: ProcessLaunchSpec) -> OwnedProcessId {
    must(registry.launch(spec)).id
}

fn descendant_tree_spec(
    workspace: WorkspaceId,
    child_pid_file: &std::path::Path,
) -> ProcessLaunchSpec {
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
    let deadline = Instant::now() + Duration::from_secs(2);
    let contents = loop {
        match fs::read_to_string(path) {
            Ok(contents) if !contents.trim().is_empty() => break contents,
            _ if Instant::now() >= deadline => {
                panic!("child pid file should become readable: {}", path.display())
            }
            _ => thread::sleep(Duration::from_millis(25)),
        }
    };
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

fn spawn_unmanaged_sleep() -> u32 {
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt as _;
        use std::process::{Command, Stdio};
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        Command::new("ping")
            .args(["-n", "60", "127.0.0.1"])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .expect("spawn unmanaged sleep")
            .id()
    }
    #[cfg(unix)]
    {
        use std::process::{Command, Stdio};
        Command::new("sleep")
            .arg("60")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn unmanaged sleep")
            .id()
    }
}

#[test]
fn process_launch_registers_alive_owned_process_tree() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(1);
    let id = launch_id(&mut registry, sleep_spec(workspace));

    assert!(
        registry.is_alive(id),
        "launched owned process should be alive"
    );
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
    let id = launch_id(
        &mut registry,
        ProcessLaunchSpec::new(synthetic_sleep_command(60).0, synthetic_sleep_command(60).1)
            .with_workspace(WorkspaceId::from_raw(2))
            .with_process_supervisor_exe(supervisor),
    );
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
fn interactive_launch_skips_supervisor_wrap() {
    let Some(supervisor) = discover_volt_supervisor_exe() else {
        return;
    };
    let mut registry = ProcessRegistry::new();
    let (program, args) = synthetic_sleep_command(60);
    let id = launch_id(
        &mut registry,
        ProcessLaunchSpec::new(program, args)
            .with_workspace(WorkspaceId::from_raw(61))
            .with_mode(ProcessSupervisionMode::Interactive)
            .with_process_supervisor_exe(supervisor),
    );
    let pid = registry.root_pid(id).expect("root pid");
    #[cfg(windows)]
    {
        let image = process_image_name(pid);
        assert!(
            !image.contains("volt"),
            "interactive Launch must not wrap ConPTY-incompatible shells with the supervisor (got `{image}`)"
        );
    }
    must(registry.application_quit(Duration::from_millis(200)));
}

#[test]
fn piped_launch_exposes_stdout_and_registers_with_workspace() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(62);
    #[cfg(windows)]
    let (program, args) = (
        "cmd".to_owned(),
        vec!["/C".to_owned(), "echo piped-launch-ok".to_owned()],
    );
    #[cfg(unix)]
    let (program, args) = ("printf".to_owned(), vec!["piped-launch-ok".to_owned()]);

    let mut launched = must(
        registry.launch(
            ProcessLaunchSpec::new(program, args)
                .with_workspace(workspace)
                .with_stdio(ProcessLaunchStdio::Piped),
        ),
    );
    let mut stdout = launched
        .stdout
        .take()
        .expect("piped launch should expose stdout");
    let mut output = String::new();
    stdout
        .read_to_string(&mut output)
        .expect("read piped stdout");
    assert!(
        output.contains("piped-launch-ok"),
        "unexpected stdout `{output}`"
    );
    assert!(
        wait_until(Duration::from_secs(2), || !registry.is_alive(launched.id)),
        "short piped command should exit"
    );
    must(registry.reclaim(launched.id));
    assert!(!registry.is_alive(launched.id));
}

#[test]
fn launch_interactive_root_registers_workspace_and_close_kills_tree() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(63);
    let root_pid = spawn_unmanaged_sleep();
    assert!(owned_process_pid_alive(root_pid));

    let id = must(registry.launch_interactive_root(root_pid, [workspace]));
    assert!(registry.is_alive(id));
    assert_eq!(registry.root_pid(id), Some(root_pid));

    must(registry.workspace_close(workspace, Duration::from_millis(200)));
    assert!(!registry.is_alive(id));
    assert!(
        wait_until(Duration::from_secs(3), || !owned_process_pid_alive(
            root_pid
        )),
        "interactive root {root_pid} must die on Workspace Close"
    );
}

#[test]
fn launch_interactive_session_runs_spawn_inside_process_launch() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(64);
    let (id, reported_pid) = must(registry.launch_interactive_session([workspace], || {
        let pid = spawn_unmanaged_sleep();
        Ok::<_, String>((pid, pid))
    }));
    assert_eq!(registry.root_pid(id), Some(reported_pid));
    must(registry.application_quit(Duration::from_millis(200)));
    assert!(
        wait_until(Duration::from_secs(3), || !owned_process_pid_alive(
            reported_pid
        )),
        "interactive session root must die on Application Quit"
    );
}

#[test]
fn job_manager_launch_is_registry_owned_and_dies_on_workspace_close() {
    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let mut jobs = JobManager::with_registry(Arc::clone(&registry));
    let workspace = WorkspaceId::from_raw(65);
    let (program, args) = synthetic_sleep_command(60);
    let handle =
        must(jobs.spawn(JobSpec::command("owned-job", program, args).with_workspace(workspace)));
    assert!(
        wait_until(Duration::from_secs(2), || {
            registry
                .lock()
                .map(|mut registry| registry.alive_count() > 0)
                .unwrap_or(false)
        }),
        "JobManager spawn should register an Owned Process"
    );
    must(
        registry
            .lock()
            .expect("registry")
            .workspace_close(workspace, Duration::from_millis(200)),
    );
    drop(handle);
    assert!(
        wait_until(Duration::from_secs(2), || {
            registry
                .lock()
                .map(|mut registry| registry.alive_count() == 0)
                .unwrap_or(false)
        }),
        "Workspace Close must clear JobManager-launched Owned Processes"
    );
}

#[test]
fn workspace_close_kills_descendant_tree_not_just_root() {
    let child_pid_file = unique_temp_path("volt-owned-child-pid");
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(3);
    let id = launch_id(
        &mut registry,
        descendant_tree_spec(workspace, &child_pid_file),
    );
    let root_pid = registry.root_pid(id).expect("root pid");
    let child_pid = read_child_pid(&child_pid_file);
    assert_ne!(
        root_pid, child_pid,
        "synthetic tree must include a distinct descendant"
    );
    assert!(
        owned_process_pid_alive(child_pid),
        "descendant should be alive before close"
    );

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

    let keep_id = launch_id(&mut registry, sleep_spec(keep));
    let close_id = launch_id(&mut registry, sleep_spec(close));
    let close_pid = registry.root_pid(close_id).expect("close pid");

    must(registry.workspace_close(close, Duration::from_millis(200)));

    assert!(
        registry.is_alive(keep_id),
        "other workspace process must survive"
    );
    assert!(
        !registry.is_alive(close_id),
        "closed workspace process must die"
    );
    assert!(
        wait_until(Duration::from_secs(2), || !owned_process_pid_alive(
            close_pid
        )),
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

    let first_id = launch_id(&mut registry, first_spec);
    let second_id = launch_id(&mut registry, second_spec);
    assert_eq!(
        first_id, second_id,
        "share key should reuse the owned process"
    );

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
    let a = launch_id(&mut registry, sleep_spec(WorkspaceId::from_raw(31)));
    let b = launch_id(&mut registry, sleep_spec(WorkspaceId::from_raw(32)));
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
    let id = launch_id(&mut registry, sleep_spec(WorkspaceId::from_raw(41)));
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
    let id = launch_id(
        &mut registry,
        descendant_tree_spec(WorkspaceId::from_raw(51), &child_pid_file),
    );
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

#[test]
fn protocol_stdio_launch_exposes_stdin_and_stdout() {
    let mut registry = ProcessRegistry::new();
    let (program, args) = synthetic_sleep_command(60);
    let launched = must(
        registry.launch(with_supervisor(
            ProcessLaunchSpec::new(program, args)
                .with_workspace(WorkspaceId::from_raw(61))
                .with_stdio(ProcessLaunchStdio::Protocol),
        )),
    );
    assert!(
        launched.stdin.is_some(),
        "protocol launch must expose stdin"
    );
    assert!(
        launched.stdout.is_some(),
        "protocol launch must expose stdout"
    );
    assert!(launched.stderr.is_none(), "protocol launch discards stderr");
    assert!(registry.is_alive(launched.id));
    must(registry.application_quit(Duration::from_millis(200)));
}

#[test]
fn language_tooling_shared_share_key_keeps_then_kills_on_last_close() {
    let mut registry = ProcessRegistry::new();
    let first = WorkspaceId::from_raw(71);
    let second = WorkspaceId::from_raw(72);
    let share = language_server_share_key("rust-analyzer", Some(Path::new("/demo/root")));
    let (program, args) = synthetic_sleep_command(60);

    let first_id = launch_id(
        &mut registry,
        with_supervisor(
            ProcessLaunchSpec::new(program.clone(), args.clone())
                .with_workspace(first)
                .with_share_key(share.clone())
                .with_stdio(ProcessLaunchStdio::Protocol),
        ),
    );
    let second_id = launch_id(
        &mut registry,
        with_supervisor(
            ProcessLaunchSpec::new(program, args)
                .with_workspace(second)
                .with_share_key(share)
                .with_stdio(ProcessLaunchStdio::Protocol),
        ),
    );
    assert_eq!(first_id, second_id);
    let pid = registry.root_pid(first_id).expect("shared pid");

    must(registry.workspace_close(first, Duration::from_millis(200)));
    assert!(registry.is_alive(first_id));
    assert!(owned_process_pid_alive(pid));

    must(registry.workspace_close(second, Duration::from_millis(200)));
    assert!(!registry.is_alive(first_id));
    assert!(wait_until(Duration::from_secs(2), || {
        !owned_process_pid_alive(pid)
    }));
}

#[test]
fn language_tooling_single_tagged_helper_dies_on_quit() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(81);
    let (program, args) = synthetic_sleep_command(60);
    let id = launch_id(
        &mut registry,
        with_supervisor(
            ProcessLaunchSpec::new(program, args)
                .with_workspace(workspace)
                .with_stdio(ProcessLaunchStdio::Protocol),
        ),
    );
    let pid = registry.root_pid(id).expect("helper pid");
    must(registry.application_quit(Duration::from_millis(200)));
    assert!(!registry.is_alive(id));
    assert!(wait_until(Duration::from_secs(2), || {
        !owned_process_pid_alive(pid)
    }));
}

#[test]
fn run_captured_registers_then_reclaims_silent_command() {
    let mut registry = ProcessRegistry::new();
    let workspace = WorkspaceId::from_raw(91);
    #[cfg(windows)]
    let (program, args) = (
        "cmd".to_owned(),
        vec!["/C".to_owned(), "echo captured-ok".to_owned()],
    );
    #[cfg(unix)]
    let (program, args) = ("printf".to_owned(), vec!["captured-ok".to_owned()]);

    let output = must(
        registry.run_captured(
            ProcessLaunchSpec::new(program, args)
                .with_workspace(workspace)
                .with_stdio(ProcessLaunchStdio::Piped),
        ),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("captured-ok"),
        "unexpected stdout `{stdout}`"
    );
    assert_eq!(output.exit_code, Some(0));
    assert_eq!(registry.alive_count(), 0);
}

#[test]
fn release_shaped_quit_leaves_no_strays_and_frees_project_dir() {
    let project_dir = unique_temp_path("volt-release-quit-project");
    fs::create_dir_all(&project_dir).expect("create project dir");
    let lock_probe = project_dir.join("lock-probe.txt");
    fs::write(&lock_probe, b"probe").expect("write lock probe");

    let registry = Arc::new(Mutex::new(ProcessRegistry::new()));
    let mut jobs = JobManager::with_registry(Arc::clone(&registry));
    let workspace = WorkspaceId::from_raw(101);

    let (program, args) = synthetic_sleep_command(60);
    let job_handle = must(
        jobs.spawn(
            JobSpec::command("release-quit-job", program, args)
                .with_workspace(workspace)
                .with_cwd(project_dir.clone()),
        ),
    );

    let interactive_pid = spawn_unmanaged_sleep();
    let (helper_id, interactive_id, helper_pid) = {
        let mut registry = registry.lock().expect("registry");
        let helper_id = launch_id(
            &mut registry,
            with_supervisor(
                ProcessLaunchSpec::new(
                    synthetic_sleep_command(60).0,
                    synthetic_sleep_command(60).1,
                )
                .with_workspace(workspace)
                .with_share_key(ShareKey::new("helper:release-quit"))
                .with_stdio(ProcessLaunchStdio::Protocol)
                .with_current_dir(&project_dir),
            ),
        );
        let interactive_id = must(registry.launch_interactive_root(interactive_pid, [workspace]));
        let helper_pid = registry.root_pid(helper_id).expect("helper pid");
        (helper_id, interactive_id, helper_pid)
    };
    assert!(owned_process_pid_alive(helper_pid));
    assert!(owned_process_pid_alive(interactive_pid));

    {
        let mut registry = registry.lock().expect("registry");
        must(registry.application_quit(Duration::from_millis(200)));
        assert!(!registry.is_alive(helper_id));
        assert!(!registry.is_alive(interactive_id));
    }
    let _ = job_handle.wait();
    assert!(wait_until(Duration::from_secs(3), || {
        !owned_process_pid_alive(helper_pid) && !owned_process_pid_alive(interactive_pid)
    }));

    // Project directory must be free of Volt-started locks after quit.
    fs::remove_file(&lock_probe).expect("remove lock probe after quit");
    fs::remove_dir_all(&project_dir).expect("remove project dir after quit");
}
