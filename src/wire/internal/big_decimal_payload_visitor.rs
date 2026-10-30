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

pub(super) struct BigDecimalPayloadVisitor;

impl<'de> Visitor<'de> for BigDecimalPayloadVisitor {
    type Value = BigDecimalPayload;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("struct BigDecimalPayload")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
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
