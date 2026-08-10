// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload behavior.

#[test]
fn test_borrowed_payload_omits_envelope() {
    use qubit_value::Value;
    use qubit_value::ValueWirePayloadRefV1;

    assert_eq!(
        serde_json::to_value(
            ValueWirePayloadRefV1::try_from(&Value::Int32(1)).unwrap()
        )
        .unwrap(),
        serde_json::json!({"scalar": {"int32": 1}})
    );
}
