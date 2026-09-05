// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Processing Error Types
//!
//! Defines various errors that may occur during value processing.

#[cfg(all(feature = "converter", feature = "json"))]
use qubit_budget::MeasuredBudgetError;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::ConversionResource;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionError;
#[cfg(feature = "converter")]
use qubit_datatype::DataConversionErrorKind;
#[cfg(feature = "converter")]
use qubit_datatype::DataListConversionError;
use qubit_datatype::DataType;
use thiserror::Error;

use crate::ValueMissing;

/// Value processing error type
///
/// Defines various error conditions that may occur during value operations.
/// Downstream matches must include a wildcard arm because this enum is
/// non-exhaustive and may gain new error variants.
///
/// # Features
///
/// - Type mismatch error
/// - Structured missing-value errors
/// - Structured single-value conversion errors when `converter` is enabled
/// - Structured list conversion errors, including the failing item index, when
///   `converter` is enabled
///
/// # Examples
///
/// ```rust
/// use qubit_datatype::DataType;
/// use qubit_value::{ValueError, ValueMissing};
///
/// let error = ValueError::Missing(ValueMissing::UnsetScalar {
///     data_type: DataType::String,
/// });
/// assert_eq!(error.to_string(), "Missing value: unset scalar with declared type string");
/// ```
#[non_exhaustive]
#[must_use]
#[derive(Debug, Clone, Error)]
pub enum ValueError {
    /// Resource rejection before materializing a natural JSON projection.
    #[cfg(all(feature = "converter", feature = "json"))]
    #[error("JSON projection limit for {data_type} at collection index {source_index:?}: {source}")]
    JsonProjectionLimit {
        /// Runtime scalar or collection element type being projected.
        data_type: DataType,
        /// Collection element index, or `None` for a scalar or outer shape.
        source_index: Option<usize>,
        /// Exact rejected resource measurement, including its configured bound.
        #[source]
        source: MeasuredBudgetError<ConversionResource, u64>,
    },

    /// No concrete item is available from typed runtime storage or conversion.
    #[error("Missing value: {0}")]
    Missing(
        /// Structured typed storage state that caused the missing value.
        ValueMissing,
    ),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, actual {actual}")]
    TypeMismatch {
        /// Expected data type
        expected: DataType,
        /// Actual data type
        actual: DataType,
    },

    /// Error returned by the shared single-value conversion layer.
    #[cfg(feature = "converter")]
    #[error("Conversion error: {0}")]
    Conversion(
        /// Structured conversion failure from `qubit-datatype`.
        #[source]
        DataConversionError,
    ),

    /// Error returned by the shared list conversion layer.
    #[cfg(feature = "converter")]
    #[error("List conversion error: {0}")]
    ListConversion(
        /// Structured list conversion failure, including the source index.
        #[source]
        DataListConversionError,
    ),
}

impl PartialEq for ValueError {
    /// Compares structured error facts without depending on diagnostic text.
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Missing(left), Self::Missing(right)) => left == right,
            (
                Self::TypeMismatch {
                    expected: le,
                    actual: la,
                },
                Self::TypeMismatch {
                    expected: re,
                    actual: ra,
                },
            ) => le == re && la == ra,
            #[cfg(feature = "converter")]
            (Self::Conversion(left), Self::Conversion(right)) => left == right,
            #[cfg(feature = "converter")]
            (Self::ListConversion(left), Self::ListConversion(right)) => left == right,
            #[cfg(all(feature = "converter", feature = "json"))]
            (
                Self::JsonProjectionLimit {
                    data_type: lt,
                    source_index: li,
                    source: ls,
                },
                Self::JsonProjectionLimit {
                    data_type: rt,
                    source_index: ri,
                    source: rs,
                },
            ) => {
                lt == rt
                    && li == ri
                    && ls.resource() == rs.resource()
                    && ls.budget_error() == rs.budget_error()
                    && ls.quantity_error() == rs.quantity_error()
            }
            _ => false,
        }
    }
}

impl Eq for ValueError {}

impl ValueError {
    /// Reports whether this error describes a missing value.
    ///
    /// # Returns
    ///
    /// `true` only for [`Self::Missing`].
    #[must_use]
    #[inline(always)]
    pub const fn is_missing(&self) -> bool {
        matches!(self, Self::Missing(_))
    }

    /// Returns the structured missing-value reason, when present.
    ///
    /// # Returns
    ///
    /// `Some(reason)` for [`Self::Missing`] and `None` for every other variant.
    #[must_use]
    #[inline(always)]
    pub const fn missing(&self) -> Option<&ValueMissing> {
        match self {
            Self::Missing(missing) => Some(missing),
            Self::TypeMismatch { .. } => None,
            #[cfg(all(feature = "converter", feature = "json"))]
            Self::JsonProjectionLimit { .. } => None,
            #[cfg(feature = "converter")]
            Self::Conversion(_) | Self::ListConversion(_) => None,
        }
    }
}

#[cfg(feature = "converter")]
impl From<DataConversionError> for ValueError {
    fn from(error: DataConversionError) -> Self {
        if error.is_missing()
            && let Some(from) = error.from_type()
        {
            return Self::Missing(ValueMissing::Conversion {
                from,
                to: error.to_type(),
            });
        }
        if error.kind() == DataConversionErrorKind::EmptyCollection {
            return Self::Missing(ValueMissing::EmptyCollectionConversion { to: error.to_type() });
        }
        Self::Conversion(error)
    }
}

#[cfg(feature = "converter")]
impl From<DataListConversionError> for ValueError {
    fn from(error: DataListConversionError) -> Self {
        let (source_index, source) = error.into_parts();
        if source.is_missing()
            && let Some(from) = source.from_type()
        {
            return Self::Missing(ValueMissing::CollectionItem {
                source_index,
                from,
                to: source.to_type(),
            });
        }
        if source.kind() == DataConversionErrorKind::EmptyCollection {
            return Self::Missing(ValueMissing::EmptyCollectionConversion { to: source.to_type() });
        }
        Self::ListConversion(DataListConversionError::new(source_index, source))
    }
}

/// Result returned by value processing operations.
///
/// # Type Parameters
///
/// * `T` - Successful value returned by the operation.
///
/// # Examples
///
/// ```
/// use qubit_value::{Value, ValueResult};
///
/// fn read_int(value: &Value) -> ValueResult<i32> {
///     value.get_int32()
/// }
///
/// assert_eq!(read_int(&Value::from(42_i32)).unwrap(), 42);
/// ```
pub type ValueResult<T> = Result<T, ValueError>;
