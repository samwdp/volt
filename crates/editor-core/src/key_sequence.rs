//! Ambiguous-prefix key sequence resolution.
//!
//! When a registered chord is both an exact binding and a proper prefix of a longer
//! binding, the short chord waits for [`DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS`] (or a
//! configured override) before firing. Completing a longer chord within the window
//! fires the long binding and cancels the short. Incompatible input clears the
//! pending short without firing it.

use crate::{KeymapRegistry, KeymapScope, KeymapVimMode};

/// Default ambiguous-prefix timeout in milliseconds (`ui.keymap.ambiguous_prefix_timeout_ms`).
pub const DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS: u64 = 250;

/// Idle timeout for incomplete (non-ambiguous) multi-key prefixes before they are dropped.
pub const DEFAULT_SEQUENCE_IDLE_TIMEOUT_MS: u64 = 1200;

/// Tunables for key sequence resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySequenceOptions {
    /// Milliseconds to wait before firing a short chord that is also a longer prefix.
    pub ambiguous_prefix_timeout_ms: u64,
    /// Milliseconds after which an incomplete prefix-only sequence is dropped.
    pub sequence_idle_timeout_ms: u64,
}

impl Default for KeySequenceOptions {
    fn default() -> Self {
        Self {
            ambiguous_prefix_timeout_ms: DEFAULT_AMBIGUOUS_PREFIX_TIMEOUT_MS,
            sequence_idle_timeout_ms: DEFAULT_SEQUENCE_IDLE_TIMEOUT_MS,
        }
    }
}

/// Pending multi-key sequence state, keyed by monotonic milliseconds for testability.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingKeySequence {
    /// Normalized tokens collected so far.
    pub tokens: Vec<String>,
    /// Clock time when this pending state began (or was last extended).
    pub started_at_ms: u64,
    /// Exact short chord waiting on the ambiguous-prefix timeout, if any.
    pub ambiguous_short: Option<String>,
}

/// Result of feeding one key token into the sequence resolver.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySequencePush {
    /// Keep waiting for more input (prefix incomplete and/or ambiguous short pending).
    Wait(PendingKeySequence),
    /// Fire this chord now and clear pending state.
    Execute {
        /// Normalized chord to execute.
        chord: String,
    },
    /// Clear pending ambiguous short without firing it.
    Cancel,
    /// Token does not continue a sequence; pending cleared if any.
    Miss,
}

/// Result of polling a pending sequence against the clock.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeySequenceTick {
    /// Still within the wait window.
    Pending,
    /// Ambiguous short timed out — fire it.
    Execute {
        /// Normalized short chord to execute.
        chord: String,
    },
    /// Incomplete prefix-only sequence idle-expired — drop without firing.
    Expired,
}

/// Pushes `token` onto `pending` (if still live) and resolves against the registry.
pub fn push_key_sequence(
    registry: &KeymapRegistry,
    scope: &KeymapScope,
    vim_mode: KeymapVimMode,
    pending: Option<PendingKeySequence>,
    token: &str,
    now_ms: u64,
    options: &KeySequenceOptions,
) -> KeySequencePush {
    let live_pending = pending.and_then(|state| retain_live_pending(state, now_ms, options));
    let mut tokens = live_pending.map(|state| state.tokens).unwrap_or_default();
    tokens.push(normalize_token(token));
    resolve_tokens(registry, scope, vim_mode, tokens, now_ms)
}

/// Polls `pending` for ambiguous-prefix fire or idle expiry.
pub fn tick_key_sequence(
    pending: &PendingKeySequence,
    now_ms: u64,
    options: &KeySequenceOptions,
) -> KeySequenceTick {
    let elapsed = now_ms.saturating_sub(pending.started_at_ms);
    if let Some(chord) = pending.ambiguous_short.as_ref() {
        if elapsed >= options.ambiguous_prefix_timeout_ms {
            return KeySequenceTick::Execute {
                chord: chord.clone(),
            };
        }
        return KeySequenceTick::Pending;
    }
    if elapsed >= options.sequence_idle_timeout_ms {
        return KeySequenceTick::Expired;
    }
    KeySequenceTick::Pending
}

fn retain_live_pending(
    state: PendingKeySequence,
    now_ms: u64,
    options: &KeySequenceOptions,
) -> Option<PendingKeySequence> {
    match tick_key_sequence(&state, now_ms, options) {
        KeySequenceTick::Pending => Some(state),
        // Timed-out ambiguous short should already have been fired by tick; drop so the
        // new token starts a fresh sequence rather than extending a dead short.
        KeySequenceTick::Execute { .. } | KeySequenceTick::Expired => None,
    }
}

fn resolve_tokens(
    registry: &KeymapRegistry,
    scope: &KeymapScope,
    vim_mode: KeymapVimMode,
    tokens: Vec<String>,
    now_ms: u64,
) -> KeySequencePush {
    let chord = tokens.join(" ");
    let exact = registry.contains_for_mode(scope, vim_mode, &chord);
    let longer_prefix = registry.has_sequence_prefix_for_mode(scope, vim_mode, &tokens);

    if exact && longer_prefix {
        return KeySequencePush::Wait(PendingKeySequence {
            tokens,
            started_at_ms: now_ms,
            ambiguous_short: Some(chord),
        });
    }
    if exact {
        return KeySequencePush::Execute { chord };
    }
    if longer_prefix {
        return KeySequencePush::Wait(PendingKeySequence {
            tokens,
            started_at_ms: now_ms,
            ambiguous_short: None,
        });
    }

    // Incompatible continuation: if we had been building a sequence, cancel without
    // firing any pending short (caller had already moved into this resolve after a live
    // pending). Distinguish single-token miss from cancel of a multi-token attempt.
    if tokens.len() > 1 {
        KeySequencePush::Cancel
    } else {
        KeySequencePush::Miss
    }
}

fn normalize_token(token: &str) -> String {
    crate::keymaps::normalize_chord_token(token)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::CommandSource;

    fn registry_with_space_w() -> KeymapRegistry {
        let mut registry = KeymapRegistry::new();
        registry
            .register(
                "Space w",
                "buffer.save",
                KeymapScope::Workspace,
                CommandSource::Core,
            )
            .expect("register short");
        registry
            .register(
                "Space w n",
                "workspace.next",
                KeymapScope::Workspace,
                CommandSource::Core,
            )
            .expect("register long");
        registry
    }

    fn options(ambiguous_ms: u64) -> KeySequenceOptions {
        KeySequenceOptions {
            ambiguous_prefix_timeout_ms: ambiguous_ms,
            ..KeySequenceOptions::default()
        }
    }

    #[test]
    fn ambiguous_short_waits_then_fires_on_timeout() {
        let registry = registry_with_space_w();
        let opts = options(250);

        let after_space = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            None,
            "Space",
            0,
            &opts,
        );
        let KeySequencePush::Wait(pending) = after_space else {
            panic!("expected wait after Space, got {after_space:?}");
        };
        assert!(pending.ambiguous_short.is_none());

        let after_w = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "w",
            10,
            &opts,
        );
        let KeySequencePush::Wait(pending) = after_w else {
            panic!("expected ambiguous wait after Space w, got {after_w:?}");
        };
        assert_eq!(pending.ambiguous_short.as_deref(), Some("Space w"));

        assert_eq!(
            tick_key_sequence(&pending, 259, &opts),
            KeySequenceTick::Pending
        );
        assert_eq!(
            tick_key_sequence(&pending, 260, &opts),
            KeySequenceTick::Execute {
                chord: "Space w".to_owned(),
            }
        );
    }

    #[test]
    fn longer_chord_within_window_cancels_short() {
        let registry = registry_with_space_w();
        let opts = options(250);

        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            None,
            "Space",
            0,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected wait, got {other:?}"),
        };
        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "w",
            10,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected ambiguous wait, got {other:?}"),
        };

        let result = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "n",
            100,
            &opts,
        );
        assert_eq!(
            result,
            KeySequencePush::Execute {
                chord: "Space w n".to_owned(),
            }
        );
    }

    #[test]
    fn incompatible_input_clears_pending_short_without_firing() {
        let registry = registry_with_space_w();
        let opts = options(250);

        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            None,
            "Space",
            0,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected wait, got {other:?}"),
        };
        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "w",
            10,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected ambiguous wait, got {other:?}"),
        };

        let result = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "x",
            50,
            &opts,
        );
        assert_eq!(result, KeySequencePush::Cancel);
    }

    #[test]
    fn ambiguous_prefix_timeout_is_configurable() {
        let registry = registry_with_space_w();
        let opts = options(100);

        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            None,
            "Space",
            0,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected wait, got {other:?}"),
        };
        let pending = match push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "w",
            0,
            &opts,
        ) {
            KeySequencePush::Wait(pending) => pending,
            other => panic!("expected ambiguous wait, got {other:?}"),
        };

        assert_eq!(
            tick_key_sequence(&pending, 99, &opts),
            KeySequenceTick::Pending
        );
        assert_eq!(
            tick_key_sequence(&pending, 100, &opts),
            KeySequenceTick::Execute {
                chord: "Space w".to_owned(),
            }
        );
    }

    #[test]
    fn exact_chord_without_longer_prefix_fires_immediately() {
        let mut registry = KeymapRegistry::new();
        registry
            .register(
                "Space w",
                "buffer.save",
                KeymapScope::Workspace,
                CommandSource::Core,
            )
            .expect("register");

        let after_space = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            None,
            "Space",
            0,
            &KeySequenceOptions::default(),
        );
        let KeySequencePush::Wait(pending) = after_space else {
            panic!("expected prefix wait, got {after_space:?}");
        };
        assert!(pending.ambiguous_short.is_none());

        let result = push_key_sequence(
            &registry,
            &KeymapScope::Workspace,
            KeymapVimMode::Any,
            Some(pending),
            "w",
            10,
            &KeySequenceOptions::default(),
        );
        assert_eq!(
            result,
            KeySequencePush::Execute {
                chord: "Space w".to_owned(),
            }
        );
    }
}
