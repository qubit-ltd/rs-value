// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed runtime source retained for V1 payload preparation.

use crate::MultiValues;
use crate::Value;

/// Borrowed scalar-or-collection source used to build a V1 wire shape on
/// demand.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed runtime value or collection.
#[derive(Clone, Copy)]
pub(in crate::value_wire) enum WireSourceRef<'a> {
    /// One runtime scalar value.
    Scalar(
        /// Runtime scalar borrowed for `'a` without cloning or taking
        /// ownership.
        &'a Value,
    ),
    /// One homogeneous runtime collection.
    Collection(
        /// Homogeneous collection borrowed for `'a` without copying its
        /// elements.
        &'a MultiValues,
    ),
}
