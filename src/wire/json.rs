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
    DuplicateKeyRejectingJsonValue::deserialize(deserializer).map(DuplicateKeyRejectingJsonValue::into_inner)
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
#[must_use]
#[cfg(feature = "converter")]
pub(crate) fn canonicalize_json_value(value: &Value) -> Value {
    let mut pending = vec![(value, false)];
    let mut completed = Vec::new();
    while let Some((node, ready)) = pending.pop() {
        match node {
            Value::Array(values) if !ready => {
                pending.push((node, true));
                pending.extend(values.iter().rev().map(|child| (child, false)));
            }
            Value::Object(values) if !ready => {
                pending.push((node, true));
                pending.extend(values.values().rev().map(|child| (child, false)));
            }
            Value::Array(values) => {
                let children = completed.split_off(completed.len() - values.len());
                completed.push(Value::Array(children));
            }
            Value::Object(values) => {
                let children = completed.split_off(completed.len() - values.len());
                let mut entries: Vec<_> = values.keys().zip(children).collect();
                entries.sort_unstable_by_key(|(left, _)| *left);
                let mut object = Map::with_capacity(entries.len());
                for (key, child) in entries {
                    object.insert(key.clone(), child);
                }
                completed.push(Value::Object(object));
            }
            _ => completed.push(node.clone()),
        }
    }
    completed.pop().expect("one root is completed")
}
