// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashMap;
use std::time::Duration;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::DateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "chrono")]
use chrono::NaiveDateTime;
#[cfg(feature = "chrono")]
use chrono::NaiveTime;
#[cfg(feature = "chrono")]
use chrono::Utc;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueMissingReason;
#[cfg(feature = "json")]
use serde_json::Value as JsonValue;
#[cfg(feature = "url")]
use url::Url;

#[test]
fn test_multi_values_converters_convert_first_list_and_value() {
    let values = MultiValues::String(vec!["1".to_string(), "2".to_string()]);
    assert_eq!(values.to_first::<i32>().unwrap(), 1);
    assert_eq!(values.to_list::<i32>().unwrap(), vec![1, 2]);
    assert_eq!(values.first_value(), Value::String("1".to_string()));
}

#[test]
fn test_multi_values_converters_report_list_conversion_index() {
    let values = MultiValues::String(vec!["1".to_string(), "bad".to_string()]);
    let error = values.to_list::<i32>().unwrap_err();
    assert!(matches!(
        error,
        ValueError::ListConversion(ref error)
            if error.source_index() == 1
                && error.conversion_error() == &DataConversionError::invalid(
                    DataType::String,
                    DataType::Int32,
                    InvalidValueReason::InvalidSyntax {
                        expected: "integer",
                    },
                )
    ));
}

#[test]
fn test_multi_values_empty_conversion_preserves_conversion_semantics() {
    let values = MultiValues::String(Vec::new());
    let error = values
        .to_first::<i32>()
        .expect_err("empty collection has no first converted value");

    let ValueError::Missing(missing) = error else {
        panic!("expected an empty collection conversion error");
    };
    assert_eq!(missing.reason(), ValueMissingReason::EmptyCollection);
    assert_eq!(missing.source_type(), Some(DataType::String));
    assert_eq!(missing.target_type(), Some(DataType::Int32));
}

macro_rules! assert_multi_values_identity_conversion {
    ($value:expr, $ty:ty, $first:expr, $list:expr) => {{
        let values = $value;
        assert_eq!(values.to_first::<$ty>().unwrap(), $first);
        assert_eq!(values.to_list::<$ty>().unwrap(), $list);
    }};
}

#[test]
fn test_multi_values_converters_cover_every_runtime_variant() {
    assert_multi_values_identity_conversion!(MultiValues::Bool(vec![true, false]), bool, true, vec![true, false]);
    assert_multi_values_identity_conversion!(MultiValues::Char(vec!['a', 'b']), char, 'a', vec!['a', 'b']);
    assert_multi_values_identity_conversion!(MultiValues::Int8(vec![-1, 2]), i8, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int16(vec![-1, 2]), i16, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int32(vec![-1, 2]), i32, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int64(vec![-1, 2]), i64, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int128(vec![-1, 2]), i128, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt8(vec![1, 2]), u8, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt16(vec![1, 2]), u16, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt32(vec![1, 2]), u32, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt64(vec![1, 2]), u64, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt128(vec![1, 2]), u128, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Float32(vec![1.5, 2.5]), f32, 1.5, vec![1.5, 2.5]);
    assert_multi_values_identity_conversion!(MultiValues::Float64(vec![1.5, 2.5]), f64, 1.5, vec![1.5, 2.5]);
    assert_multi_values_identity_conversion!(
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
        String,
        "a".to_string(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_multi_values_identity_conversion!(
        MultiValues::Duration(vec![Duration::from_secs(1)]),
        Duration,
        Duration::from_secs(1),
        vec![Duration::from_secs(1)]
    );

    #[cfg(feature = "chrono")]
    {
        let date = NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_multi_values_identity_conversion!(MultiValues::Date(vec![date]), NaiveDate, date, vec![date]);
        let time = NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        assert_multi_values_identity_conversion!(MultiValues::Time(vec![time]), NaiveTime, time, vec![time]);
        let datetime = date.and_time(time);
        assert_multi_values_identity_conversion!(
            MultiValues::DateTime(vec![datetime]),
            NaiveDateTime,
            datetime,
            vec![datetime]
        );
        let instant = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
        assert_multi_values_identity_conversion!(
            MultiValues::Instant(vec![instant]),
            DateTime<Utc>,
            instant,
            vec![instant]
        );
    }

    #[cfg(feature = "big-integer")]
    {
        let value = BigInt::from(123);
        assert_multi_values_identity_conversion!(
            MultiValues::BigInteger(vec![value.clone()]),
            BigInt,
            value.clone(),
            vec![value]
        );
    }
    #[cfg(feature = "big-decimal")]
    {
        let value = "1.25".parse::<BigDecimal>().unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::BigDecimal(vec![value.clone()]),
            BigDecimal,
            value.clone(),
            vec![value]
        );
    }
    #[cfg(feature = "url")]
    {
        let value = Url::parse("https://example.com").unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::Url(vec![value.clone()]),
            Url,
            value.clone(),
            vec![value]
        );
    }
    let map = HashMap::from([("key".to_string(), "value".to_string())]);
    assert_multi_values_identity_conversion!(
        MultiValues::StringMap(vec![map.clone()]),
        HashMap<String, String>,
        map.clone(),
        vec![map]
    );
    #[cfg(feature = "json")]
    {
        let value = serde_json::json!({"key": 1});
        assert_multi_values_identity_conversion!(
            MultiValues::Json(vec![value.clone()]),
            JsonValue,
            value.clone(),
            vec![value]
        );
    }
}
