// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an owned V1 envelope.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::WireEnvelopeOwned;
pub(super) struct WireEnvelopeOwnedVisitor;

impl<'de> Visitor<'de> for WireEnvelopeOwnedVisitor {
    type Value = WireEnvelopeOwned;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("struct WireEnvelopeOwned")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let version = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let value = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(WireEnvelopeOwned { version, value })
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
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
