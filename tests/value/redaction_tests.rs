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
    MaskPolicy,
    Redact as _,
    RedactValue as _,
    RedactionPolicy,
    Sensitivity,
};
use qubit_value::{
    MultiValues,
    NamedMultiValues,
    NamedValue,
    Value,
};

#[test]
fn test_value_redacted_view_masks_sensitive_string_map_entries() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let policy = RedactionPolicy::builder()
        .raise("api_key", Sensitivity::Secret)
        .expect("the test builder input should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[test]
fn test_value_redacted_view_preserves_scalar_without_key_context() {
    let value = Value::String("visible-without-key".to_owned());
    let policy = RedactionPolicy::builder()
        .raise("password", Sensitivity::Secret)
        .expect("the test builder input should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(output.contains("visible-without-key"));
}

#[test]
fn test_value_redact_value_masks_non_strings_with_configured_opaque_value() {
    let value = Value::Int32(12345);
    let masking = RedactionPolicy::builder()
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("the test mask policy should be valid")
        .build()
        .expect("policy should build")
        .masking()
        .clone();

    let output = value.redact_value(Sensitivity::Low, &masking);

    assert_eq!(format!("{output:?}"), "\"OPAQUE\"");
}

#[test]
fn test_named_value_redaction_uses_text_masking_for_sensitive_strings() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = RedactionPolicy::builder()
        .raise("token", Sensitivity::Low)
        .expect("the test builder input should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "MASK", 0),
        )
        .expect("the test mask policy should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("secret-token"));
    assert!(output.contains("****"));
}

#[test]
fn test_named_multi_values_redaction_masks_sensitive_collections_as_opaque() {
    let value = NamedMultiValues::new(
        "tokens",
        MultiValues::String(vec![
            "first-secret".to_owned(),
            "second-secret".to_owned(),
        ]),
    );
    let policy = RedactionPolicy::builder()
        .raise("tokens", Sensitivity::Low)
        .expect("the test builder input should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("the test mask policy should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("first-secret"));
    assert!(!output.contains("second-secret"));
    assert!(output.contains("OPAQUE"));
}
