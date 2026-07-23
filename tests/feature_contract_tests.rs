// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[cfg(feature = "big-decimal")]
use std::str::FromStr;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "chrono")]
use chrono::NaiveDate;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
#[cfg(feature = "converter")]
use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConversionTarget,
    DataConverter,
    DataType,
    DataTypeOf,
};
#[cfg(any(
    feature = "converter",
    feature = "chrono",
    feature = "big-integer",
    feature = "big-decimal",
    feature = "url",
    feature = "json",
))]
use qubit_value::MultiValues;
#[cfg(any(
    feature = "converter",
    feature = "chrono",
    feature = "big-integer",
    feature = "big-decimal",
    feature = "url",
    feature = "json",
    feature = "redact",
))]
use qubit_value::Value;
#[cfg(feature = "converter")]
use qubit_value::ValueContainer;
#[cfg(any(
    feature = "converter",
    feature = "chrono",
    feature = "big-integer",
    feature = "big-decimal",
    feature = "url",
    feature = "json",
))]
use serde::Serialize;
#[cfg(any(
    feature = "converter",
    feature = "chrono",
    feature = "big-integer",
    feature = "big-decimal",
    feature = "url",
    feature = "json",
))]
use serde::de::DeserializeOwned;
#[cfg(feature = "url")]
use url::Url;

#[cfg(feature = "redact")]
use std::collections::HashMap;

#[cfg(feature = "redact")]
use qubit_redact::{
    Redact as _,
    RedactionPolicy,
    Sensitivity,
};

#[cfg(any(
    feature = "converter",
    feature = "chrono",
    feature = "big-integer",
    feature = "big-decimal",
    feature = "url",
    feature = "json",
))]
fn assert_json_round_trip<T>(value: &T)
where
    T: Serialize + DeserializeOwned + PartialEq + std::fmt::Debug,
{
    let encoded = serde_json::to_string(value).expect("serialize value");
    let decoded: T = serde_json::from_str(&encoded).expect("deserialize value");
    assert_eq!(&decoded, value);
}

#[cfg(feature = "converter")]
#[test]
fn converter_feature_converts_core_values() {
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![43_i32, 44]);

    assert_eq!(scalar.to::<i64>().expect("convert scalar"), 42);
    assert_eq!(
        collection.to_list::<i64>().expect("convert collection"),
        vec![43, 44]
    );
    assert_json_round_trip(&collection);
}

/// A downstream conversion target used to validate rs-value's public bounds.
#[cfg(feature = "converter")]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Port(u16);

#[cfg(feature = "converter")]
impl DataTypeOf for Port {
    const DATA_TYPE: DataType = DataType::UInt16;
}

#[cfg(feature = "converter")]
impl DataConversionTarget for Port {
    fn convert_from(
        source: &DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError> {
        u16::convert_from(source, options).map(Self)
    }
}

/// Verifies every value shape accepts a downstream-owned target directly.
#[cfg(feature = "converter")]
#[test]
fn converter_feature_accepts_target_side_extension() {
    assert_eq!(Value::from("8080").to::<Port>().unwrap(), Port(8080));
    assert_eq!(
        MultiValues::from(vec!["8080", "8081"])
            .to_list::<Port>()
            .unwrap(),
        vec![Port(8080), Port(8081)]
    );
    assert_eq!(
        ValueContainer::from("8082").to::<Port>().unwrap(),
        Port(8082)
    );
}

#[cfg(feature = "chrono")]
#[test]
fn chrono_feature_preserves_values_and_wire_payloads() {
    let date = NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date");
    let scalar = Value::Date(date);
    let collection = MultiValues::Date(vec![date]);

    assert_eq!(scalar.get::<NaiveDate>().expect("read date"), date);
    assert_eq!(collection.get_dates().expect("read dates"), &[date]);
    assert_json_round_trip(&scalar);
    assert_json_round_trip(&collection);
}

#[cfg(feature = "big-integer")]
#[test]
fn big_integer_feature_preserves_values_and_wire_payloads() {
    let integer = BigInt::from(123_456_789_i64);
    let integer_value = Value::BigInteger(integer.clone());
    let integers = MultiValues::BigInteger(vec![integer.clone()]);

    assert_eq!(
        integer_value.get::<BigInt>().expect("read big integer"),
        integer
    );
    assert_eq!(
        integers.get_bigintegers().expect("read big integers"),
        &[integer]
    );
    assert_json_round_trip(&integer_value);
    assert_json_round_trip(&integers);
}

#[cfg(feature = "big-decimal")]
#[test]
fn big_decimal_feature_preserves_values_and_wire_payloads() {
    let decimal = BigDecimal::from_str("123.4500").expect("valid decimal");
    let decimal_value = Value::BigDecimal(decimal.clone());
    let decimals = MultiValues::BigDecimal(vec![decimal.clone()]);

    assert_eq!(
        decimal_value.get::<BigDecimal>().expect("read big decimal"),
        decimal
    );
    assert_eq!(
        decimals.get_bigdecimals().expect("read big decimals"),
        &[decimal]
    );
    assert_json_round_trip(&decimal_value);
    assert_json_round_trip(&decimals);
}

#[cfg(feature = "url")]
#[test]
fn url_feature_preserves_values_and_wire_payloads() {
    let url = Url::parse("https://example.com/path?q=1").expect("valid URL");
    let scalar = Value::Url(url.clone());
    let collection = MultiValues::Url(vec![url.clone()]);

    assert_eq!(scalar.get::<Url>().expect("read URL"), url);
    assert_eq!(collection.get_urls().expect("read URLs"), &[url]);
    assert_json_round_trip(&scalar);
    assert_json_round_trip(&collection);
}

#[cfg(feature = "json")]
#[test]
fn json_feature_preserves_values_and_wire_payloads() {
    let json = serde_json::json!({"nested": [true, 42]});
    let scalar = Value::Json(json.clone());
    let collection = MultiValues::Json(vec![json.clone()]);

    assert_eq!(
        scalar.get::<serde_json::Value>().expect("read JSON value"),
        json
    );
    assert_eq!(collection.get_jsons().expect("read JSON values"), &[json]);
    assert_json_round_trip(&scalar);
    assert_json_round_trip(&collection);
}

#[cfg(feature = "redact")]
#[test]
fn redact_feature_masks_sensitive_string_map_entries() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let policy = RedactionPolicy::empty_builder()
        .raise("api_key", Sensitivity::Secret)
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[cfg(all(feature = "redact", feature = "json"))]
#[test]
fn redact_feature_recursively_masks_sensitive_json_object_entries() {
    let value = Value::Json(serde_json::json!({
        "profile": {
            "api_key": "nested-secret",
            "label": "visible"
        },
        "items": [
            { "token": "array-secret" },
            "unkeyed-value"
        ]
    }));
    let policy = RedactionPolicy::empty_builder()
        .raise("api_key", Sensitivity::Secret)
        .raise("token", Sensitivity::Secret)
        .build()
        .expect("policy should build");

    let debug = format!("{:#?}", value.redacted_with(&policy));
    let display = format!("{}", value.redacted_with(&policy));

    assert!(!debug.contains("nested-secret"));
    assert!(!debug.contains("array-secret"));
    assert!(debug.contains("visible"));
    assert!(debug.contains("unkeyed-value"));
    assert!(!display.contains("nested-secret"));
    assert!(!display.contains("array-secret"));
    assert!(!display.contains('\n'));
}

#[cfg(all(feature = "converter", feature = "chrono"))]
#[test]
fn converter_chrono_features_convert_text_to_date() {
    let expected = NaiveDate::from_ymd_opt(2026, 7, 15).expect("valid date");
    assert_eq!(
        Value::from("2026-07-15")
            .to::<NaiveDate>()
            .expect("convert text to date"),
        expected
    );
}

#[cfg(all(feature = "converter", feature = "big-integer"))]
#[test]
fn converter_big_integer_features_convert_text_to_big_integer() {
    assert_eq!(
        Value::from("123456789")
            .to::<BigInt>()
            .expect("convert text to big integer"),
        BigInt::from(123_456_789_i64)
    );
}

#[cfg(all(feature = "converter", feature = "big-decimal"))]
#[test]
fn converter_big_decimal_features_convert_text_to_big_decimal() {
    assert_eq!(
        Value::from("123.4500")
            .to::<BigDecimal>()
            .expect("convert text to big decimal"),
        BigDecimal::from_str("123.4500").expect("valid decimal")
    );
}

#[cfg(all(feature = "converter", feature = "url"))]
#[test]
fn converter_url_features_convert_text_to_url() {
    let expected = Url::parse("https://example.com/path").expect("valid URL");
    assert_eq!(
        Value::from("https://example.com/path")
            .to::<Url>()
            .expect("convert text to URL"),
        expected
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn converter_json_features_convert_text_to_json() {
    assert_eq!(
        Value::from(r#"{"answer":42}"#)
            .to::<serde_json::Value>()
            .expect("convert text to JSON"),
        serde_json::json!({"answer": 42})
    );
}
