// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashMap;

use qubit_redact::{
    Redact as _,
    RedactionPolicy,
    Sensitivity,
};
use qubit_value::Value;

#[test]
fn test_rs_metadata_feature_profile_masks_sensitive_map_values() {
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
fn test_rs_metadata_feature_profile_keeps_converter_api_available() {
    assert_eq!(
        Value::from("42")
            .to::<i32>()
            .expect("convert metadata value"),
        42
    );
}
