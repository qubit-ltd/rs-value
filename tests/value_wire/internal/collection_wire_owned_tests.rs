// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned collection V1 wire behavior.

use qubit_value::ValueContainer;
use qubit_value::ValueWireV1;
use serde_json::to_value;

use crate::decode_value_wire_value;

#[test]
fn test_owned_collection_wire_round_trip() {
    let wire = ValueWireV1::try_from(ValueContainer::from(vec![1_i32])).expect("construct wire");
    assert_eq!(
        decode_value_wire_value(to_value(wire).expect("wire value should serialize"))
            .expect("wire value should decode")
            .into_container(),
        ValueContainer::from(vec![1_i32])
    );
}
