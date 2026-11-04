// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value_fuzz::preflight_input::bounded_input;
use qubit_value_fuzz::preflight_input::value_from_bytes;

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
fn test_preflight_generator_bounds_and_decodes_input_deterministically() {
    for length in [0, 3, 258, 259, 4096, 4097] {
        let input = vec![0x5a; length];
        let bounded = bounded_input(&input);
        assert!(bounded.len() <= 4096);
        assert_eq!(value_from_bytes(bounded), value_from_bytes(bounded));
        if length >= 3 {
            let mut integer_input = vec![1; length];
            integer_input[0] = 6;
            let integer = value_from_bytes(&integer_input);
            let integer = integer.as_scalar().unwrap().get_ref::<BigInt>().unwrap();
            assert!(integer.bits() <= 256 * 8);
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
    use qubit_value_fuzz::preflight_sequence::run_sequence;
    use qubit_value_fuzz::structured_wire_input::container_from_bytes;

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
