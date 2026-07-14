// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================
//! Golden tests for the type-preserving tagged Serde representation.

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    TimeZone,
    Utc,
};
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::{
    MultiValues,
    Value,
};
use serde_json::{
    Map,
    Value as JsonValue,
    json,
};
use url::Url;

fn value_fixtures() -> Vec<(Value, JsonValue)> {
    vec![
        (Value::Unset(DataType::Int32), json!({"Unset": "int32"})),
        (Value::Bool(true), json!({"Bool": true})),
        (Value::Char('界'), json!({"Char": "界"})),
        (Value::Int8(-8), json!({"Int8": -8})),
        (Value::Int16(-16), json!({"Int16": -16})),
        (Value::Int32(-32), json!({"Int32": -32})),
        (Value::Int64(-64), json!({"Int64": -64})),
        (
            Value::Int128(i128::MIN),
            json!({"Int128": i128::MIN.to_string()}),
        ),
        (Value::UInt8(8), json!({"UInt8": 8})),
        (Value::UInt16(16), json!({"UInt16": 16})),
        (Value::UInt32(32), json!({"UInt32": 32})),
        (Value::UInt64(64), json!({"UInt64": 64})),
        (
            Value::UInt128(u128::MAX),
            json!({"UInt128": u128::MAX.to_string()}),
        ),
        (Value::Float32(1.25), json!({"Float32": 1.25})),
        (Value::Float64(2.5), json!({"Float64": 2.5})),
        (
            Value::BigInteger(BigInt::from(123)),
            json!({"BigInteger": "123"}),
        ),
        (
            Value::BigDecimal(
                BigDecimal::from_str("123.4500").expect("valid decimal"),
            ),
            json!({"BigDecimal": "123.4500"}),
        ),
        (Value::String("text".to_string()), json!({"String": "text"})),
        (
            Value::Date(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()),
            json!({"Date": "2026-07-14"}),
        ),
        (
            Value::Time(NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
            json!({"Time": "01:02:03"}),
        ),
        (
            Value::DateTime(
                NaiveDateTime::parse_from_str(
                    "2026-07-14 01:02:03",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
            ),
            json!({"DateTime": "2026-07-14T01:02:03"}),
        ),
        (
            Value::Instant(Utc.with_ymd_and_hms(2026, 7, 14, 1, 2, 3).unwrap()),
            json!({"Instant": "2026-07-14T01:02:03Z"}),
        ),
        (
            Value::Duration(Duration::new(1, 2)),
            json!({"Duration": {"secs": 1, "nanos": 2}}),
        ),
        (
            Value::Url(Url::parse("https://example.com/path").unwrap()),
            json!({"Url": "https://example.com/path"}),
        ),
        (
            Value::StringMap(HashMap::from([(
                "key".to_string(),
                "value".to_string(),
            )])),
            json!({"StringMap": {"key": "value"}}),
        ),
        (
            Value::Json(json!({"nested": true})),
            json!({"Json": {"nested": true}}),
        ),
    ]
}

#[test]
fn value_tagged_json_matches_all_variant_fixtures() {
    for (value, expected) in value_fixtures() {
        let encoded = serde_json::to_value(&value).unwrap_or_else(|error| {
            panic!("failed to serialize {value:?}: {error}")
        });
        assert_eq!(encoded, expected, "unexpected wire value for {value:?}");
        assert_eq!(serde_json::from_value::<Value>(encoded).unwrap(), value,);
    }
}

#[test]
fn multi_values_tagged_json_matches_all_variant_fixtures() {
    for (value, expected_value) in value_fixtures() {
        let values = MultiValues::from(value);
        let expected = match expected_value {
            JsonValue::Object(object) if !object.contains_key("Unset") => {
                let (variant, item) = object.into_iter().next().unwrap();
                JsonValue::Object(Map::from_iter([(
                    variant,
                    JsonValue::Array(vec![item]),
                )]))
            }
            expected => expected,
        };

        let encoded = serde_json::to_value(&values).unwrap_or_else(|error| {
            panic!("failed to serialize {values:?}: {error}")
        });
        assert_eq!(encoded, expected, "unexpected wire value for {values:?}",);
        assert_eq!(
            serde_json::from_value::<MultiValues>(encoded).unwrap(),
            values,
        );
    }
}

#[test]
fn tagged_wide_integer_payloads_require_decimal_strings() {
    assert!(serde_json::from_value::<Value>(json!({"Int128": 128})).is_err());
    assert!(serde_json::from_value::<Value>(json!({"Int128": "12x"})).is_err());
    assert!(serde_json::from_value::<Value>(json!({"UInt128": "-1"})).is_err());
    assert!(
        serde_json::from_value::<MultiValues>(json!({"UInt128": ["1", 2]}))
            .is_err()
    );
}

#[test]
fn tagged_big_number_payloads_require_decimal_strings() {
    assert!(
        serde_json::from_value::<Value>(json!({"BigInteger": [1, [123]]}))
            .is_err()
    );
    assert!(
        serde_json::from_value::<Value>(json!({"BigInteger": "12x"})).is_err()
    );
    assert!(
        serde_json::from_value::<Value>(json!({"BigDecimal": 12.5})).is_err()
    );
}

#[test]
fn tagged_duration_payload_rejects_out_of_range_nanos() {
    assert!(
        serde_json::from_value::<Value>(
            json!({"Duration": {"secs": 1, "nanos": 1_000_000_000}}),
        )
        .is_err()
    );
}
