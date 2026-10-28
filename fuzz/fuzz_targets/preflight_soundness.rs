// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Checks that successful bounded Wire V1 encodings pass preflight.

use libfuzzer_sys::fuzz_target;
use qubit_budget::json::JsonEncodeLimits;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;
use qubit_value_fuzz::preflight_input::MAX_DEPTH;
use qubit_value_fuzz::preflight_input::MAX_INPUT_BYTES;
use qubit_value_fuzz::preflight_input::MAX_ITEMS;
use qubit_value_fuzz::preflight_input::MAX_OUTPUT_BYTES;
use qubit_value_fuzz::preflight_input::bounded_input;
use qubit_value_fuzz::preflight_input::value_from_bytes;

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
