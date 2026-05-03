/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations for value conversion support.
//!
//! This module focuses on conversion helpers and `ValueConverter<T>` impls. The
//! generic trait impls for getter/setter/constructor are defined in
//! `value_getter.rs`, `value_setter.rs`, and `value_constructor.rs` to keep
//! trait concerns separate.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use num_bigint::BigInt;

use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConvertTo,
    DataConverter,
};

use super::value::Value;
use super::value_converter::ValueConverter;
use crate::value_error::{
    ValueError,
    ValueResult,
};

/// Maps a shared single-value conversion error into `ValueError`.
fn map_data_conversion_error(error: DataConversionError) -> ValueError {
    match error {
        DataConversionError::NoValue => ValueError::NoValue,
        DataConversionError::ConversionFailed { from, to } => {
            ValueError::ConversionFailed { from, to }
        }
        DataConversionError::ConversionError(message) => ValueError::ConversionError(message),
        DataConversionError::JsonSerializationError(message) => {
            ValueError::JsonSerializationError(message)
        }
        DataConversionError::JsonDeserializationError(message) => {
            ValueError::JsonDeserializationError(message)
        }
    }
}

/// Wraps a `Value` into the common conversion helper for the `qubit_datatype`
/// conversion API.
fn data_converter_from_value(value: &Value) -> DataConverter<'_> {
    match value {
        Value::Empty(data_type) => DataConverter::Empty(*data_type),
        Value::Bool(value) => DataConverter::from(value),
        Value::Char(value) => DataConverter::from(value),
        Value::Int8(value) => DataConverter::from(value),
        Value::Int16(value) => DataConverter::from(value),
        Value::Int32(value) => DataConverter::from(value),
        Value::Int64(value) => DataConverter::from(value),
        Value::Int128(value) => DataConverter::from(value),
        Value::UInt8(value) => DataConverter::from(value),
        Value::UInt16(value) => DataConverter::from(value),
        Value::UInt32(value) => DataConverter::from(value),
        Value::UInt64(value) => DataConverter::from(value),
        Value::UInt128(value) => DataConverter::from(value),
        Value::IntSize(value) => DataConverter::from(value),
        Value::UIntSize(value) => DataConverter::from(value),
        Value::Float32(value) => DataConverter::from(value),
        Value::Float64(value) => DataConverter::from(value),
        Value::BigInteger(value) => DataConverter::from(value),
        Value::BigDecimal(value) => DataConverter::from(value),
        Value::String(value) => DataConverter::from(value),
        Value::Date(value) => DataConverter::from(value),
        Value::Time(value) => DataConverter::from(value),
        Value::DateTime(value) => DataConverter::from(value),
        Value::Instant(value) => DataConverter::from(value),
        Value::Duration(value) => DataConverter::from(value),
        Value::Url(value) => DataConverter::from(value),
        Value::StringMap(value) => DataConverter::from(value),
        Value::Json(value) => DataConverter::from(value),
    }
}

/// Converts a single `Value` into `T` using shared conversion helpers.
fn convert_with_data_converter<T>(value: &Value) -> ValueResult<T>
where
    for<'a> DataConverter<'a>: DataConvertTo<T>,
{
    convert_with_data_converter_with(value, &DataConversionOptions::default())
}

/// Converts a single `Value` into `T` using shared conversion helpers and options.
///
/// # Parameters
///
/// * `value` - Source value to convert.
/// * `options` - Conversion options forwarded to `qubit_datatype`.
///
/// # Returns
///
/// Returns the converted value.
///
/// # Errors
///
/// Returns a `ValueError` mapped from the shared conversion error when the
/// source value is missing, unsupported, or invalid for `T`.
pub(super) fn convert_with_data_converter_with<T>(
    value: &Value,
    options: &DataConversionOptions,
) -> ValueResult<T>
where
    for<'a> DataConverter<'a>: DataConvertTo<T>,
{
    data_converter_from_value(value)
        .to_with::<T>(options)
        .map_err(map_data_conversion_error)
}

macro_rules! impl_data_value_converter {
    ($type:ty) => {
        impl ValueConverter<$type> for Value {
            #[inline]
            fn convert(&self) -> ValueResult<$type> {
                convert_with_data_converter(self)
            }
        }
    };
}

impl_data_value_converter!(String);
impl_data_value_converter!(bool);
impl_data_value_converter!(char);
impl_data_value_converter!(i8);
impl_data_value_converter!(i16);
impl_data_value_converter!(i32);
impl_data_value_converter!(i64);
impl_data_value_converter!(i128);
impl_data_value_converter!(u8);
impl_data_value_converter!(u16);
impl_data_value_converter!(u32);
impl_data_value_converter!(u64);
impl_data_value_converter!(u128);
impl_data_value_converter!(isize);
impl_data_value_converter!(usize);
impl_data_value_converter!(f32);
impl_data_value_converter!(f64);
impl_data_value_converter!(NaiveDate);
impl_data_value_converter!(NaiveTime);
impl_data_value_converter!(NaiveDateTime);
impl_data_value_converter!(DateTime<Utc>);
impl_data_value_converter!(BigInt);
impl_data_value_converter!(BigDecimal);
impl_data_value_converter!(Duration);
impl_data_value_converter!(url::Url);
impl_data_value_converter!(HashMap<String, String>);
impl_data_value_converter!(serde_json::Value);
