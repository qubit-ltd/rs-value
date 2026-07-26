// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests runtime value type lookup.

use qubit_datatype::DataType;
use qubit_value::Value;

/// Constructs a concrete value for one datatype catalog entry.
///
/// # Parameters
///
/// * `data_type` - Catalog entry whose concrete value variant is required.
///
/// # Returns
///
/// A concrete value carrying `data_type`.
#[cfg(feature = "all")]
fn value_for_data_type(data_type: DataType) -> Value {
    match data_type {
        DataType::Bool => Value::Bool(false),
        DataType::Char => Value::Char('\0'),
        DataType::Int8 => Value::Int8(0),
        DataType::Int16 => Value::Int16(0),
        DataType::Int32 => Value::Int32(0),
        DataType::Int64 => Value::Int64(0),
        DataType::Int128 => Value::Int128(0),
        DataType::UInt8 => Value::UInt8(0),
        DataType::UInt16 => Value::UInt16(0),
        DataType::UInt32 => Value::UInt32(0),
        DataType::UInt64 => Value::UInt64(0),
        DataType::UInt128 => Value::UInt128(0),
        DataType::Float32 => Value::Float32(0.0),
        DataType::Float64 => Value::Float64(0.0),
        DataType::String => Value::String(String::new()),
        DataType::Date => Value::Date(
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .expect("contract test date should be valid"),
        ),
        DataType::Time => Value::Time(
            chrono::NaiveTime::from_hms_opt(0, 0, 0).expect("contract test time should be valid"),
        ),
        DataType::DateTime => Value::DateTime(
            chrono::NaiveDate::from_ymd_opt(1970, 1, 1)
                .expect("contract test date should be valid")
                .and_hms_opt(0, 0, 0)
                .expect("contract test datetime should be valid"),
        ),
        DataType::Instant => Value::Instant(
            chrono::DateTime::from_timestamp(0, 0).expect("contract test instant should be valid"),
        ),
        DataType::BigInteger => Value::BigInteger(num_bigint::BigInt::from(0)),
        DataType::BigDecimal => Value::BigDecimal(bigdecimal::BigDecimal::from(0)),
        DataType::Duration => Value::Duration(std::time::Duration::ZERO),
        DataType::Url => Value::new(
            url::Url::parse("https://example.com").expect("contract test URL should be valid"),
        ),
        DataType::StringMap => Value::StringMap(std::collections::HashMap::new()),
        DataType::Json => Value::Json(serde_json::Value::Null),
    }
}

/// Verifies values report their concrete runtime data type.
#[test]
fn test_value_type_table_reports_concrete_type() {
    assert_eq!(Value::Int32(7).data_type(), DataType::Int32);
}

/// Verifies the complete value type table covers the datatype catalog.
#[cfg(feature = "all")]
#[test]
fn test_value_type_table_covers_all_data_types() {
    let actual = DataType::ALL
        .iter()
        .copied()
        .map(value_for_data_type)
        .map(|value| value.data_type())
        .collect::<Vec<_>>();

    assert_eq!(actual, DataType::ALL);
}

/// Verifies the scalar container stays compact when the URL family is enabled.
#[cfg(feature = "all")]
#[test]
fn test_value_layout_with_all_features_is_compact() {
    assert_eq!(std::mem::size_of::<Value>(), 64);
    assert_eq!(std::mem::size_of::<qubit_value::ValueContainer>(), 64);
}
