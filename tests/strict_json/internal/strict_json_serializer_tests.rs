// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Public behavior tests for the strict JSON serializer.

use qubit_value::Value;
use serde_json::json;

use super::oversized_length_hint_tests::OversizedLengthHint;

/// Verifies pathological Serde hints cannot force unbounded preallocation.
#[test]
fn test_from_serializable_bounds_compound_length_hints() {
    for (source, expected) in [
        (OversizedLengthHint::Sequence, json!([])),
        (OversizedLengthHint::TupleVariant, json!({"Tuple": []})),
        (OversizedLengthHint::Map, json!({})),
        (OversizedLengthHint::StructVariant, json!({"Struct": {}})),
    ] {
        assert_eq!(
            Value::from_serializable(&source)
                .expect("oversized length hints must not force allocation"),
            Value::Json(expected),
        );
    }
}
