// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueMissing;

#[test]
fn test_value_getter_is_strict_and_default_is_empty_only() {
    let value = Value::Int32(5);
    assert_eq!(value.get::<i32>().unwrap(), 5);
    assert!(matches!(
        value.get::<String>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));

    let empty = Value::Unset(DataType::String);
    assert_eq!(
        empty.get::<String>(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::String,
        })),
    );
    assert_eq!(empty.get_or::<String>("missing").unwrap(), "missing");
}

macro_rules! assert_scalar_getter_contract {
    ($value:expr, $getter:ident, $expected:expr, $data_type:expr, $wrong_type:expr) => {{
        let value = $value;
        assert_eq!(value.$getter().unwrap(), $expected);
        assert_eq!(
            Value::Unset($data_type).$getter(),
            Err(ValueError::Missing(ValueMissing::UnsetScalar {
                data_type: $data_type,
            }))
        );
        assert!(matches!(
            Value::Unset($wrong_type).$getter(),
            Err(ValueError::TypeMismatch { expected, actual })
                if expected == $data_type && actual == $wrong_type
        ));
    }};
}

#[test]
fn test_value_scalar_getters_cover_all_builtin_types() {
    assert_scalar_getter_contract!(
        Value::Bool(true),
        get_bool,
        true,
        DataType::Bool,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Char('x'),
        get_char,
        'x',
        DataType::Char,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Int8(-8),
        get_int8,
        -8,
        DataType::Int8,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Int16(-16),
        get_int16,
        -16,
        DataType::Int16,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Int32(-32),
        get_int32,
        -32,
        DataType::Int32,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Int64(-64),
        get_int64,
        -64,
        DataType::Int64,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Int128(-128),
        get_int128,
        -128,
        DataType::Int128,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::UInt8(8),
        get_uint8,
        8,
        DataType::UInt8,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::UInt16(16),
        get_uint16,
        16,
        DataType::UInt16,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::UInt32(32),
        get_uint32,
        32,
        DataType::UInt32,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::UInt64(64),
        get_uint64,
        64,
        DataType::UInt64,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::UInt128(128),
        get_uint128,
        128,
        DataType::UInt128,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Float32(1.5),
        get_float32,
        1.5,
        DataType::Float32,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::Float64(2.5),
        get_float64,
        2.5,
        DataType::Float64,
        DataType::String
    );
    assert_scalar_getter_contract!(
        Value::String("text".to_string()),
        get_string,
        "text",
        DataType::String,
        DataType::Bool
    );
    assert_scalar_getter_contract!(
        Value::Duration(std::time::Duration::from_secs(3)),
        get_duration,
        std::time::Duration::from_secs(3),
        DataType::Duration,
        DataType::String
    );
}

#[cfg(feature = "chrono")]
#[test]
fn test_value_chrono_getters_cover_unset_and_values() {
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use chrono::Utc;

    assert_eq!(
        Value::Date(NaiveDate::from_ymd_opt(2025, 1, 2).unwrap())
            .get_date()
            .unwrap(),
        NaiveDate::from_ymd_opt(2025, 1, 2).unwrap()
    );
    assert_eq!(
        Value::Time(NaiveTime::from_hms_opt(3, 4, 5).unwrap())
            .get_time()
            .unwrap(),
        NaiveTime::from_hms_opt(3, 4, 5).unwrap()
    );
    let datetime = NaiveDateTime::new(
        NaiveDate::from_ymd_opt(2025, 1, 2).unwrap(),
        NaiveTime::from_hms_opt(3, 4, 5).unwrap(),
    );
    assert_eq!(Value::DateTime(datetime).get_datetime().unwrap(), datetime);
    let instant = DateTime::<Utc>::from_timestamp(1_735_776_000, 0).unwrap();
    assert_eq!(Value::Instant(instant).get_instant().unwrap(), instant);
}

#[cfg(feature = "big-integer")]
#[test]
fn test_value_big_integer_getters_cover_owned_and_borrowed_access() {
    let value = Value::BigInteger(num_bigint::BigInt::from(123));
    assert_eq!(
        value.get_biginteger().unwrap(),
        num_bigint::BigInt::from(123)
    );
    assert_eq!(
        value.get_biginteger_ref().unwrap(),
        &num_bigint::BigInt::from(123)
    );
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_value_big_decimal_getters_cover_owned_and_borrowed_access() {
    let decimal = "123.45".parse::<bigdecimal::BigDecimal>().unwrap();
    let value = Value::BigDecimal(decimal.clone());
    assert_eq!(value.get_bigdecimal().unwrap(), decimal);
    assert_eq!(value.get_bigdecimal_ref().unwrap(), &decimal);
}

#[cfg(feature = "url")]
#[test]
fn test_value_url_getters_cover_owned_and_borrowed_access() {
    let url = url::Url::parse("https://example.com/path").unwrap();
    let value = Value::Url(url.clone());
    assert_eq!(value.get_url().unwrap(), url);
    assert_eq!(value.get_url_ref().unwrap(), &url);
}

#[test]
fn test_value_map_getters_cover_owned_and_borrowed_access() {
    let map = std::collections::HashMap::from([("key".to_string(), "value".to_string())]);
    let value = Value::StringMap(map.clone());
    assert_eq!(value.get_string_map().unwrap(), map);
    assert_eq!(value.get_string_map_ref().unwrap(), &map);
}

#[cfg(feature = "json")]
#[test]
fn test_value_json_getters_cover_owned_and_borrowed_access() {
    let json = serde_json::json!({"key": 1});
    let value = Value::Json(json.clone());
    assert_eq!(value.get_json().unwrap(), json);
    assert_eq!(value.get_json_ref().unwrap(), &json);
}

#[cfg(feature = "big-integer")]
#[test]
fn test_value_big_integer_borrowed_getter_reports_storage_errors() {
    assert!(matches!(
        Value::Unset(DataType::BigInteger).get_biginteger_ref(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::BigInteger,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).get_biginteger_ref(),
        Err(ValueError::TypeMismatch {
            expected: DataType::BigInteger,
            actual: DataType::Int32,
        })
    ));
}

#[cfg(feature = "big-decimal")]
#[test]
fn test_value_big_decimal_borrowed_getter_reports_storage_errors() {
    assert!(matches!(
        Value::Unset(DataType::BigDecimal).get_bigdecimal_ref(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::BigDecimal,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).get_bigdecimal_ref(),
        Err(ValueError::TypeMismatch {
            expected: DataType::BigDecimal,
            actual: DataType::Int32,
        })
    ));
}

#[cfg(feature = "url")]
#[test]
fn test_value_url_borrowed_getter_reports_storage_errors() {
    assert!(matches!(
        Value::Unset(DataType::Url).get_url_ref(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::Url,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).get_url_ref(),
        Err(ValueError::TypeMismatch {
            expected: DataType::Url,
            actual: DataType::Int32,
        })
    ));
}

#[test]
fn test_value_string_map_borrowed_getter_reports_storage_errors() {
    assert!(matches!(
        Value::Unset(DataType::StringMap).get_string_map_ref(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::StringMap,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).get_string_map_ref(),
        Err(ValueError::TypeMismatch {
            expected: DataType::StringMap,
            actual: DataType::Int32,
        })
    ));
}

#[cfg(feature = "json")]
#[test]
fn test_value_json_borrowed_getter_reports_storage_errors() {
    assert!(matches!(
        Value::Unset(DataType::Json).get_json_ref(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::Json,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).get_json_ref(),
        Err(ValueError::TypeMismatch {
            expected: DataType::Json,
            actual: DataType::Int32,
        })
    ));
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_value_json_accessors_cover_serialization_and_deserialization() {
    let value =
        Value::from_serializable(&serde_json::json!({"answer": 42})).expect("serialize JSON value");
    let decoded: std::collections::HashMap<String, i32> =
        value.deserialize_json().expect("deserialize JSON value");
    assert_eq!(decoded.get("answer"), Some(&42));
    assert!(matches!(
        Value::Unset(DataType::Json).deserialize_json::<serde_json::Value>(),
        Err(ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::Json,
        }))
    ));
    assert!(matches!(
        Value::Int32(1).deserialize_json::<serde_json::Value>(),
        Err(ValueError::TypeMismatch {
            expected: DataType::Json,
            actual: DataType::Int32,
        })
    ));
    assert!(matches!(
        Value::Json(serde_json::json!("not an integer")).deserialize_json::<i32>(),
        Err(ValueError::Conversion(_))
    ));
}
