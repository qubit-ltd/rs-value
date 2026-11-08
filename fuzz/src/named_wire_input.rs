// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded generators and rejection checks for named Wire V1 wrappers.
//!
//! Generated names are limited to 96 UTF-8 bytes and reference encodings to
//! 65,536 bytes. Raw documents are decoded with a 4096-byte ceiling.

use qubit_budget::BudgetError;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::NamedMultiValues;
use qubit_value::NamedValue;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

const MAX_INPUT_BYTES: usize = 4096;
const REFERENCE_BYTES: usize = 65_536;

/// Exercises successful named scalar and collection wrapper encodings.
///
/// The borrowed input selects a bounded name; arbitrary bytes are converted
/// to valid UTF-8 and truncated on a character boundary. Every wrapper uses
/// the same 65,536-byte reference input and output ceilings.
pub fn check_structured(data: &[u8]) {
    let container = crate::structured_wire_input::container_from_bytes(data);
    let name = name_from_bytes(data);
    match container {
        ValueContainer::Scalar(value) => {
            let named = NamedValue::new(name, value);
            let encoded = named
                .to_json_vec_with_limits(encode_limits(REFERENCE_BYTES))
                .expect("bounded named scalar must encode within reference limits");
            let decoded =
                NamedValue::decode_json_slice_with_limits(&encoded, decode_limits(REFERENCE_BYTES))
                    .expect("encoded named scalar must decode within reference limits");
            assert_eq!(decoded.name(), named.name());
            assert_eq!(decoded.value(), named.value());
            assert_eq!(decode_scalar_reordered(&encoded), named);
        }
        ValueContainer::Collection(values) => {
            let named = NamedMultiValues::new(name, values);
            let encoded = named
                .to_json_vec_with_limits(encode_limits(REFERENCE_BYTES))
                .expect("bounded named collection must encode within reference limits");
            let decoded = NamedMultiValues::decode_json_slice_with_limits(
                &encoded,
                decode_limits(REFERENCE_BYTES),
            )
            .expect("encoded named collection must decode within reference limits");
            assert_eq!(decoded.name(), named.name());
            assert_eq!(decoded.values(), named.values());
            assert_eq!(decode_collection_reordered(&encoded), named);
        }
    }
}

/// Applies bounded decode profiles to both named wrapper schemas.
///
/// Syntax, schema and resource-budget rejection are expected; unexpected
/// errors panic. Successful values must round-trip under the relaxed reference
/// profile without losing name or payload identity.
pub fn check_raw(data: &[u8]) {
    check_named_value(data, selected_limits(data, 0));
    check_named_multi_values(data, selected_limits(data, 1));
}

fn name_from_bytes(data: &[u8]) -> String {
    // Keep explicit empty, Unicode and JSON-special names reachable.
    let special = ["", "λ雪", "quote\"slash\\", "control\n\t\u{0001}"];
    let selector = data.first().copied().unwrap_or(0);
    if selector & 3 != 0 {
        return special[usize::from(selector) % special.len()].to_owned();
    }
    let text = String::from_utf8_lossy(data.get(1..).unwrap_or_default());
    let mut name = String::new();
    for character in text.chars() {
        if name.len() + character.len_utf8() > 96 {
            break;
        }
        name.push(character);
    }
    name
}

fn selected_limits(data: &[u8], offset: usize) -> JsonDecodeLimits {
    let input_limits = [0, 32, MAX_INPUT_BYTES];
    let structures = [0, 1, 8, 64, 4096];
    let selector = data.get(offset).copied().unwrap_or(0);
    let base = ValueWireV1::default_json_decode_limits();
    let values = base
        .value_limits()
        .into_builder()
        .max_depth(structures[usize::from(selector) % structures.len()])
        .max_nodes(structures[usize::from(selector.rotate_left(2)) % structures.len()])
        .max_sequence_items(structures[usize::from(selector.rotate_left(4)) % structures.len()])
        .max_map_entries(structures[usize::from(selector.rotate_left(6)) % structures.len()])
        .build();
    JsonDecodeLimits::builder()
        .max_input_bytes(input_limits[usize::from(selector) % input_limits.len()])
        .value_limits(values)
        .build()
}

fn decode_limits(max_input: usize) -> JsonDecodeLimits {
    ValueWireV1::default_json_decode_limits()
        .into_builder()
        .max_input_bytes(max_input)
        .build()
}

fn encode_limits(max_output: usize) -> qubit_budget::json::JsonEncodeLimits {
    ValueWireV1::default_json_encode_limits()
        .into_builder()
        .max_output_bytes(max_output)
        .build()
}

fn check_named_value(data: &[u8], limits: JsonDecodeLimits) {
    let max_input = limits
        .max_input_bytes()
        .expect("all fuzz profiles are bounded");
    let result = NamedValue::decode_json_slice_with_limits(data, limits);
    check_input_budget(&result, data.len(), max_input);
    match result {
        Ok(value) => {
            let encoded = value
                .to_json_vec_with_limits(encode_limits(REFERENCE_BYTES))
                .expect("accepted named scalar must serialize within reference limits");
            let decoded =
                NamedValue::decode_json_slice_with_limits(&encoded, decode_limits(REFERENCE_BYTES))
                    .expect("reference named scalar must decode");
            assert_eq!(decoded, value);
        }
        Err(
            ValueWireDecodeError::Syntax(_)
            | ValueWireDecodeError::InvalidJson(_)
            | ValueWireDecodeError::Budget(_)
            | ValueWireDecodeError::UnsupportedVersion { .. },
        ) => {}
        Err(error) => panic!("named scalar returned unexpected error: {error}"),
    }
}

fn check_named_multi_values(data: &[u8], limits: JsonDecodeLimits) {
    let max_input = limits
        .max_input_bytes()
        .expect("all fuzz profiles are bounded");
    let result = NamedMultiValues::decode_json_slice_with_limits(data, limits);
    check_input_budget(&result, data.len(), max_input);
    match result {
        Ok(value) => {
            let encoded = value
                .to_json_vec_with_limits(encode_limits(REFERENCE_BYTES))
                .expect("accepted named collection must serialize within reference limits");
            let decoded = NamedMultiValues::decode_json_slice_with_limits(
                &encoded,
                decode_limits(REFERENCE_BYTES),
            )
            .expect("reference named collection must decode");
            assert_eq!(decoded, value);
        }
        Err(
            ValueWireDecodeError::Syntax(_)
            | ValueWireDecodeError::InvalidJson(_)
            | ValueWireDecodeError::Budget(_)
            | ValueWireDecodeError::UnsupportedVersion { .. },
        ) => {}
        Err(error) => panic!("named collection returned unexpected error: {error}"),
    }
}

fn check_input_budget<T: std::fmt::Debug>(
    result: &Result<T, ValueWireDecodeError>,
    actual: usize,
    maximum: usize,
) {
    if actual <= maximum {
        return;
    }
    assert!(
        matches!(result,
        Err(ValueWireDecodeError::Budget(
            BudgetError::LimitExceeded { resource: JsonResource::InputBytes, maximum: found, .. }
            | BudgetError::Insufficient { resource: JsonResource::InputBytes, limit: found, .. }
        )) if *found == maximum),
        "oversized named document must fail input-byte budget: {result:?}"
    );
}

fn decode_scalar_reordered(encoded: &[u8]) -> NamedValue {
    let mut value: serde_json::Value =
        serde_json::from_slice(encoded).expect("encoded wrapper is JSON");
    let object = value.as_object_mut().expect("wrapper is object");
    let reordered = serde_json::json!({"value": object.remove("value").expect("value field"), "name": object.remove("name").expect("name field")});
    NamedValue::decode_json_slice_with_limits(
        &serde_json::to_vec(&reordered).expect("serialize reordered wrapper"),
        decode_limits(REFERENCE_BYTES),
    )
    .expect("field order does not affect named scalar decode")
}

fn decode_collection_reordered(encoded: &[u8]) -> NamedMultiValues {
    let mut value: serde_json::Value =
        serde_json::from_slice(encoded).expect("encoded wrapper is JSON");
    let object = value.as_object_mut().expect("wrapper is object");
    let reordered = serde_json::json!({"value": object.remove("value").expect("value field"), "name": object.remove("name").expect("name field")});
    NamedMultiValues::decode_json_slice_with_limits(
        &serde_json::to_vec(&reordered).expect("serialize reordered wrapper"),
        decode_limits(REFERENCE_BYTES),
    )
    .expect("field order does not affect named collection decode")
}
