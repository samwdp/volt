//! Large-JSON edit latency probe (hang is during edit, not open).
//!
//! ```text
//! cargo test -p editor-sdl large_json_edit_timing_probe -- --ignored --nocapture
//! ```

use std::path::PathBuf;
use std::time::{Duration, Instant};

use editor_buffer::TextPoint;

use super::*;

const RENDER_WIDTH: u32 = 960;
const RENDER_HEIGHT: u32 = 640;
const CELL_WIDTH: i32 = 8;
const LINE_HEIGHT: i32 = 16;

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../uk-valhalla-load-results.json")
}

fn percentile(sorted_ms: &[f64], p: f64) -> f64 {
    if sorted_ms.is_empty() {
        return 0.0;
    }
    let index = ((sorted_ms.len() as f64 - 1.0) * p).round() as usize;
    sorted_ms[index.min(sorted_ms.len().saturating_sub(1))]
}

fn summarize(label: &str, samples_ms: &mut [f64]) {
    samples_ms.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    let n = samples_ms.len();
    let sum: f64 = samples_ms.iter().sum();
    eprintln!(
        "{label}: n={n} min={:.1}ms p50={:.1}ms p95={:.1}ms max={:.1}ms avg={:.1}ms",
        samples_ms.first().copied().unwrap_or(0.0),
        percentile(samples_ms, 0.50),
        percentile(samples_ms, 0.95),
        samples_ms.last().copied().unwrap_or(0.0),
        if n == 0 { 0.0 } else { sum / n as f64 },
    );
}

/// Open real fixture, wait for syntax, then simulate insert-mode typing like the shell loop.
#[test]
#[ignore = "manual large-file edit timing probe"]
fn large_json_edit_timing_probe() -> Result<(), String> {
    let path = fixture_path();
    if !path.is_file() {
        eprintln!("SKIP: missing {}", path.display());
        return Ok(());
    }

    let mut state = state_with_user_library()?;
    let open_started = Instant::now();
    let buffer_id = open_workspace_file(&mut state.runtime, &path)?;
    let open_elapsed = open_started.elapsed();
    eprintln!("open_workspace_file: {open_elapsed:?}");

    let syntax_started = Instant::now();
    wait_for_buffer_syntax_refresh(&mut state.runtime, buffer_id)?;
    eprintln!(
        "wait_for_buffer_syntax_refresh: {:?}",
        syntax_started.elapsed()
    );

    let layout_started = Instant::now();
    sync_active_buffer_layout_for_test(&mut state)?;
    eprintln!(
        "first sync_visible_buffer_layouts (wrap cache build): {:?}",
        layout_started.elapsed()
    );

    {
        let ui = shell_ui_mut(&mut state.runtime)?;
        ui.enter_insert_mode();
    }
    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        let mid = buffer.line_count() / 2;
        buffer.set_cursor(TextPoint::new(mid, 0));
    }
    sync_active_buffer_layout_for_test(&mut state)?;

    let mut char_samples = Vec::with_capacity(40);
    for _ in 0..40 {
        let started = Instant::now();
        state
            .handle_text_input("x")
            .map_err(|error| error.to_string())?;
        state
            .sync_visible_buffer_layouts(RENDER_WIDTH, RENDER_HEIGHT, CELL_WIDTH, LINE_HEIGHT)
            .map_err(|error| error.to_string())?;
        char_samples.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    summarize("char_insert+layout", &mut char_samples);

    let mut newline_samples = Vec::with_capacity(10);
    for _ in 0..10 {
        let started = Instant::now();
        state
            .handle_text_input("\n")
            .map_err(|error| error.to_string())?;
        state
            .sync_visible_buffer_layouts(RENDER_WIDTH, RENDER_HEIGHT, CELL_WIDTH, LINE_HEIGHT)
            .map_err(|error| error.to_string())?;
        newline_samples.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    summarize("newline_insert+layout", &mut newline_samples);

    state.last_text_input_at = Some(Instant::now() - Duration::from_millis(200));
    let idle_started = Instant::now();
    let _ = state.refresh_pending_syntax(false);
    let syntax_idle = idle_started.elapsed();
    let lsp_started = Instant::now();
    let _ = state.refresh_pending_lsp(false);
    let lsp_idle = lsp_started.elapsed();
    eprintln!("idle refresh_pending_syntax: {syntax_idle:?}");
    eprintln!("idle refresh_pending_lsp: {lsp_idle:?}");

    let buffer = shell_buffer(&state.runtime, buffer_id)?;
    eprintln!(
        "buffer lines={} wrap={} language={:?} wrap_cache={}",
        buffer.line_count(),
        buffer.line_wrap,
        buffer.language_id(),
        buffer.wrap_cache.is_some()
    );

    char_samples.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let char_p95 = percentile(&char_samples, 0.95);
    assert!(
        char_p95 < 50.0,
        "char insert+layout p95 {char_p95:.1}ms exceeds 50ms debug budget (edit hang)"
    );

    Ok(())
}
