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

use qubit_datatype::DataType;
#[cfg(feature = "converter")]
use qubit_datatype::{
    DataConversionError,
    DataListConversionError,
};
use thiserror::Error;

use crate::ValueAbsence;

/// Value processing error type
///
/// Defines various error conditions that may occur during value operations.
/// Downstream matches must include a wildcard arm because this enum is
/// non-exhaustive and may gain new error variants.
///
/// # Features
///
/// - Type mismatch error
/// - No value error
/// - Structured single-value conversion errors when `converter` is enabled
/// - Structured list conversion errors, including the failing item index, when
///   `converter` is enabled
///
/// # Examples
///
/// ```rust
/// use qubit_datatype::DataType;
/// use qubit_value::{ValueAbsence, ValueError};
///
/// let error = ValueError::NoValue(ValueAbsence::UnsetScalar {
///     data_type: DataType::String,
/// });
/// assert_eq!(error.to_string(), "No value: unset scalar with declared type string");
/// ```
#[non_exhaustive]
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum ValueError {
    /// No concrete item is available from typed runtime storage.
    #[error("No value: {0}")]
    NoValue(
        /// Structured typed storage state that caused the absence.
        ValueAbsence,
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
    #[error("Data conversion error: {0}")]
    DataConversion(
        /// Structured conversion failure from `qubit-datatype`.
        #[from]
        DataConversionError,
    ),

    /// Error returned by the shared list conversion layer.
    #[cfg(feature = "converter")]
    #[error("Data list conversion error: {0}")]
    DataListConversion(
        /// Structured list conversion failure, including the source index.
        #[from]
        DataListConversionError,
    ),
}

/// Value processing result type
pub type ValueResult<T> = Result<T, ValueError>;
