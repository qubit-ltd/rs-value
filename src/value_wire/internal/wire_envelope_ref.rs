// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Borrowed envelope used by V1 serialization.

use serde::Serialize;

use super::WireShapeRef;

/// Borrowed V1 envelope emitted during serialization.
#[derive(Serialize)]
pub(in crate::value_wire) struct WireEnvelopeRef<'a> {
    /// Numeric V1 protocol version.
    pub(in crate::value_wire) version: u8,
    /// Borrowed scalar-or-collection payload.
    pub(in crate::value_wire) value: WireShapeRef<'a>,
}
