// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Checks that successful bounded Wire V1 encodings pass preflight.

use std::time::Duration;

use bigdecimal::BigDecimal;
use libfuzzer_sys::fuzz_target;
use num_bigint::BigInt;
use qubit_budget::json::JsonEncodeLimits;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;

const MAX_INPUT_BYTES: usize = 4096;
const MAX_ITEMS: usize = 32;
const MAX_DEPTH: usize = 60;
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

/// Limits the fuzz input slice to the target's maximum processing size.
fn bounded_input(data: &[u8]) -> &[u8] {
    &data[..data.len().min(MAX_INPUT_BYTES)]
}

fn read_u128(data: &[u8]) -> u128 {
    let mut bytes = [0_u8; 16];
    let count = data.len().min(bytes.len());
    bytes[..count].copy_from_slice(&data[..count]);
    u128::from_le_bytes(bytes)
}

fn read_i128(data: &[u8]) -> i128 {
    let mut bytes = [0_u8; 16];
    let count = data.len().min(bytes.len());
    bytes[..count].copy_from_slice(&data[..count]);
    i128::from_le_bytes(bytes)
}

fn read_u64(data: &[u8]) -> u64 {
    let mut bytes = [0_u8; 8];
    let count = data.len().min(bytes.len());
    bytes[..count].copy_from_slice(&data[..count]);
    u64::from_le_bytes(bytes)
}

fn read_u32(data: &[u8]) -> u32 {
    let mut bytes = [0_u8; 4];
    let count = data.len().min(bytes.len());
    bytes[..count].copy_from_slice(&data[..count]);
    u32::from_le_bytes(bytes)
}

/// Converts a bounded input suffix into valid UTF-8 text.
fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data.get(1..data.len().min(65)).unwrap_or_default()).into_owned()
}

/// Builds a JSON value whose nesting depth is selected by the input.
fn nested_json(data: &[u8]) -> serde_json::Value {
    let depth = usize::from(data.get(3).copied().unwrap_or_default()) % (MAX_DEPTH + 1);
    let leaf = serde_json::json!({
        "text": text(data),
        "finite": f64::from(data.get(1).copied().unwrap_or_default()) / 10.0,
    });
    (0..depth).fold(leaf, |value, _| serde_json::json!([value]))
}

/// Maps fuzz bytes to a bounded runtime value container.
fn value_from_bytes(data: &[u8]) -> ValueContainer {
    let input = bounded_input(data);
    let tag = input.first().copied().unwrap_or_default() % 13;
    let number = i64::from(input.get(1).copied().unwrap_or_default());
    let float64 = f64::from_bits(read_u64(input.get(2..).unwrap_or_default()));
    let float32 = f32::from_bits(read_u32(input.get(2..).unwrap_or_default()));
    let coefficient_bytes = input.get(3..259).unwrap_or_default();
    let coefficient = BigInt::from_signed_bytes_le(coefficient_bytes);
    let scales = [0_i64, 1, -1, 150_000, -150_000, 150_001, -150_001];
    let scale = scales[usize::from(input.get(1).copied().unwrap_or_default()) % scales.len()];
    match tag {
        0 => ValueContainer::Scalar(Value::String(text(input))),
        1 => ValueContainer::Scalar(Value::Int64(number)),
        2 => ValueContainer::Scalar(Value::UInt128(read_u128(input.get(1..).unwrap_or_default()))),
        3 => ValueContainer::Scalar(Value::Int128(read_i128(input.get(1..).unwrap_or_default()))),
        4 => ValueContainer::Scalar(Value::Float64(float64)),
        5 => ValueContainer::Scalar(Value::Float32(float32)),
        6 => ValueContainer::Scalar(Value::BigInteger(coefficient.clone())),
        7 => ValueContainer::Scalar(Value::BigDecimal(BigDecimal::new(coefficient, scale))),
        8 => ValueContainer::Scalar(Value::Duration(Duration::new(
            u64::from(input.get(1).copied().unwrap_or_default()),
            read_u32(input.get(2..).unwrap_or_default()) % 1_000_000_000,
        ))),
        9 => ValueContainer::Scalar(Value::Unset(DataType::String)),
        10 => ValueContainer::Scalar(Value::Json(nested_json(input))),
        11 => ValueContainer::Collection(MultiValues::String(
            input
                .chunks(16)
                .take(MAX_ITEMS)
                .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
                .collect(),
        )),
        12 => ValueContainer::Collection(MultiValues::Json(
            input
                .chunks(16)
                .take(MAX_ITEMS)
                .map(|chunk| match chunk.first().copied().unwrap_or_default() % 5 {
                    0 => serde_json::Value::Null,
                    1 => serde_json::Value::Bool(chunk.get(1).copied().unwrap_or_default() & 1 == 1),
                    2 => serde_json::Value::from(i64::from(chunk.get(1).copied().unwrap_or_default())),
                    3 => serde_json::Value::String(text(chunk)),
                    _ => serde_json::json!({"text": text(chunk)}),
                })
                .collect(),
        )),
        _ => ValueContainer::Collection(MultiValues::Int128(
            input.chunks(16).take(MAX_ITEMS).map(read_i128).collect(),
        )),
    }
}

fn varied_limit(input: &[u8], index: usize, wide: usize) -> usize {
    match input.get(index).copied().unwrap_or_default() % 4 {
        0 => 0,
        1 => 1,
        2 => usize::from(input.get(index + 1).copied().unwrap_or_default()),
        _ => wide,
    }
}

/// Creates JSON encoding limits varied by the bounded fuzz input.
fn encode_limits(data: &[u8]) -> JsonEncodeLimits {
    let input = bounded_input(data);
    JsonEncodeLimits::builder()
        .max_output_bytes(varied_limit(input, 8, MAX_OUTPUT_BYTES).min(MAX_OUTPUT_BYTES))
        .max_depth(varied_limit(input, 10, MAX_DEPTH).min(MAX_DEPTH))
        .max_nodes(varied_limit(input, 12, MAX_ITEMS * MAX_ITEMS).min(MAX_ITEMS * MAX_ITEMS))
        .max_sequence_items(varied_limit(input, 14, MAX_ITEMS).min(MAX_ITEMS))
        .max_map_entries(varied_limit(input, 16, MAX_ITEMS).min(MAX_ITEMS))
        .max_key_bytes(varied_limit(input, 18, MAX_INPUT_BYTES).min(MAX_INPUT_BYTES))
        .max_string_bytes(varied_limit(input, 20, MAX_INPUT_BYTES).min(MAX_INPUT_BYTES))
        .max_number_bytes(varied_limit(input, 22, MAX_INPUT_BYTES).min(MAX_INPUT_BYTES))
        .max_payload_bytes(varied_limit(input, 24, MAX_OUTPUT_BYTES).min(MAX_OUTPUT_BYTES))
        .build()
}

fn check_successful_encoding(value: &ValueContainer, limits: JsonEncodeLimits) {
    let wire = match ValueWireV1::try_from(value.clone()) {
        Ok(wire) => wire,
        Err(_) => return,
    };
    if wire.to_json_vec_with_limits(limits.clone()).is_ok() {
        ValueWireEncodePreflight::new(limits)
            .check_container(value)
            .expect("successful bounded encoding must pass preflight");
    }
}

fuzz_target!(|data: &[u8]| {
    let input = bounded_input(data);
    let value = value_from_bytes(input);
    assert_eq!(value, value_from_bytes(input), "input decoding must be deterministic");
    let wide = JsonEncodeLimits::builder()
        .max_output_bytes(MAX_OUTPUT_BYTES)
        .max_depth(MAX_DEPTH + 8)
        .max_nodes(MAX_ITEMS * MAX_ITEMS)
        .max_sequence_items(MAX_ITEMS + 2)
        .max_map_entries(MAX_ITEMS + 2)
        .max_key_bytes(MAX_INPUT_BYTES)
        .max_string_bytes(MAX_INPUT_BYTES)
        .max_number_bytes(MAX_INPUT_BYTES)
        .max_payload_bytes(MAX_OUTPUT_BYTES)
        .build();
    if let Ok(wire) = ValueWireV1::try_from(value.clone())
        && let Ok(encoded) = wire.to_json_vec_with_limits(wide)
    {
        let size = encoded.len();
        for output_limit in [
            size.saturating_sub(1),
            size,
            size.saturating_add(1).min(MAX_OUTPUT_BYTES),
        ] {
            let limits = JsonEncodeLimits::builder()
                .max_output_bytes(output_limit)
                .max_depth(MAX_DEPTH + 8)
                .max_nodes(MAX_ITEMS * MAX_ITEMS)
                .max_sequence_items(MAX_ITEMS + 2)
                .max_map_entries(MAX_ITEMS + 2)
                .max_key_bytes(MAX_INPUT_BYTES)
                .max_string_bytes(MAX_INPUT_BYTES)
                .max_number_bytes(MAX_INPUT_BYTES)
                .max_payload_bytes(MAX_OUTPUT_BYTES)
                .build();
            check_successful_encoding(&value, limits);
        }
    }
    check_successful_encoding(&value, encode_limits(input));
});
