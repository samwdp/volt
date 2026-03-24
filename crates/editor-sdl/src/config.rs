use std::{error::Error, fmt};

use editor_render::{RenderBackend, RenderError};

/// Configures the demo shell loop.
#[derive(Debug, Clone)]
pub struct ShellConfig {
    /// Window title.
    pub title: String,
    /// Window width in pixels.
    pub width: u32,
    /// Window height in pixels.
    pub height: u32,
    /// Monospace font size in pixels.
    pub font_size: u32,
    /// Whether the window should start hidden.
    pub hidden: bool,
    /// Renderer backend requested by the shell configuration.
    pub render_backend: RenderBackend,
    /// Optional frame limit used for smoke tests.
    pub frame_limit: Option<u32>,
    /// Enables detailed typing/input latency profiling and writes a report on exit.
    pub profile_input_latency: bool,
}

impl Default for ShellConfig {
    fn default() -> Self {
        Self {
            title: "volt shell demo".to_owned(),
            width: 1200,
            height: 760,
            font_size: 18,
            hidden: false,
            render_backend: RenderBackend::SdlCanvas,
            frame_limit: None,
            profile_input_latency: false,
        }
    }
}

/// Summary written when typing profiling is enabled.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingProfileSummary {
    /// Output path of the written typing profile log.
    pub log_path: String,
    /// Number of captured frames retained in the report.
    pub frames_captured: usize,
    /// Number of retained frames that included text input.
    pub input_frames_captured: usize,
    /// Slowest retained frame duration in microseconds.
    pub slowest_frame_micros: u128,
}

/// Summary returned after the demo shell exits.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShellSummary {
    /// Number of frames presented.
    pub frames_rendered: u32,
    /// Number of visible panes.
    pub pane_count: usize,
    /// Whether the picker popup was visible when the loop exited.
    pub popup_visible: bool,
    /// Renderer backend used for the shell session.
    pub render_backend: RenderBackend,
    /// Concrete SDL renderer chosen for the shell session.
    pub renderer_name: String,
    /// Font path selected by the text renderer.
    pub font_path: String,
    /// Typing profile report metadata when input profiling was enabled.
    pub typing_profile: Option<TypingProfileSummary>,
}

/// Errors raised while creating or running the SDL demo shell.
#[derive(Debug)]
pub enum ShellError {
    /// SDL initialization or rendering failed.
    Sdl(String),
    /// Font lookup failed before SDL_ttf could load the font.
    Render(RenderError),
    /// Runtime or shell orchestration failed.
    Runtime(String),
}

impl fmt::Display for ShellError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sdl(error) => write!(formatter, "SDL error: {error}"),
            Self::Render(error) => error.fmt(formatter),
            Self::Runtime(error) => write!(formatter, "runtime error: {error}"),
        }
    }
}

impl Error for ShellError {}

impl From<RenderError> for ShellError {
    fn from(error: RenderError) -> Self {
        Self::Render(error)
    }
}
