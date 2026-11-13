// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed continuation frames for projection admission.

use std::slice::Iter as SliceIter;

use serde_json::Value;
use serde_json::map::Iter as MapIter;

/// Iterates one container without retaining all sibling nodes on the stack.
///
/// Entries borrow the source container for `'a`; advancing the frame neither
/// clones values nor allocates storage for siblings.
///
/// # Type Parameters
///
/// - `'a`: Lifetime of the source container and its borrowed entries.
pub(in crate::json) enum JsonChildren<'a> {
    /// Remaining array elements and their root-inclusive child depth.
    Array(
        /// Remaining borrowed array entries.
        SliceIter<'a, Value>,
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
    /// Advances this frame by one entry in the source container's iteration
    /// order.
    ///
    /// The returned key and value borrow the source for `'a`, independently of
    /// the mutable borrow of this frame. The stored child depth is unchanged.
    ///
    /// # Returns
    ///
    /// `Some((key, value, depth))` for the next child, where `key` is `None`
    /// for an array element and `Some(name)` for an object entry. Returns
    /// `None` once the container is exhausted.
    #[inline]
    pub(in crate::json) fn next(&mut self) -> Option<(Option<&'a str>, &'a Value, usize)> {
        match self {
            Self::Array(values, depth) => values.next().map(|value| (None, value, *depth)),
            Self::Object(values, depth) => values.next().map(|(key, value)| (Some(key.as_str()), value, *depth)),
        }
    }
}
