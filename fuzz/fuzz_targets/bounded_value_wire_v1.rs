// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Fuzzes bounded V1 JSON decoding and successful wire round trips.

use libfuzzer_sys::fuzz_target;
use qubit_budget::JsonLimits;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

/// Small budget that keeps over-limit inputs common while accepting all seeds.
const MAX_JSON_BYTES: usize = 94;

fuzz_target!(|data: &[u8]| {
    let limits = JsonLimits::new().with_max_input_bytes(MAX_JSON_BYTES);
    let result = ValueWireV1::decode_json_slice_with_limits(data, limits);
    if data.len() > MAX_JSON_BYTES {
        assert!(matches!(result, Err(ValueWireDecodeError::Budget(_))));
        return;
    }

    match result {
        Ok(value) => {
            let encoded = serde_json::to_vec(&value)
                .expect("a decoded ValueWireV1 must serialize");
            let decoded = ValueWireV1::decode_json_slice_with_limits(
                &encoded,
                JsonLimits::new().with_max_input_bytes(encoded.len()),
            )
            .expect("a serialized ValueWireV1 must decode");
            assert_eq!(decoded, value);
        }
        Err(
            ValueWireDecodeError::InvalidJson(_)
            | ValueWireDecodeError::Budget(_),
        ) => {}
        Err(error) => {
            panic!("bounded input returned an unexpected error: {error}")
        }
    }
});
