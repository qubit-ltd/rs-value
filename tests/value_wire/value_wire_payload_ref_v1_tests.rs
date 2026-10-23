// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed V1 payload behavior.

use qubit_value::Value;
use qubit_value::ValueWirePayloadRefV1;
use serde_json::json;
use serde_json::to_value;

#[test]
fn test_borrowed_payload_omits_envelope() {
    assert_eq!(
        to_value(
            ValueWirePayloadRefV1::try_from(&Value::Int32(1))
                .expect("the integer value should have a borrowed V1 payload"),
        )
        .expect("the borrowed payload should serialize to a JSON value"),
        json!({"scalar": {"int32": 1}})
    );
}
