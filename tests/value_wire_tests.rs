// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Golden tests for the type-preserving versioned wire representation.

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
    NamedMultiValues,
    NamedValue,
    Value,
    ValueContainer,
    ValueWireEncodeError,
    ValueWirePayloadRefV1,
    ValueWirePayloadV1,
    ValueWireV1,
};
use serde_json::{
    Map,
    Value as JsonValue,
    json,
};
use url::Url;

#[test]
fn test_value_wire_v1_identity_preserves_shape() {
    assert_ne!(
        ValueWireV1::try_from(Value::Int32(1)).expect("construct scalar wire"),
        ValueWireV1::try_from(MultiValues::Int32(vec![1]))
            .expect("construct collection wire"),
    );
}

#[test]
fn test_value_wire_v1_preserves_f64_round_trip() {
    let value = Value::Float64(625_026_605_f64 / 3.0);
    let wire = ValueWireV1::try_from(value).expect("construct float wire");
    let encoded = serde_json::to_vec(&wire).expect("serialize float wire");
    let decoded: ValueWireV1 =
        serde_json::from_slice(&encoded).expect("deserialize float wire");

    assert_eq!(decoded, wire);
}

#[test]
fn test_value_wire_v1_serializes_string_map_keys_in_dictionary_order() {
    let map = (0..128)
        .map(|index| (format!("key-{index:03}"), index.to_string()))
        .collect::<HashMap<_, _>>();
    let wire = ValueWireV1::try_from(Value::StringMap(map))
        .expect("construct string map wire");
    let encoded =
        serde_json::to_string(&wire).expect("serialize string map wire");

    let expected_entries = (0..128)
        .map(|index| format!(r#""key-{index:03}":"{index}""#))
        .collect::<Vec<_>>()
        .join(",");
    let expected = format!(
        r#"{{"version":1,"value":{{"scalar":{{"stringmap":{{{expected_entries}}}}}}}}}"#,
    );
    assert_eq!(encoded, expected);
}

/// Rejects duplicate keys in scalar and collection string-map payloads.
#[test]
fn test_value_wire_v1_rejects_duplicate_string_map_keys() {
    for input in [
        r#"{"version":1,"value":{"scalar":{"stringmap":{"key":"first","key":"second"}}}}"#,
        r#"{"version":1,"value":{"collection":{"stringmap":[{"key":"first","key":"second"}]}}}"#,
    ] {
        assert!(
            serde_json::from_str::<ValueWireV1>(input).is_err(),
            "duplicate string-map key was accepted: {input}",
        );
    }
}

/// Rejects duplicate keys at every object level of JSON payloads.
#[test]
fn test_value_wire_v1_rejects_duplicate_nested_json_keys() {
    for input in [
        r#"{"version":1,"value":{"scalar":{"json":{"key":"first","key":"second"}}}}"#,
        r#"{"version":1,"value":{"collection":{"json":[{"nested":{"key":"first","key":"second"}}]}}}"#,
    ] {
        assert!(
            serde_json::from_str::<ValueWireV1>(input).is_err(),
            "duplicate JSON key was accepted: {input}",
        );
    }
}

/// Rejects JSON objects that collide with serde_json's number marker.
#[test]
fn test_value_wire_v1_rejects_reserved_json_number_key() {
    let value = Value::Json(json!({
        "$serde_json::private::Number": "123",
        "other": true,
    }));

    assert!(matches!(
        ValueWireV1::try_from(value),
        Err(ValueWireEncodeError::ReservedJsonObjectKey { .. })
    ));
}

/// Serializes a string-map collection with dictionary-ordered keys.
#[test]
fn test_value_wire_v1_serializes_string_map_collection_keys_in_dictionary_order()
 {
    let map = HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
        ("m".to_owned(), "13".to_owned()),
    ]);
    let wire = ValueWireV1::try_from(MultiValues::StringMap(vec![map]))
        .expect("construct string-map collection wire");

    assert_eq!(
        serde_json::to_string(&wire)
            .expect("serialize string-map collection wire"),
        r#"{"version":1,"value":{"collection":{"stringmap":[{"a":"1","m":"13","z":"26"}]}}}"#,
    );
}

/// Serializes a borrowed string-map payload with dictionary-ordered keys.
#[test]
fn test_value_wire_v1_borrowed_string_map_keys_in_dictionary_order() {
    let value = Value::StringMap(HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
        ("m".to_owned(), "13".to_owned()),
    ]));
    let payload = ValueWirePayloadRefV1::try_from(&value)
        .expect("construct borrowed string-map payload");

    assert_eq!(
        serde_json::to_string(&payload)
            .expect("serialize borrowed string-map payload"),
        r#"{"scalar":{"stringmap":{"a":"1","m":"13","z":"26"}}}"#,
    );
}

/// Verifies the standalone V1 envelope wraps an unversioned V1 payload.
#[test]
fn test_value_wire_v1_wraps_unversioned_payload_and_rejects_non_finite_float() {
    let payload = ValueWirePayloadV1::try_from(Value::Int32(7))
        .expect("finite scalar should fit the V1 payload");
    assert_eq!(
        serde_json::to_value(payload).expect("payload should serialize"),
        json!({"scalar": {"int32": 7}}),
    );

    let wire = ValueWireV1::try_from(Value::Int32(7))
        .expect("finite scalar should fit the V1 envelope");
    assert_eq!(
        serde_json::to_value(wire).expect("envelope should serialize"),
        json!({"version": 1, "value": {"scalar": {"int32": 7}}}),
    );
    assert!(matches!(
        ValueWireV1::try_from(Value::Float64(f64::NAN)),
        Err(ValueWireEncodeError::NonFiniteFloat {
            data_type: DataType::Float64,
        })
    ));
}

/// Rejects arbitrary-precision decimal exponents outside V1's bounded range.
#[cfg(feature = "big-decimal")]
#[test]
fn test_value_wire_v1_rejects_excessive_big_decimal_scale() {
    let value = BigDecimal::new(BigInt::from(1), 150_001);

    assert!(matches!(
        ValueWireV1::try_from(Value::BigDecimal(value)),
        Err(ValueWireEncodeError::BigDecimalScaleTooLarge {
            scale: 150_001,
            maximum_absolute_scale: 150_000,
        })
    ));
}

/// Rejects decimal payload scales that would permit resource-exhausting values.
#[cfg(feature = "big-decimal")]
#[test]
fn test_value_wire_v1_rejects_excessive_big_decimal_scale_on_decode() {
    let input = scalar_wire(
        "bigdecimal",
        json!({"coefficient": "1", "scale": 150_001}),
    );

    let error = serde_json::from_value::<ValueWireV1>(input)
        .expect_err("excessive decimal scale must be rejected");

    assert!(error.to_string().contains("maximum absolute scale"));
}

/// Handles the minimum signed exponent without overflowing scale validation.
#[cfg(feature = "big-decimal")]
#[test]
fn test_value_wire_v1_rejects_minimum_big_decimal_scale_on_decode() {
    let input = scalar_wire(
        "bigdecimal",
        json!({"coefficient": "1", "scale": i64::MIN}),
    );

    assert!(serde_json::from_value::<ValueWireV1>(input).is_err());
}

/// Rejects URL spellings that parse successfully but are not canonical V1
/// payloads.
#[cfg(feature = "url")]
#[test]
fn test_value_wire_v1_rejects_noncanonical_url_payload() {
    let input =
        r#"{"version":1,"value":{"scalar":{"url":"HTTPS://example.com/"}}}"#;

    assert!(serde_json::from_str::<ValueWireV1>(input).is_err());
}

#[derive(Debug)]
struct ValueFixture {
    data_type: DataType,
    value: Value,
    tag: &'static str,
    payload: JsonValue,
}

fn tagged_payload(tag: &str, payload: JsonValue) -> JsonValue {
    JsonValue::Object(Map::from_iter([(tag.to_string(), payload)]))
}

fn wire_value(shape: &str, tag: &str, payload: JsonValue) -> JsonValue {
    json!({
        "version": 1,
        "value": shaped_value(shape, tag, payload),
    })
}

fn shaped_value(shape: &str, tag: &str, payload: JsonValue) -> JsonValue {
    JsonValue::Object(Map::from_iter([(
        shape.to_string(),
        tagged_payload(tag, payload),
    )]))
}

fn scalar_wire(tag: &str, payload: JsonValue) -> JsonValue {
    wire_value("scalar", tag, payload)
}

fn collection_wire(tag: &str, payload: JsonValue) -> JsonValue {
    wire_value("collection", tag, payload)
}

fn value_fixtures() -> Vec<ValueFixture> {
    vec![
        ValueFixture {
            data_type: DataType::Bool,
            value: Value::Bool(true),
            tag: "bool",
            payload: json!(true),
        },
        ValueFixture {
            data_type: DataType::Char,
            value: Value::Char('界'),
            tag: "char",
            payload: json!("界"),
        },
        ValueFixture {
            data_type: DataType::Int8,
            value: Value::Int8(-8),
            tag: "int8",
            payload: json!(-8),
        },
        ValueFixture {
            data_type: DataType::Int16,
            value: Value::Int16(-16),
            tag: "int16",
            payload: json!(-16),
        },
        ValueFixture {
            data_type: DataType::Int32,
            value: Value::Int32(-32),
            tag: "int32",
            payload: json!(-32),
        },
        ValueFixture {
            data_type: DataType::Int64,
            value: Value::Int64(-64),
            tag: "int64",
            payload: json!(-64),
        },
        ValueFixture {
            data_type: DataType::Int128,
            value: Value::Int128(i128::MIN),
            tag: "int128",
            payload: json!(i128::MIN.to_string()),
        },
        ValueFixture {
            data_type: DataType::UInt8,
            value: Value::UInt8(8),
            tag: "uint8",
            payload: json!(8),
        },
        ValueFixture {
            data_type: DataType::UInt16,
            value: Value::UInt16(16),
            tag: "uint16",
            payload: json!(16),
        },
        ValueFixture {
            data_type: DataType::UInt32,
            value: Value::UInt32(32),
            tag: "uint32",
            payload: json!(32),
        },
        ValueFixture {
            data_type: DataType::UInt64,
            value: Value::UInt64(64),
            tag: "uint64",
            payload: json!(64),
        },
        ValueFixture {
            data_type: DataType::UInt128,
            value: Value::UInt128(u128::MAX),
            tag: "uint128",
            payload: json!(u128::MAX.to_string()),
        },
        ValueFixture {
            data_type: DataType::Float32,
            value: Value::Float32(1.25),
            tag: "float32",
            payload: json!(1.25),
        },
        ValueFixture {
            data_type: DataType::Float64,
            value: Value::Float64(2.5),
            tag: "float64",
            payload: json!(2.5),
        },
        ValueFixture {
            data_type: DataType::BigInteger,
            value: Value::BigInteger(BigInt::from(123)),
            tag: "biginteger",
            payload: json!("123"),
        },
        ValueFixture {
            data_type: DataType::BigDecimal,
            value: Value::BigDecimal(
                BigDecimal::from_str("123.4500").expect("valid decimal"),
            ),
            tag: "bigdecimal",
            payload: json!({"coefficient": "1234500", "scale": 4}),
        },
        ValueFixture {
            data_type: DataType::String,
            value: Value::String("text".to_string()),
            tag: "string",
            payload: json!("text"),
        },
        ValueFixture {
            data_type: DataType::Date,
            value: Value::Date(NaiveDate::from_ymd_opt(2026, 7, 14).unwrap()),
            tag: "date",
            payload: json!("2026-07-14"),
        },
        ValueFixture {
            data_type: DataType::Time,
            value: Value::Time(NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
            tag: "time",
            payload: json!("01:02:03"),
        },
        ValueFixture {
            data_type: DataType::DateTime,
            value: Value::DateTime(
                NaiveDateTime::parse_from_str(
                    "2026-07-14 01:02:03",
                    "%Y-%m-%d %H:%M:%S",
                )
                .unwrap(),
            ),
            tag: "datetime",
            payload: json!("2026-07-14T01:02:03"),
        },
        ValueFixture {
            data_type: DataType::Instant,
            value: Value::Instant(
                Utc.with_ymd_and_hms(2026, 7, 14, 1, 2, 3).unwrap(),
            ),
            tag: "instant",
            payload: json!("2026-07-14T01:02:03Z"),
        },
        ValueFixture {
            data_type: DataType::Duration,
            value: Value::Duration(Duration::new(1, 2)),
            tag: "duration",
            payload: json!({"secs": 1, "nanos": 2}),
        },
        ValueFixture {
            data_type: DataType::Url,
            value: Value::new(Url::parse("https://example.com/path").unwrap()),
            tag: "url",
            payload: json!("https://example.com/path"),
        },
        ValueFixture {
            data_type: DataType::StringMap,
            value: Value::StringMap(HashMap::from([(
                "key".to_string(),
                "value".to_string(),
            )])),
            tag: "stringmap",
            payload: json!({"key": "value"}),
        },
        ValueFixture {
            data_type: DataType::Json,
            value: Value::Json(json!({"nested": true})),
            tag: "json",
            payload: json!({"nested": true}),
        },
    ]
}

#[test]
fn value_wire_v1_fixtures_cover_every_data_type() {
    let mut actual = value_fixtures()
        .into_iter()
        .map(|fixture| fixture.data_type)
        .collect::<Vec<_>>();
    let mut expected = DataType::ALL.to_vec();
    actual.sort_by_key(|data_type| data_type.as_str());
    expected.sort_by_key(|data_type| data_type.as_str());
    assert_eq!(actual, expected);
}

#[test]
fn value_wire_v1_unset_tags_cover_every_data_type() {
    for &data_type in DataType::ALL {
        let scalar = ValueContainer::Scalar(Value::Unset(data_type));
        let collection =
            ValueContainer::Collection(MultiValues::Unset(data_type));
        let expected_scalar = scalar_wire("unset", json!(data_type.as_str()));
        let expected_collection =
            collection_wire("unset", json!(data_type.as_str()));

        assert_eq!(
            serde_json::to_value(
                ValueWireV1::try_from(scalar.clone())
                    .expect("construct scalar wire"),
            )
            .expect("serialize unset scalar"),
            expected_scalar
        );
        assert_eq!(
            serde_json::from_value::<ValueWireV1>(expected_scalar)
                .expect("deserialize unset scalar")
                .into_container(),
            scalar
        );
        assert_eq!(
            serde_json::to_value(
                ValueWireV1::try_from(collection.clone())
                    .expect("construct collection wire"),
            )
            .expect("serialize unset collection"),
            expected_collection
        );
        assert_eq!(
            serde_json::from_value::<ValueWireV1>(expected_collection)
                .expect("deserialize unset collection")
                .into_container(),
            collection
        );
    }
}

#[test]
fn value_wire_v1_scalar_golden_round_trips_all_types() {
    for fixture in value_fixtures() {
        let expected = scalar_wire(fixture.tag, fixture.payload);
        let dto = ValueWireV1::try_from(fixture.value.clone())
            .expect("construct scalar wire");
        assert_eq!(serde_json::to_value(&dto).unwrap(), expected);
        let restored = serde_json::from_value::<ValueWireV1>(expected).unwrap();
        assert_eq!(
            ValueContainer::from(restored),
            ValueContainer::Scalar(fixture.value),
        );
    }
}

#[test]
fn value_wire_v1_collection_golden_round_trips_all_types() {
    for fixture in value_fixtures() {
        let values = MultiValues::from(fixture.value);
        let expected = collection_wire(fixture.tag, json!([fixture.payload]));
        let dto = ValueWireV1::try_from(values.clone())
            .expect("construct collection wire");
        assert_eq!(serde_json::to_value(&dto).unwrap(), expected);
        let restored = serde_json::from_value::<ValueWireV1>(expected).unwrap();
        assert_eq!(
            ValueContainer::from(restored),
            ValueContainer::Collection(values),
        );
    }
}

#[test]
fn value_wire_v1_borrowed_payload_golden_round_trips_all_types() {
    for fixture in value_fixtures() {
        let expected_scalar = shaped_value(
            "scalar",
            fixture.tag,
            fixture.payload.clone(),
        );
        let scalar = ValueWirePayloadRefV1::try_from(&fixture.value)
            .expect("construct borrowed scalar payload");
        assert_eq!(serde_json::to_value(&scalar).unwrap(), expected_scalar);
        assert_eq!(
            serde_json::from_value::<ValueWirePayloadV1>(expected_scalar)
                .unwrap()
                .into_container(),
            ValueContainer::Scalar(fixture.value.clone()),
        );

        let values = MultiValues::from(fixture.value.clone());
        let expected_collection = shaped_value(
            "collection",
            fixture.tag,
            json!([fixture.payload]),
        );
        let collection = ValueWirePayloadRefV1::try_from(&values)
            .expect("construct borrowed collection payload");
        assert_eq!(
            serde_json::to_value(&collection).unwrap(),
            expected_collection,
        );
        assert_eq!(
            serde_json::from_value::<ValueWirePayloadV1>(expected_collection)
                .unwrap()
                .into_container(),
            ValueContainer::Collection(values),
        );
    }
}

#[test]
fn value_wire_v1_preserves_unset_empty_singleton_and_json_null() {
    let cases = [
        (
            ValueContainer::Scalar(Value::Unset(DataType::Int32)),
            scalar_wire("unset", json!("int32")),
        ),
        (
            ValueContainer::Collection(MultiValues::Unset(DataType::Int32)),
            collection_wire("unset", json!("int32")),
        ),
        (
            ValueContainer::Collection(MultiValues::Int32(Vec::new())),
            collection_wire("int32", json!([])),
        ),
        (
            ValueContainer::Collection(MultiValues::Int32(vec![42])),
            collection_wire("int32", json!([42])),
        ),
        (
            ValueContainer::Scalar(Value::Json(JsonValue::Null)),
            scalar_wire("json", JsonValue::Null),
        ),
        (
            ValueContainer::Scalar(Value::Unset(DataType::Json)),
            scalar_wire("unset", json!("json")),
        ),
    ];
    for (container, expected) in cases {
        assert_eq!(
            serde_json::to_value(
                ValueWireV1::try_from(container.clone())
                    .expect("construct V1 wire"),
            )
            .unwrap(),
            expected
        );
        assert_eq!(
            serde_json::from_value::<ValueWireV1>(expected)
                .unwrap()
                .into_container(),
            container,
        );
    }
}

#[test]
fn value_wire_v1_owned_conversions_preserve_shape() {
    let into_container: fn(ValueWireV1) -> ValueContainer =
        ValueWireV1::into_container;
    let scalar =
        ValueWireV1::try_from(Value::Int32(42)).expect("construct scalar wire");
    assert_eq!(
        scalar.container(),
        &ValueContainer::Scalar(Value::Int32(42)),
    );
    let collection = ValueWireV1::try_from(MultiValues::Int32(vec![42]))
        .expect("construct collection wire");
    assert_eq!(
        collection.container(),
        &ValueContainer::Collection(MultiValues::Int32(vec![42])),
    );
    assert_eq!(
        ValueContainer::from(scalar),
        ValueContainer::Scalar(Value::Int32(42)),
    );
    assert_eq!(
        into_container(collection),
        ValueContainer::Collection(MultiValues::Int32(vec![42])),
    );
    let container =
        ValueContainer::Scalar(Value::String("explicit".to_string()));
    assert_eq!(
        into_container(
            ValueWireV1::try_from(container.clone())
                .expect("construct explicit-shape wire")
        ),
        container,
    );
}

#[test]
fn named_values_keep_outer_fields_and_embed_value_wire_v1() {
    let named = NamedValue::new("port", Value::Int32(8080));
    let expected = json!({
        "name": "port",
        "value": scalar_wire("int32", json!(8080)),
    });
    assert_eq!(serde_json::to_value(&named).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<NamedValue>(expected).unwrap(),
        named
    );

    let named =
        NamedMultiValues::new("ports", MultiValues::Int32(vec![8080, 8081]));
    let expected = json!({
        "name": "ports",
        "value": collection_wire("int32", json!([8080, 8081])),
    });
    assert_eq!(serde_json::to_value(&named).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<NamedMultiValues>(expected).unwrap(),
        named,
    );
}

#[test]
fn value_wire_v1_rejects_invalid_envelopes_and_unknown_tags() {
    let valid_value = json!({"scalar": {"int32": 42}});
    for invalid in [
        json!({"value": valid_value}),
        json!({"version": "1", "value": valid_value}),
        json!({"version": 2, "value": valid_value}),
        json!({"version": 1}),
        json!({"version": 1, "value": valid_value, "extra": true}),
        json!({"version": 1, "value": {"unknown": {"int32": 42}}}),
        json!({"version": 1, "value": {"scalar": {"unknown": 42}}}),
        json!({"version": 1, "value": {"scalar": {"int32": 42, "bool": true}}}),
    ] {
        assert!(
            serde_json::from_value::<ValueWireV1>(invalid.clone()).is_err(),
            "unexpectedly accepted {invalid}",
        );
    }
}

#[test]
fn value_wire_v1_rejects_all_legacy_external_tag_shapes() {
    for legacy in [
        json!({"Int32": 42}),
        json!({"Unset": "int32"}),
        json!({"Scalar": {"Int32": 42}}),
        json!({"Collection": {"Int32": [42]}}),
    ] {
        assert!(serde_json::from_value::<ValueWireV1>(legacy).is_err());
    }
}

#[test]
fn value_wire_v1_wide_integer_payloads_require_canonical_decimal_strings() {
    for invalid in [
        scalar_wire("int128", json!(128)),
        scalar_wire("int128", json!("12x")),
        scalar_wire("int128", json!("+1")),
        scalar_wire("int128", json!("01")),
        scalar_wire("uint128", json!("-1")),
        scalar_wire("uint128", json!("01")),
    ] {
        assert!(serde_json::from_value::<ValueWireV1>(invalid).is_err());
    }
    for invalid in [
        collection_wire("uint128", json!(["1", 2])),
        collection_wire("uint128", json!(["1", "02"])),
    ] {
        assert!(serde_json::from_value::<ValueWireV1>(invalid).is_err());
    }
}

#[test]
fn value_wire_v1_big_number_payloads_require_canonical_structures() {
    for invalid in [
        scalar_wire("biginteger", json!([1, [123]])),
        scalar_wire("biginteger", json!("12x")),
        scalar_wire("biginteger", json!("+1")),
        scalar_wire("biginteger", json!("001")),
        scalar_wire("bigdecimal", json!(12.5)),
        scalar_wire("bigdecimal", json!("1.0")),
        scalar_wire("bigdecimal", json!({"coefficient": "01", "scale": 1})),
        scalar_wire(
            "bigdecimal",
            json!({"coefficient": "1", "scale": 1, "extra": true}),
        ),
    ] {
        assert!(serde_json::from_value::<ValueWireV1>(invalid).is_err());
    }
    assert!(
        serde_json::from_value::<ValueWireV1>(collection_wire(
            "biginteger",
            json!(["1", "02"]),
        ))
        .is_err(),
    );
}

#[test]
fn value_wire_v1_duration_payload_is_strict() {
    assert!(
        serde_json::from_value::<ValueWireV1>(scalar_wire(
            "duration",
            json!({"secs": 1, "nanos": 1_000_000_000}),
        ))
        .is_err(),
    );
    assert!(
        serde_json::from_value::<ValueWireV1>(scalar_wire(
            "duration",
            json!({"secs": 1, "nanos": 2, "extra": 3}),
        ))
        .is_err(),
    );
    assert!(
        serde_json::from_value::<ValueWireV1>(collection_wire(
            "duration",
            json!([{"secs": 1, "nanos": 2, "extra": 3}]),
        ))
        .is_err(),
    );
}
