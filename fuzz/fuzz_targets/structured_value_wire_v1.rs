// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
#![no_main]

//! Fuzzes the structured V1 encode/decode path from generated runtime values.

use libfuzzer_sys::fuzz_target;
use qubit_budget::ResourceLimit;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::ValueWireV1;
use qubit_value_fuzz::named_wire_input::check_structured;
use qubit_value_fuzz::structured_wire_input::check_non_finite;
use qubit_value_fuzz::structured_wire_input::container_from_bytes;

/// Bounds generated structured values during both sides of the wire round trip.
#[must_use]
#[inline]
fn fuzz_decode_limits() -> JsonDecodeLimits {
    let base = ValueWireV1::default_json_decode_limits();
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 4_096))
        .value_limits(base.into_value_limits())
        .build()
}

/// Bounds generated JSON output while preserving the default value profile.
#[must_use]
#[inline]
fn fuzz_encode_limits() -> JsonEncodeLimits {
    let base = ValueWireV1::default_json_encode_limits();
    JsonEncodeLimits::builder()
        .output_bytes_limit(ResourceLimit::new(JsonResource::OutputBytes, 4_096))
        .value_limits(base.into_value_limits())
        .build()
}

fuzz_target!(|data: &[u8]| {
    let container = container_from_bytes(data);
    let wire = ValueWireV1::try_from(container.clone());

    match wire {
        Ok(wire) => {
            let encoded = wire
                .to_json_vec_with_limits(fuzz_encode_limits())
                .expect("validated V1 values serialize within fuzz limits");
            let decoded = ValueWireV1::decode_json_slice_with_limits(&encoded, fuzz_decode_limits())
                .expect("serialized V1 values deserialize within fuzz limits");
            assert_eq!(decoded, wire);
            assert_eq!(decoded.into_container(), container);
        }
        Err(error) => {
            panic!("generated structured value must be V1 encodable: {error}")
        }
    }
    check_non_finite(data);
    check_structured(data);
});
