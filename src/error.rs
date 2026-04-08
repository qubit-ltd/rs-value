/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Processing Error Types
//!
//! Defines various errors that may occur during value processing.
//!
//! # Author
//!
//! Haixing Hu

use qubit_common::lang::DataType;
use thiserror::Error;

/// Value processing error type
///
/// Defines various error conditions that may occur during value operations.
///
/// # Features
///
/// - Type mismatch error
/// - No value error
/// - Type conversion failure error
/// - Conversion error
///
/// # Example
///
/// ```rust,ignore
/// use common_rs::util::value::{ValueError, DataType};
///
/// let error = ValueError::NoValue;
/// assert_eq!(error.to_string(), "No value");
/// ```
///
/// # Author
///
/// Haixing Hu
///
#[derive(Debug, Error)]
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

    /// Type conversion failed
    #[error("Type conversion failed: from {from} to {to}")]
    ConversionFailed {
        /// Source data type
        from: DataType,
        /// Target data type
        to: DataType,
    },

    /// Conversion error (with detailed information)
    #[error("Conversion error: {0}")]
    ConversionError(String),

    /// Index out of bounds
    #[error("Index out of bounds: index {index}, length {len}")]
    IndexOutOfBounds {
        /// Accessed index
        index: usize,
        /// Actual length
        len: usize,
    },

    /// JSON serialization error
    #[error("JSON serialization error: {0}")]
    JsonSerializationError(String),

    /// JSON deserialization error
    #[error("JSON deserialization error: {0}")]
    JsonDeserializationError(String),
}

/// Value processing result type
pub type ValueResult<T> = Result<T, ValueError>;
