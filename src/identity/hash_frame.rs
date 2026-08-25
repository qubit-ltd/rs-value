// =============================================================================
//    Copyright (c) 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Pending operations for iterative JSON hashing.

use serde_json::Value;
use serde_json::map::Iter;

/// One pending operation in the iterative JSON hashing traversal.
pub(super) enum HashFrame<'a> {
    /// Visits one JSON node at its root-inclusive depth.
    Visit(
        /// JSON value whose structural tag and contents are hashed next.
        &'a Value,
        /// Root-inclusive depth retained for child continuations.
        usize,
    ),
    /// Hashes an array length before its elements are visited.
    HashArrayLength(
        /// Element count incorporated before any array element.
        usize,
    ),
    /// Continues visiting an array from its next element.
    VisitArray {
        /// Array elements being visited.
        values: &'a [Value],
        /// Root-inclusive depth of each array element.
        depth: usize,
        /// Index of the next element to visit.
        next: usize,
    },
    /// Continues visiting an object from its next entry.
    VisitObject {
        /// Object entries being visited.
        entries: Iter<'a>,
        /// Root-inclusive depth of each object value.
        depth: usize,
    },
    /// Starts an object entry with its independent identity hasher.
    StartObjectEntry(
        /// Object key that seeds the entry's independent hasher.
        &'a str,
    ),
    /// Finishes an object entry and adds its hash to the current object.
    FinishObjectEntry,
    /// Finishes an object and writes its order-independent aggregates.
    FinishObject,
}
