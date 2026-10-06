//! Silent External Command helpers for Builtin Plugins.

use abi_stable::std_types::{ROption, RString, RVec};
use serde_json::Value;

use super::with_host;

/// Result of a Silent Command.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilentCommandResult {
    pub code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Runs `program` with `args` in `cwd` (or workspace root when `cwd` is None).
///
/// Uses Process Launch on the host. Blocks until the process exits.
pub fn run_silent(program: &str, args: &[&str], cwd: Option<&str>) -> Option<SilentCommandResult> {
    with_host(|host| {
        let args: RVec<RString> = args
            .iter()
            .map(|arg| RString::from(*arg))
            .collect::<Vec<_>>()
            .into();
        let cwd = match cwd {
            Some(path) => ROption::RSome(RString::from(path)),
            None => ROption::RNone,
        };
        let payload = (host.job_run_silent)(RString::from(program), args, cwd);
        parse_silent_result(payload.as_str())
    })
    .flatten()
}

fn parse_silent_result(payload: &str) -> Option<SilentCommandResult> {
    let value = serde_json::from_str::<Value>(payload).ok()?;
    let code = value.get("code")?.as_i64()? as i32;
    let stdout = value
        .get("stdout")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    let stderr = value
        .get("stderr")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_owned();
    Some(SilentCommandResult {
        code,
        stdout,
        stderr,
    })
}
