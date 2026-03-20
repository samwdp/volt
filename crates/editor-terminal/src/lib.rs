#![doc = r#"Terminal transcript sessions and editor-facing command execution surfaces."#]

use editor_jobs::{JobError, JobManager, JobResult, JobSpec};

/// Human-readable summary of this crate's responsibility.
pub const ROLE: &str = "Terminal transcript sessions and editor-facing command execution surfaces.";

/// Returns the responsibility summary for this crate.
pub const fn role() -> &'static str {
    ROLE
}

/// Distinguishes stdout and stderr transcript lines.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalStream {
    /// Normal stdout output.
    Stdout,
    /// Error stream output.
    Stderr,
}

/// One line in a terminal transcript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalLine {
    stream: TerminalStream,
    text: String,
}

impl TerminalLine {
    fn new(stream: TerminalStream, text: impl Into<String>) -> Self {
        Self {
            stream,
            text: text.into(),
        }
    }

    /// Returns the originating stream.
    pub const fn stream(&self) -> TerminalStream {
        self.stream
    }

    /// Returns the line text without a trailing newline.
    pub fn text(&self) -> &str {
        &self.text
    }
}

/// Collected transcript for a terminal session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalTranscript {
    lines: Vec<TerminalLine>,
    exit_code: Option<i32>,
}

impl TerminalTranscript {
    /// Returns all transcript lines.
    pub fn lines(&self) -> &[TerminalLine] {
        &self.lines
    }

    /// Returns the number of transcript lines.
    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    /// Returns the exit code, if any.
    pub const fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Reports whether the session exited successfully.
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Materialized terminal session suitable for editor buffers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminalSession {
    title: String,
    command_label: String,
    transcript: TerminalTranscript,
}

impl TerminalSession {
    /// Runs a terminal command to completion and captures its transcript.
    pub fn run(
        jobs: &mut JobManager,
        title: impl Into<String>,
        spec: JobSpec,
    ) -> Result<Self, JobError> {
        let handle = jobs.spawn(spec.with_kind(editor_jobs::JobKind::Terminal))?;
        let result = handle.wait()?;
        Ok(Self::from_job_result(title, result))
    }

    /// Returns the terminal title.
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Returns the executed command label.
    pub fn command_label(&self) -> &str {
        &self.command_label
    }

    /// Returns the collected transcript.
    pub fn transcript(&self) -> &TerminalTranscript {
        &self.transcript
    }

    fn from_job_result(title: impl Into<String>, result: JobResult) -> Self {
        let mut lines = Vec::new();
        append_lines(&mut lines, TerminalStream::Stdout, result.stdout());
        append_lines(&mut lines, TerminalStream::Stderr, result.stderr());

        Self {
            title: title.into(),
            command_label: result.spec().label().to_owned(),
            transcript: TerminalTranscript {
                lines,
                exit_code: result.exit_code(),
            },
        }
    }
}

fn append_lines(lines: &mut Vec<TerminalLine>, stream: TerminalStream, text: &str) {
    for line in text.lines() {
        lines.push(TerminalLine::new(stream, line));
    }
}

#[cfg(test)]
mod tests {
    use editor_jobs::{JobManager, JobSpec};

    use super::{TerminalSession, TerminalStream};

    fn must<T, E: std::fmt::Debug>(result: Result<T, E>) -> T {
        match result {
            Ok(value) => value,
            Err(error) => panic!("unexpected error: {error:?}"),
        }
    }

    #[test]
    fn terminal_session_captures_transcript_lines() {
        let mut jobs = JobManager::new();
        let session = must(TerminalSession::run(
            &mut jobs,
            "Terminal",
            JobSpec::terminal("cargo-version", "cargo", ["--version"]),
        ));

        assert_eq!(session.title(), "Terminal");
        assert_eq!(session.command_label(), "cargo-version");
        assert!(session.transcript().succeeded());
        assert!(session.transcript().line_count() >= 1);
        assert_eq!(
            session.transcript().lines()[0].stream(),
            TerminalStream::Stdout
        );
        assert!(session.transcript().lines()[0].text().contains("cargo"));
    }
}
