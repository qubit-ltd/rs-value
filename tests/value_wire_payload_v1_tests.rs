// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetError;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use serde_json::json;
use serde_json::to_value;

/// Verifies unversioned V1 payloads retain an explicit collection shape.
#[test]
fn test_value_wire_payload_v1_preserves_collection_shape() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::from(vec![42_i32])).expect("construct V1 payload");

    assert_eq!(
        to_value(payload).expect("serialize V1 payload"),
        json!({"collection": {"int32": [42]}}),
    );
}

#[test]
fn test_value_wire_payload_v1_decode_json_slice_honors_limits() {
    let input = br#"{"scalar": {"int32": 42}}"#;
    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        JsonDecodeLimits::builder().max_input_bytes(input.len()).build(),
    )
    .expect("decode bounded V1 payload");
    assert_eq!(payload.into_container(), ValueContainer::from(42_i32));

    let error = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        JsonDecodeLimits::builder().max_input_bytes(input.len() - 1).build(),
    )
    .expect_err("reject payload larger than limit");
    assert!(matches!(
        error,
        ValueWireDecodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::InputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::InputBytes,
                ..
            }
        )
    ));
}

#[test]
fn test_value_wire_payload_v1_owned_conversions_cover_all_shapes() {
    let scalar = ValueWirePayloadV1::try_from(Value::Int32(7)).expect("construct scalar payload");
    assert_eq!(scalar.container(), &ValueContainer::from(7_i32));
    let scalar_container: ValueContainer = scalar.into();
    assert_eq!(scalar_container, ValueContainer::from(7_i32));

    let collection = ValueWirePayloadV1::try_from(MultiValues::Int32(vec![7])).expect("construct collection payload");
    assert_eq!(collection.container(), &ValueContainer::from(vec![7_i32]));
    let collection_container: ValueContainer = collection.into();
    assert_eq!(collection_container, ValueContainer::from(vec![7_i32]));

    let explicit = ValueContainer::Scalar(Value::String("shape".to_string()));
    let payload = ValueWirePayloadV1::try_from(explicit.clone()).expect("construct explicit payload");
    assert_eq!(payload.into_container(), explicit);
}

#[test]
fn test_value_wire_payload_v1_default_encoding_round_trips() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::from(42_i32)).expect("construct V1 payload");
    let encoded = payload.to_json_vec().expect("default limits should encode payload");

    assert_eq!(
        ValueWirePayloadV1::decode_json_slice(&encoded).expect("default limits should decode payload"),
        payload
    );
}

#[test]
fn test_value_wire_payload_ref_v1_bounded_encoding_matches_owned_payload() {
    let value = ValueContainer::from(vec![1_i32, 2]);
    let owned = ValueWirePayloadV1::try_from(value.clone()).expect("construct V1 payload");
    let borrowed = ValueWirePayloadRefV1::try_from(&value).expect("construct borrowed V1 payload");

    assert_eq!(
        borrowed.to_json_vec().expect("borrowed payload should encode"),
        owned.to_json_vec().expect("owned payload should encode")
    );
}

#[test]
fn test_value_wire_payload_ref_v1_default_writer_matches_vec() {
    let value = ValueContainer::from(42_i32);
    let borrowed = ValueWirePayloadRefV1::try_from(&value).expect("construct borrowed V1 payload");
    let mut output = Vec::new();

    borrowed
        .to_json_writer(&mut output)
        .expect("borrowed payload should encode to writer");

    assert_eq!(output, borrowed.to_json_vec().expect("borrowed payload should encode"));
}

/// The explicit unversioned seed supports embedded payloads and rejects
/// malformed shape data without requiring a complete wire envelope.
#[test]
fn test_payload_seed_decodes_scalar_and_collection_shapes() {
    for (input, expected) in [
        (r#"{"scalar":{"int32":42}}"#, ValueContainer::from(42_i32)),
        (
            r#"{"collection":{"int32":[1,2]}}"#,
            ValueContainer::from(vec![1_i32, 2]),
        ),
    ] {
        let mut decoder = Deserializer::from_str(input);
        let payload = ValueWirePayloadV1Seed::new()
            .deserialize(&mut decoder)
            .expect("valid payload");
        assert_eq!(payload.into_container(), expected);
        decoder.end().expect("complete test input");
    }
    let mut invalid = Deserializer::from_str(r#"{"scalar":{"int32":"wrong-type"}}"#);
    assert!(ValueWirePayloadV1Seed::new().deserialize(&mut invalid).is_err());
}

/// Owned DOM inputs must retain canonical integer strings just like streaming
/// decoders, including rejection of alternate numeric spellings.
#[test]
fn test_payload_seed_decodes_owned_canonical_integer_strings() {
    for (input, expected) in [
        (json!({"scalar": {"int128": "-42"}}), ValueContainer::from(-42_i128)),
        (json!({"scalar": {"uint128": "42"}}), ValueContainer::from(42_u128)),
    ] {
        let payload = ValueWirePayloadV1Seed::new()
            .deserialize(input)
            .expect("owned canonical integer");
        assert_eq!(payload.into_container(), expected);
    }
    for input in [
        json!({"scalar": {"int128": "042"}}),
        json!({"scalar": {"uint128": "-1"}}),
    ] {
        assert!(ValueWirePayloadV1Seed::new().deserialize(input).is_err());
    }

    #[cfg(feature = "big-integer")]
    {
        let input = json!({"scalar": {"biginteger": "1234567890123456789012345678901234567890"}});
        let expected = ValueWirePayloadV1::decode_json_slice(
            br#"{"scalar":{"biginteger":"1234567890123456789012345678901234567890"}}"#,
        )
        .expect("streamed arbitrary-precision integer");
        let owned = ValueWirePayloadV1Seed::new()
            .deserialize(input)
            .expect("owned arbitrary-precision integer");
        assert_eq!(owned, expected);
        assert!(
            ValueWirePayloadV1Seed::new()
                .deserialize(json!({"scalar": {"biginteger": 42}}))
                .is_err()
        );
    }
}
