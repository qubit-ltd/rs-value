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

/// Borrowed wire representation of a named collection.
///
/// # Type Parameters
///
/// * `'a` - Lifetime of the borrowed name and collection payload.
#[derive(Serialize)]
pub(in crate::named_multi_values) struct NamedMultiValuesWireRef<'a> {
    /// Name associated with the collection.
    pub(in crate::named_multi_values) name: &'a str,
    /// Independently versioned collection.
    pub(in crate::named_multi_values) value: ValueWireRefV1<'a>,
}
