// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use thiserror::Error;

/// Describes why two [`crate::Value`] instances cannot be numerically ordered.
#[must_use]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[non_exhaustive]
pub enum NumericComparisonError {
    /// The left operand is not a concrete numeric value.
    #[error("left value is not numeric: {actual}")]
    LeftNotNumeric {
        /// Actual runtime type of the left operand.
        actual: DataType,
    },
    /// The right operand is not a concrete numeric value.
    #[error("right value is not numeric: {actual}")]
    RightNotNumeric {
        /// Actual runtime type of the right operand.
        actual: DataType,
    },
    /// At least one operand is NaN and therefore unordered.
    #[error("NaN is unordered")]
    UnorderedNaN,
}
