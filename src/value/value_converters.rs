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

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionSession;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataConverter;

use super::Value;
use super::ValueRepr;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

/// Expands the shared value table into a `DataConverter` construction match.
macro_rules! value_data_converter_match {
    ($value:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match &$value.repr {
            ValueRepr::Unset(data_type) => DataConverter::Unset(*data_type),
            $($(#[$cfg])* ValueRepr::$variant(value) => DataConverter::from(value_storage_ref!($variant, value)),)+
        }
    };
}

/// Wraps a `Value` into the common conversion helper for the `qubit_datatype`
/// conversion API.
///
/// # Parameters
///
/// * `value` - Runtime value whose payload is borrowed by the converter.
///
/// # Returns
///
/// A shared converter view preserving the runtime data type.
fn data_converter_from_value(value: &Value) -> DataConverter<'_> {
    for_each_value_type!(value_data_converter_match, value)
}

impl<'a> From<&'a Value> for DataConverter<'a> {
    /// Borrows a runtime value as a shared conversion source.
    ///
    /// # Parameters
    ///
    /// * `value` - Runtime value whose storage is exposed to the converter.
    ///
    /// # Returns
    ///
    /// A [`DataConverter`] borrowing rich payloads from `value` without
    /// cloning them.
    #[inline(always)]
    fn from(value: &'a Value) -> Self {
        data_converter_from_value(value)
    }
}

/// Converts a single `Value` into `T` using shared conversion helpers,
/// conversion policy, and resource limits.
///
/// # Type Parameters
///
/// * `T` - Target type supported by the shared conversion layer.
///
/// # Parameters
///
/// * `value` - Source value to convert.
/// * `policy` - Conversion policy forwarded to `qubit_datatype`.
/// * `limits` - Conversion limits forwarded to `qubit_datatype`.
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
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<T>
where
    T: DataConversionTarget,
{
    data_converter_from_value(value)
        .to_with::<T>(policy, limits)
        .map_err(ValueError::from)
}

/// Converts a single `Value` into `T` using an existing conversion session.
///
/// # Type Parameters
///
/// * `T` - Target type supported by the shared conversion layer.
///
/// # Parameters
///
/// * `value` - Source runtime value.
/// * `session` - Caller-owned session providing policy, limits, and budget.
///
/// # Returns
///
/// The converted target value.
///
/// # Errors
///
/// Returns a mapped missing, conversion, or budget error.
pub(super) fn convert_with_data_converter_in<T>(value: &Value, session: &mut ConversionSession<'_>) -> ValueResult<T>
where
    T: DataConversionTarget,
{
    data_converter_from_value(value)
        .to_in::<T>(session)
        .map_err(ValueError::from)
}
