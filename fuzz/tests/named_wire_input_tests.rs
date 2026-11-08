// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Deterministic coverage for named Wire V1 wrappers.

use qubit_budget::BudgetError;
use qubit_budget::json::{JsonDecodeLimits, JsonResource};
use qubit_value::{
    MultiValues, NamedMultiValues, NamedValue, Value, ValueWireDecodeError, ValueWireV1,
};
use qubit_value_fuzz::named_wire_input::{check_raw, check_structured};

fn wide_limits(input: usize) -> JsonDecodeLimits {
    let base = ValueWireV1::default_json_decode_limits();
    let values = base
        .value_limits()
        .into_builder()
        .max_depth(4096)
        .max_nodes(4096)
        .max_sequence_items(4096)
        .max_map_entries(4096)
        .max_string_bytes(65_536)
        .build();
    JsonDecodeLimits::builder()
        .max_input_bytes(input)
        .value_limits(values)
        .build()
}

fn assert_resource<T: std::fmt::Debug>(
    result: &Result<T, ValueWireDecodeError>,
    resource: JsonResource,
) {
    assert!(
        matches!(result, Err(ValueWireDecodeError::Budget(
        BudgetError::LimitExceeded { resource: actual, .. } | BudgetError::Insufficient { resource: actual, .. }
    )) if *actual == resource),
        "unexpected result: {result:?}, expected {resource:?}"
    );
}

#[test]
fn structured_covers_empty_unicode_escaped_scalar_collection_and_typed_unset() {
    // Selector bytes choose empty, Unicode, quote/backslash and control names.
    for selector in [1_u8, 2, 3, 7] {
        for scalar in [false, true] {
            let data = [selector, 0, 0, 0, u8::from(scalar)];
            check_structured(&data);
        }
    }
    let unset = NamedValue::new("", Value::Unset(qubit_datatype::DataType::Int32));
    let bytes = unset.to_json_vec().expect("typed unset encodes");
    assert_eq!(
        NamedValue::decode_json_slice(&bytes).expect("typed unset decodes"),
        unset
    );
    let empty = NamedMultiValues::new("λ", MultiValues::Int32(Vec::new()));
    let bytes = empty.to_json_vec().expect("empty collection encodes");
    assert_eq!(
        NamedMultiValues::decode_json_slice(&bytes).expect("empty collection decodes"),
        empty
    );
    let unset_collection = NamedMultiValues::new(
        "unset",
        MultiValues::Unset(qubit_datatype::DataType::String),
    );
    let bytes = unset_collection
        .to_json_vec()
        .expect("typed unset collection encodes");
    assert_eq!(
        NamedMultiValues::decode_json_slice(&bytes).expect("typed unset collection decodes"),
        unset_collection
    );
}

#[test]
fn raw_checker_accepts_valid_ordered_wrappers_and_rejects_duplicate_fields() {
    let scalar = br#"{"name":"x","value":{"version":1,"value":{"scalar":{"int32":1}}}}"#;
    let reordered = br#"{"value":{"version":1,"value":{"scalar":{"int32":1}}},"name":"x"}"#;
    assert_eq!(
        NamedValue::decode_json_slice(scalar).unwrap(),
        NamedValue::decode_json_slice(reordered).unwrap()
    );
    check_raw(scalar);
    check_raw(reordered);
    let duplicate =
        br#"{"name":"x","name":"y","value":{"version":1,"value":{"scalar":{"int32":1}}}}"#;
    assert!(matches!(
        NamedValue::decode_json_slice(duplicate),
        Err(ValueWireDecodeError::InvalidJson(_))
    ));
    check_raw(duplicate);
    let duplicate_value = br#"{"name":"x","value":{"version":1,"value":{"scalar":{"int32":1}}},"value":{"version":1,"value":{"scalar":{"int32":1}}}}"#;
    assert!(NamedValue::decode_json_slice(duplicate_value).is_err());
    check_raw(duplicate_value);
    // A scalar envelope is invalid in the NamedMultiValues value position.
    let wrong_shape = br#"{"name":"x","value":{"version":1,"value":{"scalar":{"int32":1}}}}"#;
    assert!(NamedMultiValues::decode_json_slice(wrong_shape).is_err());
    check_raw(wrong_shape);
    let collection_duplicate =
        br#"{"name":"x","name":"y","value":{"version":1,"value":{"collection":{"int32":[]}}}}"#;
    assert!(NamedMultiValues::decode_json_slice(collection_duplicate).is_err());
    check_raw(collection_duplicate);
}

#[test]
fn each_schema_mutation_is_independently_rejected() {
    let valid =
        serde_json::json!({"name":"x","value":{"version":1,"value":{"scalar":{"int32":1}}}});
    let variants = [
        serde_json::json!({"value":valid["value"]}),
        serde_json::json!({"name":"x"}),
        serde_json::json!({"name":"x","value":valid["value"],"extra":0}),
        serde_json::json!({"name":7,"value":valid["value"]}),
    ];
    for variant in variants {
        let bytes = serde_json::to_vec(&variant).unwrap();
        assert!(
            NamedValue::decode_json_slice_with_limits(&bytes, wide_limits(4096)).is_err(),
            "{variant}"
        );
        check_raw(&bytes);
    }
    let trailing = br#"{"name":"x","value":{"version":1,"value":{"scalar":{"int32":1}}}} {}"#;
    assert!(NamedValue::decode_json_slice_with_limits(trailing, wide_limits(4096)).is_err());
    check_raw(trailing);
}

#[test]
fn input_and_output_byte_limits_are_checked_at_n_minus_n_and_n_plus_one() {
    let named = NamedValue::new("λ\"\\\n", Value::Int32(1));
    let encoded = named.to_json_vec().unwrap();
    let n = encoded.len();
    for (maximum, succeeds) in [(n - 1, false), (n, true), (n + 1, true)] {
        let result = NamedValue::decode_json_slice_with_limits(&encoded, wide_limits(maximum));
        if succeeds {
            assert_eq!(result.unwrap(), named);
        } else {
            assert_resource(&result, JsonResource::InputBytes);
        }
        let encode_limits = ValueWireV1::default_json_encode_limits()
            .into_builder()
            .max_output_bytes(maximum)
            .build();
        let output = named.to_json_vec_with_limits(encode_limits);
        assert_eq!(output.is_ok(), succeeds, "output max {maximum}");
    }
}

#[test]
fn maximum_generated_name_fits_with_payload_under_wide_limits() {
    let wrapper = NamedValue::new("é".repeat(48), Value::Int32(1)); // exactly 96 UTF-8 bytes
    let bytes = wrapper.to_json_vec().unwrap();
    assert_eq!(
        NamedValue::decode_json_slice_with_limits(&bytes, wide_limits(4096)).unwrap(),
        wrapper
    );
}
