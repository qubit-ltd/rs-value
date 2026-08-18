// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! External tests for policy-aware [`qubit_value::Value`] redaction.

use std::collections::HashMap;

use qubit_budget::StructureLimits;
use qubit_redact::MaskPolicy;
use qubit_redact::RedactionPolicy;
use qubit_redact::Sensitivity;
use qubit_redact::domain::Redact as _;
use qubit_redact::domain::RedactValue as _;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;

/// Builds a policy that classifies one field and limits domain nodes.
fn sensitive_policy_with_nodes(
    field: &str,
    max_nodes: usize,
) -> RedactionPolicy {
    let limits = StructureLimits::builder()
        .max_nodes(max_nodes)
        .max_sequence_items(1024)
        .max_depth(32)
        .build();
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .raise(field, Sensitivity::Secret)
        .expect("the test field rule should be valid");
    builder.limits().domain(limits);
    builder.build().expect("policy should build")
}

#[test]
fn test_value_redacted_view_masks_sensitive_string_map_entries() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .raise("api_key", Sensitivity::Secret)
        .expect("the test builder input should be valid");
    let policy = builder.build().expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[test]
fn test_value_redacted_view_preserves_scalar_without_key_context() {
    let value = Value::String("visible-without-key".to_owned());
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .raise("password", Sensitivity::Secret)
        .expect("the test builder input should be valid");
    let policy = builder.build().expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(output.contains("visible-without-key"));
}

#[test]
fn test_value_redact_value_masks_non_strings_with_configured_opaque_value() {
    let value = Value::Int32(12345);
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("the test mask policy should be valid");
    let masking = builder
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
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .raise("token", Sensitivity::Low)
        .expect("the test builder input should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "MASK", 0),
        )
        .expect("the test mask policy should be valid");
    let policy = builder.build().expect("policy should build");

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
    let mut builder = RedactionPolicy::builder();
    builder
        .edit_fields()
        .raise("tokens", Sensitivity::Low)
        .expect("the test builder input should be valid")
        .mask(
            Sensitivity::Low,
            MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
        )
        .expect("the test mask policy should be valid");
    let policy = builder.build().expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("first-secret"));
    assert!(!output.contains("second-secret"));
    assert!(output.contains("OPAQUE"));
}

#[test]
fn test_named_value_exact_wrapper_node_budget_is_complete() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = sensitive_policy_with_nodes("token", 5);

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("secret-token"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_named_value_one_less_wrapper_node_truncates() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = sensitive_policy_with_nodes("token", 4);

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("secret-token"), "{output}");
    assert!(output.contains("<truncated>"), "{output}");
}

#[test]
fn test_named_multi_values_exact_wrapper_node_budget_is_complete() {
    let value = NamedMultiValues::new(
        "tokens",
        MultiValues::String(vec!["first-secret".to_owned()]),
    );
    let policy = sensitive_policy_with_nodes("tokens", 5);

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("first-secret"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}
