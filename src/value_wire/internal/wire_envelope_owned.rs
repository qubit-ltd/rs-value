// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned envelope used by V1 deserialization.

use serde::Deserialize;

use super::WireShapeOwned;

/// Owned V1 envelope accepted during deserialization.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(in crate::value_wire) struct WireEnvelopeOwned {
    /// Numeric protocol version supplied by the input.
    pub(in crate::value_wire) version: u8,
    /// Owned scalar-or-collection payload.
    pub(in crate::value_wire) value: WireShapeOwned,
}
