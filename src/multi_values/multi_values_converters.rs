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
        match self {
            MultiValues::Empty(dt) => Value::Empty(*dt),
            MultiValues::Bool(v) => v
                .first()
                .copied()
                .map(Value::Bool)
                .unwrap_or(Value::Empty(DataType::Bool)),
            MultiValues::Char(v) => v
                .first()
                .copied()
                .map(Value::Char)
                .unwrap_or(Value::Empty(DataType::Char)),
            MultiValues::Int8(v) => v
                .first()
                .copied()
                .map(Value::Int8)
                .unwrap_or(Value::Empty(DataType::Int8)),
            MultiValues::Int16(v) => v
                .first()
                .copied()
                .map(Value::Int16)
                .unwrap_or(Value::Empty(DataType::Int16)),
            MultiValues::Int32(v) => v
                .first()
                .copied()
                .map(Value::Int32)
                .unwrap_or(Value::Empty(DataType::Int32)),
            MultiValues::Int64(v) => v
                .first()
                .copied()
                .map(Value::Int64)
                .unwrap_or(Value::Empty(DataType::Int64)),
            MultiValues::Int128(v) => v
                .first()
                .copied()
                .map(Value::Int128)
                .unwrap_or(Value::Empty(DataType::Int128)),
            MultiValues::UInt8(v) => v
                .first()
                .copied()
                .map(Value::UInt8)
                .unwrap_or(Value::Empty(DataType::UInt8)),
            MultiValues::UInt16(v) => v
                .first()
                .copied()
                .map(Value::UInt16)
                .unwrap_or(Value::Empty(DataType::UInt16)),
            MultiValues::UInt32(v) => v
                .first()
                .copied()
                .map(Value::UInt32)
                .unwrap_or(Value::Empty(DataType::UInt32)),
            MultiValues::UInt64(v) => v
                .first()
                .copied()
                .map(Value::UInt64)
                .unwrap_or(Value::Empty(DataType::UInt64)),
            MultiValues::UInt128(v) => v
                .first()
                .copied()
                .map(Value::UInt128)
                .unwrap_or(Value::Empty(DataType::UInt128)),
            MultiValues::IntSize(v) => v
                .first()
                .copied()
                .map(Value::IntSize)
                .unwrap_or(Value::Empty(DataType::IntSize)),
            MultiValues::UIntSize(v) => v
                .first()
                .copied()
                .map(Value::UIntSize)
                .unwrap_or(Value::Empty(DataType::UIntSize)),
            MultiValues::Float32(v) => v
                .first()
                .copied()
                .map(Value::Float32)
                .unwrap_or(Value::Empty(DataType::Float32)),
            MultiValues::Float64(v) => v
                .first()
                .copied()
                .map(Value::Float64)
                .unwrap_or(Value::Empty(DataType::Float64)),
            MultiValues::BigInteger(v) => v
                .first()
                .cloned()
                .map(Value::BigInteger)
                .unwrap_or(Value::Empty(DataType::BigInteger)),
            MultiValues::BigDecimal(v) => v
                .first()
                .cloned()
                .map(Value::BigDecimal)
                .unwrap_or(Value::Empty(DataType::BigDecimal)),
            MultiValues::String(v) => v
                .first()
                .cloned()
                .map(Value::String)
                .unwrap_or(Value::Empty(DataType::String)),
            MultiValues::Date(v) => v
                .first()
                .copied()
                .map(Value::Date)
                .unwrap_or(Value::Empty(DataType::Date)),
            MultiValues::Time(v) => v
                .first()
                .copied()
                .map(Value::Time)
                .unwrap_or(Value::Empty(DataType::Time)),
            MultiValues::DateTime(v) => v
                .first()
                .copied()
                .map(Value::DateTime)
                .unwrap_or(Value::Empty(DataType::DateTime)),
            MultiValues::Instant(v) => v
                .first()
                .copied()
                .map(Value::Instant)
                .unwrap_or(Value::Empty(DataType::Instant)),
            MultiValues::Duration(v) => v
                .first()
                .copied()
                .map(Value::Duration)
                .unwrap_or(Value::Empty(DataType::Duration)),
            MultiValues::Url(v) => v
                .first()
                .cloned()
                .map(Value::Url)
                .unwrap_or(Value::Empty(DataType::Url)),
            MultiValues::StringMap(v) => v
                .first()
                .cloned()
                .map(Value::StringMap)
                .unwrap_or(Value::Empty(DataType::StringMap)),
            MultiValues::Json(v) => v
                .first()
                .cloned()
                .map(Value::Json)
                .unwrap_or(Value::Empty(DataType::Json)),
        }
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

        match (self, other) {
            (MultiValues::Bool(v), MultiValues::Bool(o)) => v.extend_from_slice(o),
            (MultiValues::Char(v), MultiValues::Char(o)) => v.extend_from_slice(o),
            (MultiValues::Int8(v), MultiValues::Int8(o)) => v.extend_from_slice(o),
            (MultiValues::Int16(v), MultiValues::Int16(o)) => v.extend_from_slice(o),
            (MultiValues::Int32(v), MultiValues::Int32(o)) => v.extend_from_slice(o),
            (MultiValues::Int64(v), MultiValues::Int64(o)) => v.extend_from_slice(o),
            (MultiValues::Int128(v), MultiValues::Int128(o)) => v.extend_from_slice(o),
            (MultiValues::UInt8(v), MultiValues::UInt8(o)) => v.extend_from_slice(o),
            (MultiValues::UInt16(v), MultiValues::UInt16(o)) => v.extend_from_slice(o),
            (MultiValues::UInt32(v), MultiValues::UInt32(o)) => v.extend_from_slice(o),
            (MultiValues::UInt64(v), MultiValues::UInt64(o)) => v.extend_from_slice(o),
            (MultiValues::UInt128(v), MultiValues::UInt128(o)) => v.extend_from_slice(o),
            (MultiValues::Float32(v), MultiValues::Float32(o)) => v.extend_from_slice(o),
            (MultiValues::Float64(v), MultiValues::Float64(o)) => v.extend_from_slice(o),
            (MultiValues::String(v), MultiValues::String(o)) => v.extend_from_slice(o),
            (MultiValues::Date(v), MultiValues::Date(o)) => v.extend_from_slice(o),
            (MultiValues::Time(v), MultiValues::Time(o)) => v.extend_from_slice(o),
            (MultiValues::DateTime(v), MultiValues::DateTime(o)) => v.extend_from_slice(o),
            (MultiValues::Instant(v), MultiValues::Instant(o)) => v.extend_from_slice(o),
            (MultiValues::BigInteger(v), MultiValues::BigInteger(o)) => v.extend_from_slice(o),
            (MultiValues::BigDecimal(v), MultiValues::BigDecimal(o)) => v.extend_from_slice(o),
            (MultiValues::IntSize(v), MultiValues::IntSize(o)) => v.extend_from_slice(o),
            (MultiValues::UIntSize(v), MultiValues::UIntSize(o)) => v.extend_from_slice(o),
            (MultiValues::Duration(v), MultiValues::Duration(o)) => v.extend_from_slice(o),
            (MultiValues::Url(v), MultiValues::Url(o)) => v.extend_from_slice(o),
            (MultiValues::StringMap(v), MultiValues::StringMap(o)) => v.extend(o.iter().cloned()),
            (MultiValues::Json(v), MultiValues::Json(o)) => v.extend(o.iter().cloned()),
            (slot @ MultiValues::Empty(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }

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
