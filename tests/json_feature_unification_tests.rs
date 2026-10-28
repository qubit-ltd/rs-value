// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Public JSON contracts that must hold across unified `serde_json` features.

#[cfg(feature = "converter")]
use std::collections::HashMap;

use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonDecodeSession;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionLimits;
#[cfg(feature = "converter")]
use qubit_datatype::ConversionOperationLimits;
#[cfg(feature = "converter")]
use qubit_datatype::InvalidValueReason;
use qubit_json::decode::JsonDecoder;
#[cfg(feature = "converter")]
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueContainer;
#[cfg(feature = "converter")]
use qubit_value::ValueError;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use serde::Deserialize;
use serde_json::Value as JsonValue;
use serde_json::error::Category;

fn limits() -> JsonDecodeLimits {
    JsonDecodeLimits::builder()
        .max_input_bytes(4096usize)
        .max_depth(160usize)
        .max_nodes(4096usize)
        .max_number_bytes(128usize)
        .build()
}

#[cfg(feature = "converter")]
fn assert_projection_conversion_reason(error: ValueError, expected: InvalidValueReason) {
    let ValueError::Conversion(source) = error else {
        panic!("expected conversion error, got {error:?}");
    };
    assert_eq!(source.reason(), Some(&expected));
}

#[cfg(feature = "converter")]
fn assert_wide_integer_is_rejected(input: &str, expected: InvalidValueReason) {
    let number: JsonValue = serde_json::from_str(input).expect("valid JSON number fixture");
    // Without arbitrary_precision serde_json may already have rounded this integer
    // to f64.
    if serde_json::to_vec(&number).expect("number must serialize").as_slice() != input.as_bytes() {
        return;
    }
    assert_projection_conversion_reason(
        Value::Json(number.clone()).to_json_value().unwrap_err(),
        expected.clone(),
    );
    assert_projection_conversion_reason(
        Value::Json(serde_json::json!({"outer": [number.clone()]}))
            .to_json_value()
            .unwrap_err(),
        expected.clone(),
    );
    let values = MultiValues::Json(vec![serde_json::json!(0), number.clone()]);
    let error = values.to_json_value().unwrap_err();
    let ValueError::ListConversion(source) = error else {
        panic!("expected indexed list conversion error, got {error:?}");
    };
    assert_eq!(source.source_index(), 1);
    assert_eq!(source.conversion_error().reason(), Some(&expected));
    assert_projection_conversion_reason(
        ValueContainer::from(Value::Json(number)).to_json_value().unwrap_err(),
        expected,
    );
}

#[cfg(feature = "converter")]
#[test]
fn test_projection_rejects_unified_wide_json_number() {
    assert_wide_integer_is_rejected("18446744073709551616", InvalidValueReason::OutOfRange);
    assert_wide_integer_is_rejected("-9223372036854775809", InvalidValueReason::OutOfRange);

    // Ordinary serde_json builds may reject this token while parsing; the
    // arbitrary_precision matrix entry must preserve it for projection.
    if let Ok(non_finite) = serde_json::from_str::<JsonValue>("1e400")
        && serde_json::to_vec(&non_finite).expect("number must serialize") == b"1e400"
    {
        assert_projection_conversion_reason(
            Value::Json(non_finite.clone()).to_json_value().unwrap_err(),
            InvalidValueReason::NonFinite,
        );
        assert_projection_conversion_reason(
            Value::Json(serde_json::json!({"outer": [non_finite.clone()]}))
                .to_json_value()
                .unwrap_err(),
            InvalidValueReason::NonFinite,
        );
        let values = MultiValues::Json(vec![serde_json::json!(0), non_finite.clone()]);
        let ValueError::ListConversion(source) = values.to_json_value().unwrap_err() else {
            panic!("expected indexed list conversion error");
        };
        assert_eq!(source.source_index(), 1);
        assert_eq!(source.conversion_error().reason(), Some(&InvalidValueReason::NonFinite));
        assert_projection_conversion_reason(
            ValueContainer::from(Value::Json(non_finite.clone()))
                .to_json_value()
                .unwrap_err(),
            InvalidValueReason::NonFinite,
        );

        let limits = ConversionLimits::builder()
            .operation_limits(
                ConversionOperationLimits::builder()
                    .max_structured_payload_bytes(0)
                    .build(),
            )
            .build();
        assert!(matches!(
            Value::Json(non_finite.clone()).to_json_value_with(&qubit_datatype::ConversionPolicy::default(), &limits,),
            Err(ValueError::JsonProjectionLimit { .. })
        ));
    }

    for input in ["18446744073709551615", "-9223372036854775808"] {
        let value: JsonValue = serde_json::from_str(input).expect("valid integer boundary");
        assert_eq!(
            serde_json::to_vec(&value).expect("number must serialize"),
            input.as_bytes(),
            "boundary fixture must stay exact"
        );
        assert_eq!(Value::Json(value.clone()).to_json_value().unwrap(), value);
    }
    for input in ["1.25e100", "0.125", "-0.0"] {
        let value: JsonValue = serde_json::from_str(input).expect("valid finite float");
        assert_eq!(Value::Json(value.clone()).to_json_value().unwrap(), value);
    }

    let marker = serde_json::json!({"$serde_json::private::Number": "7"});
    assert_eq!(Value::Json(marker.clone()).to_json_value().unwrap(), marker);
}

#[test]
fn test_json_feature_unification_preserves_native_numbers_in_wire_paths() {
    let limits = limits();
    let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits));
    let wire = decoder
        .decode_unique_seed_utf8(
            ValueWireV1Seed::new(),
            br#"{"version":1,"value":{"scalar":{"uint64":18446744073709551615}}}"#,
        )
        .expect("native u64 events must survive dependency feature unification");
    assert_eq!(wire.into_container(), ValueContainer::from(u64::MAX));

    let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits));
    let payload = decoder
        .decode_unique_seed_utf8(
            ValueWirePayloadV1Seed::new(),
            br#"{"scalar":{"uint64":18446744073709551615}}"#,
        )
        .expect("native u64 events must survive payload seed decoding");
    assert_eq!(payload.into_container(), ValueContainer::from(u64::MAX));

    let envelope = ValueWireV1::decode_json_slice_with_limits(
        br#"{"version":1,"value":{"scalar":{"uint64":18446744073709551615}}}"#,
        limits,
    )
    .expect("bounded envelope should preserve native u64");
    assert_eq!(envelope.into_container(), ValueContainer::from(u64::MAX));

    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(br#"{"scalar":{"int32":-2147483648}}"#, limits)
        .expect("native i32 payload should decode");
    assert_eq!(payload.into_container(), ValueContainer::from(i32::MIN));

    let payload =
        ValueWirePayloadV1::decode_json_slice_with_limits(br#"{"scalar":{"uint64":18446744073709551615}}"#, limits)
            .expect("bounded payload should preserve native u64");
    assert_eq!(payload.into_container(), ValueContainer::from(u64::MAX));

    let float = ValueWirePayloadV1::decode_json_slice_with_limits(br#"{"scalar":{"float64":1.25e100}}"#, limits)
        .expect("finite f64 payload should decode");
    assert_eq!(float.into_container(), ValueContainer::from(1.25e100_f64));

    let float =
        ValueWireV1::decode_json_slice_with_limits(br#"{"version":1,"value":{"scalar":{"float64":-0.125}}}"#, limits)
            .expect("bounded envelope should preserve finite f64");
    assert_eq!(float.into_container(), ValueContainer::from(-0.125_f64));

    let named = NamedValue::decode_json_slice_with_limits(
        br#"{"name":"max","value":{"version":1,"value":{"scalar":{"uint64":18446744073709551615}}}}"#,
        limits,
    )
    .expect("named scalar should preserve native u64");
    assert_eq!(named.value(), &Value::UInt64(u64::MAX));

    let named_many = NamedMultiValues::decode_json_slice_with_limits(
        br#"{"name":"numbers","value":{"version":1,"value":{"collection":{"int32":[1,2]}}}}"#,
        limits,
    )
    .expect("named collection wrapper should decode");
    assert_eq!(named_many.values().len(), 2);
    assert_eq!(named_many.values().get::<i32>().expect("int32 values"), vec![1, 2]);
}

#[derive(Debug, Deserialize)]
struct EmbeddedWires {
    left: NamedValue,
    right: NamedValue,
}

#[test]
fn test_json_feature_unification_embedded_fields_share_budget() {
    let input = br#"{"left":{"name":"left","value":{"version":1,"value":{"scalar":{"int32":1}}}},"right":{"name":"right","value":{"version":1,"value":{"scalar":{"int32":2}}}}}"#;
    let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(
        JsonDecodeLimits::builder()
            .max_input_bytes(input.len())
            .max_nodes(14usize)
            .build(),
    ));
    let error = decoder
        .decode_unique_utf8::<EmbeddedWires>(input)
        .expect_err("both fields must consume one shared JSON budget");
    assert!(error.to_string().contains("Nodes"), "unexpected error: {error}");
    let mut decoder = JsonDecoder::new(JsonDecodeSession::from_limits(
        JsonDecodeLimits::builder()
            .max_input_bytes(input.len())
            .max_nodes(64usize)
            .build(),
    ));
    let decoded = decoder
        .decode_unique_utf8::<EmbeddedWires>(input)
        .expect("the same fields fit within a larger shared budget");
    assert_eq!(decoded.left.name(), "left");
    assert_eq!(decoded.left.value(), &Value::Int32(1));
    assert_eq!(decoded.right.name(), "right");
    assert_eq!(decoded.right.value(), &Value::Int32(2));
}

#[test]
fn test_json_feature_unification_rejects_duplicate_keys() {
    let error = ValueWireV1::decode_json_slice(br#"{"version":1,"value":{"scalar":{"int32":1,"int32":2}}}"#)
        .expect_err("duplicate wire keys must be rejected");
    let ValueWireDecodeError::InvalidJson(error) = error else {
        panic!("duplicate typed JSON keys should be rejected as invalid data");
    };
    assert_eq!(error.classify(), Category::Data);
}

#[test]
fn test_json_feature_unification_preserves_private_number_marker_object_key() {
    let input = br#"{"scalar":{"json":{"$serde_json::private::Number":"7"}}}"#;
    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(input, limits())
        .expect("literal marker-shaped user object must remain an object");
    let ValueContainer::Scalar(value) = payload.into_container() else {
        panic!("expected a JSON scalar payload");
    };
    assert_eq!(
        value.get_json_ref().expect("JSON value"),
        &serde_json::json!({"$serde_json::private::Number": "7"})
    );
}

#[test]
fn test_json_feature_unification_preserves_numeric_payload_tree() {
    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(
        br#"{"scalar":{"json":{"n":1.5,"u64":18446744073709551615}}}"#,
        limits(),
    )
    .expect("native numbers in a JSON payload tree should survive feature unification");
    let ValueContainer::Scalar(value) = payload.into_container() else {
        panic!("expected a JSON scalar payload");
    };
    assert_eq!(
        value.get_json_ref().expect("JSON payload tree"),
        &serde_json::json!({"n": 1.5, "u64": u64::MAX})
    );
}

#[test]
fn test_json_feature_unification_enforces_strict_number_ranges() {
    for input in [
        br#"{"scalar":{"int64":-9223372036854775809}}"#.as_slice(),
        br#"{"scalar":{"uint64":18446744073709551616}}"#.as_slice(),
        br#"{"scalar":{"float64":1e400}}"#.as_slice(),
    ] {
        assert!(
            ValueWirePayloadV1::decode_json_slice_with_limits(input, limits()).is_err(),
            "out-of-range or non-finite JSON number must fail: {}",
            String::from_utf8_lossy(input)
        );
    }
    for (input, expected) in [
        (
            br#"{"scalar":{"int64":-9223372036854775808}}"#.as_slice(),
            Value::Int64(i64::MIN),
        ),
        (
            br#"{"scalar":{"uint64":18446744073709551615}}"#.as_slice(),
            Value::UInt64(u64::MAX),
        ),
    ] {
        let payload = ValueWirePayloadV1::decode_json_slice_with_limits(input, limits())
            .expect("representable integer boundary should decode");
        assert_eq!(payload.into_container(), ValueContainer::Scalar(expected));
    }
}

#[cfg(feature = "converter")]
#[test]
fn test_json_feature_unification_canonicalizes_nested_object_order() {
    fn assert_string_map_keys_are_sorted(bytes: &[u8]) {
        let first = bytes
            .windows(b"\"a\"".len())
            .position(|window| window == b"\"a\"")
            .expect("canonical output should contain the a key");
        let second = bytes
            .windows(b"\"z\"".len())
            .position(|window| window == b"\"z\"")
            .expect("canonical output should contain the z key");
        assert!(first < second, "keys should be emitted in lexical order: {bytes:?}");
    }

    let first: JsonValue = serde_json::from_str(r#"{"outer":{"z":1,"a":2}}"#).unwrap();
    let second: JsonValue = serde_json::from_str(r#"{"outer":{"a":2,"z":1}}"#).unwrap();
    let first_value = Value::Json(first);
    let second_value = Value::Json(second);
    let first_wire = ValueWireV1::try_from(first_value.clone())
        .expect("first value converts to wire")
        .to_json_vec()
        .expect("first wire encodes");
    let second_wire = ValueWireV1::try_from(second_value.clone())
        .expect("second value converts to wire")
        .to_json_vec()
        .expect("second wire encodes");
    assert_eq!(first_wire, second_wire);
    assert_eq!(
        serde_json::to_vec(&first_value.to_json_value().unwrap()).unwrap(),
        serde_json::to_vec(&second_value.to_json_value().unwrap()).unwrap()
    );

    let mut first_map = HashMap::new();
    first_map.insert("z".to_owned(), "last".to_owned());
    first_map.insert("a".to_owned(), "first".to_owned());
    let mut second_map = HashMap::new();
    second_map.insert("a".to_owned(), "first".to_owned());
    second_map.insert("z".to_owned(), "last".to_owned());
    let first_map = Value::StringMap(first_map);
    let second_map = Value::StringMap(second_map);
    let first_wire = ValueWireV1::try_from(first_map.clone())
        .expect("first string map converts to wire")
        .to_json_vec()
        .expect("first string map wire encodes");
    let second_wire = ValueWireV1::try_from(second_map.clone())
        .expect("second string map converts to wire")
        .to_json_vec()
        .expect("second string map wire encodes");
    assert_eq!(first_wire, second_wire);
    assert_string_map_keys_are_sorted(&first_wire);
    let first_natural = serde_json::to_vec(&first_map.to_json_value().unwrap()).unwrap();
    let second_natural = serde_json::to_vec(&second_map.to_json_value().unwrap()).unwrap();
    assert_eq!(first_natural, second_natural);
    assert_string_map_keys_are_sorted(&first_natural);
}

#[test]
fn test_json_feature_unification_uses_configured_depth_for_unique_decoders() {
    let mut input = String::from(r#"{"scalar":{"json":"#);
    input.extend(std::iter::repeat_n('[', 130));
    input.push('0');
    input.extend(std::iter::repeat_n(']', 130));
    input.push_str("}}");
    let input = input.into_bytes();
    let limits = JsonDecodeLimits::builder()
        .max_input_bytes(input.len())
        .max_depth(160usize)
        .max_nodes(512usize)
        .build();
    let mut source_decoder = JsonDecoder::new(JsonDecodeSession::from_limits(limits));
    let source_tree = source_decoder.decode_unique_utf8::<JsonValue>(&input);
    assert!(
        source_tree.is_ok(),
        "the source scanner and generic JSON tree target should admit 130 levels: {source_tree:?}"
    );
    assert!(
        serde_json::from_slice::<JsonValue>(&input).is_err(),
        "serde_json::Value source-tree construction should expose its default recursion boundary"
    );
    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(&input, limits)
        .expect("typed payload decoding should follow the configured depth budget");
    assert!(matches!(payload.container(), ValueContainer::Scalar(_)));
}
