// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer for tuple enum variants.

use serde::Serialize;
use serde::ser::SerializeTupleVariant;
use serde_json::Map;
use serde_json::Value;

use crate::strict_json::Result;
use crate::strict_json::StrictJsonError;
use crate::strict_json::to_value;

/// Accumulates a tuple variant into a single-key JSON object.
pub(in crate::strict_json) struct TupleVariantSerializer {
    /// Variant name used as the object key.
    pub(in crate::strict_json) variant: String,
    /// Serialized tuple fields.
    pub(in crate::strict_json) values: Vec<Value>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and appends one tuple variant field.
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        to_value(value).map(|value| self.values.push(value))
    }

    /// Returns the tuple variant as a single-key JSON object.
    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.variant, Value::Array(self.values));
        Ok(Value::Object(object))
    }
}
