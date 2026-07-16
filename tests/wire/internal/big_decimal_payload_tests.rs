// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::str::FromStr;

use bigdecimal::BigDecimal;
use qubit_value::Value;
use serde_json::json;

#[test]
fn test_big_decimal_wire_payload_preserves_coefficient_and_scale() {
    let value = Value::BigDecimal(BigDecimal::from_str("123.4500").unwrap());
    assert_eq!(
        serde_json::to_value(&value).unwrap(),
        json!({
            "version": 1,
            "value": {
                "scalar": {
                    "bigdecimal": {"coefficient": "1234500", "scale": 4}
                }
            }
        })
    );
}

#[test]
fn test_big_decimal_wire_payload_rejects_noncanonical_coefficient() {
    for coefficient in ["01", "not-an-integer"] {
        assert!(
            serde_json::from_value::<Value>(json!({
                "version": 1,
                "value": {
                    "scalar": {
                        "bigdecimal": {
                            "coefficient": coefficient,
                            "scale": 1
                        }
                    }
                }
            }))
            .is_err()
        );
    }
}
