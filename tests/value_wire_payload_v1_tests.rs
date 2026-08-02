// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_value::{
    ValueContainer,
    ValueWireDecodeError,
    ValueWireLimits,
    ValueWirePayloadV1,
};

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
        ValueWireLimits::new(input.len()),
    )
    .expect("decode bounded V1 payload");
    assert_eq!(payload.into_container(), ValueContainer::from(42_i32));

    let error = ValueWirePayloadV1::decode_json_slice_with_limits(
        input,
        ValueWireLimits::new(input.len() - 1),
    )
    .expect_err("reject payload larger than limit");
    assert!(matches!(
        error,
        ValueWireDecodeError::InputTooLarge {
            input_bytes,
            max_input_bytes,
        } if input_bytes == input.len() && max_input_bytes == input.len() - 1
    ));
}
