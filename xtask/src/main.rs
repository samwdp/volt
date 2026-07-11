use std::{
    env,
    path::Path,
    process::{Command, ExitCode, Stdio},
};

const HELP: &str = "Usage: cargo xtask <command>\n\nCommands:\n  fmt        Run cargo fmt --all\n  fmt-check  Run cargo fmt --all --check\n  check      Run cargo check --workspace\n  clippy     Run cargo clippy --workspace --all-targets -- -D warnings\n  test       Run cargo test --workspace\n  ci         Run fmt-check, check, clippy, and test\n";

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args().skip(1);
    let Some(command) = args.next() else {
        print!("{HELP}");
        return Ok(());
    };

    if args.next().is_some() {
        return Err(format!("unexpected extra arguments after `{command}`"));
    }

    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| "failed to resolve workspace root".to_string())?;

    match command.as_str() {
        "fmt" => cargo(workspace_root, ["fmt", "--all"]),
        "fmt-check" => cargo(workspace_root, ["fmt", "--all", "--check"]),
        "check" => cargo(workspace_root, ["check", "--workspace"]),
        "clippy" => cargo(
            workspace_root,
            [
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        ),
        "test" => cargo(workspace_root, ["test", "--workspace"]),
        "ci" => {
            cargo(workspace_root, ["fmt", "--all", "--check"])?;
            cargo(workspace_root, ["check", "--workspace"])?;
            cargo(
                workspace_root,
                [
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            cargo(workspace_root, ["test", "--workspace"])
        }
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            Ok(())
        }
        other => Err(format!("unknown xtask command `{other}`\n\n{HELP}")),
    }
}

fn cargo<I, S>(workspace_root: &Path, args: I) -> Result<(), String>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let args = args
        .into_iter()
        .map(|arg| arg.as_ref().to_owned())
        .collect::<Vec<_>>();

    println!("> cargo {}", args.join(" "));

    let status = Command::new("cargo")
        .args(&args)
        .current_dir(workspace_root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("failed to spawn cargo {}: {error}", args.join(" ")))?;

    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "cargo {} exited with status {status}",
            args.join(" ")
        ))
    }
}
