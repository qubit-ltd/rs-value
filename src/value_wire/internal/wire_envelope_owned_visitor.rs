// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for the version and value fields of an owned wire envelope.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::WireEnvelopeOwned;
/// Reads an envelope with exactly one `version` and one `value` field.
///
/// This visitor validates the envelope's structure and field types. Ordered
/// sequence input is accepted only when the format is non-human-readable.
/// Whether the decoded version is supported is checked by a later part of the
/// V1 deserialization path.
pub(super) struct WireEnvelopeOwnedVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl WireEnvelopeOwnedVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for WireEnvelopeOwnedVisitor {
    type Value = WireEnvelopeOwned;

    /// Describes the envelope shape expected by this visitor.
    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("wire envelope record with version and value fields")
    }

    /// Reads exactly two permitted sequence elements, in `version`, then
    /// `value` order.
    ///
    /// Returns a deserializer error for a missing or extra element, or when a
    /// field value cannot be decoded.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        // Struct sequences preserve the declared version, value field order
        // and require exactly two elements.
        let version = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let value = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(WireEnvelopeOwned { version, value })
    }

    /// Reads the `version` and `value` map fields, in either order.
    ///
    /// Returns a deserializer error for an unknown, duplicate, or missing
    /// field, or when a field value cannot be decoded.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Map representations require both recognized fields exactly once.
        let mut version = None;
        let mut value = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "version" => {
                    if version.is_some() {
                        return Err(Error::duplicate_field("version"));
                    }
                    version = Some(map.next_value()?);
                }
                "value" => {
                    if value.is_some() {
                        return Err(Error::duplicate_field("value"));
                    }
                    value = Some(map.next_value()?);
                }
                _ => return Err(Error::unknown_field(&key, &["version", "value"])),
            }
        }
        Ok(WireEnvelopeOwned {
            version: version.ok_or_else(|| Error::missing_field("version"))?,
            value: value.ok_or_else(|| Error::missing_field("value"))?,
        })
    }
}
