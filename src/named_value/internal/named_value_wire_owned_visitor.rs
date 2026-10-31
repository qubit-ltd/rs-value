// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an owned named scalar value.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::NamedValueWireOwned;
use crate::ValueWireV1;
use crate::ValueWireV1Seed;

pub(super) struct NamedValueWireOwnedVisitor {
    /// Whether ordered struct sequences are permitted by the format.
    allow_sequence: bool,
}

impl NamedValueWireOwnedVisitor {
    /// Creates a visitor with the sequence policy for the current format.
    pub(super) fn new(allow_sequence: bool) -> Self {
        Self { allow_sequence }
    }
}

impl<'de> Visitor<'de> for NamedValueWireOwnedVisitor {
    type Value = NamedValueWireOwned;

    /// Describes the named scalar record expected by this visitor.
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("named scalar record with name and value fields")
    }

    /// Reads the `name` and `value` fields in order when sequences are allowed.
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
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(NamedValueWireOwned { name, value })
    }
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
        Ok(NamedValueWireOwned {
            name: name.ok_or_else(|| Error::missing_field("name"))?,
            value: value.ok_or_else(|| Error::missing_field("value"))?,
        })
    }
}
