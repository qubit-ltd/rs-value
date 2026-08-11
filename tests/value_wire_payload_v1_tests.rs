// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetError;
use qubit_budget::JsonDecodeLimits;
use qubit_budget::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;

use crate::json_budget_test_support_tests::JsonDecodeLimitsExt;

/// Verifies unversioned V1 payloads retain an explicit collection shape.
#[test]
fn test_value_wire_payload_v1_preserves_collection_shape() {
    let payload =
        ValueWirePayloadV1::try_from(ValueContainer::from(vec![42_i32]))
            .expect("construct V1 payload");

    assert_eq!(
        serde_json::to_value(payload).expect("serialize V1 payload"),
        serde_json::json!({"collection": {"int32": [42]}}),
    );
}

#[test]
fn test_value_wire_payload_v1_decode_json_slice_honors_limits() {
    let input = br#"{"scalar": {"int32": 42}}"#;
    let payload = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        JsonDecodeLimits::default().with_max_input_bytes(
            u64::try_from(input.len()).expect("input length must fit"),
        ),
    )
    .expect("decode bounded V1 payload");
    assert_eq!(payload.into_container(), ValueContainer::from(42_i32));

    let error = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        JsonDecodeLimits::default().with_max_input_bytes(
            u64::try_from(input.len() - 1).expect("input length must fit"),
        ),
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
    let scalar = ValueWirePayloadV1::try_from(Value::Int32(7))
        .expect("construct scalar payload");
    assert_eq!(scalar.container(), &ValueContainer::from(7_i32));
    let scalar_container: ValueContainer = scalar.into();
    assert_eq!(scalar_container, ValueContainer::from(7_i32));

    let collection = ValueWirePayloadV1::try_from(MultiValues::Int32(vec![7]))
        .expect("construct collection payload");
    assert_eq!(collection.container(), &ValueContainer::from(vec![7_i32]));
    let collection_container: ValueContainer = collection.into();
    assert_eq!(collection_container, ValueContainer::from(vec![7_i32]));

    let explicit = ValueContainer::Scalar(Value::String("shape".to_string()));
    let payload = ValueWirePayloadV1::try_from(explicit.clone())
        .expect("construct explicit payload");
    assert_eq!(payload.into_container(), explicit);
}

#[test]
fn test_value_wire_payload_v1_default_encoding_round_trips() {
    let payload = ValueWirePayloadV1::try_from(ValueContainer::from(42_i32))
        .expect("construct V1 payload");
    let encoded = payload
        .to_json_vec()
        .expect("default limits should encode payload");

    assert_eq!(
        ValueWirePayloadV1::decode_json_slice(&encoded)
            .expect("default limits should decode payload"),
        payload
    );
}

#[test]
fn test_value_wire_payload_ref_v1_bounded_encoding_matches_owned_payload() {
    let value = ValueContainer::from(vec![1_i32, 2]);
    let owned = ValueWirePayloadV1::try_from(value.clone())
        .expect("construct V1 payload");
    let borrowed = ValueWirePayloadRefV1::try_from(&value)
        .expect("construct borrowed V1 payload");

    assert_eq!(
        borrowed
            .to_json_vec()
            .expect("borrowed payload should encode"),
        owned.to_json_vec().expect("owned payload should encode")
    );
}

#[test]
fn test_value_wire_payload_ref_v1_default_writer_matches_vec() {
    let value = ValueContainer::from(42_i32);
    let borrowed = ValueWirePayloadRefV1::try_from(&value)
        .expect("construct borrowed V1 payload");
    let mut output = Vec::new();

    borrowed
        .to_json_writer(&mut output)
        .expect("borrowed payload should encode to writer");

    assert_eq!(
        output,
        borrowed
            .to_json_vec()
            .expect("borrowed payload should encode")
    );
}
