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
