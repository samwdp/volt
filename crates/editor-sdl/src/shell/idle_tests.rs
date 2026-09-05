
use super::*;

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
fn idle_wait_timeout_polls_while_interacting_or_showing_fps() {
    let now = Instant::now();
    let deadline = now + Duration::from_millis(40);
    assert_eq!(idle_wait_timeout(now, &[deadline], true, false), None);
    assert_eq!(idle_wait_timeout(now, &[deadline], false, true), None);
}

#[test]
fn ping_without_sdl_attach_is_noop() {
    ping_shell_wakeup();
    ping_shell_wakeup();
    assert!(!ShellWakeup::global().inner.pending.load(Ordering::Acquire));
}
