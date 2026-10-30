// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Iterator frames for non-recursive traversal of JSON containers.

use serde_json::Value;

/// Iterates over the children of one JSON container without collecting them.
pub(in crate::wire) enum JsonNumberChildren<'a> {
    Array(std::slice::Iter<'a, Value>),
    Object(serde_json::map::Iter<'a>),
}

impl<'a> JsonNumberChildren<'a> {
    /// Creates a child iterator when `value` is a JSON container.
    pub(in crate::wire) fn new(value: &'a Value) -> Option<Self> {
        match value {
            Value::Array(values) => Some(Self::Array(values.iter())),
            Value::Object(values) => Some(Self::Object(values.iter())),
            _ => None,
        }
    }

    /// Returns the next child value, if any.
    pub(in crate::wire) fn next(&mut self) -> Option<&'a Value> {
        match self {
            Self::Array(values) => values.next(),
            Self::Object(values) => values.next().map(|(_, value)| value),
        }
    }
}
