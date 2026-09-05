// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed continuation frames for projection admission.

use serde_json::Value;
use serde_json::map::Iter as MapIter;

/// Iterates one container without retaining all sibling nodes on the stack.
pub(super) enum JsonChildren<'a> {
    /// Remaining array elements and their root-inclusive child depth.
    Array(
        /// Remaining borrowed array entries.
        std::slice::Iter<'a, Value>,
        /// Root-inclusive depth of those entries.
        usize,
    ),
    /// Remaining object entries and their root-inclusive child depth.
    Object(
        /// Remaining borrowed object entries.
        MapIter<'a>,
        /// Root-inclusive depth of those entries.
        usize,
    ),
}

impl<'a> JsonChildren<'a> {
    /// Returns the next optional object key, value, and child depth.
    pub(super) fn next(&mut self) -> Option<(Option<&'a str>, &'a Value, usize)> {
        match self {
            Self::Array(values, depth) => values.next().map(|value| (None, value, *depth)),
            Self::Object(values, depth) => values.next().map(|(key, value)| (Some(key.as_str()), value, *depth)),
        }
    }
}
