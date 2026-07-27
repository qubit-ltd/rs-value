// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed collection V1 wire behavior.

#[test]
fn test_borrowed_collection_wire_serializes() {
    use qubit_value::{
        ValueContainer,
        ValueWireRefV1,
    };

    let value = ValueContainer::from(vec![1_i32]);
    assert!(
        serde_json::to_value(ValueWireRefV1::try_from(&value).unwrap()).is_ok()
    );
}
