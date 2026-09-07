// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared helpers for integration tests.

use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use serde::de::DeserializeSeed;

pub(crate) fn decode_value_wire_str(input: &str) -> Result<ValueWireV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

pub(crate) fn decode_value_wire_value(input: serde_json::Value) -> Result<ValueWireV1, serde_json::Error> {
    decode_value_wire_str(&serde_json::to_string(&input).expect("test JSON value should serialize"))
}

pub(crate) fn decode_value_wire_slice(input: &[u8]) -> Result<ValueWireV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

pub(crate) fn decode_value_wire_payload_str(input: &str) -> Result<ValueWirePayloadV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    ValueWirePayloadV1Seed::new().deserialize(&mut deserializer)
}

pub(crate) fn decode_value_wire_payload_value(
    input: serde_json::Value,
) -> Result<ValueWirePayloadV1, serde_json::Error> {
    decode_value_wire_payload_str(&serde_json::to_string(&input).expect("test JSON value should serialize"))
}
