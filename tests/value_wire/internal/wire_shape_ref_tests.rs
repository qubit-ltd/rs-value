// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 shape behavior.

#[test]
fn test_borrowed_wire_shape_preserves_collection() {
    use qubit_value::MultiValues;
    use qubit_value::ValueWireRefV1;

    assert_eq!(
        serde_json::to_value(
            ValueWireRefV1::try_from(&MultiValues::Int32(vec![1])).unwrap()
        )
        .unwrap()["value"]["collection"]["int32"],
        serde_json::json!([1])
    );
}
