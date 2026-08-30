// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests recursive JSON redaction through the public `Value` API.

use qubit_redact::MaskPolicy;
use qubit_redact::RedactionPolicy;
use qubit_redact::Redactor;
use qubit_redact::Sensitivity;
use qubit_value::Value;

/// Verifies sensitive JSON containers are masked without exposing their data.
#[test]
fn test_redacted_json_masks_sensitive_non_string_values() {
    let value = Value::Json(serde_json::json!({
        "secret_number": 42,
        "secret_object": {"label": "nested-object-secret"},
        "secret_array": ["array-secret"],
        "secret_null": null,
        "visible": "public"
    }));
    let policy = RedactionPolicy::builder()
        .fields(|fields| {
            fields
                .raise("secret_number", Sensitivity::Low)
                .raise("secret_object", Sensitivity::Low)
                .raise("secret_array", Sensitivity::Low)
                .raise("secret_null", Sensitivity::Low)
                .mask(Sensitivity::Low, MaskPolicy::preserve_edges(1, 1, "OPAQUE", 0));
        })
        .expect("the test policy should be valid")
        .build()
        .expect("policy should build");

    let output = Redactor::new(policy)
        .redact(&value)
        .into_complete_text()
        .expect("test output must be complete")
        .into_string();

    assert!(!output.contains("42"));
    assert!(!output.contains("nested-object-secret"));
    assert!(!output.contains("array-secret"));
    assert_eq!(output.matches("OPAQUE").count(), 4);
    assert!(output.contains("public"));
}
