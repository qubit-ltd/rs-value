// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an arbitrary-precision decimal payload.

use std::fmt::Formatter;
use std::fmt::Result as FmtResult;

use serde::de::Error;
use serde::de::IgnoredAny;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::BigDecimalPayload;
use super::big_decimal_payload_field::BigDecimalPayloadField;

/// Decodes coefficient and scale records without validating decimal
/// canonicality.
///
/// Named fields are always accepted; ordered fields are accepted only for
/// formats whose deserializer reports a non-human-readable representation.
pub(super) struct BigDecimalPayloadVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl BigDecimalPayloadVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    ///
    /// # Parameters
    ///
    /// * `allow_sequence` - Whether coefficient-then-scale sequences are
    ///   accepted.
    ///
    /// # Returns
    ///
    /// A visitor enforcing the supplied representation policy.
    #[must_use]
    #[inline]
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for BigDecimalPayloadVisitor {
    /// The decoded record, before canonical coefficient and scale validation.
    type Value = BigDecimalPayload;

    /// Describes the decimal payload record expected by this visitor.
    ///
    /// # Parameters
    ///
    /// * `f` - Formatter receiving the expected record description.
    ///
    /// # Returns
    ///
    /// Success after writing the description.
    ///
    /// # Errors
    ///
    /// Returns the formatter's error if writing fails.
    fn expecting(&self, f: &mut Formatter<'_>) -> FmtResult {
        f.write_str("decimal payload record with coefficient and scale fields")
    }

    /// Consumes an ordered coefficient and scale record when sequences are
    /// allowed.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Sequence access supplied by the deserializer.
    ///
    /// # Parameters
    ///
    /// * `seq` - Owned access to the coefficient followed by the scale.
    ///
    /// # Returns
    ///
    /// The decoded payload without canonical decimal validation.
    ///
    /// # Errors
    ///
    /// Rejects forbidden sequences, missing or extra elements, and element
    /// deserialization failures.
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        let coefficient = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(0, &self))?;
        let scale = seq
            .next_element()?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(BigDecimalPayload { coefficient, scale })
    }

    /// Consumes a named record containing exactly one coefficient and one
    /// scale.
    ///
    /// # Type Parameters
    ///
    /// * `A` - Map access supplied by the deserializer.
    ///
    /// # Parameters
    ///
    /// * `map` - Owned access to named fields in any order.
    ///
    /// # Returns
    ///
    /// The decoded payload without canonical decimal validation.
    ///
    /// # Errors
    ///
    /// Rejects unknown, duplicate, or missing fields and propagates key or
    /// value deserialization failures.
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut coefficient = None;
        let mut scale = None;
        while let Some(key) = map.next_key::<BigDecimalPayloadField>()? {
            match key {
                BigDecimalPayloadField::Coefficient => {
                    if coefficient.is_some() {
                        return Err(Error::duplicate_field("coefficient"));
                    }
                    coefficient = Some(map.next_value()?);
                }
                BigDecimalPayloadField::Scale => {
                    if scale.is_some() {
                        return Err(Error::duplicate_field("scale"));
                    }
                    scale = Some(map.next_value()?);
                }
            }
        }
        Ok(BigDecimalPayload {
            coefficient: coefficient.ok_or_else(|| Error::missing_field("coefficient"))?,
            scale: scale.ok_or_else(|| Error::missing_field("scale"))?,
        })
    }
}
