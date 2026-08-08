// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer for JSON map and struct values.

use serde::Serialize;
use serde::ser::SerializeMap;
use serde::ser::SerializeStruct;
use serde_json::Map;
use serde_json::Value;

use super::MapKeySerializer;
use crate::strict_json::Result;
use crate::strict_json::StrictJsonError;
use crate::strict_json::to_value;

/// Accumulates map entries or struct fields into one JSON object.
pub(in crate::strict_json) struct ObjectSerializer {
    /// Serialized object entries.
    pub(in crate::strict_json) values: Map<String, Value>,
    /// Serialized key awaiting its corresponding map value.
    pub(in crate::strict_json) next_key: Option<String>,
}

impl SerializeMap for ObjectSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and retains the next map key.
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    /// Serializes the value corresponding to the retained map key.
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take().ok_or(StrictJsonError::Serialization)?;
        self.values.insert(key, to_value(value)?);
        Ok(())
    }

    /// Returns the completed JSON object.
    fn end(self) -> Result<Value> {
        if self.next_key.is_some() {
            return Err(StrictJsonError::Serialization);
        }
        Ok(Value::Object(self.values))
    }
}

impl SerializeStruct for ObjectSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and inserts one named struct field.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.insert(key.to_string(), to_value(value)?);
        Ok(())
    }

    /// Returns the completed struct as a JSON object.
    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.values))
    }
}
