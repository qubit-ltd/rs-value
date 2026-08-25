// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! External tests for policy-aware [`qubit_value::Value`] redaction.

use std::collections::HashMap;

use qubit_redact::MaskPolicy;
use qubit_redact::Redact;
use qubit_redact::RedactionCompletion;
use qubit_redact::RedactionPolicy;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use serde_json::json;

/// Renders a domain value through one explicit policy snapshot.
fn redacted_text<T: Redact>(value: &T, policy: &RedactionPolicy) -> String {
    Redactor::new(policy.clone())
        .redact(value)
        .into_complete_text()
        .expect("test output must be complete")
        .into_string()
}

/// Builds a policy that classifies one field and limits domain nodes.
fn sensitive_policy_with_nodes(
    field: &str,
    max_nodes: usize,
) -> RedactionPolicy {
    RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise(field, Sensitivity::Secret);
        })
        .expect("the test field rule should be valid")
        .limits(|limits_builder| {
            limits_builder
                .max_nodes(max_nodes)
                .max_collection_items(1024)
                .max_depth(32);
        })
        .expect("the domain limits should be valid")
        .build()
        .expect("policy should build")
}

#[test]
fn test_value_redacted_view_masks_sensitive_string_map_entries() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("api_key", Sensitivity::Secret);
        })
        .expect("the test builder input should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[test]
fn test_multi_values_json_recursively_masks_sensitive_entries() {
    let values = MultiValues::Json(vec![json!({
        "profile": {
            "password": "raw-json-secret",
            "attempts": 3,
        },
        "label": "visible",
    })]);
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("password", Sensitivity::Secret);
        })
        .expect("the test field rule should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&values, &policy);

    assert!(!output.contains("raw-json-secret"), "{output}");
    assert!(output.contains("visible"), "{output}");
}

#[test]
fn test_multi_values_json_masks_sensitive_non_string_entries() {
    let values = MultiValues::Json(vec![
        json!({"password": 12345}),
        json!({"password": true, "label": "visible"}),
    ]);
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("password", Sensitivity::Secret);
        })
        .expect("the test field rule should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&values, &policy);

    assert!(!output.contains("12345"), "{output}");
    assert!(!output.contains("true"), "{output}");
    assert!(output.contains("visible"), "{output}");
}

#[test]
fn test_multi_values_empty_json_collection_is_preserved() {
    let values = MultiValues::Json(Vec::new());

    let output = redacted_text(&values, &RedactionPolicy::standard());

    assert_eq!(output, "[]");
}

#[test]
fn test_multi_values_json_stops_before_unadmitted_items() {
    let values = MultiValues::Json(vec![json!("first"), json!("second")]);
    let policy = RedactionPolicy::builder()
        .limits(|limits| {
            limits.max_nodes(64).max_collection_items(1).max_depth(8);
        })
        .expect("the test limits should be valid")
        .build()
        .expect("policy should build");

    let result = Redactor::new(policy).redact(&values);
    let output = result.text().as_str();

    assert_eq!(
        result.summary().completion(),
        RedactionCompletion::Truncated
    );
    assert!(output.contains("first"), "{output}");
    assert!(!output.contains("second"), "{output}");
    assert!(output.contains("<truncated>"), "{output}");
}

#[test]
fn test_value_redacted_view_preserves_scalar_without_key_context() {
    let value = Value::String("visible-without-key".to_owned());
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("password", Sensitivity::Secret);
        })
        .expect("the test builder input should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&value, &policy);

    assert!(output.contains("visible-without-key"));
}

#[test]
fn test_named_value_masks_non_strings_with_configured_opaque_value() {
    let value = NamedValue::new("token", Value::Int32(12345));
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("token", Sensitivity::Low).mask(
                Sensitivity::Low,
                MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
            );
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");
    assert!(redacted_text(&value, &policy).contains("OPAQUE"));
}

#[test]
fn test_named_value_redaction_uses_text_masking_for_sensitive_strings() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields.raise("token", Sensitivity::Low).mask(
                Sensitivity::Low,
                MaskPolicy::preserve_edges(1, 1, "MASK", 0),
            );
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("secret-token"));
    assert!(!output.contains("secret-token"));
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
        .fields(|fields| {
            fields.raise("tokens", Sensitivity::Low).mask(
                Sensitivity::Low,
                MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0),
            );
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("first-secret"));
    assert!(!output.contains("second-secret"));
    assert!(output.contains("OPAQUE"));
}

#[test]
fn test_named_value_exact_wrapper_node_budget_is_complete() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = sensitive_policy_with_nodes("token", 5);

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("secret-token"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}

#[test]
fn test_named_value_one_less_wrapper_node_truncates() {
    let value =
        NamedValue::new("token", Value::String("secret-token".to_owned()));
    let policy = sensitive_policy_with_nodes("token", 4);

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("secret-token"), "{output}");
    assert!(!output.contains("secret-token"), "{output}");
}

#[test]
fn test_named_multi_values_exact_wrapper_node_budget_is_complete() {
    let value = NamedMultiValues::new(
        "tokens",
        MultiValues::String(vec!["first-secret".to_owned()]),
    );
    let policy = sensitive_policy_with_nodes("tokens", 5);

    let output = redacted_text(&value, &policy);

    assert!(!output.contains("first-secret"), "{output}");
    assert!(!output.contains("<truncated>"), "{output}");
}
