// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One-pass JSON value serializer with strict finite-float validation.

use std::fmt::{self, Display};

use serde::Serialize;
use serde::ser::{
    Impossible, SerializeMap, SerializeSeq, SerializeStruct, SerializeStructVariant,
    SerializeTuple, SerializeTupleStruct, SerializeTupleVariant, Serializer,
};
use serde_json::{Map, Number, Value};

/// Stable error categories needed by the public conversion layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StrictJsonError {
    /// A non-finite float was encountered at any nesting level.
    NonFinite,
    /// The input could not be represented as a JSON value.
    Serialization,
}

impl Display for StrictJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => formatter.write_str("non-finite float"),
            Self::Serialization => formatter.write_str("JSON serialization failed"),
        }
    }
}

impl std::error::Error for StrictJsonError {}

impl serde::ser::Error for StrictJsonError {
    fn custom<T>(_message: T) -> Self
    where
        T: Display,
    {
        Self::Serialization
    }
}

type Result<T> = std::result::Result<T, StrictJsonError>;

/// Serializes a value to JSON while rejecting every non-finite float.
pub(crate) fn to_value<T>(value: &T) -> Result<Value>
where
    T: ?Sized + Serialize,
{
    value.serialize(StrictJsonSerializer)
}

#[derive(Clone, Copy)]
struct StrictJsonSerializer;

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

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<Value> {
        Ok(Value::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<Value> {
        self.serialize_i64(value.into())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_i128(self, value: i128) -> Result<Value> {
        if let Ok(value) = u64::try_from(value) {
            self.serialize_u64(value)
        } else if let Ok(value) = i64::try_from(value) {
            self.serialize_i64(value)
        } else {
            Err(StrictJsonError::Serialization)
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<Value> {
        self.serialize_u64(value.into())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<Value> {
        Ok(Value::Number(value.into()))
    }

    fn serialize_u128(self, value: u128) -> Result<Value> {
        u64::try_from(value)
            .map_err(|_| StrictJsonError::Serialization)
            .and_then(|value| self.serialize_u64(value))
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<Value> {
        self.serialize_f64(value.into())
    }

    fn serialize_f64(self, value: f64) -> Result<Value> {
        Number::from_f64(value)
            .map(Value::Number)
            .ok_or(StrictJsonError::NonFinite)
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<Value> {
        Ok(Value::String(value.to_string()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<Value> {
        Ok(Value::Array(
            value
                .iter()
                .map(|value| Value::Number((*value).into()))
                .collect(),
        ))
    }

    #[inline]
    fn serialize_none(self) -> Result<Value> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<Value> {
        Ok(Value::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Value> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Value> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<Value>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

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

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SequenceSerializer {
            values: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

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

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(ObjectSerializer {
            values: Map::with_capacity(len.unwrap_or(0)),
            next_key: None,
        })
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

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

    fn collect_str<T>(self, value: &T) -> Result<Value>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}

struct SequenceSerializer {
    values: Vec<Value>,
}

impl SerializeSeq for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Array(self.values))
    }
}

impl SerializeTuple for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

impl SerializeTupleStruct for SequenceSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Value> {
        SerializeSeq::end(self)
    }
}

struct TupleVariantSerializer {
    variant: String,
    values: Vec<Value>,
}

impl SerializeTupleVariant for TupleVariantSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.push(to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.variant, Value::Array(self.values));
        Ok(Value::Object(object))
    }
}

struct ObjectSerializer {
    values: Map<String, Value>,
    next_key: Option<String>,
}

impl SerializeMap for ObjectSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        let key = self.next_key.take().ok_or(StrictJsonError::Serialization)?;
        self.values.insert(key, to_value(value)?);
        Ok(())
    }

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

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.insert(key.to_string(), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        Ok(Value::Object(self.values))
    }
}

struct StructVariantSerializer {
    variant: String,
    values: Map<String, Value>,
}

impl SerializeStructVariant for StructVariantSerializer {
    type Ok = Value;
    type Error = StrictJsonError;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.values.insert(key.to_string(), to_value(value)?);
        Ok(())
    }

    fn end(self) -> Result<Value> {
        let mut object = Map::new();
        object.insert(self.variant, Value::Object(self.values));
        Ok(Value::Object(object))
    }
}

#[derive(Clone, Copy)]
struct MapKeySerializer;

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

    fn serialize_f32(self, value: f32) -> Result<String> {
        if value.is_finite() {
            Ok(value.to_string())
        } else {
            Err(StrictJsonError::NonFinite)
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String> {
        if value.is_finite() {
            Ok(value.to_string())
        } else {
            Err(StrictJsonError::NonFinite)
        }
    }

    fn serialize_char(self, value: char) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_none(self) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_unit(self) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_string())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

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

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(StrictJsonError::Serialization)
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(StrictJsonError::Serialization)
    }

    fn collect_str<T>(self, value: &T) -> Result<String>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}
