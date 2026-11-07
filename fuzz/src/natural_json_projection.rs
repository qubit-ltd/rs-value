// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Bounded Natural JSON projection properties shared by fuzz and regression
//! tests.

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataType;
use qubit_datatype::StructuredConversionLimits;
use qubit_value::ValueContainer;
use qubit_value::ValueError;
use serde_json::Value as JsonValue;

use crate::preflight_input::bounded_input;
use crate::preflight_input::value_from_bytes;
use crate::structured_wire_input::container_from_bytes;

const LOOSE_LIMIT: u64 = 512;

/// Checks deterministic, bounded Natural JSON projection properties for input
/// bytes.
pub fn check_input(data: &[u8]) {
    let input = bounded_input(data);
    let value_container = value_from_bytes(input);
    let structured_container = container_from_bytes(input);

    for container in [&value_container, &structured_container] {
        check_container(container, input);
    }
}

fn check_container(container: &ValueContainer, input: &[u8]) {
    let before = container.clone();
    let policies = [
        ConversionPolicy::default(),
        ConversionPolicy::strict(),
        ConversionPolicy::lossy(),
        ConversionPolicy::env_friendly(),
    ];
    let strict = limits(input, false);
    let loose = limits(input, true);

    for policy in &policies {
        let first = container.to_json_value_with(policy, &strict);
        let second = container.to_json_value_with(policy, &strict);
        assert_eq!(
            result_signature(&first),
            result_signature(&second),
            "projection result must be deterministic"
        );
        assert_eq!(&before, container, "projection must not mutate the source");

        let strict_error = first.as_ref().err();
        let loose_after_failure = container.to_json_value_with(policy, &loose);
        let loose_independent = container.to_json_value_with(policy, &loose);
        assert_eq!(
            result_signature(&loose_after_failure),
            result_signature(&loose_independent),
            "a failed call must not contaminate a later call"
        );
        assert_projected_shape(container, &first);
        assert_projected_shape(container, &loose_after_failure);
        if strict_error.is_none() {
            assert_eq!(
                result_signature(&first),
                result_signature(&loose_after_failure),
                "simultaneously relaxed limits must preserve successful output"
            );
        }
    }
}

fn assert_projected_shape(container: &ValueContainer, result: &Result<JsonValue, ValueError>) {
    let Ok(projected) = result else {
        return;
    };
    if container.is_unset() {
        assert_eq!(
            projected,
            &JsonValue::Null,
            "unset scalar and collection project to null"
        );
    } else {
        match container {
            ValueContainer::Scalar(source) if source.data_type() == DataType::Json => {
                assert_eq!(
                    projected,
                    source.get_json_ref().expect("JSON data type has JSON payload"),
                    "scalar JSON projection must preserve its JSON value"
                );
            }
            ValueContainer::Collection(source) if source.data_type() == DataType::Json => {
                assert_eq!(
                    projected,
                    &JsonValue::Array(source.get_jsons().expect("JSON data type has JSON payload").to_vec()),
                    "JSON collection projection must preserve its source items"
                );
            }
            ValueContainer::Collection(_) => {
                let JsonValue::Array(items) = projected else {
                    panic!("concrete collection must project to a JSON array");
                };
                assert_eq!(
                    items.len(),
                    container.len(),
                    "projected collection length must match source"
                );
            }
            ValueContainer::Scalar(_) => {}
        }
    }
}

fn limits(input: &[u8], loose: bool) -> ConversionLimits {
    let selector = u64::from(input.get(1).copied().unwrap_or_default() % 4);
    let cap = if loose { LOOSE_LIMIT } else { selector };
    ConversionLimits::builder()
        .operation_limits(
            ConversionOperationLimits::builder()
                .max_items(cap)
                .max_output_bytes(if loose { LOOSE_LIMIT * 8 } else { selector * 8 })
                .max_structured_nodes(if loose { LOOSE_LIMIT } else { selector + 1 })
                .max_structured_payload_bytes(if loose { LOOSE_LIMIT * 8 } else { selector * 8 })
                .build(),
        )
        .structured_limits(
            StructuredConversionLimits::builder()
                .max_text_bytes(if loose { LOOSE_LIMIT } else { selector * 8 })
                .max_depth(if loose { LOOSE_LIMIT } else { selector + 1 })
                .max_sequence_items(if loose { LOOSE_LIMIT } else { selector })
                .max_map_entries(if loose { LOOSE_LIMIT } else { selector })
                .build(),
        )
        .build()
}

#[derive(Debug, PartialEq, Eq)]
enum ErrorSignature {
    ProjectionLimit(Option<usize>),
    Missing,
    TypeMismatch,
    Conversion,
    ListConversion(usize),
}

fn result_signature(result: &Result<JsonValue, ValueError>) -> Result<&JsonValue, ErrorSignature> {
    result.as_ref().map_err(|error| match error {
        ValueError::JsonProjectionLimit { source_index, .. } => ErrorSignature::ProjectionLimit(*source_index),
        ValueError::Missing(_) => ErrorSignature::Missing,
        ValueError::TypeMismatch { .. } => ErrorSignature::TypeMismatch,
        ValueError::Conversion(_) => ErrorSignature::Conversion,
        ValueError::ListConversion(error) => ErrorSignature::ListConversion(error.source_index()),
        _ => unreachable!("ValueError is non-exhaustive"),
    })
}
