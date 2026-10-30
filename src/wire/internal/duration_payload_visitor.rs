// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for a duration payload.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::DurationPayload;

pub(super) struct DurationPayloadVisitor;

impl<'de> Visitor<'de> for DurationPayloadVisitor {
    type Value = DurationPayload;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("struct DurationPayload")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let secs = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let nanos = seq.next_element()?.ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(DurationPayload { secs, nanos })
    }
    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut secs = None;
        let mut nanos = None;
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "secs" => {
                    if secs.is_some() {
                        return Err(Error::duplicate_field("secs"));
                    }
                    secs = Some(map.next_value()?);
                }
                "nanos" => {
                    if nanos.is_some() {
                        return Err(Error::duplicate_field("nanos"));
                    }
                    nanos = Some(map.next_value()?);
                }
                _ => return Err(Error::unknown_field(&key, &["secs", "nanos"])),
            }
        }
        Ok(DurationPayload {
            secs: secs.ok_or_else(|| Error::missing_field("secs"))?,
            nanos: nanos.ok_or_else(|| Error::missing_field("nanos"))?,
        })
    }
}
