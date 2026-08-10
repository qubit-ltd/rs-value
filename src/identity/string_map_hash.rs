// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Iteration-order-independent hashing for string maps.

use std::collections::HashMap;
use std::hash::BuildHasher;
use std::hash::BuildHasherDefault;
use std::hash::Hash;
use std::hash::Hasher;

type IdentityHasher =
    BuildHasherDefault<std::collections::hash_map::DefaultHasher>;

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
    let mut sum = 0_u64;
    let mut xor = 0_u64;
    for (key, value) in value {
        let mut entry = IdentityHasher::default().build_hasher();
        key.hash(&mut entry);
        value.hash(&mut entry);
        let hash = entry.finish();
        sum = sum.wrapping_add(hash);
        xor ^= hash.rotate_left(17);
    }
    sum.hash(state);
    xor.hash(state);
}
