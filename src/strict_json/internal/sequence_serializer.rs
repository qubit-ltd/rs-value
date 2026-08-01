// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer for JSON array-like compound values.

use serde::Serialize;
use serde::ser::{SerializeSeq, SerializeTuple, SerializeTupleStruct};
use serde_json::Value;

use crate::strict_json::{Result, StrictJsonError, to_value};

/// Accumulates sequence and tuple elements into one JSON array.
pub(in crate::strict_json) struct SequenceSerializer {
    /// Serialized array elements.
    pub(in crate::strict_json) values: Vec<Value>,
}

impl SerializeSeq for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and appends one sequence element.
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(to_value(value)?);
        Ok(())
    }

    /// Returns the accumulated JSON array.
    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.values))
    }
}

impl SerializeTuple for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and appends one tuple element.
    #[inline(always)]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    /// Returns the accumulated tuple as a JSON array.
    #[inline(always)]
    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    /// Serializes and appends one tuple-struct field.
    #[inline(always)]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    /// Returns the accumulated tuple struct as a JSON array.
    #[inline(always)]
    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}
