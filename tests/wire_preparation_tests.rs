// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::json::JsonEncodeLimits;
use qubit_value::Value;
use qubit_value::ValueWireRefV1;

#[test]
fn test_bounded_wire_entry_still_serializes_valid_values() {
    let limits = JsonEncodeLimits::builder().max_output_bytes(1024).build();
    let value = Value::from(42_i32);
    let wire = ValueWireRefV1::from_value(&value).unwrap();
    let encoded = wire.to_json_vec_with_limits(limits).unwrap();
    assert!(!encoded.is_empty());
}
