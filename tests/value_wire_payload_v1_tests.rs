// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetError;
use qubit_budget::JsonLimits;
use qubit_budget::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWirePayloadV1;

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
        JsonLimits::new().with_max_input_bytes(input.len()),
    )
    .expect("decode bounded V1 payload");
    assert_eq!(payload.into_container(), ValueContainer::from(42_i32));

    let error = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        JsonLimits::new().with_max_input_bytes(input.len() - 1),
    )
    .expect_err("reject payload larger than limit");
    assert!(matches!(
        error,
        ValueWireDecodeError::Budget(BudgetError::LimitExceeded {
            resource: JsonResource::InputBytes,
            actual,
            maximum,
        }) if actual == input.len() && maximum == input.len() - 1
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
