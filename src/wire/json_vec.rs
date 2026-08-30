// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

// qubit-style: allow source-test-pair
//! Canonical Serde adapter for collections of JSON values.

use qubit_json::value::DuplicateKeyRejectingJsonValue;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serializer;
use serde::ser::SerializeSeq;
use serde_json::Value;

use super::internal::CanonicalJson;

/// Serializes JSON values in a collection with recursively ordered keys.
///
/// # Type Parameters
///
/// * `S` - Destination serializer type.
///
/// # Parameters
///
/// * `values` - JSON trees to serialize in order.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's sequence result.
///
/// # Errors
///
/// Returns `S::Error` when sequence serialization fails.
pub(crate) fn serialize<S>(values: &[Value], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut sequence = serializer.serialize_seq(Some(values.len()))?;
    for value in values {
        sequence.serialize_element(&CanonicalJson(value))?;
    }
    sequence.end()
}

/// Deserializes a collection of JSON values.
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
/// Decoded JSON trees in source order.
///
/// # Errors
///
/// Returns `D::Error` when decoding fails or any object repeats a key.
pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Value>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<DuplicateKeyRejectingJsonValue>::deserialize(deserializer).map(|values| {
        values
            .into_iter()
            .map(DuplicateKeyRejectingJsonValue::into_inner)
            .collect()
    })
}
