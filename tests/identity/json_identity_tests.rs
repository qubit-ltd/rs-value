// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests JSON identity normalization.

#[cfg(feature = "json")]
use std::mem::ManuallyDrop;
#[cfg(feature = "json")]
use std::process::Command;

use qubit_value::Value;

#[cfg(feature = "json")]
const DEEP_JSON_IDENTITY_CHILD_ENV: &str = "QUBIT_VALUE_DEEP_JSON_IDENTITY_CHILD";
#[cfg(feature = "json")]
const DEEP_JSON_DEPTH: usize = 10_000;

/// Verifies JSON object key order does not affect public value identity.
#[test]
fn test_json_identity_ignores_object_key_order() {
    let left = Value::Json(serde_json::json!({"first": 1, "second": 2}));
    let right = Value::Json(serde_json::json!({"second": 2, "first": 1}));
    assert_eq!(left, right);
}

/// Verifies deeply nested JSON equality is evaluated without recursion.
#[cfg(feature = "json")]
#[test]
fn test_deep_json_identity_equality_does_not_recurse() {
    let output = Command::new(std::env::current_exe().expect("locate test binary"))
        .arg("--exact")
        .arg("identity::json_identity_tests::test_deep_json_identity_equality_child")
        .arg("--ignored")
        .env(DEEP_JSON_IDENTITY_CHILD_ENV, "1")
        .output()
        .expect("run deep JSON identity child test");

    assert!(
        output.status.success(),
        "deep JSON identity child failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Exercises JSON equality in a process isolated from stack overflow.
#[cfg(feature = "json")]
#[test]
#[ignore = "run only through test_deep_json_identity_equality_does_not_recurse"]
fn test_deep_json_identity_equality_child() {
    assert!(
        std::env::var_os(DEEP_JSON_IDENTITY_CHILD_ENV).is_some(),
        "this helper test must run through its parent test"
    );

    let left = ManuallyDrop::new(Value::Json(build_deep_json_value(1)));
    let equal = ManuallyDrop::new(Value::Json(build_deep_json_value(1)));
    let unequal = ManuallyDrop::new(Value::Json(build_deep_json_value(2)));

    assert!(
        *left == *equal,
        "matching deeply nested JSON values must be equal"
    );
    assert!(
        *left != *unequal,
        "leaf differences in deeply nested JSON values must not be equal"
    );

    // SAFETY: each wrapper is manually dropped and its payload is moved out
    // exactly once for iterative teardown.
    unsafe {
        dismantle_json_value(read_json_fixture(&left));
        dismantle_json_value(read_json_fixture(&equal));
        dismantle_json_value(read_json_fixture(&unequal));
    }
}

/// Builds an array chain whose innermost value is the supplied integer.
#[cfg(feature = "json")]
fn build_deep_json_value(leaf: i64) -> serde_json::Value {
    let mut value = serde_json::Value::Number(leaf.into());
    for _ in 0..DEEP_JSON_DEPTH {
        value = serde_json::Value::Array(vec![value]);
    }
    value
}

/// Moves a JSON fixture out of a manually dropped value.
///
/// # Safety
///
/// Callers must ensure the value is never dropped normally after this function
/// returns and that its payload is moved out exactly once, because this
/// function transfers ownership of the JSON payload.
#[cfg(feature = "json")]
unsafe fn read_json_fixture(value: &ManuallyDrop<Value>) -> serde_json::Value {
    // SAFETY: the caller keeps `value` manually dropped and moves the payload
    // exactly once into `dismantle_json_value`.
    unsafe {
        std::ptr::read(
            value
                .get_json_ref()
                .expect("test fixture must retain its JSON payload"),
        )
    }
}

/// Iteratively consumes a JSON fixture so its destructor cannot recurse.
#[cfg(feature = "json")]
fn dismantle_json_value(value: serde_json::Value) {
    let mut pending = vec![value];
    while let Some(value) = pending.pop() {
        match value {
            serde_json::Value::Array(values) => pending.extend(values),
            serde_json::Value::Object(values) => pending.extend(values.into_values()),
            serde_json::Value::Null
            | serde_json::Value::Bool(_)
            | serde_json::Value::Number(_)
            | serde_json::Value::String(_) => {}
        }
    }
}
