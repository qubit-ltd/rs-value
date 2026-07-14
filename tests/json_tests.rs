// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::collections::HashMap;
use std::str::FromStr;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{NaiveDate, TimeZone, Utc};
use num_bigint::BigInt;
use qubit_datatype::{DataConversionError, DataType, InvalidValueReason};
use qubit_value::{MultiValues, Value, ValueError};
use serde::Serialize;
use serde::ser::SerializeMap;
use serde_json::json;
use url::Url;

#[test]
fn test_value_natural_json_projection() {
    assert_eq!(
        Value::Empty(DataType::Int32).to_json_value().unwrap(),
        serde_json::Value::Null
    );
    assert_eq!(Value::Bool(true).to_json_value().unwrap(), json!(true));
    assert_eq!(Value::Int64(-42).to_json_value().unwrap(), json!(-42));
    assert_eq!(
        Value::Int128(i128::MAX).to_json_value().unwrap(),
        json!(i128::MAX.to_string())
    );
    assert_eq!(
        Value::UInt128(u128::MAX).to_json_value().unwrap(),
        json!(u128::MAX.to_string())
    );
    assert_eq!(
        Value::BigInteger(BigInt::from(i128::MAX))
            .to_json_value()
            .unwrap(),
        json!(i128::MAX.to_string())
    );
    assert_eq!(
        Value::BigDecimal(BigDecimal::from_str("123.4500").unwrap())
            .to_json_value()
            .unwrap(),
        json!("123.4500")
    );
    assert_eq!(Value::Char('界').to_json_value().unwrap(), json!("界"));
    assert_eq!(
        Value::String("text".to_string()).to_json_value().unwrap(),
        json!("text")
    );

    let date = NaiveDate::from_ymd_opt(2026, 7, 14).unwrap();
    assert_eq!(
        Value::Date(date).to_json_value().unwrap(),
        json!("2026-07-14")
    );
    let instant = Utc.with_ymd_and_hms(2026, 7, 14, 1, 2, 3).unwrap();
    assert_eq!(
        Value::Instant(instant).to_json_value().unwrap(),
        json!(instant.to_string())
    );
    assert_eq!(
        Value::Duration(Duration::from_nanos(1_500_000))
            .to_json_value()
            .unwrap(),
        json!("2ms")
    );
    let url = Url::parse("https://example.com/path").unwrap();
    assert_eq!(
        Value::Url(url.clone()).to_json_value().unwrap(),
        json!(url.to_string())
    );

    let map = HashMap::from([("key".to_string(), "value".to_string())]);
    assert_eq!(
        Value::StringMap(map).to_json_value().unwrap(),
        json!({"key": "value"})
    );
    let nested = json!({"items": [1, null, true]});
    assert_eq!(Value::Json(nested.clone()).to_json_value().unwrap(), nested);
}

#[test]
fn test_multi_values_natural_json_projection_uses_cardinality() {
    assert_eq!(
        MultiValues::Empty(DataType::Int32).to_json_value().unwrap(),
        serde_json::Value::Null
    );
    assert_eq!(
        MultiValues::Int32(Vec::new()).to_json_value().unwrap(),
        json!([])
    );
    assert_eq!(
        MultiValues::Int32(vec![42]).to_json_value().unwrap(),
        json!(42)
    );
    assert_eq!(
        MultiValues::Int32(vec![1, 2, 3]).to_json_value().unwrap(),
        json!([1, 2, 3])
    );
    assert_eq!(
        MultiValues::StringMap(vec![HashMap::from([(
            "key".to_string(),
            "value".to_string(),
        )])])
        .to_json_value()
        .unwrap(),
        json!({"key": "value"})
    );
}

#[test]
fn test_natural_json_projection_reports_non_finite_values() {
    assert!(matches!(
        Value::Float64(f64::NAN).to_json_value(),
        Err(ValueError::DataConversion(
            DataConversionError::InvalidValue {
                from: DataType::Float64,
                to: DataType::Json,
                reason: InvalidValueReason::NonFinite,
            }
        ))
    ));

    assert!(matches!(
        MultiValues::Float32(vec![1.0, f32::INFINITY]).to_json_value(),
        Err(ValueError::DataListConversion(error))
            if error.source_index == 1
                && error.source == DataConversionError::InvalidValue {
                    from: DataType::Float32,
                    to: DataType::Json,
                    reason: InvalidValueReason::NonFinite,
                }
    ));
}

#[derive(Serialize)]
struct SerializablePayload {
    values: Vec<f64>,
    missing: Option<String>,
}

struct NonFiniteMapKey;

impl Serialize for NonFiniteMapKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut map = serializer.serialize_map(Some(1))?;
        map.serialize_entry(&f64::INFINITY, &1_u8)?;
        map.end()
    }
}

#[test]
fn test_from_serializable_rejects_nested_non_finite_floats() {
    assert!(matches!(
        Value::from_serializable(&f64::NAN),
        Err(ValueError::DataConversion(
            DataConversionError::InvalidValue {
                from: DataType::Json,
                to: DataType::Json,
                reason: InvalidValueReason::NonFinite,
            }
        ))
    ));

    let payload = SerializablePayload {
        values: vec![1.0, f64::NEG_INFINITY],
        missing: None,
    };
    assert!(matches!(
        Value::from_serializable(&payload),
        Err(ValueError::DataConversion(
            DataConversionError::InvalidValue {
                reason: InvalidValueReason::NonFinite,
                ..
            }
        ))
    ));

    assert!(matches!(
        Value::from_serializable(&NonFiniteMapKey),
        Err(ValueError::DataConversion(
            DataConversionError::InvalidValue {
                reason: InvalidValueReason::NonFinite,
                ..
            }
        ))
    ));
}

#[test]
fn test_from_serializable_preserves_legitimate_null() {
    let payload = SerializablePayload {
        values: vec![1.0, 2.0],
        missing: None,
    };
    assert_eq!(
        Value::from_serializable(&payload).unwrap(),
        Value::Json(json!({"values": [1.0, 2.0], "missing": null}))
    );
}
