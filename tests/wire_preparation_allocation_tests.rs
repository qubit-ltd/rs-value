// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::json::JsonEncodeLimits;
use qubit_value::MultiValues;
use qubit_value::ValueWirePayloadRefV1;

#[test]
fn test_bounded_collection_entry_accepts_empty_collection() {
    let limits = JsonEncodeLimits::builder().max_output_bytes(1024).build();
    let values = MultiValues::Int32(Vec::new());
    let wire = ValueWirePayloadRefV1::from_values(&values).unwrap();
    assert!(!wire.to_json_vec_with_limits(limits).unwrap().is_empty());
}
