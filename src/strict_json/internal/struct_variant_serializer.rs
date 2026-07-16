// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer for struct enum variants.

use serde::Serialize;
use serde::ser::SerializeStructVariant;
use serde_json::{
    Map,
    Value,
};

use crate::strict_json::{
    Result,
    StrictJsonError,
    to_value,
};

/// Accumulates a struct variant into a nested single-key JSON object.
pub(in crate::strict_json) struct StructVariantSerializer {
    /// Variant name used as the outer object key.
    pub(in crate::strict_json) variant: String,
    /// Serialized struct fields.
    pub(in crate::strict_json) values: Map<String, Value>,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and inserts one named struct variant field.
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.insert(key.to_string(), to_value(value)?);
        Ok(())
    }

    /// Returns the struct variant as a nested single-key JSON object.
    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.variant, Value::Object(self.values));
        Ok(Value::Object(object))
    }
}
