// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

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
    ValueError,
    ValueResult,
};

macro_rules! value_data_converter_match {
    ($value:expr; $(([$($cfg:meta),*], [$($value_attr:meta),*], [$($multi_attr:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            Value::Unset(data_type) => DataConverter::Empty(*data_type),
            $($(#[$cfg])* Value::$variant(value) => DataConverter::from(value),)+
        }
    };
}

/// Wraps a `Value` into the common conversion helper for the `qubit_datatype`
/// conversion API.
fn data_converter_from_value(value: &Value) -> DataConverter<'_> {
    for_each_value_type!(value_data_converter_match, value)
}

/// Converts a single `Value` into `T` using shared conversion helpers and
/// options.
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
        .map_err(ValueError::from)
}
