// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Root serializer for strict JSON value construction.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::Serializer;
use serde_json::{
    Map,
    Number,
    Value,
};

use crate::strict_json::{
    Result,
    StrictJsonError,
};

use super::{
    ObjectSerializer,
    SequenceSerializer,
    StructVariantSerializer,
    TupleVariantSerializer,
};

/// Builds a JSON value while rejecting every non-finite floating-point value.
#[derive(Clone, Copy)]
pub(in crate::strict_json) struct StrictJsonSerializer;

impl Serializer for StrictJsonSerializer {
    type Ok = Value;
    type Error = StrictJsonError;
    type SerializeSeq = SequenceSerializer;
    type SerializeTuple = SequenceSerializer;
    type SerializeTupleStruct = SequenceSerializer;
    type SerializeTupleVariant = TupleVariantSerializer;
    type SerializeMap = ObjectSerializer;
    type SerializeStruct = ObjectSerializer;
    type SerializeStructVariant = StructVariantSerializer;

    /// Serializes a Boolean JSON value.
    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value> {
        Ok(Value::Bool(value))
    }

    /// Serializes an 8-bit signed integer.
    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    /// Serializes a 16-bit signed integer.
    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    /// Serializes a 32-bit signed integer.
    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    /// Serializes a JSON-compatible signed integer.
    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    /// Serializes a 128-bit signed integer when it fits a JSON integer.
    fn serialize_i128(self, value: i128) -> Result<Value> {
        if let Ok(value) = u64::try_from(value) {
            self.serialize_u64(value)
        } else if let Ok(value) = i64::try_from(value) {
            self.serialize_i64(value)
        } else {
            Err(StrictJsonError::Serialization)
        }
    }

    /// Serializes an 8-bit unsigned integer.
    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    /// Serializes a 16-bit unsigned integer.
    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    /// Serializes a 32-bit unsigned integer.
    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    /// Serializes a JSON-compatible unsigned integer.
    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    /// Serializes a 128-bit unsigned integer when it fits a JSON integer.
    fn serialize_u128(self, value: u128) -> Result<Value> {
        u64::try_from(value)
            .map_err(|_| StrictJsonError::Serialization)
            .and_then(|value| self.serialize_u64(value))
    }

    /// Serializes a finite 32-bit floating-point value.
    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value> {
        self.serialize_f64(value.into())
    }

    /// Serializes a finite 64-bit floating-point value.
    fn serialize_f64(self, value: f64) -> Result<Value> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or(StrictJsonError::NonFinite)
    }

    /// Serializes a character as a JSON string.
    #[inline]
    fn serialize_char(self, value: char) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    /// Serializes a borrowed string.
    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    /// Serializes bytes as a JSON number array.
    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        Ok(Value::Array(
            value
                .iter()
                .map(|value| Value::Number((*value).into()))
                .collect(),
        ))
    }

    /// Serializes an absent option as JSON null.
    #[inline]
    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit()
    }

    /// Serializes the value inside a present option.
    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    /// Serializes a unit value as JSON null.
    #[inline]
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    /// Serializes a unit struct as JSON null.
    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    /// Serializes a unit variant through its variant name.
    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    /// Delegates a newtype struct to its wrapped value.
    #[inline]
    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    /// Serializes a newtype variant as a single-key JSON object.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        let mut object = Map::new();
        object.insert(variant.to_string(), value.serialize(self)?);
        Ok(Value::Object(object))
    }

    /// Creates a sequence serializer with the declared capacity.
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SequenceSerializer {
            values: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    /// Creates a tuple serializer with the declared capacity.
    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    /// Creates a tuple-struct serializer with the declared capacity.
    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    /// Creates a tuple-variant serializer with the declared capacity.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(TupleVariantSerializer {
            variant: variant.to_string(),
            values: Vec::with_capacity(len),
        })
    }

    /// Creates a map serializer with the declared capacity.
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(ObjectSerializer {
            values: Map::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    /// Creates a struct serializer with the declared capacity.
    #[inline]
    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    /// Creates a struct-variant serializer with the declared capacity.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Ok(StructVariantSerializer {
            variant: variant.to_string(),
            values: Map::with_capacity(len),
        })
    }

    /// Serializes a displayable value as a JSON string.
    fn collect_str<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}
