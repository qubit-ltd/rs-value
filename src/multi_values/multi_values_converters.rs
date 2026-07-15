// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal conversion and interoperability implementations for `MultiValues`.
//!
//! This module keeps generic conversion logic (`to` and `to_list`).

use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConvertTo,
    DataConverter,
    DataConverters,
    DataTypeOf,
};

use crate::IntoValueDefault;
use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;

macro_rules! multi_values_convert_first_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(from) => Err(DataConversionError::Missing {
                from: *from,
                to: T::DATA_TYPE,
            }
            .into()),
            $(
                $(#[$cfg])*
                MultiValues::$variant(values) => {
                    convert_first_with(DataConverters::from(values), $options)
                }
            )+
        }
    };
}

macro_rules! multi_values_convert_list_match {
    ($value:expr, $options:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match $value {
            MultiValues::Unset(from) => Err(DataConversionError::Missing {
                from: *from,
                to: T::DATA_TYPE,
            }
            .into()),
            $(
                $(#[$cfg])*
                MultiValues::$variant(values) => {
                    convert_values_with(DataConverters::from(values), $options)
                }
            )+
        }
    };
}

// ============================================================================
// Inherent conversion APIs
// ============================================================================

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
/// Returns the mapped single-value conversion error for an empty source or an
/// invalid first source value.
#[inline]
fn convert_first_with<'a, T, I>(
    values: DataConverters<I>,
    options: &DataConversionOptions,
) -> ValueResult<T>
where
    DataConverter<'a>: DataConvertTo<T>,
    T: DataTypeOf,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values.to_first_with(options).map_err(ValueError::from)
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
    values: DataConverters<I>,
    options: &DataConversionOptions,
) -> ValueResult<Vec<T>>
where
    DataConverter<'a>: DataConvertTo<T>,
    T: DataTypeOf,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values.to_vec_with(options).map_err(ValueError::from)
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
    /// Returns a structured missing-value conversion error when the container
    /// is unset, an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to<T>(&self) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        self.to_with(DataConversionOptions::default_ref())
    }

    /// Converts the first stored value to `T`, or returns `default` when no
    /// value is stored.
    #[inline]
    pub fn to_or<T>(&self, default: impl IntoValueDefault<T>) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        match self.to() {
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts the first stored value to `T` using conversion options.
    ///
    /// Stored strings are collection items and are never split again by scalar
    /// string collection options.
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
    /// Returns a structured missing-value conversion error when the container
    /// is unset, an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to_with<T>(&self, options: &DataConversionOptions) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        for_each_value_type!(multi_values_convert_first_match, self, options)
    }

    /// Converts the first stored value to `T` using conversion options, or
    /// returns `default` when the container is unset.
    #[inline]
    pub fn to_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        options: &DataConversionOptions,
    ) -> ValueResult<T>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        match self.to_with(options) {
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts all stored values to `T`.
    ///
    /// Unlike [`Self::get`], this method uses shared `DataConverter` conversion
    /// rules for every element instead of strict type matching. A concrete
    /// empty vector returns an empty vector; an unset container reports a
    /// missing-value conversion error.
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
        T: DataTypeOf,
    {
        self.to_list_with(DataConversionOptions::default_ref())
    }

    /// Converts all stored values to `T`, or returns `default` when the
    /// container is unset.
    #[inline]
    pub fn to_list_or<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
    ) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        match self.to_list() {
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
            result => result,
        }
    }

    /// Converts all stored values to `T` using conversion options.
    ///
    /// Stored strings are collection items and are never split again by scalar
    /// string collection options.
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
    pub fn to_list_with<T>(
        &self,
        options: &DataConversionOptions,
    ) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        for_each_value_type!(multi_values_convert_list_match, self, options)
    }

    /// Converts all stored values to `T` using conversion options, or returns
    /// `default` when the container is unset.
    #[inline]
    pub fn to_list_or_with<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
        options: &DataConversionOptions,
    ) -> ValueResult<Vec<T>>
    where
        for<'a> DataConverter<'a>: DataConvertTo<T>,
        T: DataTypeOf,
    {
        match self.to_list_with(options) {
            Err(ValueError::DataConversion(DataConversionError::Missing {
                ..
            })) => Ok(default.into_value_default()),
            result => result,
        }
    }
}
