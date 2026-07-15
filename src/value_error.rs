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

/// Value processing error type
///
/// Defines various error conditions that may occur during value operations.
///
/// # Features
///
/// - Type mismatch error
/// - No value error
/// - Structured single-value conversion errors when `converter` is enabled
/// - Structured list conversion errors, including the failing item index, when
///   `converter` is enabled
///
/// # Example
///
/// ```rust
/// use qubit_value::ValueError;
///
/// let error = ValueError::NoValue;
/// assert_eq!(error.to_string(), "No value");
/// ```
#[non_exhaustive]
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ValueError {
    /// No value
    #[error("No value")]
    NoValue,

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
    DataConversion(#[from] DataConversionError),

    /// Error returned by the shared list conversion layer.
    #[cfg(feature = "converter")]
    #[error("Data list conversion error: {0}")]
    DataListConversion(#[from] DataListConversionError),
}

/// Value processing result type
pub type ValueResult<T> = Result<T, ValueError>;
