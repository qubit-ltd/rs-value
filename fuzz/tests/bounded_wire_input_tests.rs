// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Regression tests for raw bounded Wire V1 inputs.

use qubit_budget::BudgetError;
use qubit_budget::json::JsonDecodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::ValueContainer;
use qubit_value::ValueWireDecodeError;
use qubit_value::ValueWireV1;
use qubit_value_fuzz::bounded_wire_input::check_input;

#[test]
fn test_long_document_reaches_wide_decode_and_round_trips() {
    let input = format!(
        r#"{{"version":1,"value":{{"scalar":{{"string":"{}"}}}}}}"#,
        "x".repeat(256),
    );
    assert!(input.len() > 94);
    assert!(check_input(input.as_bytes()));
}

fn decode_limits(max_input_bytes: usize) -> JsonDecodeLimits {
    ValueWireV1::default_json_decode_limits()
        .into_builder()
        .max_input_bytes(max_input_bytes)
        .build()
}

fn assert_resource(input: &[u8], limits: JsonDecodeLimits, resource: JsonResource) {
    let result = ValueWireV1::decode_json_slice_with_limits(input, limits);
    assert!(
        matches!(
            result,
            Err(ValueWireDecodeError::Budget(
                BudgetError::LimitExceeded { resource: actual, .. }
                    | BudgetError::Insufficient { resource: actual, .. }
            )) if actual == resource
        ),
        "unexpected resource error: {result:?}, expected {resource:?}"
    );
}

fn nested_document(depth: usize) -> String {
    format!(
        r#"{{"version":1,"value":{{"scalar":{{"json":{}null{}}}}}}}"#,
        "[".repeat(depth),
        "]".repeat(depth),
    )
}

#[test]
fn test_input_byte_boundaries_preserve_full_document() {
    for length in [94, 95, 4096, 4097] {
        let mut input = br#"{"version":1,"value":{"scalar":{"int32":42}}}"#.to_vec();
        input.resize(length, b' ');
        for maximum in [94, 4096] {
            let limits = decode_limits(maximum);
            if length > maximum {
                assert_resource(&input, limits, JsonResource::InputBytes);
            } else {
                let _ = ValueWireV1::decode_json_slice_with_limits(&input, limits)
                    .expect("a valid document at the byte limit must decode");
            }
        }
        assert_eq!(check_input(&input), length <= 4096, "length: {length}");
    }
}

#[test]
fn test_depth_budget_rejects_and_accepts_nested_payload() {
    let input = nested_document(4);
    assert_resource(
        input.as_bytes(),
        decode_limits(4096).into_builder().max_depth(4).build(),
        JsonResource::Depth,
    );
    let _ = ValueWireV1::decode_json_slice_with_limits(
        input.as_bytes(),
        decode_limits(4096).into_builder().max_depth(16).build(),
    )
    .expect("a relaxed depth limit must admit the nested payload");
    assert!(check_input(input.as_bytes()));
}

#[test]
fn test_default_depth_budget_rejects_deep_payload() {
    let input = nested_document(65);
    assert_resource(input.as_bytes(), decode_limits(4096), JsonResource::Depth);
    assert!(!check_input(input.as_bytes()));
}

#[test]
fn test_nodes_budget_rejects_and_accepts_collection() {
    let value = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3])).expect("construct integer collection");
    let input = value.to_json_vec().expect("serialize integer collection");
    assert_resource(
        &input,
        decode_limits(4096).into_builder().max_nodes(1).build(),
        JsonResource::Nodes,
    );
    let _ =
        ValueWireV1::decode_json_slice_with_limits(&input, decode_limits(4096).into_builder().max_nodes(64).build())
            .expect("a relaxed node limit must admit the collection");
    assert!(check_input(&input));
}

#[test]
fn test_sequence_budget_rejects_and_accepts_collection() {
    let value = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3])).expect("construct integer collection");
    let input = value.to_json_vec().expect("serialize integer collection");
    assert_resource(
        &input,
        decode_limits(4096).into_builder().max_sequence_items(2).build(),
        JsonResource::SequenceItems,
    );
    let _ = ValueWireV1::decode_json_slice_with_limits(
        &input,
        decode_limits(4096).into_builder().max_sequence_items(3).build(),
    )
    .expect("a sequence at its item limit must decode");
    assert!(check_input(&input));
}

#[test]
fn test_map_budget_rejects_and_accepts_json_payload() {
    let input = br#"{"version":1,"value":{"scalar":{"json":{"a":1,"b":2,"c":3}}}}"#;
    assert_resource(
        input,
        decode_limits(4096).into_builder().max_map_entries(2).build(),
        JsonResource::MapEntries,
    );
    let _ = ValueWireV1::decode_json_slice_with_limits(
        input,
        decode_limits(4096).into_builder().max_map_entries(3).build(),
    )
    .expect("a map at its entry limit must decode");
    assert!(check_input(input));
}

#[test]
fn test_long_duplicate_key_reaches_invalid_json_rejection() {
    let input = include_bytes!("../corpus/bounded_value_wire_v1/duplicate-key");
    assert!(input.len() > 94 && input.len() < 4096);
    let result = ValueWireV1::decode_json_slice_with_limits(input, decode_limits(4096));
    assert!(
        matches!(result, Err(ValueWireDecodeError::InvalidJson(_))),
        "{result:?}"
    );
    assert!(!check_input(input));
}

#[test]
fn test_long_malformed_tail_reaches_syntax_rejection() {
    let input = include_bytes!("../corpus/bounded_value_wire_v1/malformed-tail");
    assert!(input.len() > 94 && input.len() < 4096);
    let result = ValueWireV1::decode_json_slice_with_limits(input, decode_limits(4096));
    assert!(matches!(result, Err(ValueWireDecodeError::Syntax(_))), "{result:?}");
    assert!(!check_input(input));
}

#[test]
fn test_valid_corpus_documents_reach_wide_decode() {
    assert!(check_input(include_bytes!(
        "../corpus/bounded_value_wire_v1/long-string"
    )));
    assert!(check_input(include_bytes!(
        "../corpus/bounded_value_wire_v1/nested-array"
    )));
}
