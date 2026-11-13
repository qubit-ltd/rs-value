// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Same-crate tests for bounded wire APIs that are inlined at downstream call
//! sites and therefore need crate-local call coverage.

use crate::MultiValues;
use crate::NamedMultiValues;
use crate::NamedValue;
use crate::Value;
use crate::ValueContainer;
use crate::ValueWirePayloadV1;
use crate::ValueWireV1;

#[test]
fn test_duration_wire_adapters_serialize_scalar_and_collection_to_sink() {
    let duration = std::time::Duration::new(7, 123);

    let mut scalar_serializer = serde_json::Serializer::new(std::io::sink());
    crate::wire::duration::serialize(&duration, &mut scalar_serializer)
        .expect("scalar duration wire adapter writes to sink");

    let mut collection_serializer = serde_json::Serializer::new(std::io::sink());
    crate::wire::duration_vec::serialize(&[duration], &mut collection_serializer)
        .expect("duration collection wire adapter writes to sink");
}

#[test]
fn test_wire_json_error_classifies_io_and_preserves_source_coordinates() {
    let error = crate::ValueWireJsonError::new(serde_json::error::Category::Io, 3, 9);

    assert_eq!(error.classify(), serde_json::error::Category::Io);
    assert!(error.is_io());
    assert!(!error.is_syntax());
    assert!(!error.is_data());
    assert!(!error.is_eof());
    assert_eq!(error.line(), 3);
    assert_eq!(error.column(), 9);
    assert_eq!(
        error.to_string(),
        "JSON deserialization failed (Io) at line 3, column 9"
    );
}

#[test]
fn test_named_multi_values_bounded_json_api_round_trip_and_mutation() {
    let mut named = NamedMultiValues::new("ports", MultiValues::Int32(vec![80, 443]));
    let encoded = named.to_json_vec().expect("named collection encodes");
    assert_eq!(
        NamedMultiValues::decode_json_slice(&encoded).expect("named collection decodes"),
        named
    );

    let explicit_bytes = named
        .to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
        .expect("named collection encodes with explicit limits");
    assert_eq!(explicit_bytes, encoded);
    assert_eq!(
        NamedMultiValues::decode_json_slice_with_limits(&explicit_bytes, ValueWireV1::default_json_decode_limits(),)
            .expect("named collection decodes with explicit limits"),
        named
    );

    let mut output = Vec::new();
    named.to_json_writer(&mut output).expect("named collection writes");
    assert_eq!(output, encoded);
    let mut explicit_output = Vec::new();
    named
        .to_json_writer_with_limits(&mut explicit_output, ValueWireV1::default_json_encode_limits())
        .expect("named collection writes with explicit limits");
    assert_eq!(explicit_output, encoded);
    named
        .to_json_writer(&mut std::io::sink())
        .expect("named collection writes to a sink");

    named.set_name("secure-ports");
    assert_eq!(named.name(), "secure-ports");
    assert_eq!(named.values(), &MultiValues::Int32(vec![80, 443]));
}

#[test]
fn test_named_value_bounded_json_api_round_trip_and_mutation() {
    let mut named = NamedValue::new("service", Value::String("database".to_owned()));
    let encoded = named.to_json_vec().expect("named scalar encodes");
    assert_eq!(
        NamedValue::decode_json_slice(&encoded).expect("named scalar decodes"),
        named
    );

    let explicit_bytes = named
        .to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
        .expect("named scalar encodes with explicit limits");
    assert_eq!(explicit_bytes, encoded);
    assert_eq!(
        NamedValue::decode_json_slice_with_limits(&explicit_bytes, ValueWireV1::default_json_decode_limits(),)
            .expect("named scalar decodes with explicit limits"),
        named
    );

    let mut output = Vec::new();
    named.to_json_writer(&mut output).expect("named scalar writes");
    assert_eq!(output, encoded);
    let mut explicit_output = Vec::new();
    named
        .to_json_writer_with_limits(&mut explicit_output, ValueWireV1::default_json_encode_limits())
        .expect("named scalar writes with explicit limits");
    assert_eq!(explicit_output, encoded);

    named.set_name("database");
    assert_eq!(named.name(), "database");
    assert_eq!(named.value(), &Value::String("database".to_owned()));
}

#[test]
fn test_value_wire_v1_bounded_json_api_round_trip_and_container_conversion() {
    let container = ValueContainer::from(Value::Int32(42));
    let wire = ValueWireV1::try_from(container.clone()).expect("container converts to V1");
    assert_eq!(ValueContainer::from(wire.clone()), container);

    let collection = ValueContainer::from(vec![7_i32, 11]);
    let collection_wire = ValueWireV1::try_from(collection.clone()).expect("collection container converts to V1");
    assert_eq!(ValueContainer::from(collection_wire.clone()), collection);
    assert_eq!(
        ValueWireV1::decode_json_slice(&collection_wire.to_json_vec().expect("collection wire encodes"),)
            .expect("collection wire decodes"),
        collection_wire
    );

    let encoded = wire.to_json_vec().expect("V1 document encodes");
    assert_eq!(
        ValueWireV1::decode_json_slice(&encoded).expect("V1 document decodes"),
        wire
    );

    let explicit_bytes = wire
        .to_json_vec_with_limits(ValueWireV1::default_json_encode_limits())
        .expect("V1 document encodes with explicit limits");
    assert_eq!(explicit_bytes, encoded);
    assert_eq!(
        ValueWireV1::decode_json_slice_with_limits(&explicit_bytes, ValueWireV1::default_json_decode_limits(),)
            .expect("V1 document decodes with explicit limits"),
        wire
    );

    let mut output = Vec::new();
    wire.to_json_writer(&mut output).expect("V1 document writes");
    assert_eq!(output, encoded);
    let mut explicit_output = Vec::new();
    wire.to_json_writer_with_limits(&mut explicit_output, ValueWireV1::default_json_encode_limits())
        .expect("V1 document writes with explicit limits");
    assert_eq!(explicit_output, encoded);
    wire.to_json_writer(&mut std::io::sink())
        .expect("V1 document writes to a sink");
}

#[test]
fn test_value_wire_payload_v1_default_json_decode_round_trip() {
    let scalar = ValueWirePayloadV1::decode_json_slice(br#"{"scalar":{"int32":42}}"#)
        .expect("standalone scalar payload decodes with default limits");
    assert_eq!(scalar.into_container(), ValueContainer::from(Value::Int32(42)));

    let collection = ValueWirePayloadV1::decode_json_slice(br#"{"collection":{"int32":[7,11]}}"#)
        .expect("standalone collection payload decodes with default limits");
    assert_eq!(collection.into_container(), ValueContainer::from(vec![7_i32, 11]),);
}
