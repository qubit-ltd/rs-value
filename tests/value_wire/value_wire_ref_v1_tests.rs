// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for borrowed V1 envelope serialization.

use qubit_value::ValueContainer;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;

#[test]
fn test_value_wire_ref_v1_preserves_the_owned_wire_contract() {
    let value = ValueContainer::from(vec!["api".to_owned(), "worker".to_owned()]);
    let wire = ValueWireRefV1::try_from(&value).expect("construct borrowed V1 wire");
    let encoded = serde_json::to_value(wire).expect("serialize borrowed V1 wire");
    let decoded: ValueWireV1 = serde_json::from_value(encoded).expect("decode V1 wire");

    assert_eq!(decoded.into_container(), value);
}
