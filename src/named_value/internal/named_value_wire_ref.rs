// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed wire representation for one named scalar value.

use serde::Serialize;

use crate::ValueWireRefV1;

/// Borrowed wire representation of a named scalar value.
#[derive(Serialize)]
pub(in crate::named_value) struct NamedValueWireRef<'a> {
    /// Name associated with the scalar value.
    pub(in crate::named_value) name: &'a str,
    /// Independently versioned scalar value.
    pub(in crate::named_value) value: ValueWireRefV1<'a>,
}
