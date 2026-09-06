// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde adapter for JSON values.

use qubit_json::value::DuplicateKeyRejectingJsonValue;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
#[cfg(feature = "converter")]
use serde_json::Map;
use serde_json::Value;

use super::internal::CanonicalJson;

/// Serializes one JSON value with recursively ordered object keys.
///
/// # Type Parameters
///
/// * `S` - Destination serializer type.
///
/// # Parameters
///
/// * `value` - JSON tree to serialize canonically.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's result.
///
/// # Errors
///
/// Returns `S::Error` when serialization fails.
pub(crate) fn serialize<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    CanonicalJson(value).serialize(serializer)
}

/// Deserializes one JSON value from any Serde representation.
///
/// # Type Parameters
///
/// * `D` - Source deserializer type.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// The decoded JSON tree after duplicate-key validation.
///
/// # Errors
///
/// Returns `D::Error` when decoding fails or an object repeats a key.
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    DuplicateKeyRejectingJsonValue::deserialize(deserializer)
        .map(DuplicateKeyRejectingJsonValue::into_inner)
}

/// Returns a recursively canonical JSON value for natural projections.
///
/// # Parameters
///
/// * `value` - JSON tree whose object maps are recursively reordered.
///
/// # Returns
///
/// An owned JSON tree with lexicographically ordered object keys.
#[cfg(feature = "converter")]
pub(crate) fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => value.clone(),
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_json_value).collect()),
        Value::Object(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_unstable_by_key(|(left, _)| *left);
            let mut object = Map::with_capacity(entries.len());
            for (key, value) in entries {
                object.insert(key.clone(), canonicalize_json_value(value));
            }
            Value::Object(object)
        }
    }
}
