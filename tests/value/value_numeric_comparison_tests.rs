// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cmp::Ordering;
#[cfg(feature = "big-number")]
use std::str::FromStr;

#[cfg(feature = "big-number")]
use bigdecimal::BigDecimal;
#[cfg(feature = "big-number")]
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::NumericComparisonError;
use qubit_value::Value;

/// Verifies numeric equality is distinct from representation identity.
#[test]
fn test_value_numeric_cmp_compares_across_numeric_variants() {
    assert_ne!(Value::Int32(1), Value::Int64(1));
    assert_eq!(
        Value::Int32(1).numeric_cmp(&Value::Int64(1), NumericComparisonPolicy::Exact),
        Ok(Ordering::Equal)
    );
    assert_eq!(
        Value::Int128(-1).numeric_cmp(&Value::UInt128(0), NumericComparisonPolicy::Exact),
        Ok(Ordering::Less)
    );
}

/// Verifies exact and approximate decimal/float policy behavior.
#[cfg(feature = "big-number")]
#[test]
fn test_value_numeric_cmp_applies_explicit_policy() {
    let decimal = Value::BigDecimal(BigDecimal::from_str("0.1").unwrap());
    let float = Value::Float64(0.1);
    assert_eq!(
        decimal.numeric_cmp(&float, NumericComparisonPolicy::Exact),
        Ok(Ordering::Less)
    );
    assert_eq!(
        decimal.numeric_cmp(&float, NumericComparisonPolicy::Approximate),
        Ok(Ordering::Equal)
    );
}

/// Verifies missing and non-numeric operands retain their side and type.
#[test]
fn test_value_numeric_cmp_distinguishes_missing_and_non_numeric_operands() {
    assert_eq!(
        Value::Unset(DataType::Int32).numeric_cmp(&Value::Int32(1), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftMissing {
            declared: DataType::Int32,
        }),
    );
    assert_eq!(
        Value::Int32(1).numeric_cmp(
            &Value::Unset(DataType::Float64),
            NumericComparisonPolicy::Exact,
        ),
        Err(NumericComparisonError::RightMissing {
            declared: DataType::Float64,
        }),
    );
    assert_eq!(
        Value::Unset(DataType::Int32).numeric_cmp(
            &Value::Unset(DataType::Int64),
            NumericComparisonPolicy::Exact,
        ),
        Err(NumericComparisonError::LeftMissing {
            declared: DataType::Int32,
        }),
    );
    assert_eq!(
        Value::String("x".to_owned()).numeric_cmp(&Value::Int32(1), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftNotNumeric {
            actual: DataType::String,
        }),
    );
    assert_eq!(
        Value::Int32(1).numeric_cmp(&Value::Bool(true), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNotNumeric {
            actual: DataType::Bool,
        }),
    );
}

/// Verifies NaN positions are classified after concrete operand types.
#[test]
fn test_value_numeric_cmp_reports_nan_position_after_type_validation() {
    let nan = Value::Float64(f64::NAN);
    let number = Value::Float64(0.0);
    assert_eq!(
        nan.numeric_cmp(&number, NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::LeftNaN),
    );
    assert_eq!(
        number.numeric_cmp(&nan, NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNaN),
    );
    assert_eq!(
        nan.numeric_cmp(&Value::Float32(f32::NAN), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::BothNaN),
    );
    assert_eq!(
        nan.numeric_cmp(&Value::Bool(true), NumericComparisonPolicy::Exact),
        Err(NumericComparisonError::RightNotNumeric {
            actual: DataType::Bool,
        }),
    );
}

/// Verifies only concrete NaN float values are classified as NaN.
#[test]
fn test_value_is_nan_classifies_only_nan_floats() {
    assert!(Value::Float32(f32::NAN).is_nan());
    assert!(Value::Float64(f64::NAN).is_nan());
    assert!(!Value::Float32(1.0).is_nan());
    assert!(!Value::Float64(f64::INFINITY).is_nan());
    assert!(!Value::Int32(1).is_nan());
    assert!(!Value::String("NaN".to_owned()).is_nan());
    assert!(!Value::Unset(DataType::Float64).is_nan());
}

/// Exercises the lower-level numeric projection for every numeric variant.
#[test]
fn test_value_numeric_cmp_covers_every_numeric_variant() {
    let values = vec![
        Value::Int8(1),
        Value::Int16(1),
        Value::Int32(1),
        Value::Int64(1),
        Value::Int128(1),
        Value::UInt8(1),
        Value::UInt16(1),
        Value::UInt32(1),
        Value::UInt64(1),
        Value::UInt128(1),
        Value::Float32(1.0),
        Value::Float64(1.0),
        Value::BigInteger(BigInt::from(1)),
        Value::BigDecimal(BigDecimal::from(1)),
    ];

    for value in &values {
        assert_eq!(
            value.numeric_cmp(value, NumericComparisonPolicy::Exact),
            Ok(Ordering::Equal)
        );
    }
}
