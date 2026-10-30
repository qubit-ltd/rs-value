// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_budget::json::JsonEncodeLimits;
use qubit_value::Value;
use qubit_value::ValueContainer;
use qubit_value::ValueWireEncodeError;
use qubit_value::ValueWirePayloadRefV1;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWireRefV1;
use qubit_value::ValueWireV1;

#[test]
fn test_bounded_wire_entry_still_serializes_valid_values() {
    let limits = JsonEncodeLimits::builder().max_output_bytes(1024).build();
    let value = Value::from(42_i32);
    let wire = ValueWireRefV1::from_value(&value).unwrap();
    let encoded = wire.to_json_vec_with_limits(limits).unwrap();
    assert!(!encoded.is_empty(), "bounded serialization should produce JSON");
}

#[test]
fn test_direct_wire_helpers_preserve_success_bytes_and_budget_boundaries() {
    let cases = [
        ValueContainer::from(42_i32),
        ValueContainer::Collection(qubit_value::MultiValues::Int32(Vec::new())),
        ValueContainer::Scalar(Value::Unset(qubit_datatype::DataType::String)),
    ];
    for value in cases {
        assert_all_wire_helpers_match_serde_and_limits(&value);
    }

    #[cfg(feature = "big-decimal")]
    {
        use bigdecimal::BigDecimal;
        use num_bigint::BigInt;
        let decimal = ValueContainer::from(Value::BigDecimal(BigDecimal::new(BigInt::from(12345_i32), 2)));
        assert_all_wire_helpers_match_serde_and_limits(&decimal);
    }
}

fn assert_all_wire_helpers_match_serde_and_limits(value: &ValueContainer) {
    let payload_ref = ValueWirePayloadRefV1::from_container(value).expect("borrowed payload validates");
    let wire_ref = ValueWireRefV1::from_container(value).expect("borrowed envelope validates");
    let payload = ValueWirePayloadV1::try_from(value.clone()).expect("owned payload validates");
    let wire = ValueWireV1::try_from(value.clone()).expect("owned envelope validates");

    macro_rules! verify {
        ($dto:expr, $label:literal) => {{
            let expected = serde_json::to_vec($dto).expect("Serde encoding succeeds");
            assert_eq!($dto.to_json_vec().unwrap(), expected, "{} default bytes", $label);
            let exact = JsonEncodeLimits::builder().max_output_bytes(expected.len()).build();
            assert_eq!(
                $dto.to_json_vec_with_limits(exact).unwrap(),
                expected,
                "{} exact bytes",
                $label
            );

            let mut writer = Vec::new();
            $dto.to_json_writer_with_limits(&mut writer, exact).unwrap();
            assert_eq!(writer, expected, "{} exact writer bytes", $label);

            let too_small = JsonEncodeLimits::builder().max_output_bytes(expected.len() - 1).build();
            let vector_error = $dto.to_json_vec_with_limits(too_small).unwrap_err();
            assert!(
                matches!(vector_error, ValueWireEncodeError::Budget(_)),
                "{} must reject one byte below output size: {vector_error:?}",
                $label
            );
            let mut prefixed = b"prefix".to_vec();
            assert!(
                matches!(
                    $dto.to_json_writer_with_limits(&mut prefixed, too_small),
                    Err(ValueWireEncodeError::Budget(_))
                ),
                "{} writer must reject one byte below output size",
                $label
            );
            assert_eq!(prefixed, b"prefix", "{} writer preserves prefix", $label);
        }};
    }

    verify!(&payload_ref, "ValueWirePayloadRefV1");
    verify!(&wire_ref, "ValueWireRefV1");
    verify!(&payload, "ValueWirePayloadV1");
    verify!(&wire, "ValueWireV1");
}

#[test]
fn test_wire_envelope_limit_still_counts_the_envelope() {
    let value = Value::from(42_i32);
    let wire = ValueWireRefV1::from_value(&value).unwrap();
    let limits = JsonEncodeLimits::builder().max_nodes(1).build();
    let error = wire.to_json_vec_with_limits(limits).unwrap_err();
    assert!(matches!(error, ValueWireEncodeError::Budget(_)), "{error:?}");
}
