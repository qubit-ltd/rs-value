// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for the explicit V1 Serde seeds.

use qubit_value::ValueContainer;
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1Seed;
use serde::de::DeserializeSeed;

#[test]
fn test_value_wire_payload_v1_seed_decodes_scalar_shape() {
    let input = r#"{"scalar":{"int32":7}}"#;
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let payload = ValueWirePayloadV1Seed::default()
        .deserialize(&mut deserializer)
        .expect("the valid scalar V1 payload should decode");

    assert_eq!(payload.into_container(), ValueContainer::from(7_i32));
}

#[test]
fn test_value_wire_payload_v1_seed_decodes_collection_shape() {
    let input = r#"{"collection":{"int32":[7,8]}}"#;
    let mut deserializer = serde_json::Deserializer::from_str(input);
    let payload = ValueWirePayloadV1Seed::new()
        .deserialize(&mut deserializer)
        .expect("the valid collection V1 payload should decode");

    assert_eq!(
        payload.into_container(),
        ValueContainer::from(vec![7_i32, 8])
    );
}

#[test]
fn test_value_wire_payload_v1_seed_rejects_an_unknown_shape_tag() {
    let input = r#"{"unknown":{"int32":7}}"#;
    let mut deserializer = serde_json::Deserializer::from_str(input);

    let error = ValueWirePayloadV1Seed::new()
        .deserialize(&mut deserializer)
        .expect_err("an unknown payload shape must be rejected");

    assert!(
        error.to_string().contains("unknown variant `unknown`"),
        "unexpected seed error: {error}"
    );
}

#[test]
fn test_value_wire_v1_seed_preserves_the_golden_envelope() {
    let input = r#"{"version":1,"value":{"scalar":{"string":"ready"}}}"#;
    let mut deserializer = serde_json::Deserializer::from_str(input);

    let wire = ValueWireV1Seed::default()
        .deserialize(&mut deserializer)
        .expect("the valid V1 envelope should decode");

    assert_eq!(wire.container(), &ValueContainer::from("ready"));
}

#[test]
fn test_value_wire_v1_seed_rejects_an_unsupported_version_precisely() {
    let input = r#"{"version":2,"value":{"scalar":{"int32":7}}}"#;
    let mut deserializer = serde_json::Deserializer::from_str(input);

    let error = ValueWireV1Seed::new()
        .deserialize(&mut deserializer)
        .expect_err("V1 seed must reject another wire version");

    assert!(
        error
            .to_string()
            .contains("unsupported qubit-value wire version 2"),
        "unexpected seed error: {error}"
    );
}
