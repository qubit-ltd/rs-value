// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Continuation frames for iterative canonical JSON materialization.

use std::slice::Iter as ArrayIter;

use serde_json::Map;
use serde_json::Value;
use serde_json::map::Iter as ObjectIter;

/// Owns the destination container while borrowing only its unvisited source.
///
/// Invariant: an object has at most one `pending_key`; it is set exactly when
/// an object child has been yielded and is cleared when that child's owned
/// value is delivered. Completed children already belong to final output.
pub(in crate::wire) enum CanonicalJsonFrame<'a> {
    /// Holds the remaining array source and its final owned destination.
    Array {
        /// The source elements not yet visited by the projection loop.
        remaining: ArrayIter<'a, Value>,
        /// Completed children accumulated directly into the returned array.
        output: Vec<Value>,
    },
    /// Holds remaining object entries and the final output map.
    Object {
        /// The object entries not yet visited by the projection loop.
        remaining: ObjectIter<'a>,
        /// The key associated with the one child currently being projected.
        pending_key: Option<&'a str>,
        /// Completed children keyed directly in the final output map.
        output: Map<String, Value>,
    },
}

impl<'a> CanonicalJsonFrame<'a> {
    /// Creates a frame for a nonempty source container, preallocating its final
    /// destination. Empty containers are returned directly by the caller.
    ///
    /// # Parameters
    ///
    /// - `value`: borrowed JSON node whose nonempty container should be framed.
    ///
    /// # Returns
    ///
    /// `Some` containing the frame for a nonempty array or object, or `None`
    /// for a scalar or empty container.
    pub(in crate::wire) fn new(value: &'a Value) -> Option<Self> {
        match value {
            Value::Array(values) if !values.is_empty() => Some(Self::Array {
                remaining: values.iter(),
                output: Vec::with_capacity(values.len()),
            }),
            Value::Object(values) if !values.is_empty() => Some(Self::Object {
                remaining: values.iter(),
                pending_key: None,
                output: Map::with_capacity(values.len()),
            }),
            _ => None,
        }
    }

    /// Yields exactly one next child and, for objects, remembers its key.
    ///
    /// # Returns
    ///
    /// The next borrowed array element or object value, or `None` when the
    /// source container has no remaining entries.
    pub(in crate::wire) fn next_child(&mut self) -> Option<&'a Value> {
        match self {
            Self::Array { remaining, .. } => remaining.next(),
            Self::Object {
                remaining, pending_key, ..
            } => {
                debug_assert!(pending_key.is_none());
                let (key, value) = remaining.next()?;
                *pending_key = Some(key);
                Some(value)
            }
        }
    }

    /// Transfers one completed child directly into this frame's destination.
    ///
    /// # Parameters
    ///
    /// - `child`: independently owned projection of the child most recently
    ///   yielded by [`Self::next_child`].
    pub(in crate::wire) fn push(&mut self, child: Value) {
        match self {
            Self::Array { output, .. } => output.push(child),
            Self::Object {
                pending_key, output, ..
            } => {
                let key = pending_key.take().expect("object child has a pending key");
                output.insert(key.to_owned(), child);
            }
        }
    }

    /// Builds the completed owned container, sorting each object map at its
    /// own frame boundary. Each source key is cloned once into the final map.
    ///
    /// # Returns
    ///
    /// The completed array or object with all projected children.
    pub(in crate::wire) fn finish(self) -> Value {
        match self {
            Self::Array { output, .. } => Value::Array(output),
            Self::Object { mut output, .. } => {
                output.sort_keys();
                Value::Object(output)
            }
        }
    }
}
