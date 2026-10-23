// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Continuation frames for depth-bounded JSON equality traversal.

use serde_json::Map;
use serde_json::Value;
use serde_json::map::Iter as MapIter;

/// Retains only the active iterator for each enclosing container.
pub(super) enum JsonEqualityFrame<'a> {
    /// Remaining paired elements of equal-length arrays.
    Array {
        /// Left array elements not yet compared.
        left: std::slice::Iter<'a, Value>,
        /// Right array elements not yet compared.
        right: std::slice::Iter<'a, Value>,
    },
    /// Remaining left entries and the object used for matching-key lookup.
    Object {
        /// Entries not yet compared.
        left: MapIter<'a>,
        /// Right object with the same entry count.
        right: &'a Map<String, Value>,
    },
}
