// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload behavior.

#[cfg(feature = "json")]
use std::mem::ManuallyDrop;
#[cfg(feature = "json")]
use std::process::Command;

#[cfg(feature = "json")]
use qubit_value::Value;
#[cfg(feature = "json")]
use qubit_value::ValueWireEncodeError;
#[cfg(feature = "json")]
use qubit_value::ValueWirePayloadRefV1;

#[cfg(feature = "json")]
const DEEP_JSON_RESERVED_KEY_CHILD_ENV: &str =
    "QUBIT_VALUE_DEEP_JSON_RESERVED_KEY_CHILD";
#[cfg(feature = "json")]
const DEEP_JSON_DEPTH: usize = 10_000;
#[cfg(feature = "json")]
const JSON_NUMBER_TOKEN: &str = "$serde_json::private::Number";

#[test]
fn test_borrowed_payload_omits_envelope() {
    use qubit_value::Value;
    use qubit_value::ValueWirePayloadRefV1;

    assert_eq!(
        serde_json::to_value(
            ValueWirePayloadRefV1::try_from(&Value::Int32(1)).unwrap()
        )
        .unwrap(),
        serde_json::json!({"scalar": {"int32": 1}})
    );
}

/// Verifies reserved-key validation traverses deeply nested JSON iteratively.
#[cfg(feature = "json")]
#[test]
fn test_deep_json_reserved_key_validation_does_not_recurse() {
    let output = Command::new(std::env::current_exe().expect("locate test binary"))
        .arg("--exact")
        .arg("value_wire::value_wire_payload_ref_v1_tests::test_deep_json_reserved_key_validation_child")
        .arg("--ignored")
        .env(DEEP_JSON_RESERVED_KEY_CHILD_ENV, "1")
        .output()
        .expect("run deep JSON validation child test");

    assert!(
        output.status.success(),
        "deep JSON validation child failed: status={:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

/// Exercises reserved-key validation in a process isolated from stack overflow.
#[cfg(feature = "json")]
#[test]
#[ignore = "run only through test_deep_json_reserved_key_validation_does_not_recurse"]
fn test_deep_json_reserved_key_validation_child() {
    assert!(
        std::env::var_os(DEEP_JSON_RESERVED_KEY_CHILD_ENV).is_some(),
        "this helper test must run through its parent test"
    );

    let value = ManuallyDrop::new(Value::Json(build_deep_reserved_json_value()));
    let error = match ValueWirePayloadRefV1::try_from(&*value) {
        Err(error) => error,
        Ok(_) => panic!("deep reserved JSON key must be rejected"),
    };
    assert!(matches!(
        error,
        ValueWireEncodeError::ReservedJsonObjectKey {
            key: JSON_NUMBER_TOKEN
        }
    ));

    // SAFETY: `ManuallyDrop` prevents `Value` from dropping the JSON value.
    // No reference to the inner value is used after moving it out, so the
    // fixture has exactly one destructor path through iterative teardown.
    let fixture = unsafe {
        std::ptr::read(
            value
                .get_json_ref()
                .expect("test fixture must retain its JSON payload"),
        )
    };
    dismantle_json_value(fixture);
}

/// Builds an array chain with a reserved object key at its innermost leaf.
#[cfg(feature = "json")]
fn build_deep_reserved_json_value() -> serde_json::Value {
    let mut object = serde_json::Map::new();
    object.insert(
        JSON_NUMBER_TOKEN.to_owned(),
        serde_json::Value::String("123".to_owned()),
    );
    let mut value = serde_json::Value::Object(object);
    for _ in 0..DEEP_JSON_DEPTH {
        value = serde_json::Value::Array(vec![value]);
    }
    value
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
