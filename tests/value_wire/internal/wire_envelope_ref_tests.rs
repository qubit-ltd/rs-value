// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 envelope behavior.

#[test]
fn test_borrowed_wire_envelope_has_version_one() {
    use qubit_value::Value;
    use qubit_value::ValueWireRefV1;

    assert_eq!(
        serde_json::to_value(
            ValueWireRefV1::try_from(&Value::Int32(1)).unwrap()
        )
        .unwrap()["version"],
        1
    );
}
