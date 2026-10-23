// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Shared helpers for integration tests.

#[cfg(feature = "all")]
use qubit_value::ValueWirePayloadV1;
#[cfg(feature = "all")]
use qubit_value::ValueWirePayloadV1Seed;
use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use serde::de::DeserializeSeed;
use serde_json::Deserializer;
use serde_json::Error;
use serde_json::Value;

pub(crate) fn decode_value_wire_str(input: &str) -> Result<ValueWireV1, Error> {
    let mut deserializer = Deserializer::from_str(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

pub(crate) fn decode_value_wire_value(input: Value) -> Result<ValueWireV1, Error> {
    decode_value_wire_str(&serde_json::to_string(&input).expect("test JSON value should serialize"))
}

#[cfg(feature = "all")]
pub(crate) fn decode_value_wire_slice(input: &[u8]) -> Result<ValueWireV1, Error> {
    let mut deserializer = Deserializer::from_slice(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

#[cfg(feature = "all")]
pub(crate) fn decode_value_wire_payload_str(input: &str) -> Result<ValueWirePayloadV1, Error> {
    let mut deserializer = Deserializer::from_str(input);
    ValueWirePayloadV1Seed::new().deserialize(&mut deserializer)
}

#[cfg(feature = "all")]
pub(crate) fn decode_value_wire_payload_value(input: Value) -> Result<ValueWirePayloadV1, Error> {
    let input = serde_json::to_string(&input).expect("test JSON value should serialize");
    decode_value_wire_payload_str(&input)
}
