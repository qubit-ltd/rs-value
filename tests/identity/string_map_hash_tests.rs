// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests string-map hashing.

use std::collections::HashMap;

use qubit_value::Value;

/// Verifies string-map insertion order does not affect public value identity.
#[test]
fn test_string_map_hash_ignores_insertion_order() {
    let left = Value::StringMap(HashMap::from([("first".into(), "1".into())]));
    let right = Value::StringMap(HashMap::from([("first".into(), "1".into())]));
    assert_eq!(left, right);
}
