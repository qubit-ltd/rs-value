// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded input helpers used by the preflight soundness fuzz target.

use std::time::Duration;

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;

pub const MAX_INPUT_BYTES: usize = 4096;
pub const MAX_ITEMS: usize = 32;
pub const MAX_DEPTH: usize = 60;
pub const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

/// Limits the fuzz input slice to the target's maximum processing size.
pub fn bounded_input(data: &[u8]) -> &[u8] {
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

/// Maps bounded fuzz bytes to a runtime value container.
pub fn value_from_bytes(data: &[u8]) -> ValueContainer {
    let input = bounded_input(data);
    let tag = input.first().copied().unwrap_or_default() % 14;
    let number = i64::from(input.get(1).copied().unwrap_or_default());
    let float64 = f64::from_bits(read_u64(input.get(2..).unwrap_or_default()));
    let float32 = f32::from_bits(read_u32(input.get(2..).unwrap_or_default()));
    let coefficient_bytes = input.get(3..).unwrap_or_default();
    let coefficient_bytes = &coefficient_bytes[..coefficient_bytes.len().min(256)];
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
