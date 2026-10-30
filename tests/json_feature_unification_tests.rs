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
use qubit_value::MultiValues;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::Value;
use qubit_value::ValueContainer;
#[cfg(feature = "converter")]
use qubit_value::ValueError;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireRefV1;
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

#[test]
fn test_wire_json_number_round_trip_boundary() {
    let number_inputs = [
        "0.12345678901234567890123",
        "1e1",
        "1.2500",
        "18446744073709551616",
        "1e400",
        "-0",
    ];
    for input in number_inputs {
        let Ok(number) = serde_json::from_str::<JsonValue>(input) else {
            continue;
        };
        let text = number.to_string();
        let expected_standard: Option<serde_json::Number> = if text == "-0" {
            serde_json::Number::from_f64(-0.0)
        } else if text.bytes().any(|byte| matches!(byte, b'.' | b'e' | b'E')) {
            text.parse::<f64>().ok().and_then(serde_json::Number::from_f64)
        } else if text.starts_with('-') {
            text.parse::<i64>().ok().map(serde_json::Number::from)
        } else {
            text.parse::<u64>().ok().map(serde_json::Number::from)
        };
        let value = Value::Json(number.clone());
        if expected_standard.as_ref() == number.as_number() {
            let wire = ValueWireV1::try_from(value.clone()).expect("standardized number is accepted");
            let bytes = wire.to_json_vec().expect("standardized number encodes");
            assert_eq!(
                ValueWireV1::decode_json_slice(&bytes)
                    .expect("standardized number decodes")
                    .into_container(),
                ValueContainer::from(value)
            );
            continue;
        }
        assert!(
            matches!(
                ValueWirePayloadV1::try_from(value.clone()),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "scalar {input} must be rejected"
        );
        assert!(
            matches!(
                ValueWirePayloadRefV1::try_from(&value),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "borrowed scalar {input} must be rejected"
        );
        assert!(matches!(
            ValueWireV1::try_from(value.clone()),
            Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
        ));
        assert!(matches!(
            ValueWireRefV1::try_from(&value),
            Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
        ));
        let named = NamedValue::new("number", value.clone());
        assert!(serde_json::to_vec(&named).is_err());
        assert!(named.to_json_vec().is_err());
        let nested = Value::Json(serde_json::json!({"outer": [number.clone()]}));
        assert!(
            matches!(
                ValueWireV1::try_from(nested.clone()),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "nested scalar {input} must be rejected"
        );
        assert!(
            matches!(
                ValueWireRefV1::try_from(&nested),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "borrowed nested scalar {input} must be rejected"
        );
        let values = MultiValues::Json(vec![serde_json::json!(0), number]);
        assert!(
            matches!(
                ValueWirePayloadV1::try_from(values.clone()),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "collection {input} must be rejected"
        );
        assert!(
            matches!(
                ValueWirePayloadRefV1::try_from(&values),
                Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
            ),
            "borrowed collection {input} must be rejected"
        );
        assert!(matches!(
            ValueWireV1::try_from(values.clone()),
            Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
        ));
        assert!(matches!(
            ValueWireRefV1::try_from(&values),
            Err(ValueWireEncodeError::JsonNumberNotRoundTrippable)
        ));
        let named = NamedMultiValues::new("numbers", values.clone());
        assert!(serde_json::to_vec(&named).is_err());
        assert!(named.to_json_vec().is_err());
    }
}

#[test]
fn test_wire_json_number_standard_values_preserve_identity() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hash;
    use std::hash::Hasher;

    let values = [
        JsonValue::from(i64::MIN),
        JsonValue::from(u64::MAX),
        JsonValue::from(0.125_f64),
        JsonValue::from(1.25e100_f64),
        JsonValue::from(-0.0_f64),
    ];
    for json in values {
        let original = ValueContainer::from(Value::Json(json));
        let wire = ValueWireV1::try_from(original.clone()).expect("standard number is accepted");
        let bytes = wire.to_json_vec().expect("standard number encodes");
        let restored = ValueWireV1::decode_json_slice(&bytes)
            .expect("standard number decodes")
            .into_container();
        assert_eq!(restored, original);
        let mut expected_hash = DefaultHasher::new();
        original.hash(&mut expected_hash);
        let mut actual_hash = DefaultHasher::new();
        restored.hash(&mut actual_hash);
        assert_eq!(actual_hash.finish(), expected_hash.finish());
    }
}

#[test]
fn test_wire_json_number_named_standard_wrappers_round_trip() {
    let value = Value::Json(JsonValue::from(42_u64));
    let named_value = NamedValue::new("answer", value.clone());

    // Standard JSON numbers remain supported through both Serde entry points.
    let serde_bytes = serde_json::to_vec(&named_value).expect("ordinary named Serde encoding");
    let serde_value: NamedValue = serde_json::from_slice(&serde_bytes).expect("ordinary named Serde decoding");
    assert_eq!(serde_value.name(), "answer");
    assert_eq!(serde_value.value(), &value);

    let bounded_bytes = named_value.to_json_vec().expect("bounded named encoding");
    let bounded_value = NamedValue::decode_json_slice(&bounded_bytes).expect("bounded named decoding");
    assert_eq!(bounded_value.name(), "answer");
    assert_eq!(bounded_value.value(), &value);

    let values = MultiValues::Json(vec![JsonValue::from(42_u64), JsonValue::from(43_u64)]);
    let named_values = NamedMultiValues::new("answers", values.clone());
    let serde_bytes = serde_json::to_vec(&named_values).expect("ordinary named-list Serde encoding");
    let serde_values: NamedMultiValues =
        serde_json::from_slice(&serde_bytes).expect("ordinary named-list Serde decoding");
    assert_eq!(serde_values.name(), "answers");
    assert_eq!(serde_values.values(), &values);

    let bounded_bytes = named_values.to_json_vec().expect("bounded named-list encoding");
    let bounded_values = NamedMultiValues::decode_json_slice(&bounded_bytes).expect("bounded named-list decoding");
    assert_eq!(bounded_values.name(), "answers");
    assert_eq!(bounded_values.values(), &values);
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
