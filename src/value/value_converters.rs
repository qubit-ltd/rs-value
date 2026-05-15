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
//! This module focuses on conversion helpers backed by `qubit_datatype`.

use qubit_datatype::{
    DataConversionOptions,
    DataConvertTo,
    DataConverter,
};

use super::value::Value;
use crate::value_error::{
    ValueResult,
    map_data_conversion_error,
};

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
