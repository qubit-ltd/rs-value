// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 envelope behavior.

use qubit_value::Value;
use qubit_value::ValueWireRefV1;
use serde_json::to_value;

#[test]
fn test_borrowed_wire_envelope_has_version_one() {
    assert_eq!(
        to_value(ValueWireRefV1::try_from(&Value::Int32(1)).expect("construct borrowed wire envelope"),)
            .expect("serialize borrowed wire envelope")["version"],
        1
    );
}
