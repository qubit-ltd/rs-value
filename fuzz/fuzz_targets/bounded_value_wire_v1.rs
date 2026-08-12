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
use qubit_budget::ResourceLimit;
use qubit_json::JsonDecodeLimits;
use qubit_json::JsonEncodeLimits;
use qubit_json::JsonResource;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;

/// Small budget that keeps over-limit inputs common while accepting all seeds.
const MAX_JSON_BYTES: usize = 94;

/// Keeps all JSON resource dimensions active while fuzzing bounded input.
fn fuzz_decode_limits(max_input_bytes: usize) -> JsonDecodeLimits {
    ValueWireV1::default_json_decode_limits().with_input_bytes_limit(
        ResourceLimit::new(JsonResource::InputBytes, max_input_bytes),
    )
}

/// Keeps output accounting active while encoding successful values.
fn fuzz_encode_limits() -> JsonEncodeLimits {
    ValueWireV1::default_json_encode_limits().with_output_bytes_limit(
        ResourceLimit::new(JsonResource::OutputBytes, 4_096),
    )
}

fuzz_target!(|data: &[u8]| {
    let limits = fuzz_decode_limits(MAX_JSON_BYTES);
    let result = ValueWireV1::decode_json_slice_with_limits(data, limits);
    if data.len() > MAX_JSON_BYTES {
        assert!(matches!(result, Err(ValueWireDecodeError::Budget(_))));
        return;
    }

    match result {
        Ok(value) => {
            let encoded =
                value.to_json_vec_with_limits(fuzz_encode_limits()).expect(
                    "a decoded ValueWireV1 must serialize within fuzz limits",
                );
            let decoded = ValueWireV1::decode_json_slice_with_limits(
                &encoded,
                fuzz_decode_limits(encoded.len()),
            )
            .expect("a serialized ValueWireV1 must decode");
            assert_eq!(decoded, value);
        }
        Err(
            ValueWireDecodeError::InvalidJson(_)
            | ValueWireDecodeError::Budget(_)
            | ValueWireDecodeError::UnsupportedVersion { .. },
        ) => {}
        Err(error) => {
            panic!("bounded input returned an unexpected error: {error}")
        }
    }
});
