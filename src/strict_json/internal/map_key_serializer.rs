// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serializer for JSON object keys.

use std::fmt::Display;

use serde::Serialize;
use serde::ser::Impossible;
use serde::ser::Serializer;

use crate::strict_json::Result;
use crate::strict_json::StrictJsonError;

/// Converts supported scalar map keys to JSON object key strings.
#[derive(Clone, Copy)]
pub(in crate::strict_json) struct MapKeySerializer;

/// Implements canonical textual serialization for integer key types.
macro_rules! serialize_key_integer {
    ($($method:ident($type:ty)),+ $(,)?) => {
        $(
            fn $method(self, value: $type) -> Result<String> {
                Ok(value.to_string())
            }
        )+
    };
}

impl Serializer for MapKeySerializer {
    type Ok = String;
    type Error = StrictJsonError;
    type SerializeSeq = Impossible<String, StrictJsonError>;
    type SerializeTuple = Impossible<String, StrictJsonError>;
    type SerializeTupleStruct = Impossible<String, StrictJsonError>;
    type SerializeTupleVariant = Impossible<String, StrictJsonError>;
    type SerializeMap = Impossible<String, StrictJsonError>;
    type SerializeStruct = Impossible<String, StrictJsonError>;
    type SerializeStructVariant = Impossible<String, StrictJsonError>;

    /// Serializes a Boolean key through its textual representation.
    fn serialize_bool(self, value: bool) -> Result<String> {
        Ok(value.to_string())
    }

    serialize_key_integer!(
        serialize_i8(i8),
        serialize_i16(i16),
        serialize_i32(i32),
        serialize_i64(i64),
        serialize_i128(i128),
        serialize_u8(u8),
        serialize_u16(u16),
        serialize_u32(u32),
        serialize_u64(u64),
        serialize_u128(u128),
    );

    /// Serializes a finite 32-bit floating-point key.
    fn serialize_f32(self, value: f32) -> Result<String> {
        if value.is_finite() {
            Ok(value.to_string())
        } else {
            Err(StrictJsonError::NonFinite)
        }
    }

    /// Serializes a finite 64-bit floating-point key.
    fn serialize_f64(self, value: f64) -> Result<String> {
        if value.is_finite() {
            Ok(value.to_string())
        } else {
            Err(StrictJsonError::NonFinite)
        }
    }

    /// Serializes a character key.
    fn serialize_char(self, value: char) -> Result<String> {
        Ok(value.to_string())
    }

    /// Copies a string key.
    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_string())
    }

    /// Rejects byte sequences because JSON object keys are strings.
    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects absent optional keys.
    fn serialize_none(self) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects optional key wrappers.
    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects unit keys.
    fn serialize_unit(self) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects unit-struct keys.
    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    /// Serializes a unit variant key through its variant name.
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_string())
    }

    /// Delegates a newtype-struct key to its wrapped value.
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    /// Rejects newtype-variant keys.
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects sequence keys.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects tuple keys.
    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects tuple-struct keys.
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects tuple-variant keys.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects map keys.
    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects struct keys.
    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(StrictJsonError::Serialization)
    }

    /// Rejects struct-variant keys.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(StrictJsonError::Serialization)
    }

    /// Serializes a displayable key through its textual representation.
    fn collect_str<T>(self, value: &T) -> Result<String>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}
