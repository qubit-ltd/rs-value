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
    let left = NumericComparisonError::LeftNotNumeric {
        actual: DataType::String,
    };
    let right = NumericComparisonError::RightNotNumeric {
        actual: DataType::Bool,
    };
    assert_eq!(left.to_string(), "left value is not numeric: string");
    assert_eq!(right.to_string(), "right value is not numeric: bool");
    assert_eq!(
        NumericComparisonError::UnorderedNaN.to_string(),
        "NaN is unordered"
    );
}
