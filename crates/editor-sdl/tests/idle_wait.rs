use editor_sdl::{IDLE_WAIT_CAP, idle_wait_timeout};
use std::time::{Duration, Instant};

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
fn idle_wait_timeout_uses_earliest_deadline() {
    let now = Instant::now();
    let later = now + Duration::from_millis(80);
    let sooner = now + Duration::from_millis(25);
    assert_eq!(
        idle_wait_timeout(now, &[later, sooner], false, false),
        Some(Duration::from_millis(25))
    );
}

#[test]
fn idle_wait_timeout_caps_far_deadlines() {
    let now = Instant::now();
    let deadline = now + Duration::from_secs(2);
    assert_eq!(
        idle_wait_timeout(now, &[deadline], false, false),
        Some(IDLE_WAIT_CAP)
    );
}

#[test]
fn idle_wait_timeout_is_cap_when_no_deadlines() {
    let now = Instant::now();
    assert_eq!(
        idle_wait_timeout(now, &[], false, false),
        Some(IDLE_WAIT_CAP)
    );
}

#[test]
fn idle_wait_timeout_is_zero_for_past_deadline() {
    let now = Instant::now();
    let deadline = now - Duration::from_millis(5);
    assert_eq!(
        idle_wait_timeout(now, &[deadline], false, false),
        Some(Duration::ZERO)
    );
}

#[test]
fn idle_wait_timeout_polls_while_interacting() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(idle_wait_timeout(now, &[deadline], true, false), None);
}

#[test]
fn idle_wait_timeout_polls_when_fps_overlay_is_on() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(idle_wait_timeout(now, &[deadline], false, true), None);
}
