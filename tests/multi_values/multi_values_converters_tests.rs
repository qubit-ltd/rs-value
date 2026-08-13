// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueMissing;

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

    let ValueError::Missing(ValueMissing::EmptyCollectionConversion { to }) = error else {
        panic!("expected an empty collection conversion error");
    };
    assert_eq!(to, DataType::Int32);
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
    assert_multi_values_identity_conversion!(
        MultiValues::Bool(vec![true, false]),
        bool,
        true,
        vec![true, false]
    );
    assert_multi_values_identity_conversion!(
        MultiValues::Char(vec!['a', 'b']),
        char,
        'a',
        vec!['a', 'b']
    );
    assert_multi_values_identity_conversion!(MultiValues::Int8(vec![-1, 2]), i8, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int16(vec![-1, 2]), i16, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int32(vec![-1, 2]), i32, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::Int64(vec![-1, 2]), i64, -1, vec![-1, 2]);
    assert_multi_values_identity_conversion!(
        MultiValues::Int128(vec![-1, 2]),
        i128,
        -1,
        vec![-1, 2]
    );
    assert_multi_values_identity_conversion!(MultiValues::UInt8(vec![1, 2]), u8, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt16(vec![1, 2]), u16, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt32(vec![1, 2]), u32, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt64(vec![1, 2]), u64, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(MultiValues::UInt128(vec![1, 2]), u128, 1, vec![1, 2]);
    assert_multi_values_identity_conversion!(
        MultiValues::Float32(vec![1.5, 2.5]),
        f32,
        1.5,
        vec![1.5, 2.5]
    );
    assert_multi_values_identity_conversion!(
        MultiValues::Float64(vec![1.5, 2.5]),
        f64,
        1.5,
        vec![1.5, 2.5]
    );
    assert_multi_values_identity_conversion!(
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
        String,
        "a".to_string(),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_multi_values_identity_conversion!(
        MultiValues::Duration(vec![std::time::Duration::from_secs(1)]),
        std::time::Duration,
        std::time::Duration::from_secs(1),
        vec![std::time::Duration::from_secs(1)]
    );

    #[cfg(feature = "chrono")]
    {
        let date = chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::Date(vec![date]),
            chrono::NaiveDate,
            date,
            vec![date]
        );
        let time = chrono::NaiveTime::from_hms_opt(1, 2, 3).unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::Time(vec![time]),
            chrono::NaiveTime,
            time,
            vec![time]
        );
        let datetime = date.and_time(time);
        assert_multi_values_identity_conversion!(
            MultiValues::DateTime(vec![datetime]),
            chrono::NaiveDateTime,
            datetime,
            vec![datetime]
        );
        let instant =
            chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(datetime, chrono::Utc);
        assert_multi_values_identity_conversion!(
            MultiValues::Instant(vec![instant]),
            chrono::DateTime<chrono::Utc>,
            instant,
            vec![instant]
        );
    }

    #[cfg(feature = "big-integer")]
    {
        let value = num_bigint::BigInt::from(123);
        assert_multi_values_identity_conversion!(
            MultiValues::BigInteger(vec![value.clone()]),
            num_bigint::BigInt,
            value.clone(),
            vec![value]
        );
    }
    #[cfg(feature = "big-decimal")]
    {
        let value = "1.25".parse::<bigdecimal::BigDecimal>().unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::BigDecimal(vec![value.clone()]),
            bigdecimal::BigDecimal,
            value.clone(),
            vec![value]
        );
    }
    #[cfg(feature = "url")]
    {
        let value = url::Url::parse("https://example.com").unwrap();
        assert_multi_values_identity_conversion!(
            MultiValues::Url(vec![value.clone()]),
            url::Url,
            value.clone(),
            vec![value]
        );
    }
    let map = std::collections::HashMap::from([("key".to_string(), "value".to_string())]);
    assert_multi_values_identity_conversion!(
        MultiValues::StringMap(vec![map.clone()]),
        std::collections::HashMap<String, String>,
        map.clone(),
        vec![map]
    );
    #[cfg(feature = "json")]
    {
        let value = serde_json::json!({"key": 1});
        assert_multi_values_identity_conversion!(
            MultiValues::Json(vec![value.clone()]),
            serde_json::Value,
            value.clone(),
            vec![value]
        );
    }
}
