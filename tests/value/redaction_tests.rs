// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! External tests for policy-aware [`qubit_value::Value`] redaction.

use std::collections::HashMap;

use qubit_redact::{
    Redact as _,
    RedactionPolicy,
    Sensitivity,
};
use qubit_value::Value;

#[test]
fn test_value_redacted_view_masks_sensitive_string_map_entries() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let policy = RedactionPolicy::empty_builder()
        .raise("api_key", Sensitivity::Secret)
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[test]
fn test_value_redacted_view_preserves_scalar_without_key_context() {
    let value = Value::String("visible-without-key".to_owned());
    let policy = RedactionPolicy::empty_builder()
        .raise("password", Sensitivity::Secret)
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(output.contains("visible-without-key"));
}

#[cfg(feature = "json")]
#[test]
fn test_value_redacted_view_masks_sensitive_non_string_json_values() {
    let value = Value::Json(serde_json::json!({
        "secret_number": 42,
        "secret_object": {"label": "nested-object-secret"},
        "secret_array": ["array-secret"],
        "secret_null": null,
        "visible": "public"
    }));
    let policy = RedactionPolicy::empty_builder()
        .raise("secret_number", Sensitivity::Secret)
        .raise("secret_object", Sensitivity::Secret)
        .raise("secret_array", Sensitivity::Secret)
        .raise("secret_null", Sensitivity::Secret)
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("42"));
    assert!(!output.contains("nested-object-secret"));
    assert!(!output.contains("array-secret"));
    assert_eq!(output.matches("<redacted>").count(), 4);
    assert!(output.contains("public"));
}
