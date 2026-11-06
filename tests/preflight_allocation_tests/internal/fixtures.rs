// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Inputs constructed outside allocation measurement windows.

use qubit_value::Value;
use serde_json::Value as JsonValue;
use serde_json::json;

/// Builds a JSON array of integer controls or tenths represented as floating
/// numbers.
pub(crate) fn json_number_fixture(size: usize, floating: bool) -> Value {
    let values = (0..size)
        .map(|index| {
            if floating {
                json!(index as f64 / 10.0)
            } else {
                json!(index)
            }
        })
        .collect::<Vec<_>>();
    Value::Json(JsonValue::Array(values))
}
