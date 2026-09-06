use super::*;

#[test]
fn sync_visible_buffer_layouts_reuses_headerline_snapshot_while_typing() -> Result<(), String> {
    let render_width = 640;
    let render_height = 360;
    let cell_width = 8;
    let line_height = 16;
    let user_library = Arc::new(HeaderlineTestUserLibrary::default());
    let mut state =
        ShellState::new_with_user_library(default_error_log_path(), false, user_library.clone())
            .map_err(|error| error.to_string())?;
    let buffer_id = install_text_test_buffer(
        &mut state,
        "*typing-headerline-cache*",
        vec!["alpha".to_owned()],
    )?;

    let before = user_library.headerline_call_count();
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    let after_first = user_library.headerline_call_count();
    assert!(after_first > before);

    {
        let buffer = shell_buffer_mut(&mut state.runtime, buffer_id)?;
        buffer.set_cursor(TextPoint::new(0, 5));
        buffer.insert_text("!");
    }
    state.last_text_input_at = Some(Instant::now());
    state
        .sync_visible_buffer_layouts(render_width, render_height, cell_width, line_height)
        .map_err(|error| error.to_string())?;
    assert_eq!(user_library.headerline_call_count(), after_first);
    Ok(())
}

#[test]
fn frame_pacing_remaining_clamps_to_120fps_budget() {
    let now = Instant::now();
    let remaining = frame_pacing_remaining(now - Duration::from_millis(2), now);
    assert!(remaining >= Duration::from_micros(6_000));
    assert_eq!(
        frame_pacing_remaining(now - Duration::from_millis(10), now),
        Duration::from_secs(0)
    );
}

#[test]
fn secondary_refresh_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(secondary_refresh_deferred_for_typing(Some(now), now));
    assert!(secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(
        Some(now - GIT_REFRESH_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!secondary_refresh_deferred_for_typing(None, now));
}

#[test]
fn frame_pacing_is_deferred_while_typing() {
    let now = Instant::now();
    assert!(frame_pacing_deferred_for_typing(Some(now), now));
    assert!(frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD + Duration::from_millis(1)),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(
        Some(now - FRAME_PACING_TYPING_IDLE_THRESHOLD),
        now
    ));
    assert!(!frame_pacing_deferred_for_typing(None, now));
}

#[test]
fn idle_wait_timeout_equals_next_deadline_when_idle() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(
        idle_wait_timeout(now, &[deadline], false, false),
        Some(Duration::from_millis(40))
    );
}

#[test]
fn idle_wait_timeout_caps_and_skips_when_interacting() {
    let now = Instant::now();
    assert_eq!(
        idle_wait_timeout(now, &[], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_secs(5)], false, false),
        Some(IDLE_WAIT_CAP)
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], true, false),
        None
    );
    assert_eq!(
        idle_wait_timeout(now, &[now + Duration::from_millis(40)], false, true),
        None
    );
}

#[test]
fn normal_mode_text_input_does_not_activate_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;

    assert_eq!(shell_ui(&state.runtime)?.input_mode(), InputMode::Normal);
    state
        .handle_text_input("k")
        .map_err(|error| error.to_string())?;

    assert!(!state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(!state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn insert_mode_text_input_activates_typing_budget() -> Result<(), String> {
    let mut state = state_with_user_library()?;
    shell_ui_mut(&mut state.runtime)?.enter_insert_mode();

    state
        .handle_text_input("x")
        .map_err(|error| error.to_string())?;

    assert!(state.secondary_refresh_deferred_for_typing(Instant::now()));
    assert!(state.typing_refresh_budget_active(Instant::now()));
    Ok(())
}

#[test]
fn typing_event_batches_yield_once_budget_is_exhausted() {
    let now = Instant::now();
    assert!(!should_yield_after_typing_batch(
        0,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(!should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT - 1,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        TYPING_EVENT_BATCH_LIMIT,
        now
    ));
    assert!(should_yield_after_typing_batch(
        1,
        1,
        now - TYPING_EVENT_BATCH_TIME_BUDGET
    ));
}
