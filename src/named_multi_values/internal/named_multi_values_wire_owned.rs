// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned wire representation for one named collection.

use serde::Deserialize;

use crate::ValueWireV1;

/// Owned wire representation of a named collection.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::named_multi_values) struct NamedMultiValuesWireOwned {
    /// Name associated with the collection.
    pub(in crate::named_multi_values) name: String,
    /// Independently versioned collection.
    pub(in crate::named_multi_values) value: ValueWireV1,
}
