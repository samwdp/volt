//! Idle event-wait helpers for the SDL demo shell frame loop.

use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicBool, Ordering},
};
use std::time::{Duration, Instant};

use sdl3::EventSubsystem;
use sdl3::event::Event;

use crate::config::ShellError;

/// Upper bound on idle `SDL_WaitEventTimeout` so missed wakeups still recover.
pub const IDLE_WAIT_CAP: Duration = Duration::from_millis(100);

struct ShellWakeupEvent;

struct ShellWakeupInner {
    sender: Mutex<Option<sdl3::event::EventSender>>,
    pending: AtomicBool,
}

#[derive(Clone)]
struct ShellWakeup {
    inner: Arc<ShellWakeupInner>,
}

impl ShellWakeup {
    fn new() -> Self {
        Self {
            inner: Arc::new(ShellWakeupInner {
                sender: Mutex::new(None),
                pending: AtomicBool::new(false),
            }),
        }
    }

    fn global() -> &'static Self {
        static WAKEUP: OnceLock<ShellWakeup> = OnceLock::new();
        WAKEUP.get_or_init(Self::new)
    }

    fn attach(&self, events: &EventSubsystem) -> Result<(), ShellError> {
        let _ = events.register_custom_event::<ShellWakeupEvent>();
        let sender = events.event_sender();
        let mut slot = self
            .inner
            .sender
            .lock()
            .map_err(|_| ShellError::Sdl("shell wakeup mutex poisoned".to_owned()))?;
        *slot = Some(sender);
        Ok(())
    }

    fn ping(&self) {
        if self.inner.pending.swap(true, Ordering::AcqRel) {
            return;
        }
        let pushed = match self.inner.sender.lock() {
            Ok(slot) => slot
                .as_ref()
                .and_then(|sender| sender.push_custom_event(ShellWakeupEvent).ok())
                .is_some(),
            Err(_) => false,
        };
        if !pushed {
            self.inner.pending.store(false, Ordering::Release);
        }
    }

    fn clear_pending(&self) {
        self.inner.pending.store(false, Ordering::Release);
    }
}

/// Computes how long the idle shell should block for the next SDL event.
///
/// Returns `None` when the loop should keep polling and pacing at 120 Hz
/// (keys/recent typing, or the FPS overlay). Otherwise returns the time until
/// the next known deadline, capped at [`IDLE_WAIT_CAP`]. An empty deadline list
/// waits the full cap. Past-due deadlines yield [`Duration::ZERO`].
pub fn idle_wait_timeout(
    now: Instant,
    deadlines: &[Instant],
    interaction_active: bool,
    fps_overlay: bool,
) -> Option<Duration> {
    if interaction_active || fps_overlay {
        return None;
    }
    let until_deadline = deadlines
        .iter()
        .copied()
        .map(|deadline| {
            deadline
                .checked_duration_since(now)
                .unwrap_or(Duration::ZERO)
        })
        .min()
        .unwrap_or(IDLE_WAIT_CAP);
    Some(until_deadline.min(IDLE_WAIT_CAP))
}

pub(super) fn idle_wait_timeout_ms(timeout: Duration) -> u32 {
    timeout.as_millis().min(u128::from(u32::MAX)) as u32
}

pub(super) fn attach_shell_wakeup(events: &EventSubsystem) -> Result<(), ShellError> {
    ShellWakeup::global().attach(events)
}

pub(super) fn ping_shell_wakeup() {
    ShellWakeup::global().ping();
}

pub(super) fn take_shell_wakeup_event(event: &Event) -> bool {
    if !event.is_user_event() {
        return false;
    }
    if event.as_user_event_type::<ShellWakeupEvent>().is_some() {
        ShellWakeup::global().clear_pending();
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
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
}
