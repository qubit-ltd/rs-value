// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed wire representation for one named collection.

use serde::Serialize;

use crate::ValueWireRefV1;

/// Borrows a named collection's data for serialization without cloning it.
///
/// The serialized value uses the V1 wire envelope, and both fields remain
/// valid only for the lifetime of the source data borrowed by `'a`.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed name and collection payload.
#[derive(Serialize)]
pub(in crate::named_multi_values) struct NamedMultiValuesWireRef<'a> {
    /// Name to serialize, borrowed from the source named collection.
    pub(in crate::named_multi_values) name: &'a str,
    /// Collection payload to serialize inside its independently versioned V1
    /// envelope.
    pub(in crate::named_multi_values) value: ValueWireRefV1<'a>,
}
