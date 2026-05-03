/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal conversion and interoperability implementations for `MultiValues`.
//!
//! This module keeps generic conversion logic (`to`, `to_list`, `to_value`, etc.)
//! while dispatch traits are implemented in dedicated `multi_values_*` modules.

use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConvertTo,
    DataConverter,
    DataConverters,
    DataListConversionError,
    DataType,
    ScalarStringDataConverters,
};

use crate::value_error::{
    ValueError,
    ValueResult,
};
use crate::{
    IntoValueDefault,
    Value,
};

use super::multi_values::MultiValues;

macro_rules! multi_values_to_value_match {
    ($value:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match $value {
            MultiValues::Empty(data_type) => Value::Empty(*data_type),
            $(
                MultiValues::$variant(values) => values
                    .first()
                    .cloned()
                    .map(Value::$variant)
                    .unwrap_or(Value::Empty($data_type)),
            )+
        }
    };
}

macro_rules! multi_values_merge_match {
    ($left:expr, $right:expr; $(($variant:ident, $type:ty, $data_type:expr)),+ $(,)?) => {
        match ($left, $right) {
            $(
                (MultiValues::$variant(values), MultiValues::$variant(other_values)) => {
                    values.extend_from_slice(other_values)
                }
            )+
            (slot @ MultiValues::Empty(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }
    };
}

// ============================================================================
// Inherent conversion APIs and `Value` interop
// ============================================================================

/// Maps a shared single-value conversion error into `ValueError`.
///
/// # Parameters
///
/// * `error` - Error returned by `DataConverter`.
///
/// # Returns
///
/// Returns the corresponding `ValueError` variant.
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

/// Maps a shared batch conversion error into `ValueError`.
///
/// # Parameters
///
/// * `error` - Error returned by `DataConverters`.
///
/// # Returns
///
/// Returns a `ValueError::ConversionError` whose message includes the failing
/// source element index and the underlying conversion error.
#[inline]
fn map_data_list_conversion_error(error: DataListConversionError) -> ValueError {
    let source = map_data_conversion_error(error.source);
    ValueError::ConversionError(format!(
        "Cannot convert value at index {}: {}",
        error.index, source
    ))
}

/// Converts the first item from a batch converter using conversion options.
///
/// # Type Parameters
///
/// * `T` - Target type.
/// * `I` - Iterator type wrapped by `DataConverters`.
///
/// # Parameters
///
/// * `values` - Batch converter containing source values.
/// * `options` - Conversion options forwarded to `qubit_datatype`.
///
/// # Returns
///
/// Returns the converted first value.
///
/// # Errors
///
/// Returns `ValueError::NoValue` for empty sources or the mapped single-value
/// conversion error for an invalid first source value.
#[inline]
fn convert_first_with<'a, T, I>(
    values: DataConverters<'a, I>,
    options: &DataConversionOptions,
) -> ValueResult<T>
where
    DataConverter<'a>: DataConvertTo<T>,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values
        .to_first_with(options)
        .map_err(map_data_conversion_error)
}

/// Converts every item from a batch converter using conversion options.
///
/// # Type Parameters
///
/// * `T` - Target element type.
/// * `I` - Iterator type wrapped by `DataConverters`.
///
/// # Parameters
///
/// * `values` - Batch converter containing source values.
/// * `options` - Conversion options forwarded to `qubit_datatype`.
///
/// # Returns
///
/// Returns converted values in the original order.
///
/// # Errors
///
/// Returns a mapped batch conversion error containing the failing source index.
#[inline]
fn convert_values_with<'a, T, I>(
    values: DataConverters<'a, I>,
    options: &DataConversionOptions,
) -> ValueResult<Vec<T>>
where
    DataConverter<'a>: DataConvertTo<T>,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values
        .to_vec_with(options)
        .map_err(map_data_list_conversion_error)
}

impl MultiValues {
    /// Converts the first stored value to `T`.
    ///
    /// Unlike [`Self::get_first`], this method uses shared `DataConverter`
    /// conversion rules instead of strict type matching. For example, a stored
    /// `String("1")` can be converted to `bool`.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Returns
    ///
    /// The converted first value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] when no value is stored, or a conversion
    /// error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        self.to_with(&DataConversionOptions::default())
    }

    /// Converts the first stored value to `T`, or returns `default` when no
    /// value is stored.
    #[inline]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self.to() {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts the first stored value to `T` using conversion options.
    ///
    /// A `MultiValues::String` containing exactly one string is treated as a
    /// scalar string source, so collection options can split it before taking
    /// the first converted item. Multiple stored string values are treated as
    /// an already-materialized list and are converted element by element.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Parameters
    ///
    /// * `options` - Conversion options forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// The converted first value.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::NoValue`] when no value is stored, or a conversion
    /// error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self {
            MultiValues::Empty(_) => Err(ValueError::NoValue),
            MultiValues::Bool(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Char(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Int8(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Int16(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Int32(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Int64(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Int128(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UInt8(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UInt16(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UInt32(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UInt64(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UInt128(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::IntSize(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::UIntSize(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Float32(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Float64(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::BigInteger(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::BigDecimal(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::String(v) if v.len() == 1 => {
                ScalarStringDataConverters::from(v[0].as_str())
                    .to_first_with(options)
                    .map_err(map_data_conversion_error)
            }
            MultiValues::String(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Date(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Time(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::DateTime(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Instant(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Duration(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Url(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::StringMap(v) => convert_first_with(DataConverters::from(v), options),
            MultiValues::Json(v) => convert_first_with(DataConverters::from(v), options),
        }
    }

    /// Converts the first stored value to `T` using conversion options, or
    /// returns `default` when no value is stored.
    #[inline]
    pub fn to_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self.to_with(options) {
            Err(ValueError::NoValue) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts all stored values to `T`.
    ///
    /// Unlike [`Self::get`], this method uses shared `DataConverter` conversion
    /// rules for every element instead of strict type matching. Empty values
    /// return an empty vector.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Returns
    ///
    /// A vector containing all converted values in the original order.
    ///
    /// # Errors
    ///
    /// Returns the first conversion error encountered while converting an
    /// element.
    pub fn to_list<T>(&self) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        self.to_list_with(&DataConversionOptions::default())
    }

    /// Converts all stored values to `T`, or returns `default` when the
    /// converted list is empty.
    #[inline]
    pub fn to_list_or<T>(&self, default: impl IntoValueDefault<Vec<T>>) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        let values = self.to_list()?;
        if values.is_empty() {
            Ok(default.into_value_default())
        } else {
            Ok(values)
        }
    }

    /// Converts all stored values to `T` using conversion options.
    ///
    /// A `MultiValues::String` containing exactly one string is treated as a
    /// scalar string source, so collection options can split it into items.
    /// Multiple stored string values are treated as an already-materialized
    /// list and are converted element by element.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `options` - Conversion options forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// A vector containing all converted values in the original order.
    ///
    /// # Errors
    ///
    /// Returns the first conversion error encountered while converting an
    /// element.
    pub fn to_list_with<T>(&self, options: &DataConversionOptions) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        match self {
            MultiValues::Empty(_) => Ok(Vec::new()),
            MultiValues::Bool(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Char(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Int8(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Int16(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Int32(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Int64(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Int128(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UInt8(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UInt16(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UInt32(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UInt64(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UInt128(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::IntSize(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::UIntSize(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Float32(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Float64(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::BigInteger(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::BigDecimal(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::String(v) if v.len() == 1 => {
                ScalarStringDataConverters::from(v[0].as_str())
                    .to_vec_with(options)
                    .map_err(map_data_list_conversion_error)
            }
            MultiValues::String(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Date(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Time(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::DateTime(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Instant(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Duration(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Url(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::StringMap(v) => convert_values_with(DataConverters::from(v), options),
            MultiValues::Json(v) => convert_values_with(DataConverters::from(v), options),
        }
    }

    /// Converts all stored values to `T` using conversion options, or returns
    /// `default` when the converted list is empty.
    #[inline]
    pub fn to_list_or_with<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
        options: &DataConversionOptions,
    ) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
    {
        let values = self.to_list_with(options)?;
        if values.is_empty() {
            Ok(default.into_value_default())
        } else {
            Ok(values)
        }
    }

    /// Convert to a single [`Value`] by taking the first element.
    ///
    /// If there is no element, returns `Value::Empty(self.data_type())`.
    ///
    /// # Returns
    ///
    /// Returns the first element wrapped as [`Value`], or an empty value
    /// preserving the current data type.
    pub fn to_value(&self) -> Value {
        for_each_multi_value_type!(multi_values_to_value_match, self)
    }

    /// Merge another multiple values
    ///
    /// Append all values from another multiple values to the current multiple values
    ///
    /// # Parameters
    ///
    /// * `other` - The multiple values to merge
    ///
    /// # Returns
    ///
    /// If types match, returns `Ok(())`; otherwise returns an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// let mut a = MultiValues::Int32(vec![1, 2]);
    /// let b = MultiValues::Int32(vec![3, 4]);
    /// a.merge(&b).unwrap();
    /// assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);
    /// ```
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }
        if other.count() == 0 {
            return Ok(());
        }

        for_each_multi_value_type!(multi_values_merge_match, self, other);

        Ok(())
    }
}

impl Default for MultiValues {
    #[inline]
    fn default() -> Self {
        MultiValues::Empty(DataType::String)
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        match value {
            Value::Empty(dt) => MultiValues::Empty(dt),
            Value::Bool(v) => MultiValues::Bool(vec![v]),
            Value::Char(v) => MultiValues::Char(vec![v]),
            Value::Int8(v) => MultiValues::Int8(vec![v]),
            Value::Int16(v) => MultiValues::Int16(vec![v]),
            Value::Int32(v) => MultiValues::Int32(vec![v]),
            Value::Int64(v) => MultiValues::Int64(vec![v]),
            Value::Int128(v) => MultiValues::Int128(vec![v]),
            Value::UInt8(v) => MultiValues::UInt8(vec![v]),
            Value::UInt16(v) => MultiValues::UInt16(vec![v]),
            Value::UInt32(v) => MultiValues::UInt32(vec![v]),
            Value::UInt64(v) => MultiValues::UInt64(vec![v]),
            Value::UInt128(v) => MultiValues::UInt128(vec![v]),
            Value::Float32(v) => MultiValues::Float32(vec![v]),
            Value::Float64(v) => MultiValues::Float64(vec![v]),
            Value::String(v) => MultiValues::String(vec![v]),
            Value::Date(v) => MultiValues::Date(vec![v]),
            Value::Time(v) => MultiValues::Time(vec![v]),
            Value::DateTime(v) => MultiValues::DateTime(vec![v]),
            Value::Instant(v) => MultiValues::Instant(vec![v]),
            Value::BigInteger(v) => MultiValues::BigInteger(vec![v]),
            Value::BigDecimal(v) => MultiValues::BigDecimal(vec![v]),
            Value::IntSize(v) => MultiValues::IntSize(vec![v]),
            Value::UIntSize(v) => MultiValues::UIntSize(vec![v]),
            Value::Duration(v) => MultiValues::Duration(vec![v]),
            Value::Url(v) => MultiValues::Url(vec![v]),
            Value::StringMap(v) => MultiValues::StringMap(vec![v]),
            Value::Json(v) => MultiValues::Json(vec![v]),
        }
    }
}
