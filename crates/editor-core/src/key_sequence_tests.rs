
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
fn incompatible_input_fires_pending_short_then_retries_token() {
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
    assert_eq!(
        result,
        KeySequencePush::FireShortThenRetry {
            chord: "Space w".to_owned(),
        }
    );
}

#[test]
fn incompatible_input_without_pending_short_cancels() {
    let registry = registry_with_space_w();
    let opts = options(250);

    // "Space" alone is prefix-only (no exact binding), so breaking it has
    // no short chord to fire.
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
