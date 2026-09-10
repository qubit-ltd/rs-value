// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        http://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
// =============================================================================
#![no_main]

//! Checks that a successful bounded Wire V1 encoding is accepted by preflight.

use libfuzzer_sys::fuzz_target;
use qubit_budget::json::JsonEncodeLimits;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodePreflight;
use qubit_value::ValueWireV1;

const MAX_INPUT_BYTES: usize = 4096;
const MAX_ITEMS: usize = 32;
const MAX_DEPTH: usize = 8;
const MAX_OUTPUT_BYTES: usize = 4 * 1024 * 1024;

fn bounded_input(data: &[u8]) -> &[u8] {
    &data[..data.len().min(MAX_INPUT_BYTES)]
}

fn text(data: &[u8]) -> String {
    String::from_utf8_lossy(data.get(1..data.len().min(65)).unwrap_or_default()).into_owned()
}

fn nested_json(data: &[u8]) -> serde_json::Value {
    let depth = usize::from(data.first().copied().unwrap_or_default() % (MAX_DEPTH as u8 + 1));
    let leaf = serde_json::json!({
        "text": text(data),
        "finite": f64::from(data.get(1).copied().unwrap_or_default()) / 10.0,
    });
    (0..depth).fold(leaf, |value, _| serde_json::json!([value]))
}

fn value_from_bytes(data: &[u8]) -> ValueContainer {
    let input = bounded_input(data);
    let tag = input.first().copied().unwrap_or_default() % 8;
    let number = i64::from(input.get(1).copied().unwrap_or_default());
    let finite = f64::from(input.get(2).copied().unwrap_or_default()) / 10.0;
    match tag {
        0 => ValueContainer::Scalar(Value::String(text(input))),
        1 => ValueContainer::Scalar(Value::Int64(number)),
        2 => ValueContainer::Scalar(Value::UInt128(u128::from(number.unsigned_abs()))),
        3 => ValueContainer::Scalar(Value::Float64(finite)),
        4 => ValueContainer::Scalar(Value::Float64(1e100)),
        5 => ValueContainer::Scalar(Value::Json(nested_json(input))),
        6 => ValueContainer::Collection(MultiValues::String(
            input
                .chunks(16)
                .take(MAX_ITEMS)
                .map(|chunk| String::from_utf8_lossy(chunk).into_owned())
                .collect(),
        )),
        _ => ValueContainer::Collection(MultiValues::Int64(
            input.iter().take(MAX_ITEMS).map(|byte| i64::from(*byte)).collect(),
        )),
    }
}

fn limits(data: &[u8]) -> JsonEncodeLimits {
    let input = bounded_input(data);
    let value_limit = 512 + usize::from(input.get(3).copied().unwrap_or_default()) * 32;
    JsonEncodeLimits::builder()
        .max_output_bytes(MAX_OUTPUT_BYTES)
        .max_depth(MAX_DEPTH + 2)
        .max_nodes(4096)
        .max_sequence_items(MAX_ITEMS + 2)
        .max_map_entries(MAX_ITEMS + 2)
        .max_key_bytes(4096)
        .max_string_bytes(value_limit)
        .max_number_bytes(value_limit)
        .max_payload_bytes(MAX_OUTPUT_BYTES)
        .build()
}

fuzz_target!(|data: &[u8]| {
    let value = value_from_bytes(data);
    let limits = limits(data);
    let wire = match ValueWireV1::try_from(value.clone()) {
        Ok(wire) => wire,
        Err(_) => return,
    };

    if wire.to_json_vec_with_limits(limits.clone()).is_ok() {
        ValueWireEncodePreflight::new(limits)
            .check_container(&value)
            .expect("successful bounded encoding must pass preflight");
    }
});
