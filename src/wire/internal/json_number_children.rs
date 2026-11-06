// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Iterator frames for non-recursive traversal of JSON containers.

use std::slice::Iter as ArrayIter;

use serde_json::Value;
use serde_json::map::Iter as ObjectIter;

/// Iterates over the children of one JSON container without collecting them.
///
/// Child values remain borrowed from the container for its lifetime `'a`.
pub(in crate::wire) enum JsonNumberChildren<'a> {
    /// Visits array elements in their original order.
    Array(
        /// The remaining elements borrowed from the source array.
        ArrayIter<'a, Value>,
    ),
    /// Visits object values in the source map's iteration order, omitting keys.
    Object(
        /// The remaining entries borrowed from the source object.
        ObjectIter<'a>,
    ),
}

impl<'a> JsonNumberChildren<'a> {
    /// Creates a child iterator when `value` is a JSON container.
    ///
    /// # Parameters
    ///
    /// * `value` - Source value borrowed for the lifetime of the child
    ///   iterator.
    ///
    /// # Returns
    ///
    /// `Some` containing an iterator for an array or object, including empty
    /// containers; `None` for scalar values.
    #[must_use]
    #[inline]
    pub(in crate::wire) fn new(value: &'a Value) -> Option<Self> {
        match value {
            Value::Array(values) => Some(Self::Array(values.iter())),
            Value::Object(values) => Some(Self::Object(values.iter())),
            _ => None,
        }
    }

    /// Returns the next child value, if any.
    ///
    /// Advances the iterator without allocating or modifying the source value.
    ///
    /// # Returns
    ///
    /// `Some` containing the next child borrowed for `'a`, or `None` when all
    /// children have been visited.
    #[inline]
    pub(in crate::wire) fn next(&mut self) -> Option<&'a Value> {
        match self {
            Self::Array(values) => values.next(),
            Self::Object(values) => values.next().map(|(_, value)| value),
        }
    }
}
