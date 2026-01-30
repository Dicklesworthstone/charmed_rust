//! Property-based tests for bubbletea message routing and command composition.
//!
//! bd-10x1: Verify message type system invariants, batch/sequence behavior,
//! and command composition using proptest.

// These casts are safe in tests where n_some < 10
#![expect(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]

use bubbletea::message::{BatchMsg, Message, SequenceMsg};
use bubbletea::{Cmd, batch, sequence};
use proptest::prelude::*;

// =============================================================================
// Message Type System
// =============================================================================

proptest! {
    /// `is::<T>()` returns true for the original type
    #[test]
    fn message_is_matches_original_type(val in any::<i64>()) {
        let msg = Message::new(val);
        prop_assert!(msg.is::<i64>());
    }

    /// `is::<T>()` returns false for a wrong type
    #[test]
    fn message_is_rejects_wrong_type(val in any::<i64>()) {
        let msg = Message::new(val);
        prop_assert!(!msg.is::<String>());
        prop_assert!(!msg.is::<f64>());
        prop_assert!(!msg.is::<bool>());
    }

    /// `downcast_ref` succeeds for the correct type and returns the original value
    #[test]
    fn message_downcast_ref_preserves_value(val in any::<i32>()) {
        let msg = Message::new(val);
        let inner = msg.downcast_ref::<i32>().unwrap();
        prop_assert_eq!(*inner, val);
    }

    /// `downcast_ref` can be called multiple times on the same message
    #[test]
    fn message_downcast_ref_non_consuming(val in any::<i32>()) {
        let msg = Message::new(val);
        let r1 = msg.downcast_ref::<i32>().unwrap();
        let r2 = msg.downcast_ref::<i32>().unwrap();
        prop_assert_eq!(*r1, *r2);
        prop_assert_eq!(*r1, val);
    }

    /// `downcast` succeeds and returns the original value
    #[test]
    fn message_downcast_preserves_value(val in any::<i32>()) {
        let msg = Message::new(val);
        let inner = msg.downcast::<i32>().unwrap();
        prop_assert_eq!(inner, val);
    }

    /// `downcast_ref` returns None for wrong type
    #[test]
    fn message_downcast_ref_none_for_wrong_type(val in any::<i32>()) {
        let msg = Message::new(val);
        prop_assert!(msg.downcast_ref::<String>().is_none());
    }

    /// `is` and `downcast_ref` are consistent: is::<T>() iff downcast_ref::<T>().is_some()
    #[test]
    fn message_is_consistent_with_downcast_ref(val in any::<u64>()) {
        let msg = Message::new(val);

        // Correct type: both should agree
        prop_assert_eq!(msg.is::<u64>(), msg.downcast_ref::<u64>().is_some());

        // Wrong type: both should agree
        prop_assert_eq!(msg.is::<String>(), msg.downcast_ref::<String>().is_some());
    }
}

// Various message payload types for testing
#[derive(Debug, PartialEq)]
struct MsgA(i32);
#[derive(Debug, PartialEq)]
struct MsgB(String);
#[derive(Debug, PartialEq)]
struct MsgC;

#[test]
fn message_type_discrimination_across_structs() {
    let msg_a = Message::new(MsgA(42));
    let msg_b = Message::new(MsgB("hello".into()));
    let msg_c = Message::new(MsgC);

    // Each message should only match its own type
    assert!(msg_a.is::<MsgA>());
    assert!(!msg_a.is::<MsgB>());
    assert!(!msg_a.is::<MsgC>());

    assert!(!msg_b.is::<MsgA>());
    assert!(msg_b.is::<MsgB>());
    assert!(!msg_b.is::<MsgC>());

    assert!(!msg_c.is::<MsgA>());
    assert!(!msg_c.is::<MsgB>());
    assert!(msg_c.is::<MsgC>());
}

// =============================================================================
// Batch Execution
// =============================================================================

proptest! {
    /// batch of all-None produces None
    #[test]
    fn batch_all_none_returns_none(n in 0usize..20) {
        let cmds: Vec<Option<Cmd>> = (0..n).map(|_| None).collect();
        prop_assert!(batch(cmds).is_none());
    }

    /// batch of exactly one Some returns an unwrapped command (not BatchMsg)
    #[test]
    fn batch_single_returns_unwrapped(val in any::<i32>()) {
        let cmd = batch(vec![Some(Cmd::new(move || Message::new(val)))]);
        prop_assert!(cmd.is_some());
        let msg = cmd.unwrap().execute().unwrap();
        // Single element should NOT be wrapped in BatchMsg
        prop_assert!(!msg.is::<BatchMsg>());
        prop_assert!(msg.is::<i32>());
        prop_assert_eq!(msg.downcast::<i32>().unwrap(), val);
    }

    /// batch of n > 1 commands wraps them in BatchMsg with correct count
    #[test]
    fn batch_multiple_wraps_in_batch_msg(n in 2usize..20) {
        let cmds: Vec<Option<Cmd>> = (0..n)
            .map(|i| Some(Cmd::new(move || Message::new(i as i32))))
            .collect();
        let cmd = batch(cmds);
        prop_assert!(cmd.is_some());
        let msg = cmd.unwrap().execute().unwrap();
        prop_assert!(msg.is::<BatchMsg>());
        let batch_msg = msg.downcast::<BatchMsg>().unwrap();
        prop_assert_eq!(batch_msg.0.len(), n);
    }

    /// batch filters out None values correctly
    #[test]
    fn batch_filters_none_correctly(
        n_some in 0usize..10,
        n_none in 0usize..10,
    ) {
        let mut cmds: Vec<Option<Cmd>> = (0..n_some)
            .map(|i| Some(Cmd::new(move || Message::new(i as i32))))
            .collect();
        cmds.extend(std::iter::repeat_with(|| None).take(n_none));

        let result = batch(cmds);
        match n_some {
            0 => prop_assert!(result.is_none()),
            1 => {
                let msg = result.unwrap().execute().unwrap();
                prop_assert!(!msg.is::<BatchMsg>());
            }
            _ => {
                let msg = result.unwrap().execute().unwrap();
                let batch_msg = msg.downcast::<BatchMsg>().unwrap();
                prop_assert_eq!(batch_msg.0.len(), n_some);
            }
        }
    }
}

// =============================================================================
// Sequence Execution
// =============================================================================

proptest! {
    /// sequence of all-None produces None
    #[test]
    fn sequence_all_none_returns_none(n in 0usize..20) {
        let cmds: Vec<Option<Cmd>> = (0..n).map(|_| None).collect();
        prop_assert!(sequence(cmds).is_none());
    }

    /// sequence of exactly one returns unwrapped command (not SequenceMsg)
    #[test]
    fn sequence_single_returns_unwrapped(val in any::<i32>()) {
        let cmd = sequence(vec![Some(Cmd::new(move || Message::new(val)))]);
        prop_assert!(cmd.is_some());
        let msg = cmd.unwrap().execute().unwrap();
        prop_assert!(!msg.is::<SequenceMsg>());
        prop_assert!(msg.is::<i32>());
        prop_assert_eq!(msg.downcast::<i32>().unwrap(), val);
    }

    /// sequence of n > 1 commands wraps them in SequenceMsg with correct count
    #[test]
    fn sequence_multiple_wraps_in_sequence_msg(n in 2usize..20) {
        let cmds: Vec<Option<Cmd>> = (0..n)
            .map(|i| Some(Cmd::new(move || Message::new(i as i32))))
            .collect();
        let cmd = sequence(cmds);
        prop_assert!(cmd.is_some());
        let msg = cmd.unwrap().execute().unwrap();
        prop_assert!(msg.is::<SequenceMsg>());
        let seq_msg = msg.downcast::<SequenceMsg>().unwrap();
        prop_assert_eq!(seq_msg.0.len(), n);
    }

    /// sequence filters out None values correctly
    #[test]
    fn sequence_filters_none_correctly(
        n_some in 0usize..10,
        n_none in 0usize..10,
    ) {
        let mut cmds: Vec<Option<Cmd>> = (0..n_some)
            .map(|i| Some(Cmd::new(move || Message::new(i as i32))))
            .collect();
        cmds.extend(std::iter::repeat_with(|| None).take(n_none));

        let result = sequence(cmds);
        match n_some {
            0 => prop_assert!(result.is_none()),
            1 => {
                let msg = result.unwrap().execute().unwrap();
                prop_assert!(!msg.is::<SequenceMsg>());
            }
            _ => {
                let msg = result.unwrap().execute().unwrap();
                let seq_msg = msg.downcast::<SequenceMsg>().unwrap();
                prop_assert_eq!(seq_msg.0.len(), n_some);
            }
        }
    }
}

// =============================================================================
// Command Execution Properties
// =============================================================================

proptest! {
    /// Cmd::new always produces Some(Message)
    #[test]
    fn cmd_new_always_produces_message(val in any::<i32>()) {
        let cmd = Cmd::new(move || Message::new(val));
        let result = cmd.execute();
        prop_assert!(result.is_some());
    }

    /// Cmd::new_optional with Some produces message
    #[test]
    fn cmd_new_optional_some_produces_message(val in any::<i32>()) {
        let cmd = Cmd::new_optional(move || Some(Message::new(val)));
        let result = cmd.execute();
        prop_assert!(result.is_some());
    }
}

#[test]
fn cmd_new_optional_none_returns_none() {
    let cmd = Cmd::new_optional(|| None);
    let result = cmd.execute();
    assert!(result.is_none());
}

#[test]
fn cmd_none_is_none() {
    assert!(Cmd::none().is_none());
}

// =============================================================================
// Command Composition Properties
// =============================================================================

#[test]
fn batch_of_sequences_produces_correct_structure() {
    // batch([seq(a,b), c]) should produce BatchMsg with 2 commands
    let seq = sequence(vec![
        Some(Cmd::new(|| Message::new(1i32))),
        Some(Cmd::new(|| Message::new(2i32))),
    ]);
    let cmd_c = Some(Cmd::new(|| Message::new(3i32)));

    let composed = batch(vec![seq, cmd_c]);
    assert!(composed.is_some());
    let msg = composed.unwrap().execute().unwrap();
    assert!(msg.is::<BatchMsg>());
    let batch_msg = msg.downcast::<BatchMsg>().unwrap();
    assert_eq!(batch_msg.0.len(), 2);

    // First command in batch should produce a SequenceMsg (seq(a,b))
    let first_msg = batch_msg.0.into_iter().next().unwrap().execute().unwrap();
    assert!(first_msg.is::<SequenceMsg>());
}

#[test]
fn sequence_of_batches_produces_correct_structure() {
    // sequence([batch(a,b), c]) should produce SequenceMsg with 2 commands
    let bat = batch(vec![
        Some(Cmd::new(|| Message::new(1i32))),
        Some(Cmd::new(|| Message::new(2i32))),
    ]);
    let cmd_c = Some(Cmd::new(|| Message::new(3i32)));

    let composed = sequence(vec![bat, cmd_c]);
    assert!(composed.is_some());
    let msg = composed.unwrap().execute().unwrap();
    assert!(msg.is::<SequenceMsg>());
    let seq_msg = msg.downcast::<SequenceMsg>().unwrap();
    assert_eq!(seq_msg.0.len(), 2);

    // First command in sequence should produce a BatchMsg
    let first_msg = seq_msg.0.into_iter().next().unwrap().execute().unwrap();
    assert!(first_msg.is::<BatchMsg>());
}

#[test]
fn deeply_nested_composition() {
    // batch([batch([a, b]), sequence([c, d])])
    let inner_batch = batch(vec![
        Some(Cmd::new(|| Message::new(1i32))),
        Some(Cmd::new(|| Message::new(2i32))),
    ]);
    let inner_seq = sequence(vec![
        Some(Cmd::new(|| Message::new(3i32))),
        Some(Cmd::new(|| Message::new(4i32))),
    ]);

    let outer = batch(vec![inner_batch, inner_seq]);
    assert!(outer.is_some());
    let msg = outer.unwrap().execute().unwrap();
    assert!(msg.is::<BatchMsg>());
    let outer_batch = msg.downcast::<BatchMsg>().unwrap();
    assert_eq!(outer_batch.0.len(), 2);
}

// =============================================================================
// Batch/Sequence Cmd Execution Guarantees
// =============================================================================

#[test]
fn batch_all_commands_produce_messages() {
    let cmds: Vec<Option<Cmd>> = (0..5)
        .map(|i| Some(Cmd::new(move || Message::new(i))))
        .collect();
    let cmd = batch(cmds).unwrap();
    let msg = cmd.execute().unwrap();
    let batch_msg = msg.downcast::<BatchMsg>().unwrap();

    // Every command in the batch should produce a message
    for cmd in batch_msg.0 {
        assert!(cmd.execute().is_some());
    }
}

#[test]
fn sequence_all_commands_produce_messages() {
    let cmds: Vec<Option<Cmd>> = (0..5)
        .map(|i| Some(Cmd::new(move || Message::new(i))))
        .collect();
    let cmd = sequence(cmds).unwrap();
    let msg = cmd.execute().unwrap();
    let seq_msg = msg.downcast::<SequenceMsg>().unwrap();

    // Every command in the sequence should produce a message
    for cmd in seq_msg.0 {
        assert!(cmd.execute().is_some());
    }
}

#[test]
fn batch_commands_preserve_payload_values() {
    let cmds: Vec<Option<Cmd>> = (0..5)
        .map(|i: i32| Some(Cmd::new(move || Message::new(i * 10))))
        .collect();
    let cmd = batch(cmds).unwrap();
    let msg = cmd.execute().unwrap();
    let batch_msg = msg.downcast::<BatchMsg>().unwrap();

    let results: Vec<i32> = batch_msg
        .0
        .into_iter()
        .map(|c: Cmd| c.execute().unwrap().downcast::<i32>().unwrap())
        .collect();

    // Values should be 0, 10, 20, 30, 40
    assert_eq!(results, vec![0, 10, 20, 30, 40]);
}

#[test]
fn sequence_commands_preserve_payload_values() {
    let cmds: Vec<Option<Cmd>> = (0..5)
        .map(|i: i32| Some(Cmd::new(move || Message::new(i * 10))))
        .collect();
    let cmd = sequence(cmds).unwrap();
    let msg = cmd.execute().unwrap();
    let seq_msg = msg.downcast::<SequenceMsg>().unwrap();

    let results: Vec<i32> = seq_msg
        .0
        .into_iter()
        .map(|c: Cmd| c.execute().unwrap().downcast::<i32>().unwrap())
        .collect();

    // Sequence preserves order
    assert_eq!(results, vec![0, 10, 20, 30, 40]);
}
