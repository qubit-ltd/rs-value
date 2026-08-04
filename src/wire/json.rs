// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Canonical Serde adapter for JSON values.

use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};
use serde_json::Value;

use super::internal::{
    CanonicalJson,
    StrictJsonValue,
};

/// Serializes one JSON value with recursively ordered object keys.
pub(crate) fn serialize<S>(
    value: &Value,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    CanonicalJson(value).serialize(serializer)
}

/// Deserializes one JSON value from any Serde representation.
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
where
    D: Deserializer<'de>,
{
    StrictJsonValue::deserialize(deserializer).map(StrictJsonValue::into_inner)
}

/// Returns a recursively canonical JSON value for natural projections.
#[cfg(feature = "converter")]
pub(crate) fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {
            value.clone()
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(canonicalize_json_value).collect())
        }
        Value::Object(values) => {
            let mut entries: Vec<_> = values.iter().collect();
            entries.sort_unstable_by_key(|(left, _)| *left);
            let mut object = serde_json::Map::with_capacity(entries.len());
            for (key, value) in entries {
                object.insert(key.clone(), canonicalize_json_value(value));
            }
            Value::Object(object)
        }
    }
}
