// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Checks deterministic fuzz input generation and its public sequence
//! integration.

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value_fuzz::preflight_input::bounded_input;
use qubit_value_fuzz::preflight_input::value_from_bytes;
use qubit_value_fuzz::preflight_sequence::run_sequence;
use qubit_value_fuzz::structured_wire_input::container_from_bytes;
use serde_json::Value as JsonValue;

#[test]
fn test_preflight_generator_reaches_all_tags() {
    let expected = [
        (DataType::String, false),
        (DataType::Int64, false),
        (DataType::UInt128, false),
        (DataType::Int128, false),
        (DataType::Float64, false),
        (DataType::Float32, false),
        (DataType::BigInteger, false),
        (DataType::BigDecimal, false),
        (DataType::Duration, false),
        (DataType::String, false),
        (DataType::Json, false),
        (DataType::String, true),
        (DataType::Json, true),
        (DataType::Int128, true),
    ];

    for (tag, (kind, collection)) in expected.into_iter().enumerate() {
        let value = value_from_bytes(&[tag as u8, 0, 0, 1]);
        assert_eq!(value.data_type(), kind, "tag {tag}");
        assert_eq!(value.is_collection(), collection, "tag {tag}");
    }
}

#[test]
fn test_preflight_generator_short_big_numbers_are_nonzero() {
    let integer = value_from_bytes(&[6, 0, 0, 1]);
    let decimal = value_from_bytes(&[7, 0, 0, 1]);
    let integer = integer.as_scalar().unwrap().get_ref::<BigInt>().unwrap();
    let decimal = decimal.as_scalar().unwrap().get_ref::<BigDecimal>().unwrap();
    assert_eq!(integer, &BigInt::from(1));
    assert_eq!(decimal, &BigDecimal::from(1));
}

#[test]
fn test_preflight_json_number_choices_are_deterministic_and_numeric() {
    // Tag 10 selects a JSON scalar. Byte 4 selects 1.2500, while byte 3
    // keeps the value at the object leaf instead of wrapping it in arrays.
    let input = [10, 0, 0, 0, 1, 0];
    let first = value_from_bytes(&input);
    let second = value_from_bytes(&input);
    assert_eq!(first, second);

    let json = first.as_scalar().unwrap().get_ref::<JsonValue>().unwrap();
    let JsonValue::Number(number) = json.get("number").unwrap() else {
        panic!("selected JSON value should contain a number");
    };
    #[cfg(feature = "json-arbitrary-precision")]
    assert_eq!(number.to_string(), "1.2500");
    #[cfg(not(feature = "json-arbitrary-precision"))]
    assert_eq!(number.to_string(), "1.25");
}

#[test]
fn test_preflight_json_object_order_choices_are_semantically_equal() {
    let first = value_from_bytes(&[10, 0, 0, 0, 1, 0]);
    let second = value_from_bytes(&[24, 0, 0, 0, 1, 0]);
    let first = first.as_scalar().unwrap().get_ref::<JsonValue>().unwrap();
    let second = second.as_scalar().unwrap().get_ref::<JsonValue>().unwrap();
    assert_eq!(first, second);

    let first_keys: Vec<_> = first.as_object().unwrap().keys().map(String::as_str).collect();
    let second_keys: Vec<_> = second.as_object().unwrap().keys().map(String::as_str).collect();
    #[cfg(feature = "json-preserve-order")]
    assert_ne!(first_keys, second_keys);
    #[cfg(not(feature = "json-preserve-order"))]
    assert_eq!(first_keys, second_keys);
}

#[test]
fn test_preflight_generator_bounds_and_decodes_input_deterministically() {
    for length in [0, 3, 258, 259, 4096, 4097] {
        let input = vec![0x5a; length];
        let bounded = bounded_input(&input);
        assert!(bounded.len() <= 4096, "input length {length} exceeds the byte bound");
        assert_eq!(
            value_from_bytes(bounded),
            value_from_bytes(bounded),
            "input length {length} must decode deterministically"
        );
        if length >= 3 {
            let mut integer_input = vec![1; length];
            integer_input[0] = 6;
            let integer = value_from_bytes(&integer_input);
            let integer = integer.as_scalar().unwrap().get_ref::<BigInt>().unwrap();
            assert!(
                integer.bits() <= 256 * 8,
                "input length {length} exceeds the integer bit bound"
            );
        }
    }

    let input = vec![0x5a; 4096];
    let mut extended = input.clone();
    extended.push(0xa5);
    assert_eq!(bounded_input(&input), bounded_input(&extended));

    let collection = value_from_bytes(&[13, 0, 0, 1]);
    assert!(collection.is_collection());
    assert!(!collection.get_slice::<i128>().unwrap().is_empty());
}

#[test]
fn test_structured_generator_reaches_required_types_and_collection_variants() {
    let expected = [
        (15, DataType::BigDecimal, false),
        (17, DataType::Date, false),
        (18, DataType::Time, false),
        (19, DataType::DateTime, false),
        (20, DataType::Instant, false),
        (22, DataType::Url, false),
        (23, DataType::StringMap, false),
        (41, DataType::BigDecimal, true),
        (43, DataType::Date, true),
        (44, DataType::Time, true),
        (45, DataType::DateTime, true),
        (46, DataType::Instant, true),
        (48, DataType::Url, true),
        (49, DataType::StringMap, true),
    ];

    for (tag, data_type, collection) in expected {
        let mut input = vec![0_u8; 8];
        input[0] = tag;
        let value = container_from_bytes(&input);
        assert_eq!(value.data_type(), data_type, "structured tag {tag}");
        assert_eq!(value.is_collection(), collection, "structured tag {tag}");

        let mut sequence_input = vec![0; 513];
        sequence_input[1] = 2;
        sequence_input[2] = tag;
        let report = run_sequence(&sequence_input);
        assert_eq!(report.operation_count, 1, "structured tag {tag}");
        assert_eq!(report.checked_entries, 4, "structured tag {tag}");
        assert_eq!(report.successes, 1, "structured tag {tag}");
    }
}
