// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed traversal state for conservative JSON measurements.

use std::slice::Iter;

use serde_json::Value as JsonValue;
use serde_json::map::Iter as JsonObjectIter;

/// Pending JSON work; iterators retain only the remaining children of a
/// container.
///
/// Every reference borrows the checked tree for the traversal lifetime.
pub(in crate::value_wire::value_wire_encode_preflight) enum Frame<'a> {
    /// Measures one value at its supplied nesting depth.
    Visit(
        /// Borrowed value to measure.
        &'a JsonValue,
        /// One-based depth of this value.
        usize,
    ),
    /// Resumes an array after previously visited children.
    Array(
        /// Remaining borrowed array elements.
        Iter<'a, JsonValue>,
        /// One-based depth shared by these elements.
        usize,
    ),
    /// Resumes an object after previously visited members.
    Object(
        /// Remaining borrowed key-value pairs.
        JsonObjectIter<'a>,
        /// One-based depth of the member values.
        usize,
    ),
}
