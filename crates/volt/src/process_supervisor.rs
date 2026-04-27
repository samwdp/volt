use std::{
    process::{Child, Command, ExitStatus, Stdio},
    thread,
    time::{Duration, Instant},
};

use editor_jobs::{PROCESS_SUPERVISOR_FLAG, ProcessSupervisionMode};
use sysinfo::{Pid, System};

#[cfg(unix)]
use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
#[cfg(windows)]
use std::os::windows::process::CommandExt as _;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

const PARENT_POLL_INTERVAL: Duration = Duration::from_millis(250);
const CHILD_TERM_TIMEOUT: Duration = Duration::from_millis(750);

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProcessSupervisorRequest {
    parent_pid: u32,
    mode: ProcessSupervisionMode,
    program: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParentProcess {
    pid: Pid,
    start_time: u64,
}

pub(crate) fn maybe_run(args: &[String]) -> Result<bool, String> {
    let Some(request) = parse_request(args)? else {
        return Ok(false);
    };

    run_request(request)?;
    Ok(true)
}

fn parse_request(args: &[String]) -> Result<Option<ProcessSupervisorRequest>, String> {
    if args.first().map(String::as_str) != Some(PROCESS_SUPERVISOR_FLAG) {
        return Ok(None);
    }

    if args.len() < 4 {
        return Err(
            "process supervisor requires `--process-supervisor <parent-pid> -- <program>`"
                .to_owned(),
        );
    }

    let parent_pid = args[1]
        .parse::<u32>()
        .map_err(|error| format!("invalid supervisor parent pid `{}`: {error}", args[1]))?;
    let mut mode = ProcessSupervisionMode::Interactive;
    let mut index = 2;
    while let Some(argument) = args.get(index) {
        match argument.as_str() {
            "--background" => {
                mode = ProcessSupervisionMode::Background;
                index += 1;
            }
            "--" => {
                index += 1;
                break;
            }
            other => {
                return Err(format!("unknown process supervisor option `{other}`"));
            }
        }
    }

    let Some(program) = args.get(index) else {
        return Err("process supervisor target program is missing".to_owned());
    };

    Ok(Some(ProcessSupervisorRequest {
        parent_pid,
        mode,
        program: program.clone(),
        args: args[index + 1..].to_vec(),
    }))
}

fn run_request(request: ProcessSupervisorRequest) -> Result<(), String> {
    let Some(parent) = ParentProcess::capture(request.parent_pid) else {
        return Err(format!(
            "parent process `{}` is not running anymore",
            request.parent_pid
        ));
    };

    let mut child = spawn_supervised_child(&request)?;
    let status = supervise_child(parent, request.mode, &mut child)?;
    exit_with_status(status)
}

fn spawn_supervised_child(request: &ProcessSupervisorRequest) -> Result<Child, String> {
    let mut command = Command::new(&request.program);
    command
        .args(&request.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());
    configure_supervised_command(&mut command, request.mode);
    command.spawn().map_err(|error| {
        format!(
            "failed to start supervised child `{}`: {error}",
            request.program
        )
    })
}

fn configure_supervised_command(command: &mut Command, mode: ProcessSupervisionMode) {
    #[cfg(unix)]
    {
        if matches!(mode, ProcessSupervisionMode::Background) {
            command.process_group(0);
        }
    }
    #[cfg(windows)]
    if matches!(mode, ProcessSupervisionMode::Background) {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

fn supervise_child(
    parent: ParentProcess,
    mode: ProcessSupervisionMode,
    child: &mut Child,
) -> Result<ExitStatus, String> {
    loop {
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("failed to poll supervised child: {error}"))?
        {
            return Ok(status);
        }

        if !parent.is_alive() {
            terminate_supervised_child(child, mode)?;
            return child
                .wait()
                .map_err(|error| format!("failed to wait for supervised child: {error}"));
        }

        thread::sleep(PARENT_POLL_INTERVAL);
    }
}

fn terminate_supervised_child(
    child: &mut Child,
    #[cfg_attr(windows, allow(unused_variables))] mode: ProcessSupervisionMode,
) -> Result<(), String> {
    #[cfg(unix)]
    {
        terminate_supervised_child_unix(child, mode)?;
    }
    #[cfg(windows)]
    {
        terminate_supervised_child_windows(child)?;
    }
    Ok(())
}

#[cfg(unix)]
fn terminate_supervised_child_unix(
    child: &mut Child,
    mode: ProcessSupervisionMode,
) -> Result<(), String> {
    use rustix::process::{Pid, Signal, kill_process_group};

    if !matches!(mode, ProcessSupervisionMode::Background) {
        child
            .kill()
            .map_err(|error| format!("failed to kill supervised child: {error}"))?;
        return Ok(());
    }

    let Some(pid) = Pid::from_raw(child.id() as i32) else {
        child
            .kill()
            .map_err(|error| format!("failed to kill supervised child: {error}"))?;
        return Ok(());
    };

    let _ = kill_process_group(pid, Signal::TERM);
    if wait_for_child_exit(child, CHILD_TERM_TIMEOUT)? {
        return Ok(());
    }

    let _ = kill_process_group(pid, Signal::KILL);
    if wait_for_child_exit(child, CHILD_TERM_TIMEOUT)? {
        return Ok(());
    }

    child
        .kill()
        .map_err(|error| format!("failed to kill supervised child: {error}"))
}

#[cfg(windows)]
fn terminate_supervised_child_windows(child: &mut Child) -> Result<(), String> {
    let status = Command::new("taskkill")
        .args(["/T", "/F", "/PID", &child.id().to_string()])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|error| format!("failed to run taskkill for supervised child: {error}"))?;
    if status.success() || wait_for_child_exit(child, CHILD_TERM_TIMEOUT)? {
        return Ok(());
    }

    child
        .kill()
        .map_err(|error| format!("failed to kill supervised child: {error}"))
}

fn wait_for_child_exit(child: &mut Child, timeout: Duration) -> Result<bool, String> {
    let deadline = Instant::now() + timeout;
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to poll supervised child: {error}"))?
            .is_some()
        {
            return Ok(true);
        }
        if Instant::now() >= deadline {
            return Ok(false);
        }
        thread::sleep(PARENT_POLL_INTERVAL.min(Duration::from_millis(50)));
    }
}

fn exit_with_status(status: ExitStatus) -> ! {
    #[cfg(unix)]
    if let Some(signal) = status.signal() {
        std::process::exit(128 + signal);
    }

    std::process::exit(status.code().unwrap_or(1));
}

impl ParentProcess {
    fn capture(pid: u32) -> Option<Self> {
        let pid = Pid::from_u32(pid);
        let system = System::new_all();
        let process = system.process(pid)?;
        Some(Self {
            pid,
            start_time: process.start_time(),
        })
    }

    fn is_alive(self) -> bool {
        let system = System::new_all();
        system
            .process(self.pid)
            .map(|process| process.start_time() == self.start_time)
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use editor_jobs::ProcessSupervisionMode;

    use super::{ProcessSupervisorRequest, parse_request};

    #[test]
    fn parse_request_ignores_normal_launch_args() {
        assert_eq!(
            parse_request(&["--shell-hidden".to_owned()]).expect("normal args should parse"),
            None
        );
    }

    #[test]
    fn parse_request_accepts_background_targets() {
        let request = parse_request(&[
            "--process-supervisor".to_owned(),
            "42".to_owned(),
            "--background".to_owned(),
            "--".to_owned(),
            "pwsh".to_owned(),
            "-NoProfile".to_owned(),
        ])
        .expect("supervisor args should parse")
        .expect("supervisor request should be detected");
        assert_eq!(
            request,
            ProcessSupervisorRequest {
                parent_pid: 42,
                mode: ProcessSupervisionMode::Background,
                program: "pwsh".to_owned(),
                args: vec!["-NoProfile".to_owned()],
            }
        );
    }
}
