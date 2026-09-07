// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for borrowed V1 envelope serialization.

use std::io;
use std::io::Write;

use qubit_budget::BudgetError;
use qubit_budget::json::JsonEncodeLimits;
use qubit_budget::json::JsonResource;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWireRefV1;

/// Writer that rejects every wire write with a stable diagnostic.
struct RejectingWriter;

impl Write for RejectingWriter {
    fn write(&mut self, _buffer: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "wire writer rejected",
        ))
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_value_wire_ref_v1_preserves_the_owned_wire_contract() {
    let value = ValueContainer::from(vec!["api".to_owned(), "worker".to_owned()]);
    let wire = ValueWireRefV1::try_from(&value).expect("construct borrowed V1 wire");
    let encoded = serde_json::to_value(wire).expect("serialize borrowed V1 wire");
    let decoded = crate::decode_value_wire_value(encoded).expect("decode V1 wire");

    assert_eq!(decoded.into_container(), value);
}

#[test]
fn test_value_wire_ref_v1_scalar_constructor_preserves_shape() {
    let scalar = Value::Int32(7);
    let scalar_wire = ValueWireRefV1::from_value(&scalar).expect("borrow a scalar wire");
    assert_eq!(
        scalar_wire.to_json_vec().expect("encode a scalar wire"),
        br#"{"version":1,"value":{"scalar":{"int32":7}}}"#,
    );
}

#[test]
fn test_value_wire_ref_v1_collection_constructor_preserves_shape() {
    let values = MultiValues::Int32(vec![7, 8]);
    let collection_wire = ValueWireRefV1::from_values(&values).expect("borrow a collection wire");
    assert_eq!(
        collection_wire
            .to_json_vec_with_limits(JsonEncodeLimits::builder().max_output_bytes(1_024).build())
            .expect("encode a collection wire"),
        br#"{"version":1,"value":{"collection":{"int32":[7,8]}}}"#,
    );
}

#[test]
fn test_value_wire_ref_v1_default_writer_matches_golden_bytes() {
    let container = ValueContainer::from("ready");
    let wire = ValueWireRefV1::from_container(&container).expect("borrow a string wire");
    let mut output = Vec::new();
    wire.to_json_writer(&mut output)
        .expect("write the borrowed wire");
    assert_eq!(
        output,
        br#"{"version":1,"value":{"scalar":{"string":"ready"}}}"#,
    );
}

#[test]
fn test_value_wire_ref_v1_writer_error_is_precise() {
    let container = ValueContainer::from("ready");
    let wire = ValueWireRefV1::from_container(&container).expect("borrow a string wire");
    let error = wire
        .to_json_writer_with_limits(
            RejectingWriter,
            JsonEncodeLimits::builder().max_output_bytes(1_024).build(),
        )
        .expect_err("the rejecting writer must fail");
    assert!(matches!(
        error,
        ValueWireEncodeError::Io(source)
            if source.kind() == io::ErrorKind::BrokenPipe
                && source.to_string() == "wire writer rejected"
    ));
}

#[test]
fn test_value_wire_ref_v1_output_budget_error_is_precise() {
    let container = ValueContainer::from("ready");
    let wire = ValueWireRefV1::from_container(&container).expect("borrow a string wire");
    let error = wire
        .to_json_vec_with_limits(JsonEncodeLimits::builder().max_output_bytes(1).build())
        .expect_err("a one-byte budget must reject the borrowed wire");
    assert!(matches!(
        error,
        ValueWireEncodeError::Budget(
            BudgetError::LimitExceeded {
                resource: JsonResource::OutputBytes,
                ..
            } | BudgetError::Insufficient {
                resource: JsonResource::OutputBytes,
                ..
            }
        )
    ));
}
