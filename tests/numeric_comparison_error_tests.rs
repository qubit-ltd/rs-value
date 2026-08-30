// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::NumericComparisonError;

/// Verifies comparison errors retain operand-side type context.
#[test]
fn test_numeric_comparison_error_display_and_equality() {
    let cases = [
        (
            NumericComparisonError::LeftMissing {
                declared: DataType::Int32,
            },
            "left value is missing: declared type is int32",
        ),
        (
            NumericComparisonError::RightMissing {
                declared: DataType::Float64,
            },
            "right value is missing: declared type is float64",
        ),
        (
            NumericComparisonError::LeftNotNumeric {
                actual: DataType::String,
            },
            "left value is not numeric: string",
        ),
        (
            NumericComparisonError::RightNotNumeric { actual: DataType::Bool },
            "right value is not numeric: bool",
        ),
        (NumericComparisonError::LeftNaN, "left value is NaN"),
        (NumericComparisonError::RightNaN, "right value is NaN"),
        (NumericComparisonError::BothNaN, "both values are NaN"),
    ];

    for (error, expected) in cases {
        assert_eq!(error.to_string(), expected);
    }
}
