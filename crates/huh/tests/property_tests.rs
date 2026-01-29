#![allow(clippy::pedantic)]
#![allow(clippy::nursery)]
#![allow(clippy::redundant_closure_for_method_calls)]

//! Property-based tests for huh form framework:
//! validators, Input, Select, MultiSelect, SelectOption, Confirm, Note, Text.

use huh::{
    validate_email, validate_min_length_8, validate_required, validate_required_name,
    Confirm, EchoMode, Input, MultiSelect, Note, Select, SelectOption, Text,
};
use proptest::prelude::*;

// =============================================================================
// Validator properties
// =============================================================================

proptest! {
    #[test]
    fn validate_required_accepts_non_empty(s in "[a-zA-Z0-9]{1,50}") {
        let v = validate_required("field");
        prop_assert!(v(&s).is_none(), "non-empty '{}' should pass", s);
    }

    #[test]
    fn validate_required_rejects_whitespace_only(s in "[ \\t\\n]{0,20}") {
        let v = validate_required("field");
        prop_assert!(v(&s).is_some(), "whitespace-only '{}' should fail", s);
    }

    #[test]
    fn validate_required_name_same_behavior(s in "\\PC{0,30}") {
        let vr = validate_required("name");
        let vn = validate_required_name();
        // Both should agree on pass/fail (only message differs)
        prop_assert_eq!(vr(&s).is_some(), vn(&s).is_some());
    }

    #[test]
    fn validate_min_length_8_accepts_long(s in ".{8,50}") {
        let v = validate_min_length_8();
        prop_assert!(v(&s).is_none(), "length {} should pass", s.chars().count());
    }

    #[test]
    fn validate_min_length_8_rejects_short(s in ".{0,7}") {
        let v = validate_min_length_8();
        prop_assert!(v(&s).is_some(), "length {} should fail", s.chars().count());
    }

    #[test]
    fn validate_email_rejects_no_at(s in "[a-zA-Z0-9.]{1,30}") {
        // No @ sign at all
        if !s.contains('@') {
            let v = validate_email();
            prop_assert!(v(&s).is_some());
        }
    }

    #[test]
    fn validate_email_accepts_valid(
        local in "[a-zA-Z][a-zA-Z0-9]{0,10}",
        domain in "[a-zA-Z]{1,10}",
        tld in "[a-zA-Z]{2,5}",
    ) {
        let email = format!("{local}@{domain}.{tld}");
        let v = validate_email();
        prop_assert!(v(&email).is_none(), "valid email '{}' should pass", email);
    }

    #[test]
    fn validate_email_rejects_empty_local(
        domain in "[a-zA-Z]{1,10}",
        tld in "[a-zA-Z]{2,5}",
    ) {
        let email = format!("@{domain}.{tld}");
        let v = validate_email();
        prop_assert!(v(&email).is_some());
    }

    #[test]
    fn validate_email_rejects_empty_domain(
        local in "[a-zA-Z]{1,10}",
    ) {
        let email = format!("{local}@");
        let v = validate_email();
        prop_assert!(v(&email).is_some());
    }

    #[test]
    fn validate_email_rejects_no_dot_in_domain(
        local in "[a-zA-Z]{1,10}",
        domain in "[a-zA-Z]{1,10}",
    ) {
        // domain without a dot
        let email = format!("{local}@{domain}");
        let v = validate_email();
        prop_assert!(v(&email).is_some());
    }

}

#[test]
fn validate_email_rejects_empty() {
    let v = validate_email();
    assert!(v("").is_some());
}

// =============================================================================
// SelectOption properties
// =============================================================================

proptest! {
    #[test]
    fn select_option_preserves_key_value(
        key in "[a-zA-Z]{1,20}",
        val in 0i32..1000,
    ) {
        let opt = SelectOption::new(key.clone(), val);
        prop_assert_eq!(opt.key, key);
        prop_assert_eq!(opt.value, val);
    }

    #[test]
    fn select_option_default_not_selected(
        key in "[a-zA-Z]{1,10}",
    ) {
        let opt = SelectOption::new(key, 42);
        prop_assert!(!opt.selected);
    }

    #[test]
    fn select_option_selected_toggle(
        key in "[a-zA-Z]{1,10}",
    ) {
        let opt = SelectOption::new(key, 1).selected(true);
        prop_assert!(opt.selected);

        let opt2 = opt.selected(false);
        prop_assert!(!opt2.selected);
    }
}

// =============================================================================
// Input properties
// =============================================================================

proptest! {
    #[test]
    fn input_new_never_panics(
        val in "\\PC{0,50}",
        placeholder in "\\PC{0,30}",
    ) {
        let _input = Input::new()
            .value(val)
            .placeholder(placeholder);
    }

    #[test]
    fn input_char_limit_never_panics(limit in 0usize..=100) {
        let _input = Input::new().char_limit(limit);
    }

    #[test]
    fn input_echo_modes_never_panic(
        val in "[a-zA-Z]{0,20}",
    ) {
        let _normal = Input::new().value(val.clone()).echo_mode(EchoMode::Normal);
        let _password = Input::new().value(val.clone()).echo_mode(EchoMode::Password);
        let _none = Input::new().value(val).echo_mode(EchoMode::None);
    }

    #[test]
    fn input_password_shorthand(val in "[a-zA-Z]{0,20}") {
        let _input = Input::new().value(val).password(true);
    }

    #[test]
    fn input_suggestions_never_panic(
        suggestions in prop::collection::vec("[a-zA-Z]{1,10}", 0..=10),
    ) {
        let _input = Input::new().suggestions(suggestions);
    }
}

// =============================================================================
// Select properties
// =============================================================================

proptest! {
    #[test]
    fn select_with_options_never_panics(
        keys in prop::collection::vec("[a-zA-Z]{1,10}", 1..=10),
    ) {
        let options: Vec<SelectOption<i32>> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| SelectOption::new(k.clone(), i as i32))
            .collect();
        let _select: Select<i32> = Select::new().options(options);
    }

    #[test]
    fn select_filterable_toggle(enabled in any::<bool>()) {
        let _select: Select<i32> = Select::new().filterable(enabled);
    }

    #[test]
    fn select_height_options(h in 1usize..=20) {
        let _select: Select<i32> = Select::new().height_options(h);
    }
}

// =============================================================================
// MultiSelect properties
// =============================================================================

proptest! {
    #[test]
    fn multiselect_with_options_never_panics(
        keys in prop::collection::vec("[a-zA-Z]{1,10}", 1..=10),
    ) {
        let options: Vec<SelectOption<i32>> = keys
            .iter()
            .enumerate()
            .map(|(i, k)| SelectOption::new(k.clone(), i as i32))
            .collect();
        let _ms: MultiSelect<i32> = MultiSelect::new().options(options);
    }

    #[test]
    fn multiselect_limit_never_panics(limit in 0usize..=20) {
        let _ms: MultiSelect<i32> = MultiSelect::new().limit(limit);
    }

    #[test]
    fn multiselect_filterable_toggle(enabled in any::<bool>()) {
        let _ms: MultiSelect<i32> = MultiSelect::new().filterable(enabled);
    }
}

// =============================================================================
// Confirm properties
// =============================================================================

proptest! {
    #[test]
    fn confirm_value_roundtrip(val in any::<bool>()) {
        let _confirm = Confirm::new().value(val);
    }
}

// =============================================================================
// Text properties
// =============================================================================

proptest! {
    #[test]
    fn text_value_never_panics(val in "\\PC{0,100}") {
        let _text = Text::new().value(val);
    }

    #[test]
    fn text_char_limit_never_panics(limit in 0usize..=500) {
        let _text = Text::new().char_limit(limit);
    }

    #[test]
    fn text_placeholder_never_panics(p in "\\PC{0,30}") {
        let _text = Text::new().placeholder(p);
    }
}

// =============================================================================
// Note properties
// =============================================================================

#[test]
fn note_never_panics() {
    let _note = Note::new();
}

// =============================================================================
// new_options helper
// =============================================================================

proptest! {
    #[test]
    fn new_options_creates_correct_count(
        keys in prop::collection::vec("[a-zA-Z]{1,10}", 1..=10),
    ) {
        let options = huh::new_options(keys.clone());
        prop_assert_eq!(options.len(), keys.len());
        for (opt, key) in options.iter().zip(keys.iter()) {
            prop_assert_eq!(&opt.key, key);
            prop_assert_eq!(&opt.value, key);
            prop_assert!(!opt.selected);
        }
    }
}
