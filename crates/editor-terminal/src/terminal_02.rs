fn push_terminal_render_run(
    runs: &mut Vec<TerminalRenderRun>,
    col: u16,
    width_cells: u16,
    text: String,
    foreground: Rgb,
    background: Option<Rgb>,
    underline: Option<Rgb>,
) {
    if let Some(last) = runs.last_mut()
        && last.col.saturating_add(last.width_cells) == col
        && last.foreground == foreground
        && last.background == background
        && last.underline == underline
    {
        last.width_cells = last.width_cells.saturating_add(width_cells);
        last.text.push_str(&text);
        return;
    }
    runs.push(TerminalRenderRun {
        col,
        width_cells,
        text,
        foreground,
        background,
        underline,
    });
}

fn resolve_terminal_foreground(colors: &TerminalColors, color: TerminalColor, flags: Flags) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => {
            if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                rgb * DIM_FACTOR
            } else {
                rgb
            }
        }
        TerminalColor::Named(named) => {
            let resolved = if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                named.to_dim()
            } else if flags.intersects(Flags::BOLD) {
                named.to_bright()
            } else {
                named
            };
            resolve_terminal_named_color(colors, resolved)
        }
        TerminalColor::Indexed(index) => {
            let resolved_index = if flags.intersects(Flags::DIM) && !flags.intersects(Flags::BOLD) {
                match index {
                    0..=7 => NamedColor::DimBlack as usize + index as usize,
                    8..=15 => index as usize - 8,
                    _ => index as usize,
                }
            } else if flags.intersects(Flags::BOLD) && index <= 7 {
                index as usize + 8
            } else {
                index as usize
            };
            resolve_terminal_index_color(colors, resolved_index)
        }
    }
}

fn resolve_terminal_background(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

fn resolve_terminal_plain_color(colors: &TerminalColors, color: TerminalColor) -> Rgb {
    match color {
        TerminalColor::Spec(rgb) => rgb,
        TerminalColor::Named(named) => resolve_terminal_named_color(colors, named),
        TerminalColor::Indexed(index) => resolve_terminal_index_color(colors, index as usize),
    }
}

fn resolve_terminal_named_color(colors: &TerminalColors, named: NamedColor) -> Rgb {
    colors[named].unwrap_or_else(|| default_terminal_named_color(named))
}

fn resolve_terminal_index_color(colors: &TerminalColors, index: usize) -> Rgb {
    colors[index].unwrap_or_else(|| default_terminal_index_color(index))
}

fn default_terminal_index_color(index: usize) -> Rgb {
    match index {
        0 => default_terminal_named_color(NamedColor::Black),
        1 => default_terminal_named_color(NamedColor::Red),
        2 => default_terminal_named_color(NamedColor::Green),
        3 => default_terminal_named_color(NamedColor::Yellow),
        4 => default_terminal_named_color(NamedColor::Blue),
        5 => default_terminal_named_color(NamedColor::Magenta),
        6 => default_terminal_named_color(NamedColor::Cyan),
        7 => default_terminal_named_color(NamedColor::White),
        8 => default_terminal_named_color(NamedColor::BrightBlack),
        9 => default_terminal_named_color(NamedColor::BrightRed),
        10 => default_terminal_named_color(NamedColor::BrightGreen),
        11 => default_terminal_named_color(NamedColor::BrightYellow),
        12 => default_terminal_named_color(NamedColor::BrightBlue),
        13 => default_terminal_named_color(NamedColor::BrightMagenta),
        14 => default_terminal_named_color(NamedColor::BrightCyan),
        15 => default_terminal_named_color(NamedColor::BrightWhite),
        16..=231 => {
            let index = index - 16;
            let blue = index % 6;
            let green = (index / 6) % 6;
            let red = index / 36;
            Rgb {
                r: cube_color_component(red),
                g: cube_color_component(green),
                b: cube_color_component(blue),
            }
        }
        232..=255 => {
            let value = ((index - 232) * 10 + 8) as u8;
            Rgb {
                r: value,
                g: value,
                b: value,
            }
        }
        256 => default_terminal_named_color(NamedColor::Foreground),
        257 => default_terminal_named_color(NamedColor::Background),
        258 => default_terminal_named_color(NamedColor::Cursor),
        259 => default_terminal_named_color(NamedColor::DimBlack),
        260 => default_terminal_named_color(NamedColor::DimRed),
        261 => default_terminal_named_color(NamedColor::DimGreen),
        262 => default_terminal_named_color(NamedColor::DimYellow),
        263 => default_terminal_named_color(NamedColor::DimBlue),
        264 => default_terminal_named_color(NamedColor::DimMagenta),
        265 => default_terminal_named_color(NamedColor::DimCyan),
        266 => default_terminal_named_color(NamedColor::DimWhite),
        267 => default_terminal_named_color(NamedColor::BrightForeground),
        268 => default_terminal_named_color(NamedColor::DimForeground),
        _ => default_terminal_named_color(NamedColor::Foreground),
    }
}

fn cube_color_component(index: usize) -> u8 {
    if index == 0 {
        0
    } else {
        (index as u8).saturating_mul(40).saturating_add(55)
    }
}

fn default_terminal_named_color(named: NamedColor) -> Rgb {
    match named {
        NamedColor::Black => Rgb {
            r: 12,
            g: 12,
            b: 12,
        },
        NamedColor::Red => Rgb {
            r: 205,
            g: 49,
            b: 49,
        },
        NamedColor::Green => Rgb {
            r: 13,
            g: 188,
            b: 121,
        },
        NamedColor::Yellow => Rgb {
            r: 229,
            g: 229,
            b: 16,
        },
        NamedColor::Blue => Rgb {
            r: 36,
            g: 114,
            b: 200,
        },
        NamedColor::Magenta => Rgb {
            r: 188,
            g: 63,
            b: 188,
        },
        NamedColor::Cyan => Rgb {
            r: 17,
            g: 168,
            b: 205,
        },
        NamedColor::White => Rgb {
            r: 229,
            g: 229,
            b: 229,
        },
        NamedColor::BrightBlack => Rgb {
            r: 102,
            g: 102,
            b: 102,
        },
        NamedColor::BrightRed => Rgb {
            r: 241,
            g: 76,
            b: 76,
        },
        NamedColor::BrightGreen => Rgb {
            r: 35,
            g: 209,
            b: 139,
        },
        NamedColor::BrightYellow => Rgb {
            r: 245,
            g: 245,
            b: 67,
        },
        NamedColor::BrightBlue => Rgb {
            r: 59,
            g: 142,
            b: 234,
        },
        NamedColor::BrightMagenta => Rgb {
            r: 214,
            g: 112,
            b: 214,
        },
        NamedColor::BrightCyan => Rgb {
            r: 41,
            g: 184,
            b: 219,
        },
        NamedColor::BrightWhite => Rgb {
            r: 255,
            g: 255,
            b: 255,
        },
        NamedColor::Foreground => Rgb {
            r: 215,
            g: 221,
            b: 232,
        },
        NamedColor::Background => Rgb {
            r: 15,
            g: 16,
            b: 20,
        },
        NamedColor::Cursor => Rgb {
            r: 110,
            g: 170,
            b: 255,
        },
        NamedColor::DimBlack => default_terminal_named_color(NamedColor::Black) * DIM_FACTOR,
        NamedColor::DimRed => default_terminal_named_color(NamedColor::Red) * DIM_FACTOR,
        NamedColor::DimGreen => default_terminal_named_color(NamedColor::Green) * DIM_FACTOR,
        NamedColor::DimYellow => default_terminal_named_color(NamedColor::Yellow) * DIM_FACTOR,
        NamedColor::DimBlue => default_terminal_named_color(NamedColor::Blue) * DIM_FACTOR,
        NamedColor::DimMagenta => default_terminal_named_color(NamedColor::Magenta) * DIM_FACTOR,
        NamedColor::DimCyan => default_terminal_named_color(NamedColor::Cyan) * DIM_FACTOR,
        NamedColor::DimWhite => default_terminal_named_color(NamedColor::White) * DIM_FACTOR,
        NamedColor::BrightForeground => default_terminal_named_color(NamedColor::Foreground),
        NamedColor::DimForeground => {
            default_terminal_named_color(NamedColor::Foreground) * DIM_FACTOR
        }
    }
}

fn push_snapshot_line(lines: &mut Vec<String>, line: &str) {
    lines.push(line.trim_end_matches(' ').to_owned());
}

fn terminal_tty_options(config: &LiveTerminalConfig) -> TtyOptions {
    let mut env = std::env::vars().collect::<HashMap<_, _>>();
    env.remove("SHLVL");
    env.insert("COLORTERM".to_owned(), "truecolor".to_owned());
    env.insert("TERM".to_owned(), "xterm-256color".to_owned());
    env.insert("TERM_PROGRAM".to_owned(), "volt".to_owned());
    env.insert(
        "TERM_PROGRAM_VERSION".to_owned(),
        env!("CARGO_PKG_VERSION").to_owned(),
    );

    TtyOptions {
        // PTY-backed terminals must spawn the real shell directly; wrapping them in the
        // supervisor binary on Windows breaks ConPTY embedding and opens a separate window.
        shell: Some(TtyShell::new(config.program.clone(), config.args.clone())),
        working_directory: config.cwd.clone(),
        drain_on_exit: true,
        env,
        #[cfg(windows)]
        escape_args: true,
    }
}

#[cfg(windows)]
fn tty_process_id(_pty: &tty::Pty) -> Option<u32> {
    None
}

#[cfg(not(windows))]
fn tty_process_id(pty: &tty::Pty) -> Option<u32> {
    Some(pty.child().id())
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::{
        index::{Column, Line, Point},
        term::test::mock_term,
    };
    use editor_jobs::{JobManager, JobSpec};

    use super::{
        LiveTerminalConfig, LiveTerminalSession, TerminalCursorShape, TerminalKey, TerminalSession,
        TerminalStream, terminal_key_bytes, terminal_render_snapshot,
    };

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

    #[test]
    fn terminal_key_sequences_match_common_terminal_controls() {
        assert_eq!(terminal_key_bytes(TerminalKey::Enter), b"\r");
        assert_eq!(terminal_key_bytes(TerminalKey::Backspace), b"\x7f");
        assert_eq!(terminal_key_bytes(TerminalKey::Up), b"\x1b[A");
        assert_eq!(terminal_key_bytes(TerminalKey::PageDown), b"\x1b[6~");
        assert_eq!(terminal_key_bytes(TerminalKey::CtrlC), b"\x03");
    }

    #[test]
    fn live_terminal_session_spawns_and_terminates() {
        let config = if cfg!(target_os = "windows") {
            LiveTerminalConfig::new("Terminal", "cmd", ["/Q".to_owned(), "/K".to_owned()])
        } else {
            LiveTerminalConfig::new("Terminal", "/bin/sh", Vec::<String>::new())
        }
        .with_size(12, 80);
        let mut session = must(LiveTerminalSession::spawn(config));
        if cfg!(not(target_os = "windows")) {
            assert!(session.process_id().is_some());
        }
        must(session.kill());
        assert!(session.has_exited());
    }

    #[test]
    fn terminal_render_snapshot_tracks_visible_cursor() {
        let mut term = mock_term("hello\nworld");
        term.grid_mut().cursor.point = Point::new(Line(1), Column(3));
        let snapshot = terminal_render_snapshot(&term, 2, 5, None);

        assert_eq!(snapshot.rows(), 2);
        assert_eq!(snapshot.cols(), 5);
        assert_eq!(snapshot.lines()[0].runs()[0].text(), "hello");
        assert_eq!(snapshot.lines()[1].runs()[0].text(), "world");
        let cursor = snapshot.cursor().expect("cursor should be visible");
        assert_eq!(cursor.row(), 1);
        assert_eq!(cursor.col(), 3);
        assert_eq!(cursor.width_cells(), 1);
        assert_eq!(cursor.shape(), TerminalCursorShape::Block);
        assert_eq!(cursor.text(), "l");
    }

    #[test]
    fn terminal_render_snapshot_preserves_wide_character_widths() {
        let term = mock_term("界a");
        let snapshot = terminal_render_snapshot(&term, 1, 3, None);

        assert_eq!(snapshot.lines().len(), 1);
        assert_eq!(snapshot.lines()[0].runs().len(), 1);
        let run = &snapshot.lines()[0].runs()[0];
        assert_eq!(run.text(), "界a");
        assert_eq!(run.width_cells(), 3);
    }
}
