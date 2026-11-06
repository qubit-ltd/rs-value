// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
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
/// value is delivered. Completed array children already belong to final output.
pub(in crate::wire) enum CanonicalJsonFrame<'a> {
    Array {
        remaining: ArrayIter<'a, Value>,
        output: Vec<Value>,
    },
    Object {
        remaining: ObjectIter<'a>,
        pending_key: Option<&'a str>,
        completed: Vec<(&'a str, Value)>,
    },
}

impl<'a> CanonicalJsonFrame<'a> {
    /// Creates a frame for a nonempty source container, preallocating its final
    /// destination. Empty containers are returned directly by the caller.
    pub(in crate::wire) fn new(value: &'a Value) -> Option<Self> {
        match value {
            Value::Array(values) if !values.is_empty() => Some(Self::Array {
                remaining: values.iter(),
                output: Vec::with_capacity(values.len()),
            }),
            Value::Object(values) if !values.is_empty() => Some(Self::Object {
                remaining: values.iter(),
                pending_key: None,
                completed: Vec::with_capacity(values.len()),
            }),
            _ => None,
        }
    }

    /// Yields exactly one next child and, for objects, remembers its key.
    pub(in crate::wire) fn next_child(&mut self) -> Option<&'a Value> {
        match self {
            Self::Array { remaining, .. } => remaining.next(),
            Self::Object {
                remaining,
                pending_key,
                ..
            } => {
                debug_assert!(pending_key.is_none());
                let (key, value) = remaining.next()?;
                *pending_key = Some(key);
                Some(value)
            }
        }
    }

    /// Transfers one completed child directly into this frame's destination.
    pub(in crate::wire) fn push(&mut self, child: Value) {
        match self {
            Self::Array { output, .. } => output.push(child),
            Self::Object {
                pending_key,
                completed,
                ..
            } => {
                let key = pending_key.take().expect("object child has a pending key");
                completed.push((key, child));
            }
        }
    }

    /// Builds the completed owned container. Object keys remain sorted exactly
    /// as before, and are cloned only when entering the final map.
    pub(in crate::wire) fn finish(self) -> Value {
        match self {
            Self::Array { output, .. } => Value::Array(output),
            Self::Object { mut completed, .. } => {
                completed.sort_unstable_by(|(left, _), (right, _)| left.cmp(right));
                let mut output = Map::with_capacity(completed.len());
                for (key, child) in completed {
                    output.insert(key.to_owned(), child);
                }
                Value::Object(output)
            }
        }
    }
}
