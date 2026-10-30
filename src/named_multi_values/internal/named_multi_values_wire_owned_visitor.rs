// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde visitor for an owned named collection.

use serde::de::Error;
use serde::de::MapAccess;
use serde::de::SeqAccess;
use serde::de::Visitor;

use super::NamedMultiValuesWireOwned;
use crate::ValueWireV1;
use crate::ValueWireV1Seed;

pub(super) struct NamedMultiValuesWireOwnedVisitor;

impl<'de> Visitor<'de> for NamedMultiValuesWireOwnedVisitor {
    type Value = NamedMultiValuesWireOwned;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("struct NamedMultiValuesWireOwned")
    }
    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let name = seq.next_element()?.ok_or_else(|| Error::invalid_length(0, &self))?;
        let value = seq
            .next_element_seed(ValueWireV1Seed::new())?
            .ok_or_else(|| Error::invalid_length(1, &self))?;
        if seq.next_element::<serde::de::IgnoredAny>()?.is_some() {
            return Err(Error::invalid_length(3, &self));
        }
        Ok(NamedMultiValuesWireOwned { name, value })
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
        Ok(NamedMultiValuesWireOwned {
            name: name.ok_or_else(|| Error::missing_field("name"))?,
            value: value.ok_or_else(|| Error::missing_field("value"))?,
        })
    }
}
