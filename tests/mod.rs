// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! # Value Processing Module Tests
//!
//! Provides integration tests for the value processing framework.

use qubit_value::ValueWireV1;
use qubit_value::ValueWireV1Seed;
use qubit_value::ValueWirePayloadV1;
use qubit_value::ValueWirePayloadV1Seed;
use serde::de::DeserializeSeed;

fn decode_value_wire_str(input: &str) -> Result<ValueWireV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

fn decode_value_wire_value(input: serde_json::Value) -> Result<ValueWireV1, serde_json::Error> {
    decode_value_wire_str(&serde_json::to_string(&input).expect("test JSON value should serialize"))
}

fn decode_value_wire_slice(input: &[u8]) -> Result<ValueWireV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_slice(input);
    ValueWireV1Seed::new().deserialize(&mut deserializer)
}

fn decode_value_wire_payload_str(input: &str) -> Result<ValueWirePayloadV1, serde_json::Error> {
    let mut deserializer = serde_json::Deserializer::from_str(input);
    ValueWirePayloadV1Seed::new().deserialize(&mut deserializer)
}

fn decode_value_wire_payload_value(input: serde_json::Value) -> Result<ValueWirePayloadV1, serde_json::Error> {
    decode_value_wire_payload_str(&serde_json::to_string(&input).expect("test JSON value should serialize"))
}

mod contracts;
mod doc_examples_tests;
mod finite_float_tests;
mod finite_float {
    mod internal {
        mod finite_float_tests;
    }
}
mod identity;
mod into_value_default_tests;
mod json_tests;
mod multi_values;
mod named_multi_values_tests;
mod named_multi_values {
    mod internal {
        mod named_multi_values_wire_owned_tests;
        mod named_multi_values_wire_ref_tests;
    }
}
mod named_value_tests;
mod named_value {
    mod internal {
        mod named_value_wire_owned_tests;
        mod named_value_wire_ref_tests;
    }
}
mod numeric_comparison_error_tests;
mod strict_json_tests;
mod strict_value_read_tests;
mod value;
mod value_container_tests;
mod value_error_tests;
mod value_missing_tests;
mod value_type_table_tests;
mod value_wire;
mod value_wire_encode_error_tests;
mod value_wire_payload_v1_tests;
mod value_wire_tests;
mod wide_integer_tests;
mod wide_integer {
    mod internal {
        mod display_integer_tests;
        mod integer_visitor_tests;
        mod parsed_integer_tests;
    }
}
mod wire {
    mod decimal {
        mod internal {
            mod decimal_visitor_tests;
            mod display_decimal_tests;
            mod parsed_decimal_tests;
        }
        mod decimal_tests;
    }
    mod internal {
        mod big_decimal_payload_tests;
        mod duration_payload_tests;
    }
}
mod wire_tests;
