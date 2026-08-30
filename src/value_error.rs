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
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValueError {
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
pub type ValueResult<T> = Result<T, ValueError>;
