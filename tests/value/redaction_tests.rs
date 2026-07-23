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
