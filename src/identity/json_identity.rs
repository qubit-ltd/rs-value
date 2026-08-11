// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Structural equality and hashing for JSON payloads.

use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;

type IdentityHasher =
    BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

/// Compares two JSON trees using structural JSON semantics without recursion.
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
pub(crate) fn json_eq(
    left: &serde_json::Value,
    right: &serde_json::Value,
) -> bool {
    let mut pending = Vec::with_capacity(1);
    pending.push((left, right));
    while let Some((left, right)) = pending.pop() {
        match (left, right) {
            (serde_json::Value::Null, serde_json::Value::Null) => {}
            (serde_json::Value::Bool(left), serde_json::Value::Bool(right)) => {
                if left != right {
                    return false;
                }
            }
            (serde_json::Value::Number(left), serde_json::Value::Number(right)) => {
                if left != right {
                    return false;
                }
            }
            (serde_json::Value::String(left), serde_json::Value::String(right)) => {
                if left != right {
                    return false;
                }
            }
            (serde_json::Value::Array(left), serde_json::Value::Array(right)) => {
                if left.len() != right.len() {
                    return false;
                }
                for (left, right) in left.iter().rev().zip(right.iter().rev()) {
                    pending.push((left, right));
                }
            }
            (serde_json::Value::Object(left), serde_json::Value::Object(right)) => {
                if left.len() != right.len() {
                    return false;
                }
                for (key, left) in left {
                    let Some(right) = right.get(key) else {
                        return false;
                    };
                    pending.push((left, right));
                }
            }
            _ => return false,
        }
    }
    true
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
            let mut sum = 0_u64;
            let mut xor = 0_u64;
            for (key, value) in values {
                let mut entry = IdentityHasher::default().build_hasher();
                key.hash(&mut entry);
                hash_json(value, &mut entry);
                let hash = entry.finish();
                sum = sum.wrapping_add(hash);
                xor ^= hash.rotate_left(17);
            }
            sum.hash(state);
            xor.hash(state);
        }
    }
}
