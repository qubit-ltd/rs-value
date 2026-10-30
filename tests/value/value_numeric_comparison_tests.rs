// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::cmp::Ordering;
#[cfg(feature = "big-decimal")]
use std::str::FromStr;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_datatype::NumericComparisonPolicy;
use qubit_value::NumericComparisonError;
use qubit_value::Value;

fn assert_numeric_order(left: Value, right: Value, policy: NumericComparisonPolicy, expected: Ordering) {
    assert_eq!(left.numeric_cmp(&right, policy), Ok(expected));
    assert_eq!(right.numeric_cmp(&left, policy), Ok(expected.reverse()));
}

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

/// Verifies integer comparisons preserve boundaries across widths and signs.
#[test]
fn test_value_numeric_cmp_preserves_integer_boundaries() {
    let cases = [
        (
            Value::Int8(i8::MIN),
            Value::Int128(i128::from(i8::MIN)),
            Ordering::Equal,
        ),
        (
            Value::Int16(i16::MIN),
            Value::Int128(i128::from(i16::MIN)),
            Ordering::Equal,
        ),
        (
            Value::Int32(i32::MIN),
            Value::Int128(i128::from(i32::MIN)),
            Ordering::Equal,
        ),
        (
            Value::Int64(i64::MIN),
            Value::Int128(i128::from(i64::MIN)),
            Ordering::Equal,
        ),
        (Value::Int128(i128::MIN), Value::Int64(i64::MIN), Ordering::Less),
        (
            Value::UInt8(u8::MAX),
            Value::UInt128(u128::from(u8::MAX)),
            Ordering::Equal,
        ),
        (
            Value::UInt16(u16::MAX),
            Value::UInt128(u128::from(u16::MAX)),
            Ordering::Equal,
        ),
        (
            Value::UInt32(u32::MAX),
            Value::UInt128(u128::from(u32::MAX)),
            Ordering::Equal,
        ),
        (
            Value::UInt64(u64::MAX),
            Value::UInt128(u128::from(u64::MAX)),
            Ordering::Equal,
        ),
        (Value::UInt128(u128::MAX), Value::UInt64(u64::MAX), Ordering::Greater),
        (
            Value::UInt64(i64::MAX as u64 + 1),
            Value::Int64(i64::MAX),
            Ordering::Greater,
        ),
        (Value::Int8(-1), Value::UInt8(0), Ordering::Less),
    ];

    for (left, right, expected) in cases {
        assert_numeric_order(left, right, NumericComparisonPolicy::Exact, expected);
    }
}

/// Verifies float precision, signed zero, and infinity boundaries.
#[test]
fn test_value_numeric_cmp_preserves_float_boundaries() {
    assert_numeric_order(
        Value::UInt64(9_007_199_254_740_993),
        Value::Float64(9_007_199_254_740_992.0),
        NumericComparisonPolicy::Exact,
        Ordering::Greater,
    );
    assert_numeric_order(
        Value::UInt64(9_007_199_254_740_993),
        Value::Float64(9_007_199_254_740_992.0),
        NumericComparisonPolicy::Approximate,
        Ordering::Equal,
    );
    assert_numeric_order(
        Value::Float32(16_777_216.0),
        Value::UInt32(16_777_217),
        NumericComparisonPolicy::Exact,
        Ordering::Less,
    );
    assert_numeric_order(
        Value::Float32(-0.0),
        Value::Int8(0),
        NumericComparisonPolicy::Exact,
        Ordering::Equal,
    );

    for policy in [NumericComparisonPolicy::Exact, NumericComparisonPolicy::Approximate] {
        assert_numeric_order(
            Value::Float32(f32::NEG_INFINITY),
            Value::Int128(i128::MIN),
            policy,
            Ordering::Less,
        );
        assert_numeric_order(
            Value::Float64(f64::INFINITY),
            Value::UInt128(u128::MAX),
            policy,
            Ordering::Greater,
        );
    }
}

/// Verifies big integers can exceed the largest built-in unsigned integer.
#[cfg(feature = "big-integer")]
#[test]
fn test_value_numeric_cmp_preserves_big_integer_boundary() {
    assert_numeric_order(
        Value::BigInteger(BigInt::from(u128::MAX) + BigInt::from(1_u8)),
        Value::UInt128(u128::MAX),
        NumericComparisonPolicy::Exact,
        Ordering::Greater,
    );
}

/// Verifies exact and approximate decimal/float policy behavior.
#[cfg(feature = "big-decimal")]
#[test]
fn test_value_numeric_cmp_applies_explicit_policy() {
    let decimal = Value::BigDecimal(BigDecimal::from_str("0.1").unwrap());
    let float = Value::Float64(0.1);
    assert_numeric_order(
        decimal.clone(),
        float.clone(),
        NumericComparisonPolicy::Exact,
        Ordering::Less,
    );
    assert_numeric_order(decimal, float, NumericComparisonPolicy::Approximate, Ordering::Equal);
}

/// Verifies big decimal values compare exactly with built-in integers.
#[cfg(feature = "big-decimal")]
#[test]
fn test_value_numeric_cmp_preserves_big_decimal_boundary() {
    assert_numeric_order(
        Value::BigDecimal(BigDecimal::from_str("1.25").unwrap()),
        Value::Int8(2),
        NumericComparisonPolicy::Exact,
        Ordering::Less,
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
        Value::Int32(1).numeric_cmp(&Value::Unset(DataType::Float64), NumericComparisonPolicy::Exact,),
        Err(NumericComparisonError::RightMissing {
            declared: DataType::Float64,
        }),
    );
    assert_eq!(
        Value::Unset(DataType::Int32).numeric_cmp(&Value::Unset(DataType::Int64), NumericComparisonPolicy::Exact,),
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
        Err(NumericComparisonError::RightNotNumeric { actual: DataType::Bool }),
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
        Err(NumericComparisonError::RightNotNumeric { actual: DataType::Bool }),
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
#[cfg(all(feature = "big-decimal", feature = "big-integer"))]
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
