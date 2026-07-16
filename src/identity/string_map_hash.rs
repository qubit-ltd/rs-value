// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Iteration-order-independent hashing for string maps.

use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

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
