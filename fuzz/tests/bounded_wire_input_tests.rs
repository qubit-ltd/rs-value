// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::BudgetError;
use qubit_budget::ResourceLimit;
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

#[test]
fn test_input_byte_boundaries() {
    let base = r#"{"version":1,"value":{"scalar":{"int32":1}}}"#;
    for (length, expected) in [(94, true), (95, true), (4096, true), (4097, false)] {
        let mut input = base.to_owned();
        input.push_str(&" ".repeat(length - input.len()));
        assert_eq!(check_input(input.as_bytes()), expected, "length {length}");
        let result = ValueWireV1::decode_json_slice_with_limits(input.as_bytes(), bounded_limits(4096));
        if length > 4096 {
            assert_input_bytes_error(result.unwrap_err());
        } else {
            assert!(result.is_ok());
        }
        if length > 94 {
            assert_input_bytes_error(
                ValueWireV1::decode_json_slice_with_limits(input.as_bytes(), bounded_limits(94)).unwrap_err(),
            );
        } else {
            assert!(ValueWireV1::decode_json_slice_with_limits(input.as_bytes(), bounded_limits(94)).is_ok());
        }
    }
}

fn bounded_limits(max_input_bytes: usize) -> JsonDecodeLimits {
    let defaults = ValueWireV1::default_json_decode_limits();
    JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, max_input_bytes))
        .value_limits(defaults.into_value_limits())
        .build()
}

fn assert_input_bytes_error(error: ValueWireDecodeError) {
    assert!(matches!(
        error,
        ValueWireDecodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::InputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::InputBytes,
                ..
            }
        )
    ));
}

fn decode_with_values(
    input: &[u8],
    values: qubit_budget::json::JsonValueLimits,
) -> Result<qubit_value::ValueWireV1, ValueWireDecodeError> {
    let limits = JsonDecodeLimits::builder()
        .input_bytes_limit(ResourceLimit::new(JsonResource::InputBytes, 4096))
        .value_limits(values)
        .build();
    ValueWireV1::decode_json_slice_with_limits(input, limits)
}

fn assert_resource_error(error: ValueWireDecodeError, expected: JsonResource) {
    assert!(matches!(
        error,
        ValueWireDecodeError::Budget(
            BudgetError::LimitExceeded { resource, .. }
                | BudgetError::Insufficient { resource, .. }
        ) if resource == expected
    ));
}

#[test]
fn test_structural_limits_reject_and_accept_at_boundaries() {
    let defaults = ValueWireV1::default_json_decode_limits().into_value_limits();
    let deep = format!(
        r#"{{"version":1,"value":{{"scalar":{{"json":{}null{}}}}}}}"#,
        "[".repeat(65),
        "]".repeat(65),
    );
    assert_resource_error(
        decode_with_values(deep.as_bytes(), defaults.clone()).unwrap_err(),
        JsonResource::Depth,
    );

    let shallow = r#"{"version":1,"value":{"scalar":{"json":[[null]]}}}"#;
    let low_depth = defaults.clone().into_builder().max_depth(1).build();
    assert_resource_error(
        decode_with_values(shallow.as_bytes(), low_depth).unwrap_err(),
        JsonResource::Depth,
    );
    let raised_depth = defaults.clone().into_builder().max_depth(8).build();
    assert!(decode_with_values(shallow.as_bytes(), raised_depth).is_ok());

    let collection = ValueWireV1::try_from(ValueContainer::from(vec![1_i32, 2, 3]))
        .expect("construct integer sequence")
        .to_json_vec()
        .expect("encode integer sequence");
    let low_nodes = defaults.clone().into_builder().max_nodes(1).build();
    assert_resource_error(
        decode_with_values(&collection, low_nodes).unwrap_err(),
        JsonResource::Nodes,
    );
    let enough_nodes = defaults.clone().into_builder().max_nodes(64).build();
    assert!(decode_with_values(&collection, enough_nodes).is_ok());

    let low_items = defaults.clone().into_builder().max_sequence_items(2).build();
    assert_resource_error(
        decode_with_values(&collection, low_items).unwrap_err(),
        JsonResource::SequenceItems,
    );
    let enough_items = defaults.clone().into_builder().max_sequence_items(3).build();
    assert!(decode_with_values(&collection, enough_items).is_ok());

    let map = br#"{"version":1,"value":{"scalar":{"json":{"a":1,"b":2,"c":3}}}}"#;
    let low_map = defaults.clone().into_builder().max_map_entries(2).build();
    assert_resource_error(decode_with_values(map, low_map).unwrap_err(), JsonResource::MapEntries);
    let enough_map = defaults.into_builder().max_map_entries(3).build();
    assert!(decode_with_values(map, enough_map).is_ok());
}

#[test]
fn test_long_duplicate_key_and_malformed_tail_are_not_input_limits() {
    let duplicate = include_bytes!("../corpus/bounded_value_wire_v1/duplicate-key");
    let error =
        ValueWireV1::decode_json_slice_with_limits(duplicate, ValueWireV1::default_json_decode_limits()).unwrap_err();
    assert!(matches!(error, ValueWireDecodeError::InvalidJson(_)));

    let malformed = include_bytes!("../corpus/bounded_value_wire_v1/malformed-tail");
    let error =
        ValueWireV1::decode_json_slice_with_limits(malformed, ValueWireV1::default_json_decode_limits()).unwrap_err();
    assert!(matches!(error, ValueWireDecodeError::Syntax(_)));
}
