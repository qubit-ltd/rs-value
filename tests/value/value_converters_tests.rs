/*****************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/
//! # Value Converters Unit Tests
//!
//! Tests for `to::<T>()` conversion behavior.
//!
//! # Author
//!
//! Haixing Hu

use bigdecimal::BigDecimal;
use chrono::NaiveDate;
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use qubit_value::{Value, ValueError};
use std::str::FromStr;

#[test]
fn test_value_type_conversion() {
    let v = Value::Int32(42);
    assert_eq!(v.to::<i64>().unwrap(), 42i64);
    assert_eq!(v.to::<f64>().unwrap(), 42.0f64);
    assert_eq!(v.to::<String>().unwrap(), "42");
}
#[test]
fn test_value_bool_conversion() {
    let v1 = Value::Int32(1);
    assert!(v1.to::<bool>().unwrap());

    let v2 = Value::Int32(0);
    assert!(!v2.to::<bool>().unwrap());

    let v3 = Value::String("true".to_string());
    assert!(v3.to::<bool>().unwrap());
}

#[test]
fn test_value_bool_conversion_accepts_config_bool_strings() {
    let truthy_values = ["1", "true", "TRUE", "True", "  true  "];
    for raw in truthy_values {
        let value = Value::String(raw.to_string());
        assert!(
            value.to::<bool>().unwrap(),
            "expected '{raw}' to convert to true"
        );
    }

    let falsy_values = ["0", "false", "FALSE", "False", "  false  "];
    for raw in falsy_values {
        let value = Value::String(raw.to_string());
        assert!(
            !value.to::<bool>().unwrap(),
            "expected '{raw}' to convert to false"
        );
    }
}
#[test]
fn test_value_datetime_to_string() {
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Test Date to string conversion
    let date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();
    let value = Value::Date(date);
    let str_repr = value.to::<String>().unwrap();
    assert_eq!(str_repr, "2024-01-15");

    // Test Time to string conversion
    let time = NaiveTime::from_hms_opt(14, 30, 45).unwrap();
    let value = Value::Time(time);
    let str_repr = value.to::<String>().unwrap();
    assert_eq!(str_repr, "14:30:45");

    // Test DateTime to string conversion
    let datetime = NaiveDate::from_ymd_opt(2024, 1, 15)
        .unwrap()
        .and_hms_opt(14, 30, 45)
        .unwrap();
    let value = Value::DateTime(datetime);
    let str_repr = value.to::<String>().unwrap();
    assert_eq!(str_repr, "2024-01-15 14:30:45");

    // Test Instant to string conversion
    let instant = Utc::now();
    let value = Value::Instant(instant);
    let str_repr = value.to::<String>().unwrap();
    assert!(str_repr.contains('T')); // RFC3339 format contains 'T'
}
#[test]
fn test_value_as_bool_all_branches() {
    // Test all integer types to boolean conversion
    assert!(Value::Int8(1).to::<bool>().unwrap());
    assert!(!Value::Int8(0).to::<bool>().unwrap());

    assert!(Value::Int16(1).to::<bool>().unwrap());
    assert!(!Value::Int16(0).to::<bool>().unwrap());

    assert!(Value::Int64(1).to::<bool>().unwrap());
    assert!(!Value::Int64(0).to::<bool>().unwrap());

    assert!(Value::Int128(1).to::<bool>().unwrap());
    assert!(!Value::Int128(0).to::<bool>().unwrap());

    assert!(Value::UInt8(1).to::<bool>().unwrap());
    assert!(!Value::UInt8(0).to::<bool>().unwrap());

    assert!(Value::UInt16(1).to::<bool>().unwrap());
    assert!(!Value::UInt16(0).to::<bool>().unwrap());

    assert!(Value::UInt32(1).to::<bool>().unwrap());
    assert!(!Value::UInt32(0).to::<bool>().unwrap());

    assert!(Value::UInt64(1).to::<bool>().unwrap());
    assert!(!Value::UInt64(0).to::<bool>().unwrap());

    assert!(Value::UInt128(1).to::<bool>().unwrap());
    assert!(!Value::UInt128(0).to::<bool>().unwrap());

    // Test string to boolean conversion failure cases
    let value = Value::String("invalid".to_string());
    assert!(value.to::<bool>().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Bool);
    assert!(matches!(value.to::<bool>(), Err(ValueError::NoValue)));

    // Test unsupported type conversions
    let value = Value::Char('a');
    assert!(matches!(
        value.to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));
}
#[test]
fn test_value_as_int32_all_branches() {
    // Test all types that can convert to i32
    assert_eq!(Value::Int8(42).to::<i32>().unwrap(), 42);
    assert_eq!(Value::Int16(1000).to::<i32>().unwrap(), 1000);
    assert_eq!(Value::Int32(100000).to::<i32>().unwrap(), 100000);

    // Test i64 to i32 conversion success
    assert_eq!(Value::Int64(42).to::<i32>().unwrap(), 42);

    // Test i64 to i32 overflow
    let value = Value::Int64(i64::MAX);
    assert!(value.to::<i32>().is_err());

    // Test i128 to i32 conversion success
    assert_eq!(Value::Int128(42).to::<i32>().unwrap(), 42);

    // Test i128 to i32 overflow
    let value = Value::Int128(i128::MAX);
    assert!(value.to::<i32>().is_err());

    // Test unsigned integer conversion
    assert_eq!(Value::UInt8(42).to::<i32>().unwrap(), 42);
    assert_eq!(Value::UInt16(1000).to::<i32>().unwrap(), 1000);

    // Test u32 to i32 conversion success
    assert_eq!(Value::UInt32(42).to::<i32>().unwrap(), 42);

    // Test u32 to i32 overflow
    let value = Value::UInt32(u32::MAX);
    assert!(value.to::<i32>().is_err());

    // Test string to i32 conversion
    assert_eq!(
        Value::String("12345".to_string()).to::<i32>().unwrap(),
        12345
    );

    // Test string to i32 conversion failure
    let value = Value::String("invalid".to_string());
    assert!(value.to::<i32>().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Int32);
    assert!(matches!(value.to::<i32>(), Err(ValueError::NoValue)));

    // Test Bool to i32 conversion
    let value = Value::Bool(true);
    assert_eq!(value.to::<i32>().unwrap(), 1);
    let value = Value::Bool(false);
    assert_eq!(value.to::<i32>().unwrap(), 0);

    // Test Char to i32 conversion
    let value = Value::Char('A');
    assert_eq!(value.to::<i32>().unwrap(), 65);

    // Test Float32/Float64 to i32 conversion
    let value = Value::Float32(42.7);
    assert_eq!(value.to::<i32>().unwrap(), 42);
    let value = Value::Float64(99.9);
    assert_eq!(value.to::<i32>().unwrap(), 99);

    // Test BigDecimal to i32 conversion
    let value = Value::BigDecimal(BigDecimal::from(123));
    assert_eq!(value.to::<i32>().unwrap(), 123);

    // Test unsupported time types
    let value = Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionFailed { .. })
    ));
}
#[test]
fn test_value_as_int64_all_branches() {
    // Test all types that can convert to i64
    assert_eq!(Value::Int8(42).to::<i64>().unwrap(), 42);
    assert_eq!(Value::Int16(1000).to::<i64>().unwrap(), 1000);
    assert_eq!(Value::Int32(100000).to::<i64>().unwrap(), 100000);
    assert_eq!(Value::Int64(1000000).to::<i64>().unwrap(), 1000000);

    // Test i128 to i64 conversion success
    assert_eq!(Value::Int128(42).to::<i64>().unwrap(), 42);

    // Test i128 to i64 overflow
    let value = Value::Int128(i128::MAX);
    assert!(value.to::<i64>().is_err());

    // Test unsigned integer conversion
    assert_eq!(Value::UInt8(42).to::<i64>().unwrap(), 42);
    assert_eq!(Value::UInt16(1000).to::<i64>().unwrap(), 1000);
    assert_eq!(Value::UInt32(100000).to::<i64>().unwrap(), 100000);

    // Test u64 to i64 conversion success
    assert_eq!(Value::UInt64(42).to::<i64>().unwrap(), 42);

    // Test u64 to i64 overflow
    let value = Value::UInt64(u64::MAX);
    assert!(value.to::<i64>().is_err());

    // Test string to i64 conversion
    assert_eq!(
        Value::String("123456789".to_string()).to::<i64>().unwrap(),
        123456789
    );

    // Test Empty value
    let value = Value::Empty(DataType::Int64);
    assert!(matches!(value.to::<i64>(), Err(ValueError::NoValue)));
}
#[test]
fn test_value_as_float64_all_branches() {
    // Test floating point conversion
    assert_eq!(Value::Float32(3.5).to::<f64>().unwrap(), 3.5f32 as f64);
    assert_eq!(Value::Float64(2.5).to::<f64>().unwrap(), 2.5);

    // Test integers to floating point conversion
    assert_eq!(Value::Int8(42).to::<f64>().unwrap(), 42.0);
    assert_eq!(Value::Int16(1000).to::<f64>().unwrap(), 1000.0);
    assert_eq!(Value::Int32(100000).to::<f64>().unwrap(), 100000.0);
    assert_eq!(Value::Int64(1000000).to::<f64>().unwrap(), 1000000.0);

    // Test unsigned integers to floating point conversion
    assert_eq!(Value::UInt8(42).to::<f64>().unwrap(), 42.0);
    assert_eq!(Value::UInt16(1000).to::<f64>().unwrap(), 1000.0);
    assert_eq!(Value::UInt32(100000).to::<f64>().unwrap(), 100000.0);
    assert_eq!(Value::UInt64(1000000).to::<f64>().unwrap(), 1000000.0);

    // Test string to floating point conversion
    assert_eq!(Value::String("3.5".to_string()).to::<f64>().unwrap(), 3.5);

    // Test string to floating point conversion failure
    let value = Value::String("invalid".to_string());
    assert!(value.to::<f64>().is_err());

    // Test Empty value
    let value = Value::Empty(DataType::Float64);
    assert!(matches!(value.to::<f64>(), Err(ValueError::NoValue)));

    // Test Bool to f64 conversion
    let value = Value::Bool(true);
    assert_eq!(value.to::<f64>().unwrap(), 1.0);
    let value = Value::Bool(false);
    assert_eq!(value.to::<f64>().unwrap(), 0.0);

    // Test Char to f64 conversion
    let value = Value::Char('A');
    assert_eq!(value.to::<f64>().unwrap(), 65.0);

    // Test Int128/UInt128 to f64 conversion
    let value = Value::Int128(123);
    assert_eq!(value.to::<f64>().unwrap(), 123.0);
    let value = Value::UInt128(456);
    assert_eq!(value.to::<f64>().unwrap(), 456.0);

    // Test unsupported time types
    let value = Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap());
    assert!(matches!(
        value.to::<f64>(),
        Err(ValueError::ConversionFailed { .. })
    ));
}
#[test]
fn test_value_as_string_all_types() {
    // Test all types to string conversion
    assert_eq!(Value::Bool(true).to::<String>().unwrap(), "true");
    assert_eq!(Value::Bool(false).to::<String>().unwrap(), "false");
    assert_eq!(Value::Char('A').to::<String>().unwrap(), "A");

    assert_eq!(Value::Int8(42).to::<String>().unwrap(), "42");
    assert_eq!(Value::Int16(1000).to::<String>().unwrap(), "1000");
    assert_eq!(Value::Int32(100000).to::<String>().unwrap(), "100000");
    assert_eq!(Value::Int64(1000000).to::<String>().unwrap(), "1000000");
    assert_eq!(
        Value::Int128(123456789).to::<String>().unwrap(),
        "123456789"
    );

    assert_eq!(Value::UInt8(42).to::<String>().unwrap(), "42");
    assert_eq!(Value::UInt16(1000).to::<String>().unwrap(), "1000");
    assert_eq!(Value::UInt32(100000).to::<String>().unwrap(), "100000");
    assert_eq!(Value::UInt64(1000000).to::<String>().unwrap(), "1000000");
    assert_eq!(
        Value::UInt128(123456789).to::<String>().unwrap(),
        "123456789"
    );

    assert!(
        Value::Float32(3.5)
            .to::<String>()
            .unwrap()
            .starts_with("3.5")
    );
    assert!(
        Value::Float64(2.5)
            .to::<String>()
            .unwrap()
            .starts_with("2.5")
    );

    assert_eq!(
        Value::String("hello".to_string()).to::<String>().unwrap(),
        "hello"
    );

    // Test Empty value
    let value = Value::Empty(DataType::String);
    assert!(matches!(value.to::<String>(), Err(ValueError::NoValue)));
}
#[test]
fn test_value_as_int32_uint64_conversion() {
    // Test UInt64 to i32 - small values can convert successfully
    let value = Value::UInt64(100);
    assert_eq!(value.to::<i32>().unwrap(), 100);

    // Test UInt64 to i32 - large values will fail
    let value = Value::UInt64(u64::MAX);
    assert!(value.to::<i32>().is_err());

    // Test UInt128 to i32 - small values can convert successfully
    let value = Value::UInt128(200);
    assert_eq!(value.to::<i32>().unwrap(), 200);

    // Test UInt128 to i32 - large values will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.to::<i32>().is_err());
}
#[test]
fn test_value_as_int64_uint128_conversion() {
    // Test UInt128 to i64 - small values can convert successfully
    let value = Value::UInt128(1000);
    assert_eq!(value.to::<i64>().unwrap(), 1000);

    // Test UInt128 to i64 - large values will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.to::<i64>().is_err());

    // Test string to i64 conversion failure
    let value = Value::String("not a number".to_string());
    assert!(value.to::<i64>().is_err());
}
#[test]
fn test_value_as_float64_conversions() {
    // Test Int128 to f64 conversion
    let value = Value::Int128(999999);
    assert_eq!(value.to::<f64>().unwrap(), 999999.0);

    // Test UInt128 to f64 conversion
    let value = Value::UInt128(123456);
    assert_eq!(value.to::<f64>().unwrap(), 123456.0);

    // Test Bool to f64 conversion
    let value = Value::Bool(true);
    assert_eq!(value.to::<f64>().unwrap(), 1.0);
    let value = Value::Bool(false);
    assert_eq!(value.to::<f64>().unwrap(), 0.0);

    // Test Char to f64 conversion
    let value = Value::Char('B');
    assert_eq!(value.to::<f64>().unwrap(), 66.0);

    // Test string to f64 conversion failure
    let value = Value::String("invalid number".to_string());
    assert!(value.to::<f64>().is_err());
}
#[test]
fn test_big_type_conversions_for_coverage() {
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;
    use std::f64;
    use std::str::FromStr;

    // BigInt -> as_int32
    let v = Value::BigInteger(BigInt::from(123));
    assert_eq!(v.to::<i32>().unwrap(), 123);
    let v_overflow = Value::BigInteger(BigInt::from(i64::MAX));
    assert!(v_overflow.to::<i32>().is_err());

    // BigInt -> as_int64
    let v = Value::BigInteger(BigInt::from(123456i64));
    assert_eq!(v.to::<i64>().unwrap(), 123456i64);
    let v_overflow = Value::BigInteger(BigInt::from_str("123456789012345678901234567890").unwrap());
    assert!(v_overflow.to::<i64>().is_err());

    // BigInt -> as_float64
    let v = Value::BigInteger(BigInt::from(12345));
    assert!((v.to::<f64>().unwrap() - 12345.0).abs() < f64::EPSILON);
    let large_big_int_str = "1".repeat(400);
    let v_overflow = Value::BigInteger(BigInt::from_str(&large_big_int_str).unwrap());
    assert!(v_overflow.to::<f64>().is_err());

    // BigDecimal -> as_float64
    let v = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((v.to::<f64>().unwrap() - 123.456).abs() < f64::EPSILON);
    let v_overflow = Value::BigDecimal(BigDecimal::from_str("1.0e400").unwrap());
    assert!(v_overflow.to::<f64>().is_err());
}
#[test]
fn test_as_bool_string_conversion_error() {
    // Test string to boolean conversion failure returns ConversionError
    let value = Value::String("not_a_bool".to_string());
    match value.to::<bool>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_bool"));
            assert!(msg.contains("to boolean"));
        }
        _ => panic!("Expected ConversionError"),
    }

    // Test empty string to boolean conversion
    let value = Value::String("".to_string());
    assert!(matches!(
        value.to::<bool>(),
        Err(ValueError::ConversionError(_))
    ));

    // Test various invalid boolean strings
    let invalid_bools = vec!["yes", "no", "t", "f", "y", "n", "on", "off"];
    for invalid in invalid_bools {
        let value = Value::String(invalid.to_string());
        assert!(matches!(
            value.to::<bool>(),
            Err(ValueError::ConversionError(_))
        ));
    }
}
#[test]
fn test_as_int32_conversion_errors() {
    // Test i64 out of range ConversionError
    let value = Value::Int64(i64::MAX);
    match value.to::<i32>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i64"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i64 overflow"),
    }

    // Test i128 out of range ConversionError
    let value = Value::Int128(i128::MAX);
    match value.to::<i32>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test u32 out of range ConversionError
    let value = Value::UInt32(u32::MAX);
    match value.to::<i32>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u32"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u32 overflow"),
    }

    // Test string conversion failure ConversionError
    let value = Value::String("not_a_number".to_string());
    match value.to::<i32>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_number"));
            assert!(msg.contains("i32"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Test BigInteger out of range ConversionError
    let value = Value::BigInteger(BigInt::from_str("999999999999999999999").unwrap());
    match value.to::<i32>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigInteger"));
            assert!(msg.contains("i32"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for BigInteger overflow"),
    }
}
#[test]
fn test_as_int64_conversion_errors() {
    // Test i128 out of range ConversionError
    let value = Value::Int128(i128::MAX);
    match value.to::<i64>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test u64 out of range ConversionError
    let value = Value::UInt64(u64::MAX);
    match value.to::<i64>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 overflow"),
    }

    // Test string conversion failure ConversionError
    let value = Value::String("invalid_number".to_string());
    match value.to::<i64>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("invalid_number"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Test BigInteger out of range ConversionError
    let value = Value::BigInteger(BigInt::from_str("999999999999999999999999999999").unwrap());
    match value.to::<i64>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigInteger"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for BigInteger overflow"),
    }
}
#[test]
fn test_as_float64_conversion_errors() {
    // Test string conversion failure ConversionError
    let value = Value::String("not_a_float".to_string());
    match value.to::<f64>() {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_float"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // BigInteger / BigDecimal overflow error cases are covered by dedicated
    // tests below.
}
#[test]
fn test_as_int32_negative_i64_conversion() {
    // Test negative i64 out of i32 range
    let value = Value::Int64(i64::MIN);
    match value.to::<i32>() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i64 overflow"),
    }
}
#[test]
fn test_as_int32_negative_i128_conversion() {
    // Test negative i128 out of i32 range
    let value = Value::Int128(i128::MIN);
    match value.to::<i32>() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i128 overflow"),
    }
}
#[test]
fn test_as_int64_negative_i128_conversion() {
    // Test negative i128 out of i64 range
    let value = Value::Int128(i128::MIN);
    match value.to::<i64>() {
        Err(ValueError::ConversionError(_)) => {
            // Expected error
        }
        _ => panic!("Expected ConversionError for negative i128 overflow"),
    }
}
#[test]
fn test_as_int32_small_uint64_success() {
    // Test UInt64 small values can convert successfully to i32
    let value = Value::UInt64(100);
    assert_eq!(value.to::<i32>().unwrap(), 100);

    // Test UInt64 large values conversion to i32 will fail
    let value = Value::UInt64(u64::MAX);
    assert!(value.to::<i32>().is_err());
}
#[test]
fn test_as_int64_small_uint128_conversion_failed() {
    // Test UInt128 small values can convert successfully to i64
    let value = Value::UInt128(100);
    assert_eq!(value.to::<i64>().unwrap(), 100);

    // Test UInt128 large values conversion to i64 will fail
    let value = Value::UInt128(u128::MAX);
    assert!(value.to::<i64>().is_err());
}
#[test]
fn test_as_float64_int128_conversion_failed() {
    // Test Int128 to f64 conversion
    let value = Value::Int128(100);
    assert_eq!(value.to::<f64>().unwrap(), 100.0);

    // Test large number conversion (may lose precision, but still succeeds)
    let value = Value::Int128(i128::MAX);
    assert!(value.to::<f64>().is_ok());
}
#[test]
fn test_as_float64_uint128_conversion_failed() {
    // Test UInt128 to f64 conversion
    let value = Value::UInt128(100);
    assert_eq!(value.to::<f64>().unwrap(), 100.0);

    // Test large number conversion (may lose precision, but still succeeds)
    let value = Value::UInt128(u128::MAX);
    assert!(value.to::<f64>().is_ok());
}
#[test]
fn test_as_bool_direct_bool_type() {
    // Test case where type itself is Bool
    let value_true = Value::Bool(true);
    assert!(value_true.to::<bool>().unwrap());

    let value_false = Value::Bool(false);
    assert!(!value_false.to::<bool>().unwrap());
}
#[test]
fn test_as_bool_string_parse_error() {
    // Test all cases where String type parsing bool fails
    let invalid_strings = vec![
        "yes", "no", "t", "f", "y", "n", "on", "off", "", "  ", "null", "None",
    ];

    for invalid_str in invalid_strings {
        let value = Value::String(invalid_str.to_string());
        assert!(
            value.to::<bool>().is_err(),
            "String '{}' should not be able to convert to bool",
            invalid_str
        );
    }

    // Test valid bool strings
    let value_true = Value::String("true".to_string());
    assert!(value_true.to::<bool>().unwrap());

    let value_false = Value::String("false".to_string());
    assert!(!value_false.to::<bool>().unwrap());
}
#[test]
fn test_as_bool_all_unsupported_types() {
    // Test all types that do not support conversion to bool
    use chrono::{NaiveDate, NaiveTime, Utc};

    // Char type
    assert!(matches!(
        Value::Char('a').to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Float32 type
    assert!(matches!(
        Value::Float32(1.5).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Float64 type
    assert!(matches!(
        Value::Float64(2.5).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Date type
    assert!(matches!(
        Value::Date(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Time type
    assert!(matches!(
        Value::Time(NaiveTime::from_hms_opt(12, 0, 0).unwrap()).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // DateTime type
    assert!(matches!(
        Value::DateTime(
            NaiveDate::from_ymd_opt(2024, 1, 1)
                .unwrap()
                .and_hms_opt(12, 0, 0)
                .unwrap()
        )
        .to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // Instant type
    assert!(matches!(
        Value::Instant(Utc::now()).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // BigInteger type
    assert!(matches!(
        Value::BigInteger(BigInt::from(123)).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));

    // BigDecimal type
    assert!(matches!(
        Value::BigDecimal(BigDecimal::from_str("123.45").unwrap()).to::<bool>(),
        Err(ValueError::ConversionFailed { .. })
    ));
}
#[test]
fn test_as_int32_direct_int32_type() {
    // Test case where type itself is Int32
    let value = Value::Int32(42);
    assert_eq!(value.to::<i32>().unwrap(), 42);

    let value_negative = Value::Int32(-100);
    assert_eq!(value_negative.to::<i32>().unwrap(), -100);

    let value_max = Value::Int32(i32::MAX);
    assert_eq!(value_max.to::<i32>().unwrap(), i32::MAX);

    let value_min = Value::Int32(i32::MIN);
    assert_eq!(value_min.to::<i32>().unwrap(), i32::MIN);
}
#[test]
fn test_as_int32_int64_overflow_cases() {
    // Test various cases of Int64 overflow

    // Positive overflow
    let value_max = Value::Int64(i64::MAX);
    assert!(matches!(
        value_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int64(i64::MIN);
    assert!(matches!(
        value_min.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 max value
    let value_over_max = Value::Int64(i32::MAX as i64 + 1);
    assert!(matches!(
        value_over_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 min value
    let value_under_min = Value::Int64(i32::MIN as i64 - 1);
    assert!(matches!(
        value_under_min.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int64(1000);
    assert_eq!(value_in_range.to::<i32>().unwrap(), 1000);
}
#[test]
fn test_as_int32_int128_overflow_cases() {
    // Test various cases of Int128 overflow

    // Positive overflow
    let value_max = Value::Int128(i128::MAX);
    assert!(matches!(
        value_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int128(i128::MIN);
    assert!(matches!(
        value_min.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32 max value
    let value_over_max = Value::Int128(i32::MAX as i128 + 1);
    assert!(matches!(
        value_over_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int128(500);
    assert_eq!(value_in_range.to::<i32>().unwrap(), 500);
}
#[test]
fn test_as_int32_uint32_overflow_cases() {
    // Test various cases of UInt32 overflow

    // u32::MAX exceeds i32 range
    let value_max = Value::UInt32(u32::MAX);
    assert!(matches!(
        value_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i32::MAX
    let value_over_max = Value::UInt32(i32::MAX as u32 + 1);
    assert!(matches!(
        value_over_max.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::UInt32(100);
    assert_eq!(value_in_range.to::<i32>().unwrap(), 100);

    // i32::MAX should convert successfully
    let value_max_valid = Value::UInt32(i32::MAX as u32);
    assert_eq!(value_max_valid.to::<i32>().unwrap(), i32::MAX);
}
#[test]
fn test_as_int32_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("not_a_number".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Floating point string
    let value = Value::String("123.45".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Leading or trailing spaces
    let value = Value::String("  123  ".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Contains non-numeric characters
    let value = Value::String("123abc".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Number string out of range
    let value = Value::String("9999999999999999999".to_string());
    assert!(matches!(
        value.to::<i32>(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid number string should succeed
    let value = Value::String("12345".to_string());
    assert_eq!(value.to::<i32>().unwrap(), 12345);

    let value_negative = Value::String("-9876".to_string());
    assert_eq!(value_negative.to::<i32>().unwrap(), -9876);
}
#[test]
fn test_as_int64_direct_int64_type() {
    // Test case where type itself is Int64
    let value = Value::Int64(123456789);
    assert_eq!(value.to::<i64>().unwrap(), 123456789);

    let value_negative = Value::Int64(-987654321);
    assert_eq!(value_negative.to::<i64>().unwrap(), -987654321);

    let value_max = Value::Int64(i64::MAX);
    assert_eq!(value_max.to::<i64>().unwrap(), i64::MAX);

    let value_min = Value::Int64(i64::MIN);
    assert_eq!(value_min.to::<i64>().unwrap(), i64::MIN);
}
#[test]
fn test_as_int64_int128_overflow_cases() {
    // Test various cases of Int128 overflow

    // Positive overflow
    let value_max = Value::Int128(i128::MAX);
    assert!(matches!(
        value_max.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Negative overflow
    let value_min = Value::Int128(i128::MIN);
    assert!(matches!(
        value_min.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64 max value
    let value_over_max = Value::Int128(i64::MAX as i128 + 1);
    assert!(matches!(
        value_over_max.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64 min value
    let value_under_min = Value::Int128(i64::MIN as i128 - 1);
    assert!(matches!(
        value_under_min.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::Int128(999999);
    assert_eq!(value_in_range.to::<i64>().unwrap(), 999999);
}
#[test]
fn test_as_int64_uint64_overflow_cases() {
    // Test various cases of UInt64 overflow

    // u64::MAX exceeds i64 range
    let value_max = Value::UInt64(u64::MAX);
    assert!(matches!(
        value_max.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Just exceeding i64::MAX
    let value_over_max = Value::UInt64(i64::MAX as u64 + 1);
    assert!(matches!(
        value_over_max.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Values within range should succeed
    let value_in_range = Value::UInt64(123456);
    assert_eq!(value_in_range.to::<i64>().unwrap(), 123456);

    // i64::MAX should convert successfully
    let value_max_valid = Value::UInt64(i64::MAX as u64);
    assert_eq!(value_max_valid.to::<i64>().unwrap(), i64::MAX);
}
#[test]
fn test_as_int64_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("invalid_number".to_string());
    assert!(matches!(
        value.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Floating point string
    let value = Value::String("456.789".to_string());
    assert!(matches!(
        value.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Number string out of range
    let value = Value::String("99999999999999999999999999999".to_string());
    assert!(matches!(
        value.to::<i64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid number string should succeed
    let value = Value::String("123456789".to_string());
    assert_eq!(value.to::<i64>().unwrap(), 123456789);

    let value_negative = Value::String("-987654321".to_string());
    assert_eq!(value_negative.to::<i64>().unwrap(), -987654321);
}
#[test]
fn test_as_float64_direct_float64_type() {
    // Test case where type itself is Float64
    let value = Value::Float64(3.5);
    assert_eq!(value.to::<f64>().unwrap(), 3.5);

    let value_negative = Value::Float64(-2.5);
    assert_eq!(value_negative.to::<f64>().unwrap(), -2.5);

    let value_zero = Value::Float64(0.0);
    assert_eq!(value_zero.to::<f64>().unwrap(), 0.0);

    // Test special values
    let value_inf = Value::Float64(f64::INFINITY);
    assert_eq!(value_inf.to::<f64>().unwrap(), f64::INFINITY);

    let value_neg_inf = Value::Float64(f64::NEG_INFINITY);
    assert_eq!(value_neg_inf.to::<f64>().unwrap(), f64::NEG_INFINITY);

    let value_nan = Value::Float64(f64::NAN);
    assert!(value_nan.to::<f64>().unwrap().is_nan());
}
#[test]
fn test_as_float64_string_parse_error() {
    // Test various cases where String type parsing fails

    // Not a number at all
    let value = Value::String("not_a_float".to_string());
    assert!(matches!(
        value.to::<f64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Empty string
    let value = Value::String("".to_string());
    assert!(matches!(
        value.to::<f64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Contains non-numeric characters
    let value = Value::String("12.34abc".to_string());
    assert!(matches!(
        value.to::<f64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Multiple decimal points
    let value = Value::String("12.34.56".to_string());
    assert!(matches!(
        value.to::<f64>(),
        Err(ValueError::ConversionError(_))
    ));

    // Valid floating point string should succeed
    let value = Value::String("3.5".to_string());
    assert_eq!(value.to::<f64>().unwrap(), 3.5);

    let value_negative = Value::String("-2.5".to_string());
    assert_eq!(value_negative.to::<f64>().unwrap(), -2.5);

    let value_scientific = Value::String("1.23e10".to_string());
    assert_eq!(value_scientific.to::<f64>().unwrap(), 1.23e10);
}
#[test]
fn test_as_float64_biginteger_conversion_error() {
    use std::str::FromStr;

    // BigInteger within normal range should convert successfully
    let value = Value::BigInteger(BigInt::from(12345));
    assert_eq!(value.to::<f64>().unwrap(), 12345.0);

    // Very large BigInteger should fail instead of degrading to INFINITY
    let large_big_int = BigInt::from_str(&"9".repeat(400)).unwrap();
    let value = Value::BigInteger(large_big_int);
    assert!(value.to::<f64>().is_err());

    // Test negative numbers
    let value_negative = Value::BigInteger(BigInt::from(-999999));
    assert_eq!(value_negative.to::<f64>().unwrap(), -999999.0);
}
#[test]
fn test_as_float64_bigdecimal_conversion_error() {
    use std::str::FromStr;

    // BigDecimal within normal range should convert successfully
    let value = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((value.to::<f64>().unwrap() - 123.456).abs() < 1e-10);

    // Very large BigDecimal should fail instead of degrading to INFINITY
    let large_big_decimal = BigDecimal::from_str("1.0e400").unwrap();
    let value = Value::BigDecimal(large_big_decimal);
    assert!(value.to::<f64>().is_err());

    // Very small BigDecimal (negative exponent with huge magnitude) should fail
    let small_big_decimal = BigDecimal::from_str("-1.0e400").unwrap();
    let value = Value::BigDecimal(small_big_decimal);
    assert!(value.to::<f64>().is_err());

    // Test high precision decimals
    let value = Value::BigDecimal(BigDecimal::from_str("0.123456789012345").unwrap());
    assert!((value.to::<f64>().unwrap() - 0.123456789012345).abs() < 1e-15);
}
#[test]
fn test_as_int32_all_unsigned_types() {
    // Ensure all unsigned integer types are tested

    // UInt8 - Always within range
    let value = Value::UInt8(255);
    assert_eq!(value.to::<i32>().unwrap(), 255);

    // UInt16 - Always within range
    let value = Value::UInt16(65535);
    assert_eq!(value.to::<i32>().unwrap(), 65535);

    // UInt32 - May overflow
    let value_ok = Value::UInt32(100);
    assert_eq!(value_ok.to::<i32>().unwrap(), 100);

    let value_overflow = Value::UInt32(u32::MAX);
    assert!(value_overflow.to::<i32>().is_err());

    // UInt64 - May overflow
    let value_ok = Value::UInt64(200);
    assert_eq!(value_ok.to::<i32>().unwrap(), 200);

    let value_overflow = Value::UInt64(u64::MAX);
    assert!(value_overflow.to::<i32>().is_err());

    // UInt128 - May overflow
    let value_ok = Value::UInt128(300);
    assert_eq!(value_ok.to::<i32>().unwrap(), 300);

    let value_overflow = Value::UInt128(u128::MAX);
    assert!(value_overflow.to::<i32>().is_err());
}
#[test]
fn test_as_int64_all_unsigned_types() {
    // Ensure all unsigned integer types are tested

    // UInt8 - Always within range
    let value = Value::UInt8(255);
    assert_eq!(value.to::<i64>().unwrap(), 255);

    // UInt16 - Always within range
    let value = Value::UInt16(65535);
    assert_eq!(value.to::<i64>().unwrap(), 65535);

    // UInt32 - Always within range
    let value = Value::UInt32(u32::MAX);
    assert_eq!(value.to::<i64>().unwrap(), u32::MAX as i64);

    // UInt64 - May overflow
    let value_ok = Value::UInt64(1000);
    assert_eq!(value_ok.to::<i64>().unwrap(), 1000);

    let value_overflow = Value::UInt64(u64::MAX);
    assert!(value_overflow.to::<i64>().is_err());

    // UInt128 - May overflow
    let value_ok = Value::UInt128(2000);
    assert_eq!(value_ok.to::<i64>().unwrap(), 2000);

    let value_overflow = Value::UInt128(u128::MAX);
    assert!(value_overflow.to::<i64>().is_err());
}
#[test]
fn test_as_float64_all_integer_types() {
    // Ensure all integer types to float64 conversion are tested

    // Signed integers
    assert_eq!(Value::Int8(127).to::<f64>().unwrap(), 127.0);
    assert_eq!(Value::Int8(-128).to::<f64>().unwrap(), -128.0);

    assert_eq!(Value::Int16(32767).to::<f64>().unwrap(), 32767.0);
    assert_eq!(Value::Int16(-32768).to::<f64>().unwrap(), -32768.0);

    assert_eq!(Value::Int32(i32::MAX).to::<f64>().unwrap(), i32::MAX as f64);
    assert_eq!(Value::Int32(i32::MIN).to::<f64>().unwrap(), i32::MIN as f64);

    assert_eq!(Value::Int64(i64::MAX).to::<f64>().unwrap(), i64::MAX as f64);
    assert_eq!(Value::Int64(i64::MIN).to::<f64>().unwrap(), i64::MIN as f64);

    assert_eq!(
        Value::Int128(i128::MAX).to::<f64>().unwrap(),
        i128::MAX as f64
    );
    assert_eq!(
        Value::Int128(i128::MIN).to::<f64>().unwrap(),
        i128::MIN as f64
    );

    // Unsigned integers
    assert_eq!(Value::UInt8(255).to::<f64>().unwrap(), 255.0);
    assert_eq!(Value::UInt16(65535).to::<f64>().unwrap(), 65535.0);
    assert_eq!(
        Value::UInt32(u32::MAX).to::<f64>().unwrap(),
        u32::MAX as f64
    );
    assert_eq!(
        Value::UInt64(u64::MAX).to::<f64>().unwrap(),
        u64::MAX as f64
    );
    assert_eq!(
        Value::UInt128(u128::MAX).to::<f64>().unwrap(),
        u128::MAX as f64
    );
}
#[test]
fn test_as_string_direct_string_type() {
    // Test case where type itself is String
    let value = Value::String("hello world".to_string());
    assert_eq!(value.to::<String>().unwrap(), "hello world");

    let value_empty = Value::String("".to_string());
    assert_eq!(value_empty.to::<String>().unwrap(), "");

    let value_unicode = Value::String("你好世界🌍".to_string());
    assert_eq!(value_unicode.to::<String>().unwrap(), "你好世界🌍");
}
#[test]
fn test_conversion_with_edge_values() {
    // Test boundary value conversions

    // Int32 boundary values
    assert_eq!(Value::Int32(i32::MAX).to::<i64>().unwrap(), i32::MAX as i64);
    assert_eq!(Value::Int32(i32::MIN).to::<i64>().unwrap(), i32::MIN as i64);
    assert_eq!(Value::Int32(i32::MAX).to::<f64>().unwrap(), i32::MAX as f64);
    assert_eq!(Value::Int32(i32::MIN).to::<f64>().unwrap(), i32::MIN as f64);

    // Int64 boundary values
    assert_eq!(Value::Int64(i64::MAX).to::<f64>().unwrap(), i64::MAX as f64);
    assert_eq!(Value::Int64(i64::MIN).to::<f64>().unwrap(), i64::MIN as f64);

    // UInt32 boundary values
    assert_eq!(
        Value::UInt32(u32::MAX).to::<i64>().unwrap(),
        u32::MAX as i64
    );
    assert_eq!(
        Value::UInt32(u32::MAX).to::<f64>().unwrap(),
        u32::MAX as f64
    );

    // Float32 boundary values
    assert_eq!(
        Value::Float32(f32::MAX).to::<f64>().unwrap(),
        f32::MAX as f64
    );
    assert_eq!(
        Value::Float32(f32::MIN).to::<f64>().unwrap(),
        f32::MIN as f64
    );
}
#[test]
fn test_as_int32_bigdecimal_out_of_range() {
    use bigdecimal::BigDecimal;
    use std::str::FromStr;

    // Create a BigDecimal out of i32 range
    let huge_decimal = BigDecimal::from_str("999999999999999999.123").unwrap();
    let value = Value::BigDecimal(huge_decimal);

    // Attempting conversion should fail
    let result = value.to::<i32>();
    assert!(result.is_err());
}
#[test]
fn test_as_int32_non_numeric_type_conversion_failed() {
    use chrono::NaiveDate;

    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);

    // Date type cannot convert to i32
    let result = value.to::<i32>();
    assert!(result.is_err());

    // Verify error type
    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Date);
            assert_eq!(to, DataType::Int32);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Test other non-numeric types
    use chrono::NaiveTime;
    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);
    assert!(value.to::<i32>().is_err());

    use chrono::DateTime;
    let datetime = DateTime::from_timestamp(1_000_000_000, 0)
        .unwrap()
        .naive_utc();
    let value = Value::DateTime(datetime);
    assert!(value.to::<i32>().is_err());
}
#[test]
fn test_as_int64_non_numeric_type_conversion_failed() {
    use chrono::NaiveTime;

    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);

    // Time type cannot convert to i64
    let result = value.to::<i64>();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Time);
            assert_eq!(to, DataType::Int64);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Test Date type
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);
    assert!(value.to::<i64>().is_err());
}
#[test]
fn test_as_int64_big_types_edge_cases() {
    use bigdecimal::BigDecimal;
    use num_bigint::BigInt;
    use std::str::FromStr;

    // BigInteger out of i64 range
    let huge_bigint = BigInt::from_str("99999999999999999999").unwrap();
    let value = Value::BigInteger(huge_bigint);
    let result = value.to::<i64>();
    assert!(result.is_err());

    // BigDecimal with decimal part should be able to convert
    let decimal = BigDecimal::from_str("123.456").unwrap();
    let value = Value::BigDecimal(decimal);
    let result = value.to::<i64>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 123);
}
#[test]
fn test_as_float64_non_numeric_type_conversion_failed() {
    use chrono::{DateTime, Utc};

    // DateTime type cannot convert to f64
    let datetime = DateTime::from_timestamp(1_000_000_000, 0)
        .unwrap()
        .naive_utc();
    let value = Value::DateTime(datetime);

    let result = value.to::<f64>();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::DateTime);
            assert_eq!(to, DataType::Float64);
        }
        _ => panic!("Expected ConversionFailed error"),
    }

    // Instant type also cannot convert
    let instant = Utc::now();
    let value = Value::Instant(instant);
    assert!(value.to::<f64>().is_err());

    // Date type
    use chrono::NaiveDate;
    let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
    let value = Value::Date(date);
    assert!(value.to::<f64>().is_err());

    // Time type
    use chrono::NaiveTime;
    let time = NaiveTime::from_hms_opt(12, 30, 45).unwrap();
    let value = Value::Time(time);
    assert!(value.to::<f64>().is_err());
}
#[test]
fn test_float32_as_bool_conversion() {
    let f32_zero = Value::Float32(0.0);
    let f32_nonzero = Value::Float32(3.5);

    // Float32 does not support conversion to bool (only integer types support)
    assert!(f32_zero.to::<bool>().is_err());
    assert!(f32_nonzero.to::<bool>().is_err());

    // Verify error type
    match f32_zero.to::<bool>() {
        Err(ValueError::ConversionFailed { from, to }) => {
            assert_eq!(from, DataType::Float32);
            assert_eq!(to, DataType::Bool);
        }
        _ => panic!("Expected ConversionFailed error"),
    }
}
#[test]
fn test_char_numeric_conversions() {
    let char_val = Value::Char('A');

    // 'A' ASCII value is 65
    assert_eq!(char_val.to::<i32>().unwrap(), 65);
    assert_eq!(char_val.to::<i64>().unwrap(), 65);
    assert_eq!(char_val.to::<f64>().unwrap(), 65.0);
}
#[test]
fn test_float_to_int64_conversions() {
    let f32_val = Value::Float32(42.7);
    assert_eq!(f32_val.to::<i64>().unwrap(), 42);

    let f64_val = Value::Float64(123.9);
    assert_eq!(f64_val.to::<i64>().unwrap(), 123);
}
#[test]
fn test_float_to_int_conversions_reject_non_finite_values() {
    assert!(Value::Float64(f64::INFINITY).to::<i32>().is_err());
    assert!(Value::Float64(f64::NEG_INFINITY).to::<i32>().is_err());
    assert!(Value::Float64(f64::NAN).to::<i32>().is_err());

    assert!(Value::Float32(f32::INFINITY).to::<i64>().is_err());
    assert!(Value::Float32(f32::NEG_INFINITY).to::<i64>().is_err());
    assert!(Value::Float32(f32::NAN).to::<i64>().is_err());
}
#[test]
fn test_float_to_int_conversions_reject_out_of_range_values() {
    assert!(Value::Float64(i32::MAX as f64 + 1.0).to::<i32>().is_err());
    assert!(Value::Float64(i32::MIN as f64 - 1.0).to::<i32>().is_err());

    assert!(Value::Float64(i64::MAX as f64 * 2.0).to::<i64>().is_err());
    assert!(Value::Float64(i64::MIN as f64 * 2.0).to::<i64>().is_err());
}
#[test]
fn test_uint128_to_int64_overflow() {
    use std::str::FromStr;

    // Create a u128 out of i64 range
    let huge_val = u128::from_str("99999999999999999999").unwrap();
    let value = Value::UInt128(huge_val);

    let result = value.to::<i64>();
    assert!(result.is_err());

    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128 value out of i64 range"));
        }
        _ => panic!("Expected ConversionError"),
    }
}
#[test]
fn test_as_int64_bool_conversion() {
    // Bool(true) should convert to 1
    let value_true = Value::Bool(true);
    assert_eq!(value_true.to::<i64>().unwrap(), 1i64);

    // Bool(false) should convert to 0
    let value_false = Value::Bool(false);
    assert_eq!(value_false.to::<i64>().unwrap(), 0i64);
}
#[test]
fn test_as_int64_int128_overflow() {
    // Test positive overflow
    let value_max = Value::Int128(i128::MAX);
    let result = value_max.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 overflow"),
    }

    // Test negative overflow
    let value_min = Value::Int128(i128::MIN);
    let result = value_min.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("i128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for i128 underflow"),
    }
}
#[test]
fn test_as_int64_uint64_overflow() {
    // u64::MAX exceeds i64 range
    let value = Value::UInt64(u64::MAX);
    let result = value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 overflow"),
    }

    // u64 just exceeding i64::MAX
    let value = Value::UInt64(i64::MAX as u64 + 1);
    let result = value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u64"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u64 just over i64::MAX"),
    }
}
#[test]
fn test_as_int64_uint128_overflow() {
    // u128::MAX exceeds i64 range
    let value = Value::UInt128(u128::MAX);
    let result = value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for u128 overflow"),
    }

    // u128 out of i64 range
    let large_value = Value::UInt128(i64::MAX as u128 + 1000);
    let result = large_value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("u128"));
            assert!(msg.contains("i64"));
            assert!(msg.contains("range"));
        }
        _ => panic!("Expected ConversionError for large u128"),
    }
}
#[test]
fn test_as_int64_bigdecimal_conversion_failed() {
    use std::str::FromStr;

    // Create a BigDecimal out of i64 range
    let huge_decimal = BigDecimal::from_str("999999999999999999999.123").unwrap();
    let value = Value::BigDecimal(huge_decimal);

    let result = value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigDecimal"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for BigDecimal conversion"),
    }

    // Test very small negative BigDecimal
    let tiny_decimal = BigDecimal::from_str("-999999999999999999999.123").unwrap();
    let value = Value::BigDecimal(tiny_decimal);
    let result = value.to::<i64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("BigDecimal"));
            assert!(msg.contains("i64"));
        }
        _ => panic!("Expected ConversionError for negative BigDecimal conversion"),
    }
}
#[test]
fn test_as_float64_string_parse_failed() {
    // Not a number at all string
    let value = Value::String("not_a_number".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("not_a_number"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError for string parse failure"),
    }

    // Empty string
    let value = Value::String("".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Contains non-numeric characters
    let value = Value::String("12.34xyz".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Multiple decimal points
    let value = Value::String("1.2.3".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());
}
#[test]
fn test_as_float64_biginteger_conversion_edge_cases() {
    use std::str::FromStr;

    // BigInteger within normal range should convert successfully
    let normal_value = Value::BigInteger(BigInt::from(12345));
    assert_eq!(normal_value.to::<f64>().unwrap(), 12345.0);

    // Very large BigInteger should fail
    let huge_bigint = BigInt::from_str(&"9".repeat(400)).unwrap();
    let value = Value::BigInteger(huge_bigint);
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Negative very large BigInteger should also fail
    let neg_huge_bigint = BigInt::from_str(&format!("-{}", "9".repeat(400))).unwrap();
    let value = Value::BigInteger(neg_huge_bigint);
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Zero value BigInteger
    let zero_bigint = BigInt::from(0);
    let value = Value::BigInteger(zero_bigint);
    assert_eq!(value.to::<f64>().unwrap(), 0.0);
}
#[test]
fn test_as_float64_bigdecimal_conversion_edge_cases() {
    use std::str::FromStr;

    // BigDecimal within normal range should convert successfully
    let normal_value = Value::BigDecimal(BigDecimal::from_str("123.456").unwrap());
    assert!((normal_value.to::<f64>().unwrap() - 123.456).abs() < 1e-10);

    // Very large BigDecimal should fail
    let huge_decimal = BigDecimal::from_str("1e400").unwrap();
    let value = Value::BigDecimal(huge_decimal);
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Negative very large BigDecimal should also fail
    let neg_huge_decimal = BigDecimal::from_str("-1e400").unwrap();
    let value = Value::BigDecimal(neg_huge_decimal);
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Very small positive BigDecimal (close to zero)
    let tiny_decimal = BigDecimal::from_str("1e-400").unwrap();
    let value = Value::BigDecimal(tiny_decimal);
    let result = value.to::<f64>();
    assert!(result.is_ok());
    // Very small numbers may convert to 0.0
    let float_result = result.unwrap();
    assert!(float_result >= 0.0);

    // Zero value BigDecimal
    let zero_decimal = BigDecimal::from_str("0.0").unwrap();
    let value = Value::BigDecimal(zero_decimal);
    assert_eq!(value.to::<f64>().unwrap(), 0.0);
}
#[test]
fn test_as_float64_bool_type_all_branches() {
    // Test Bool(true) conversion to f64
    let value_true = Value::Bool(true);
    let result = value_true.to::<f64>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 1.0);

    // Test Bool(false) conversion to f64
    let value_false = Value::Bool(false);
    let result = value_false.to::<f64>();
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 0.0);

    // Verify both branches are tested
    assert_ne!(
        value_true.to::<f64>().unwrap(),
        value_false.to::<f64>().unwrap()
    );
}
#[test]
fn test_as_float64_string_parse_all_error_cases() {
    // Test not a number at all string
    let value = Value::String("abc".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(msg)) => {
            assert!(msg.contains("Cannot convert"));
            assert!(msg.contains("abc"));
            assert!(msg.contains("f64"));
        }
        _ => panic!("Expected ConversionError"),
    }

    // Test empty string
    let value = Value::String("".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());
    match result {
        Err(ValueError::ConversionError(_)) => {
            // Expected
        }
        _ => panic!("Expected ConversionError for empty string"),
    }

    // Test string containing letters
    let value = Value::String("12.34abc".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test multiple decimal points
    let value = Value::String("1.2.3".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test string with only symbols
    let value = Value::String("+".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    let value = Value::String("-".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test string containing spaces (Leading or trailing spaces)
    let value = Value::String("  123.45  ".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test special characters
    let value = Value::String("@#$%".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test Chinese characters
    let value = Value::String("一二三".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Test mixed characters
    let value = Value::String("123abc456".to_string());
    let result = value.to::<f64>();
    assert!(result.is_err());

    // Comparison: Valid floating point string should succeed
    let valid_value = Value::String("123.45".to_string());
    assert!(valid_value.to::<f64>().is_ok());
    assert_eq!(valid_value.to::<f64>().unwrap(), 123.45);
}
#[test]
fn test_to_unsigned_with_range_checks() {
    assert_eq!(Value::UInt8(42).to::<u16>().unwrap(), 42u16);
    assert_eq!(Value::UInt16(42).to::<u32>().unwrap(), 42u32);
    assert_eq!(Value::UInt32(42).to::<u64>().unwrap(), 42u64);
    assert_eq!(Value::UInt64(42).to::<u128>().unwrap(), 42u128);

    assert_eq!(Value::Int8(42).to::<u16>().unwrap(), 42u16);
    assert_eq!(Value::Int16(42).to::<u32>().unwrap(), 42u32);
    assert_eq!(Value::Int32(42).to::<u64>().unwrap(), 42u64);
    assert_eq!(Value::Int64(42).to::<u128>().unwrap(), 42u128);

    assert_eq!(Value::Bool(true).to::<u8>().unwrap(), 1u8);
    assert_eq!(Value::Char('A').to::<u16>().unwrap(), 65u16);
    assert_eq!(Value::String("255".to_string()).to::<u8>().unwrap(), 255u8);
}
#[test]
fn test_to_unsigned_range_failures() {
    assert!(Value::Int8(-1).to::<u16>().is_err());
    assert!(Value::Int16(-1).to::<u32>().is_err());
    assert!(Value::UInt16(256).to::<u8>().is_err());
    assert!(Value::UInt32(u16::MAX as u32 + 1).to::<u16>().is_err());
    assert!(Value::UInt64(u32::MAX as u64 + 1).to::<u32>().is_err());
    assert!(Value::UInt128(u64::MAX as u128 + 1).to::<u64>().is_err());
}
#[test]
fn test_to_f32_extended_sources() {
    assert_eq!(Value::Bool(true).to::<f32>().unwrap(), 1.0f32);
    assert_eq!(Value::Char('A').to::<f32>().unwrap(), 65.0f32);
    assert_eq!(Value::Int32(42).to::<f32>().unwrap(), 42.0f32);
    assert_eq!(Value::UInt64(42).to::<f32>().unwrap(), 42.0f32);
    assert_eq!(Value::Float64(3.5).to::<f32>().unwrap(), 3.5f32);
    assert_eq!(
        Value::String("2.5".to_string()).to::<f32>().unwrap(),
        2.5f32
    );
}
#[test]
fn test_to_f32_range_failures() {
    assert!(Value::Float64(f64::MAX).to::<f32>().is_err());
}
#[test]
fn test_big_number_to_f32_and_f64_failures() {
    let huge_big_int = BigInt::from_str(&"9".repeat(2000)).unwrap();
    assert!(Value::BigInteger(huge_big_int.clone()).to::<f32>().is_err());
    assert!(Value::BigInteger(huge_big_int).to::<f64>().is_err());

    let huge_big_decimal = BigDecimal::from_str("1.0e10000").unwrap();
    assert!(
        Value::BigDecimal(huge_big_decimal.clone())
            .to::<f32>()
            .is_err()
    );
    assert!(Value::BigDecimal(huge_big_decimal).to::<f64>().is_err());
}

#[test]
fn test_narrow_signed_integer_converters_accept_numeric_sources() {
    let cases = vec![
        (Value::Bool(true), 1i128),
        (Value::Bool(false), 0),
        (Value::Char('A'), 65),
        (Value::Int8(-8), -8),
        (Value::Int16(-16), -16),
        (Value::Int32(-32), -32),
        (Value::Int64(-64), -64),
        (Value::Int128(-128), -128),
        (Value::IntSize(-256), -256),
        (Value::UInt8(8), 8),
        (Value::UInt16(16), 16),
        (Value::UInt32(32), 32),
        (Value::UInt64(64), 64),
        (Value::UInt128(128), 128),
        (Value::UIntSize(512), 512),
        (Value::Float32(12.9), 12),
        (Value::Float64(-34.7), -34),
        (Value::String("-55".to_string()), -55),
        (Value::BigInteger(BigInt::from(99)), 99),
        (Value::BigDecimal(BigDecimal::from(123)), 123),
    ];

    for (value, expected) in cases {
        assert_eq!(value.to::<i128>().unwrap(), expected);
    }

    assert_eq!(Value::UInt16(127).to::<i8>().unwrap(), 127);
    assert_eq!(Value::Float64(-12.9).to::<i8>().unwrap(), -12);
    assert_eq!(
        Value::BigDecimal(BigDecimal::from(32_000))
            .to::<i16>()
            .unwrap(),
        32_000
    );
    assert_eq!(
        Value::String("-123".to_string()).to::<isize>().unwrap(),
        -123
    );
}

#[test]
fn test_narrow_signed_integer_converters_reject_invalid_values() {
    assert!(Value::String("128".to_string()).to::<i8>().is_err());
    assert!(Value::Char('\u{80}').to::<i8>().is_err());
    assert!(Value::Int32(i16::MAX as i32 + 1).to::<i16>().is_err());
    assert!(Value::Int128(isize::MAX as i128 + 1).to::<isize>().is_err());
    assert!(Value::UInt128(i128::MAX as u128 + 1).to::<i128>().is_err());
    assert!(Value::Float64(f64::INFINITY).to::<i128>().is_err());
    assert!(Value::Float64(f64::MAX).to::<i128>().is_err());
    assert!(Value::String("invalid".to_string()).to::<i16>().is_err());

    let too_big = BigInt::from(i128::MAX) + BigInt::from(1u8);
    assert!(Value::BigInteger(too_big).to::<i128>().is_err());
    assert!(
        Value::BigDecimal(BigDecimal::from_str("1e100").unwrap())
            .to::<i128>()
            .is_err()
    );

    let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    assert!(matches!(
        Value::Date(date).to::<i128>(),
        Err(ValueError::ConversionFailed { .. })
    ));
    assert!(matches!(
        Value::Empty(DataType::Int128).to::<i128>(),
        Err(ValueError::NoValue)
    ));
}

#[test]
fn test_usize_converter_accepts_integer_sources() {
    let cases = vec![
        (Value::Bool(true), 1usize),
        (Value::Bool(false), 0),
        (Value::Char('A'), 65),
        (Value::Int8(8), 8),
        (Value::Int16(16), 16),
        (Value::Int32(32), 32),
        (Value::Int64(64), 64),
        (Value::Int128(128), 128),
        (Value::IntSize(256), 256),
        (Value::UInt8(8), 8),
        (Value::UInt16(16), 16),
        (Value::UInt32(32), 32),
        (Value::UInt64(64), 64),
        (Value::UInt128(128), 128),
        (Value::UIntSize(512), 512),
        (Value::String("1024".to_string()), 1024),
    ];

    for (value, expected) in cases {
        assert_eq!(value.to::<usize>().unwrap(), expected);
    }
}

#[test]
fn test_usize_converter_rejects_invalid_values() {
    assert!(Value::Int8(-1).to::<usize>().is_err());
    assert!(Value::IntSize(-1).to::<usize>().is_err());
    assert!(
        Value::UInt128(usize::MAX as u128 + 1)
            .to::<usize>()
            .is_err()
    );
    assert!(Value::String("invalid".to_string()).to::<usize>().is_err());
    assert!(matches!(
        Value::Empty(DataType::UIntSize).to::<usize>(),
        Err(ValueError::NoValue)
    ));
    assert!(matches!(
        Value::Float64(1.0).to::<usize>(),
        Err(ValueError::ConversionFailed { .. })
    ));
}
