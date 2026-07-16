// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared equality and hashing for runtime value payloads.

use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

#[cfg(feature = "big-number")]
mod big_decimal_hash;
#[cfg(feature = "big-number")]
pub(crate) use big_decimal_hash::hash_big_decimal;

/// Returns canonical identity bits for an `f32` payload.
///
/// # Parameters
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[inline(always)]
pub(crate) fn canonical_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Returns canonical identity bits for an `f64` payload.
///
/// # Parameters
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[inline(always)]
pub(crate) fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Compares two JSON trees using structural JSON semantics.
///
/// # Parameters
///
/// * `left` - Left JSON tree.
/// * `right` - Right JSON tree.
///
/// # Returns
///
/// `true` when both trees are structurally equal. Object member order is not
/// significant and array element order is significant.
#[cfg(feature = "json")]
#[inline(always)]
pub(crate) fn json_eq(
    left: &serde_json::Value,
    right: &serde_json::Value,
) -> bool {
    left == right
}

/// Hashes a JSON tree using structural, object-order-independent semantics.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher.
#[cfg(feature = "json")]
pub(crate) fn hash_json<H: Hasher>(value: &serde_json::Value, state: &mut H) {
    match value {
        serde_json::Value::Null => 0_u8.hash(state),
        serde_json::Value::Bool(value) => {
            1_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Number(value) => {
            2_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::String(value) => {
            3_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Array(values) => {
            4_u8.hash(state);
            values.len().hash(state);
            for value in values {
                hash_json(value, state);
            }
        }
        serde_json::Value::Object(values) => {
            5_u8.hash(state);
            values.len().hash(state);
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            for key in keys {
                key.hash(state);
                hash_json(&values[key], state);
            }
        }
    }
}

/// Hashes a string map independently of its iteration order.
///
/// # Parameters
///
/// * `value` - Map to hash.
/// * `state` - Destination hasher.
pub(crate) fn hash_string_map<H: Hasher>(
    value: &HashMap<String, String>,
    state: &mut H,
) {
    value.len().hash(state);
    let mut entries: Vec<_> = value.iter().collect();
    entries.sort_unstable_by_key(|(left, _)| *left);
    for (key, value) in entries {
        key.hash(state);
        value.hash(state);
    }
}
