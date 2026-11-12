// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an owned named scalar value.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use serde::de::Error;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::NamedValueWireOwned;
use super::named_value_field::NamedValueField;
use crate::ValueWireV1;
use crate::ValueWireV1Seed;

/// Deserializes an owned named scalar record with exactly `name` and `value`
/// fields.
///
/// Ordered sequences are accepted only for formats that permit positional
/// records.
pub(super) struct NamedValueWireOwnedVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl NamedValueWireOwnedVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    ///
    /// # Parameters
    ///
    /// - `allow_sequence`: Whether the format permits positional records
    ///   instead of named fields.
    ///
    /// # Returns
    ///
    /// A visitor that enforces the supplied sequence policy.
    #[must_use]
    #[inline]
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for NamedValueWireOwnedVisitor {
    /// Owned record produced after both required fields have been decoded.
    type Value = NamedValueWireOwned;

    /// Describes the named scalar record expected by this visitor.
    ///
    /// # Parameters
    ///
    /// - `f`: Formatter receiving the expected-record description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Propagates an error from the formatter.
    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("named scalar record with name and value fields")
    }

    /// Reads the `name` and `value` fields in order when sequences are allowed.
    ///
    /// # Type Parameters
    ///
    /// - `A`: Sequence access supplied by the deserializer for input lifetime
    ///   `'de`.
    ///
    /// # Parameters
    ///
    /// - `seq`: Owned access to the positional name and scalar envelope.
    ///
    /// # Returns
    ///
    /// The owned record decoded from exactly two elements.
    ///
    /// # Errors
    ///
    /// Rejects disallowed sequences, missing or extra elements, and propagates
    /// element decoding errors.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        let name = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let value = seq
            .next_element_seed(ValueWireV1Seed::new())?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(NamedValueWireOwned { name, value })
    }

    /// Reads named fields in any order, rejecting duplicates and unknown keys.
    ///
    /// # Type Parameters
    ///
    /// - `A`: Map access supplied by the deserializer for input lifetime `'de`.
    ///
    /// # Parameters
    ///
    /// - `map`: Owned access to the record's named fields.
    ///
    /// # Returns
    ///
    /// The owned record containing both required fields.
    ///
    /// # Errors
    ///
    /// Rejects missing, duplicate, or unknown fields and propagates key and
    /// value decoding errors, including invalid scalar envelopes.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut value: Option<ValueWireV1> = None;
        while let Some(key) = map.next_key::<NamedValueField>()? {
            match key {
                NamedValueField::Name => {
                    if name.is_some() {
                        return Err(Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?);
                }
                NamedValueField::Value => {
                    if value.is_some() {
                        return Err(Error::duplicate_field("value"));
                    }
                    value = Some(map.next_value_seed(ValueWireV1Seed::new())?);
                }
            }
        }
        Ok(NamedValueWireOwned {
            name: name.ok_or_else(|| Error::missing_field("name"))?,
            value: value.ok_or_else(|| Error::missing_field("value"))?,
        })
    }
}
