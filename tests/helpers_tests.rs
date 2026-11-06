// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared helpers for integration tests.

#[cfg(feature = "json")]
use qubit_value::Value as RuntimeValue;
#[cfg(feature = "json")]
use qubit_value::ValueContainer;
#[cfg(feature = "json")]
use qubit_value::ValueWirePayloadV1;
#[cfg(feature = "json")]
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use serde_json::Error;
use serde_json::Value as JsonValue;
#[cfg(feature = "json")]
use serde_json::json;
use serde_json::to_string;

/// Decodes a versioned V1 envelope with the public deserialization seed.
pub(crate) fn decode_value_wire_str(input: &str) -> Result<ValueWireV1, Error> {
    let mut deserializer = Deserializer::from_str(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

/// Serializes a JSON fixture before decoding it as a versioned V1 envelope.
pub(crate) fn decode_value_wire_value(input: JsonValue) -> Result<ValueWireV1, Error> {
    decode_value_wire_str(&to_string(&input).expect("test JSON value should serialize"))
}

/// Decodes a versioned V1 envelope directly from a byte fixture.
#[cfg(feature = "json")]
pub(crate) fn decode_value_wire_slice(input: &[u8]) -> Result<ValueWireV1, Error> {
    let mut deserializer = Deserializer::from_slice(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

/// Decodes an unversioned V1 payload with the public deserialization seed.
#[cfg(feature = "json")]
pub(crate) fn decode_value_wire_payload_str(input: &str) -> Result<ValueWirePayloadV1, Error> {
    let mut deserializer = Deserializer::from_str(input);
    ValueWirePayloadV1Seed::new().deserialize(&mut deserializer)
}

/// Serializes a JSON fixture before decoding it as an unversioned V1 payload.
#[cfg(feature = "json")]
pub(crate) fn decode_value_wire_payload_value(input: JsonValue) -> Result<ValueWirePayloadV1, Error> {
    let input = to_string(&input).expect("test JSON value should serialize");
    decode_value_wire_payload_str(&input)
}

#[cfg(feature = "json")]
#[test]
fn test_wire_decode_helpers_are_available_with_json_feature() {
    let wire = crate::decode_value_wire_slice(br#"{"version":1,"value":{"scalar":{"int32":7}}}"#)
        .expect("decode the V1 envelope through the shared slice helper");
    assert_eq!(wire.into_container(), ValueContainer::Scalar(RuntimeValue::Int32(7)));

    let payload = crate::decode_value_wire_payload_value(json!({"scalar":{"int32":7}}))
        .expect("decode the unversioned V1 payload through the shared helper");
    assert_eq!(payload.into_container(), ValueContainer::Scalar(RuntimeValue::Int32(7)));
}
