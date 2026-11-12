// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for the seconds and nanoseconds fields of a duration payload.

use serde::de::Error;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::DurationPayload;
use super::duration_payload_field::DurationPayloadField;

/// Reads a duration payload with exactly one `secs` and one `nanos` field in
/// maps, or those two components in order in permitted sequences.
///
/// The nanosecond value is decoded here but its sub-billion range is checked
/// when the payload is converted into [`std::time::Duration`].
pub(super) struct DurationPayloadVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl DurationPayloadVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    ///
    /// # Parameters
    ///
    /// - `allow_sequence`: Whether the format permits ordered struct sequences.
    ///
    /// # Returns
    ///
    /// A visitor that enforces the selected sequence policy.
    #[must_use]
    #[inline]
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for DurationPayloadVisitor {
    /// The decoded components, before validation of the nanosecond range.
    type Value = DurationPayload;

    /// Describes the payload shape expected by this visitor.
    ///
    /// The deserializer uses this description when reporting a shape mismatch.
    ///
    /// # Parameters
    ///
    /// - `f`: The formatter receiving the expected payload description.
    ///
    /// # Returns
    ///
    /// `Ok(())` after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing the description fails.
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("duration payload record with secs and nanos fields")
    }

    /// Reads exactly two sequence elements, in `secs`, then `nanos` order.
    ///
    /// # Type Parameters
    ///
    /// - `A`: The deserializer's sequence access implementation.
    ///
    /// # Parameters
    ///
    /// - `seq`: The sequence access consumed to decode the two components.
    ///
    /// # Returns
    ///
    /// The decoded components, before validation of the nanosecond range.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error if sequences are forbidden, an element is
    /// missing or extra, or a value cannot be decoded as its field type.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        // The tuple representation is ordered as secs, then nanos, and has
        // exactly two elements.
        let secs = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let nanos = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(DurationPayload { secs, nanos })
    }

    /// Reads the `secs` and `nanos` map fields, in either order.
    ///
    /// # Type Parameters
    ///
    /// - `A`: The deserializer's map access implementation.
    ///
    /// # Parameters
    ///
    /// - `map`: The map access consumed to decode the named components.
    ///
    /// # Returns
    ///
    /// The decoded components, before validation of the nanosecond range.
    ///
    /// # Errors
    ///
    /// Returns a deserializer error for an unknown, duplicate, or missing
    /// field, or when a field value cannot be decoded as its field type.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        // Map representations require both recognized fields exactly once.
        let mut secs = None;
        let mut nanos = None;
        while let Some(key) = map.next_key::<DurationPayloadField>()? {
            match key {
                DurationPayloadField::Secs => {
                    if secs.is_some() {
                        return Err(Error::duplicate_field("secs"));
                    }
                    secs = Some(map.next_value()?);
                }
                DurationPayloadField::Nanos => {
                    if nanos.is_some() {
                        return Err(Error::duplicate_field("nanos"));
                    }
                    nanos = Some(map.next_value()?);
                }
            }
        }
        Ok(DurationPayload {
            secs: secs.ok_or_else(|| Error::missing_field("secs"))?,
            nanos: nanos.ok_or_else(|| Error::missing_field("nanos"))?,
        })
    }
}
