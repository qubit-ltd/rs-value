// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an arbitrary-precision decimal payload.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::BigDecimalPayload;

pub(super) struct BigDecimalPayloadVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl BigDecimalPayloadVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for BigDecimalPayloadVisitor {
    type Value = BigDecimalPayload;

    /// Describes the decimal payload record expected by this visitor.
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("decimal payload record with coefficient and scale fields")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        if !self.allow_sequence {
            return Err(Error::custom("human-readable record must use named fields"));
        }
        let coefficient = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let scale = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(BigDecimalPayload { coefficient, scale })
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut coefficient = None;
        let mut scale = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "coefficient" => {
                    if coefficient.is_some() {
                        return Err(Error::duplicate_field("coefficient"));
                    }
                    coefficient = Some(map.next_value()?);
                }
                "scale" => {
                    if scale.is_some() {
                        return Err(Error::duplicate_field("scale"));
                    }
                    scale = Some(map.next_value()?);
                }
                _ => return Err(Error::unknown_field(&key, &["coefficient", "scale"])),
            }
        }
        Ok(BigDecimalPayload {
            coefficient: coefficient.ok_or_else(|| Error::missing_field("coefficient"))?,
            scale: scale.ok_or_else(|| Error::missing_field("scale"))?,
        })
    }
}
