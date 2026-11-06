// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an owned named collection.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use serde::de::Error;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::NamedMultiValuesWireOwned;
use crate::ValueWireV1;
use crate::ValueWireV1Seed;

/// Decodes an owned named collection while enforcing the format's record shape.
///
/// Human-readable formats require named fields; other formats may supply the
/// two fields as an ordered sequence when enabled by the caller.
pub(super) struct NamedMultiValuesWireOwnedVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl NamedMultiValuesWireOwnedVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    ///
    /// # Parameters
    ///
    /// - `allow_sequence`: Whether the format permits the ordered `name`,
    ///   `value` representation in addition to named fields.
    ///
    /// # Returns
    ///
    /// A visitor that owns the supplied sequence policy.
    #[must_use]
    #[inline]
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for NamedMultiValuesWireOwnedVisitor {
    /// Owned name and versioned collection produced after validating the
    /// record.
    type Value = NamedMultiValuesWireOwned;

    /// Describes the named collection record expected by this visitor.
    ///
    /// # Parameters
    ///
    /// - `f`: Formatter receiving the expected record description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing fails.
    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("named collection record with name and value fields")
    }

    /// Reads the `name` and `value` fields in order when sequences are allowed.
    ///
    /// # Type Parameters
    ///
    /// - `A`: Sequence access supplied by the deserializer.
    ///
    /// # Parameters
    ///
    /// - `seq`: Owned access to the ordered record fields.
    ///
    /// # Returns
    ///
    /// The decoded owned name and versioned collection.
    ///
    /// # Errors
    ///
    /// Rejects sequences when disabled, missing or extra elements, and invalid
    /// field data reported by the deserializer.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        let name = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let value = seq
            .next_element_seed(ValueWireV1Seed::new())?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(NamedMultiValuesWireOwned { name, value })
    }

    /// Reads exactly one `name` and one `value` field from a named record.
    ///
    /// # Type Parameters
    ///
    /// - `A`: Map access supplied by the deserializer.
    ///
    /// # Parameters
    ///
    /// - `map`: Owned access to the record's keys and values.
    ///
    /// # Returns
    ///
    /// The decoded owned name and versioned collection, regardless of field
    /// order.
    ///
    /// # Errors
    ///
    /// Rejects duplicate, unknown, or missing fields and propagates field
    /// decoding errors from the deserializer.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut name: Option<String> = None;
        let mut value: Option<ValueWireV1> = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "name" => {
                    if name.is_some() {
                        return Err(Error::duplicate_field("name"));
                    }
                    name = Some(map.next_value()?);
                }
                "value" => {
                    if value.is_some() {
                        return Err(Error::duplicate_field("value"));
                    }
                    value = Some(map.next_value_seed(ValueWireV1Seed::new())?);
                }
                _ => return Err(Error::unknown_field(&key, &["name", "value"])),
            }
        }
        Ok(NamedMultiValuesWireOwned {
            name: name.ok_or_else(|| Error::missing_field("name"))?,
            value: value.ok_or_else(|| Error::missing_field("value"))?,
        })
    }
}
