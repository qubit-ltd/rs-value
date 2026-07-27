// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned wire representation for one named scalar value.

use serde::Deserialize;

use crate::ValueWireV1;

/// Owned wire representation of a named scalar value.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::named_value) struct NamedValueWireOwned {
    /// Name associated with the scalar value.
    pub(in crate::named_value) name: String,
    /// Independently versioned scalar value.
    pub(in crate::named_value) value: ValueWireV1,
}
