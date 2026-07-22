// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests strict JSON map-key serialization through the public API.

use std::collections::BTreeMap;

use qubit_value::Value;

/// Verifies map keys serialize to JSON object keys.
#[test]
fn test_map_key_serializer_writes_string_keys() {
    let value = Value::Json(
        serde_json::to_value(BTreeMap::from([("key", 7)])).unwrap(),
    );
    assert_eq!(
        value.to_json_value().unwrap(),
        serde_json::json!({"key": 7})
    );
}
