// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Errors that explain why runtime values cannot be numerically ordered.

use qubit_datatype::DataType;
use thiserror::Error;

/// Describes why two [`crate::Value`] instances cannot be numerically ordered.
///
/// # Examples
///
/// ```
/// use qubit_datatype::NumericComparisonPolicy;
/// use qubit_value::{NumericComparisonError, Value};
///
/// let error = Value::from("text")
///     .numeric_cmp(&Value::from(1_i32), NumericComparisonPolicy::Exact)
///     .unwrap_err();
/// assert!(matches!(error, NumericComparisonError::LeftNotNumeric { .. }));
/// ```
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum NumericComparisonError {
    /// The left operand is unset but retains a declared type.
    #[error("left value is missing: declared type is {declared}")]
    LeftMissing {
        /// Declared runtime type of the unset left operand.
        declared: DataType,
    },
    /// The right operand is unset but retains a declared type.
    #[error("right value is missing: declared type is {declared}")]
    RightMissing {
        /// Declared runtime type of the unset right operand.
        declared: DataType,
    },
    /// The concrete left operand is not numeric.
    #[error("left value is not numeric: {actual}")]
    LeftNotNumeric {
        /// Actual runtime type of the left operand.
        actual: DataType,
    },
    /// The concrete right operand is not numeric.
    #[error("right value is not numeric: {actual}")]
    RightNotNumeric {
        /// Actual runtime type of the right operand.
        actual: DataType,
    },
    /// Only the left operand is NaN.
    #[error("left value is NaN")]
    LeftNaN,
    /// Only the right operand is NaN.
    #[error("right value is NaN")]
    RightNaN,
    /// Both operands are NaN.
    #[error("both values are NaN")]
    BothNaN,
}
