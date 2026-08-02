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
use qubit_value::{MultiValues, Value, ValueContainer, ValueWireV1};

/// Builds a small, valid runtime shape from the fuzz input bytes. Keeping the
/// shape construction typed means the fuzz target exercises the wire encoder
/// and decoder independently from arbitrary JSON syntax handling.
fn container_from_bytes(data: &[u8]) -> ValueContainer {
    let tag = data.first().copied().unwrap_or_default() % 8;
    let number = i32::from_le_bytes([
        data.get(1).copied().unwrap_or_default(),
        data.get(2).copied().unwrap_or_default(),
        data.get(3).copied().unwrap_or_default(),
        data.get(4).copied().unwrap_or_default(),
    ]);
    let text_bytes = data
        .get(1..data.len().min(33))
        .unwrap_or_default();
    let text = String::from_utf8_lossy(text_bytes).into_owned();

    match tag {
        0 => ValueContainer::Scalar(Value::Bool(data.get(1).copied().unwrap_or_default() & 1 == 1)),
        1 => ValueContainer::Scalar(Value::Int32(number)),
        2 => ValueContainer::Scalar(Value::String(text)),
        3 => ValueContainer::Scalar(Value::new_unset(Value::Int32(0).data_type())),
        4 => ValueContainer::Collection(MultiValues::Int32(vec![number, number.wrapping_add(1)])),
        5 => ValueContainer::Collection(MultiValues::String(vec![text])),
        6 => ValueContainer::Scalar(Value::Float64(f64::from(number))),
        _ => ValueContainer::Collection(MultiValues::Bool(vec![true, false])),
    }
}

fuzz_target!(|data: &[u8]| {
    let container = container_from_bytes(data);
    let wire = ValueWireV1::try_from(container.clone());

    match wire {
        Ok(wire) => {
            let encoded = serde_json::to_vec(&wire).expect("validated V1 values serialize");
            let decoded: ValueWireV1 = serde_json::from_slice(&encoded)
                .expect("serialized V1 values deserialize");
            assert_eq!(decoded, wire);
            assert_eq!(decoded.into_container(), container);
        }
        Err(error) => panic!("generated structured value must be V1 encodable: {error}"),
    }
});
