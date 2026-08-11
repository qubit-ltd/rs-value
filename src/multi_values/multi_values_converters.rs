// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Internal conversion and interoperability implementations for `MultiValues`.
//!
//! This module keeps generic conversion logic (`to_first` and `to_list`).

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionSession;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataConversionTarget;
use qubit_datatype::DataConverter;
use qubit_datatype::DataConverters;

use super::multi_values::MultiValues;
use super::multi_values::MultiValuesRepr;
use crate::IntoValueDefault;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

macro_rules! multi_values_convert_first_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    convert_first_with(DataConverters::from(values), $policy, $limits)
                }
            )+
        }
    };
}

macro_rules! multi_values_convert_list_match {
    ($value:expr, $policy:expr, $limits:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    convert_values_with(DataConverters::from(values), $policy, $limits)
                }
            )+
        }
    };
}

macro_rules! multi_values_convert_first_in_match {
    ($value:expr, $session:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    DataConverters::from(values)
                        .to_first_in($session)
                        .map_err(ValueError::from)
                }
            )+
        }
    };
}

macro_rules! multi_values_convert_list_in_match {
    ($value:expr, $session:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?) => {
        match &$value.repr {
            MultiValuesRepr::Unset(from) => {
                Err(DataConversionError::missing(*from, T::DATA_TYPE).into())
            }
            $(
                $(#[$cfg])*
                MultiValuesRepr::$variant(values) => {
                    DataConverters::from(values)
                        .to_vec_in($session)
                        .map_err(ValueError::from)
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
#[inline(always)]
fn convert_first_with<'a, T, I>(
    values: DataConverters<I>,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<T>
where
    T: DataConversionTarget,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values
        .to_first_with(policy, limits)
        .map_err(ValueError::from)
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
#[inline(always)]
fn convert_values_with<'a, T, I>(
    values: DataConverters<I>,
    policy: &ConversionPolicy,
    limits: &ConversionLimits,
) -> ValueResult<Vec<T>>
where
    T: DataConversionTarget,
    I: Iterator,
    I::Item: Into<DataConverter<'a>>,
{
    values.to_vec_with(policy, limits).map_err(ValueError::from)
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
    #[inline(always)]
    pub fn to_first<T>(&self) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        self.to_first_with(
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
    }

    /// Converts the first stored value to `T`, or returns `default` when the
    /// container is unset or conversion reports a missing value.
    ///
    /// A concrete empty collection remains an error and does not use the
    /// default.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target type.
    ///
    /// # Parameters
    ///
    /// * `default` - Value returned for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted first value, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns an empty-collection error for a concrete empty vector, or a
    /// conversion error when the first value cannot be converted to `T`.
    #[inline]
    pub fn to_first_or<T>(
        &self,
        default: impl IntoValueDefault<T>,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_first() {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts the first value or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted first item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves empty-collection and concrete-value conversion errors without
    /// invoking the callback.
    #[inline]
    pub fn to_first_or_else<T, F>(&self, default: F) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_first() {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default())
            }
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
    pub fn to_first_with<T>(
        &self,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(
            multi_values_convert_first_match,
            self,
            policy,
            limits
        )
    }

    /// Converts the first stored value using an existing conversion session.
    pub fn to_first_in<T>(
        &self,
        session: &mut ConversionSession<'_>,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_first_in_match, self, session)
    }

    /// Converts the first stored value to `T` using conversion options, or
    /// returns `default` when storage is unset or conversion reports a missing
    /// value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized value used for unset storage or a
    ///   conversion-missing result.
    /// * `options` - Conversion options forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// The converted first item, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns an empty-collection error or a conversion error for concrete
    /// values that cannot be converted under `options`.
    #[inline]
    pub fn to_first_or_with<T>(
        &self,
        default: impl IntoValueDefault<T>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
    {
        match self.to_first_with(policy, limits) {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts the first value with `options`, or calls `default` when storage
    /// is unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target conversion type.
    /// * `F` - Deferred fallback producing `T`.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    /// * `options` - Conversion options forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted first item or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_first_or_else_with<T, F>(
        &self,
        default: F,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<T>
    where
        T: DataConversionTarget,
        F: FnOnce() -> T,
    {
        match self.to_first_with(policy, limits) {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default())
            }
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
        T: DataConversionTarget,
    {
        self.to_list_with(
            ConversionPolicy::default_ref(),
            ConversionLimits::default_ref(),
        )
    }

    /// Converts all stored values to `T`, or returns `default` when storage is
    /// unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used for unset storage or a
    ///   conversion-missing result.
    ///
    /// # Returns
    ///
    /// All converted items, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns the first item conversion error for concrete storage.
    #[inline]
    pub fn to_list_or<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self.to_list() {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts all values or calls `default` when storage is unset or
    /// conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element conversion type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    ///
    /// # Returns
    ///
    /// The converted list or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_list_or_else<T, F>(&self, default: F) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
        F: FnOnce() -> Vec<T>,
    {
        match self.to_list() {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default())
            }
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
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(
            multi_values_convert_list_match,
            self,
            policy,
            limits
        )
    }

    /// Converts every stored value using an existing conversion session.
    pub fn to_list_in<T>(
        &self,
        session: &mut ConversionSession<'_>,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        for_each_value_type!(multi_values_convert_list_in_match, self, session)
    }

    /// Converts all stored values to `T` using conversion options, or returns
    /// `default` when storage is unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element type.
    ///
    /// # Parameters
    ///
    /// * `default` - Lazily materialized list used for unset storage or a
    ///   conversion-missing result.
    /// * `options` - Conversion options forwarded to `qubit_datatype`.
    ///
    /// # Returns
    ///
    /// All converted items, or `default` for unset or conversion-missing
    /// storage.
    ///
    /// # Errors
    ///
    /// Returns the first item conversion error for concrete storage.
    #[inline]
    pub fn to_list_or_with<T>(
        &self,
        default: impl IntoValueDefault<Vec<T>>,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
    {
        match self.to_list_with(policy, limits) {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default.into_value_default())
            }
            result => result,
        }
    }

    /// Converts all values with `options`, or calls `default` when storage is
    /// unset or conversion reports a missing value.
    ///
    /// # Type Parameters
    ///
    /// * `T` - Target element conversion type.
    /// * `F` - Deferred fallback producing the complete list.
    ///
    /// # Parameters
    ///
    /// * `default` - Callback invoked for unset storage or a conversion-missing
    ///   result.
    /// * `options` - Conversion options forwarded to the shared converter.
    ///
    /// # Returns
    ///
    /// The converted list or the callback result.
    ///
    /// # Errors
    ///
    /// Preserves concrete-value conversion errors without invoking the
    /// callback.
    #[inline]
    pub fn to_list_or_else_with<T, F>(
        &self,
        default: F,
        policy: &ConversionPolicy,
        limits: &ConversionLimits,
    ) -> ValueResult<Vec<T>>
    where
        T: DataConversionTarget,
        F: FnOnce() -> Vec<T>,
    {
        match self.to_list_with(policy, limits) {
            Err(ValueError::Missing(missing))
                if missing.is_defaultable_for_conversion() =>
            {
                Ok(default())
            }
            result => result,
        }
    }
}
