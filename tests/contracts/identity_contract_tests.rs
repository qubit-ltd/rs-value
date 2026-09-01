// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for identity and hashing contracts.

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::Hash;
use std::hash::Hasher;

use qubit_value::Value;

fn hash(value: &Value) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

#[test]
fn float_identity_normalizes_signed_zero_and_nan_payloads() {
    assert_eq!(Value::Float64(-0.0), Value::Float64(0.0));
    assert_eq!(hash(&Value::Float64(-0.0)), hash(&Value::Float64(0.0)));

    let left = Value::Float64(f64::from_bits(0x7ff8_0000_0000_0001));
    let right = Value::Float64(f64::from_bits(0x7fff_ffff_ffff_ffff));
    assert_eq!(left, right);
    assert_eq!(hash(&left), hash(&right));
}

#[cfg(feature = "json")]
#[test]
fn map_and_json_object_identity_ignores_insertion_order() {
    let left_map = Value::StringMap(HashMap::from([
        ("a".to_owned(), "1".to_owned()),
        ("b".to_owned(), "2".to_owned()),
    ]));
    let right_map = Value::StringMap(HashMap::from([
        ("b".to_owned(), "2".to_owned()),
        ("a".to_owned(), "1".to_owned()),
    ]));
    assert_eq!(left_map, right_map);
    assert_eq!(hash(&left_map), hash(&right_map));

    let left_json = Value::Json(serde_json::json!({"b": {"x": 1}, "a": 0}));
    let right_json = Value::Json(serde_json::json!({"a": 0, "b": {"x": 1}}));
    assert_eq!(left_json, right_json);
    assert_eq!(hash(&left_json), hash(&right_json));
}
