// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Order-independent aggregates for JSON object hashing.

/// Hash aggregates for one object currently being visited.
#[derive(Default)]
pub(super) struct ObjectHash {
    /// Wrapping sum of the object's entry hashes.
    pub(super) sum: u64,
    /// Rotated xor of the object's entry hashes.
    pub(super) xor: u64,
}
