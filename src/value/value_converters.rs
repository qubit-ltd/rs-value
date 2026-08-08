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

use qubit_datatype::DataConversionOptions;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataConverter;

use super::value::Value;
use super::value::ValueRepr;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

macro_rules! value_data_converter_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            ValueRepr::Unset(data_type) => DataConverter::Unset(*data_type),
            $($(#[$cfg])* ValueRepr::$variant(value) => DataConverter::from(value_storage_ref!($variant, value)),)+
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
    T: DataConversionTarget,
{
    data_converter_from_value(value)
        .to_with::<T>(options)
        .map_err(ValueError::from)
}
