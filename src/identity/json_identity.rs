// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structural equality and hashing for JSON payloads.

use std::hash::{Hash, Hasher};

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
#[must_use]
#[inline(always)]
pub(crate) fn json_eq(left: &serde_json::Value, right: &serde_json::Value) -> bool {
    left == right
}

/// Hashes a JSON tree using structural, object-order-independent semantics.
///
/// # Parameters
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher.
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
