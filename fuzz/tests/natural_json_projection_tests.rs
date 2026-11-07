// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataType;
use qubit_datatype::StructuredConversionLimits;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value_fuzz::natural_json_projection::check_input;
use qubit_value_fuzz::preflight_input::MAX_INPUT_BYTES;
use qubit_value_fuzz::preflight_input::bounded_input;
use qubit_value_fuzz::preflight_input::value_from_bytes;
use qubit_value_fuzz::structured_wire_input::container_from_bytes;
use serde_json::Value as JsonValue;

fn structured_seed(tag: u8, collection_len: u8) -> Vec<u8> {
    let mut input = vec![0_u8; 16];
    input[0] = tag;
    input[5] = collection_len;
    input
}

#[test]
fn test_natural_json_projection_reaches_named_boundary_seeds() {
    let corpus_big_integer = include_bytes!("../corpus/natural_json_projection/big-integer");
    let corpus_unset_scalar = include_bytes!("../corpus/natural_json_projection/unset-scalar");
    let corpus_unset_collection = include_bytes!("../corpus/natural_json_projection/unset-collection");
    let corpus_empty_collection = include_bytes!("../corpus/natural_json_projection/empty-collection");
    let corpus_single_collection = include_bytes!("../corpus/natural_json_projection/single-collection");
    let corpus_nan = include_bytes!("../corpus/natural_json_projection/non-finite-nan");
    let corpus_positive_infinity = include_bytes!("../corpus/natural_json_projection/positive-infinity");
    let corpus_negative_infinity = include_bytes!("../corpus/natural_json_projection/negative-infinity");
    let corpus_negative_zero = include_bytes!("../corpus/natural_json_projection/negative-zero");
    let corpus_string_map = include_bytes!("../corpus/natural_json_projection/string-map");
    let corpus_string_scalar = include_bytes!("../corpus/natural_json_projection/string-scalar");
    let corpus_nested_json = include_bytes!("../corpus/natural_json_projection/nested-json");
    let corpus_large_json_number = include_bytes!("../corpus/natural_json_projection/large-json-number");

    assert_eq!(
        container_from_bytes(corpus_big_integer)
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::BigInteger
    );

    assert!(matches!(container_from_bytes(corpus_unset_scalar), ValueContainer::Scalar(ref value) if value.is_unset()));
    assert!(
        matches!(container_from_bytes(corpus_unset_collection), ValueContainer::Collection(ref values) if values.is_unset())
    );
    let empty_collection = container_from_bytes(corpus_empty_collection);
    assert_eq!(empty_collection.as_collection().unwrap().data_type(), DataType::Bool);
    assert!(
        empty_collection
            .as_collection()
            .unwrap()
            .get_bools()
            .unwrap()
            .is_empty()
    );
    let single_collection = container_from_bytes(corpus_single_collection);
    assert_eq!(single_collection.as_collection().unwrap().data_type(), DataType::Bool);
    assert_eq!(single_collection.as_collection().unwrap().get_bools().unwrap().len(), 1);
    let nan = value_from_bytes(corpus_nan);
    assert_eq!(nan.as_scalar().unwrap().data_type(), DataType::Float64);
    assert!(nan.as_scalar().unwrap().get_float64().unwrap().is_nan());
    let string_map = container_from_bytes(corpus_string_map);
    assert_eq!(string_map.as_scalar().unwrap().data_type(), DataType::StringMap);
    assert!(!string_map.as_scalar().unwrap().get_string_map_ref().unwrap().is_empty());
    assert_eq!(
        container_from_bytes(corpus_string_scalar)
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::String
    );
    assert_eq!(
        container_from_bytes(corpus_nested_json)
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::Json
    );
    let nested_json_container = container_from_bytes(corpus_nested_json);
    let nested_json = nested_json_container.as_scalar().unwrap().get_json_ref().unwrap();
    let inner_array = nested_json
        .as_array()
        .and_then(|items| items.first())
        .and_then(JsonValue::as_array)
        .expect("nested-json seed must reach a nested array");
    assert_eq!(inner_array, &[JsonValue::Null]);

    let large_json_number = value_from_bytes(corpus_large_json_number);
    assert_eq!(large_json_number.as_scalar().unwrap().data_type(), DataType::Json);
    let number = large_json_number
        .as_scalar()
        .unwrap()
        .get_json_ref()
        .unwrap()
        .get("number")
        .and_then(JsonValue::as_number)
        .expect("large-number seed must reach a JSON Number");
    assert!(
        number.as_f64().is_some_and(|number| number > i64::MAX as f64),
        "large-number seed must exceed the signed 64-bit range"
    );

    for (seed, negative) in [
        (corpus_positive_infinity.as_slice(), false),
        (corpus_negative_infinity.as_slice(), true),
    ] {
        let value = value_from_bytes(seed);
        let float = value.as_scalar().unwrap().get_float64().unwrap();
        assert!(float.is_infinite());
        assert_eq!(float.is_sign_negative(), negative);
    }

    assert!(matches!(
        string_map.to_json_value().unwrap(),
        JsonValue::Object(ref values) if !values.is_empty()
    ));
    let negative_zero_value = value_from_bytes(corpus_negative_zero);
    let negative_zero_float = negative_zero_value.as_scalar().unwrap().get_float64().unwrap();
    assert!(negative_zero_float.is_sign_negative() && negative_zero_float == 0.0);

    let named_corpus_seeds: [&[u8]; 13] = [
        corpus_big_integer.as_slice(),
        corpus_empty_collection,
        corpus_large_json_number,
        corpus_negative_infinity,
        corpus_negative_zero,
        corpus_nested_json,
        corpus_nan,
        corpus_positive_infinity,
        corpus_single_collection,
        corpus_string_map,
        corpus_string_scalar,
        corpus_unset_collection,
        corpus_unset_scalar,
    ];
    for seed in named_corpus_seeds {
        check_input(seed);
    }

    let unset_scalar = container_from_bytes(&structured_seed(25, 0));
    assert!(matches!(unset_scalar, ValueContainer::Scalar(ref value) if value.is_unset()));

    let unset_collection = container_from_bytes(&structured_seed(51, 0));
    assert!(matches!(unset_collection, ValueContainer::Collection(ref values) if values.is_unset()));

    for len in [0, 1] {
        let collection = container_from_bytes(&structured_seed(26, len));
        assert_eq!(collection.as_collection().unwrap().data_type(), DataType::Bool);
        assert_eq!(
            collection.as_collection().unwrap().get_bools().unwrap().len(),
            usize::from(len)
        );
        check_input(&structured_seed(26, len));
    }

    assert_eq!(
        container_from_bytes(&structured_seed(16, 0))
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::String
    );
    assert_eq!(
        container_from_bytes(&structured_seed(24, 0))
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::Json
    );
    assert_eq!(
        container_from_bytes(&structured_seed(14, 0))
            .as_scalar()
            .unwrap()
            .data_type(),
        DataType::BigInteger
    );

    let mut string_map = structured_seed(23, 0);
    string_map[6] = 1;
    let string_map_value = container_from_bytes(&string_map);
    assert_eq!(string_map_value.as_scalar().unwrap().data_type(), DataType::StringMap);
    assert!(
        !string_map_value
            .as_scalar()
            .unwrap()
            .get_string_map_ref()
            .unwrap()
            .is_empty()
    );

    let mut non_finite = vec![0_u8; 10];
    non_finite[0] = 4; // preflight_input tag 4 is Float64.
    non_finite[2] = 1; // Non-zero mantissa with an all-ones exponent is NaN.
    non_finite[8] = 0xf0;
    non_finite[9] = 0x7f;
    let non_finite_value = value_from_bytes(&non_finite);
    assert_eq!(non_finite_value.as_scalar().unwrap().data_type(), DataType::Float64);
    assert!(non_finite_value.as_scalar().unwrap().get_float64().unwrap().is_nan());
    assert!(value_from_bytes(&non_finite).to_json_value().is_err());

    let mut negative_zero = vec![0_u8; 10];
    negative_zero[0] = 4;
    negative_zero[9] = 0x80;
    let negative_zero_value = value_from_bytes(&negative_zero);
    let negative_zero_float = negative_zero_value.as_scalar().unwrap().get_float64().unwrap();
    assert!(negative_zero_float.is_sign_negative() && negative_zero_float == 0.0);

    for input in [
        Vec::new(),
        structured_seed(25, 0),
        structured_seed(51, 0),
        structured_seed(26, 0),
        structured_seed(26, 1),
        structured_seed(16, 0),
        structured_seed(24, 0),
        string_map,
        non_finite,
        negative_zero,
    ] {
        check_input(&input);
    }
}

#[test]
fn test_natural_json_projection_has_explicit_limit_and_recovery_oracles() {
    let policy = ConversionPolicy::default();
    let no_items = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_items(0).build())
        .build();
    let scalar = Value::Int32(1);
    let failed = scalar.to_json_value_with(&policy, &no_items);
    assert!(failed.is_err(), "max_items=0 rejects a concrete scalar");
    assert_eq!(scalar.to_json_value().unwrap(), JsonValue::from(1));

    let short_text = ConversionLimits::builder()
        .structured_limits(StructuredConversionLimits::builder().max_text_bytes(2).build())
        .build();
    assert!(Value::from("three").to_json_value_with(&policy, &short_text).is_err());

    for value in [
        Value::Float32(f32::NAN),
        Value::Float64(f64::INFINITY),
        Value::Float64(f64::NEG_INFINITY),
    ] {
        assert!(value.to_json_value_with(&policy, &ConversionLimits::default()).is_err());
    }

    let oversized = vec![0_u8; MAX_INPUT_BYTES + 1];
    assert_eq!(bounded_input(&oversized).len(), MAX_INPUT_BYTES);
    assert_eq!(bounded_input(&oversized), &oversized[..MAX_INPUT_BYTES]);
    check_input(&oversized);
    check_input(&oversized[..MAX_INPUT_BYTES]);
}
