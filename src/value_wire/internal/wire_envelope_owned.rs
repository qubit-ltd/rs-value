// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned envelope used by V1 deserialization.

use serde::Deserialize;
use serde::Deserializer;

use super::WireShapeOwned;
use super::wire_envelope_owned_visitor::WireEnvelopeOwnedVisitor;

/// Owned V1 envelope accepted during deserialization.
pub(in crate::value_wire) struct WireEnvelopeOwned {
    /// Numeric protocol version supplied by the input.
    pub(in crate::value_wire) version: u8,
    /// Owned scalar-or-collection payload.
    pub(in crate::value_wire) value: WireShapeOwned,
}

impl<'de> Deserialize<'de> for WireEnvelopeOwned {
    /// Requests the named struct representation while preserving ordered
    /// sequences only for non-human-readable formats.
    ///
    /// The visitor requires one `version` and one `value` field in a map.
    /// Human-readable sequences are rejected; other formats may use exactly
    /// two sequence elements in version/value order. Errors decoding either
    /// field are propagated from the nested deserializer. Support for the
    /// decoded version is checked downstream.
    ///
    /// # Errors
    ///
    /// Returns the deserializer's error for a malformed envelope shape, an
    /// unknown, duplicate, or missing map field, or a field value that cannot
    /// be decoded.
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let allow_sequence = !deserializer.is_human_readable();
        deserializer.deserialize_struct(
            "WireEnvelopeRef",
            &["version", "value"],
            WireEnvelopeOwnedVisitor::new(allow_sequence),
        )
    }
}
