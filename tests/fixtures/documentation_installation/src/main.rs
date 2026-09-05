// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Exercises the public types from the documentation installation dependencies.

use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::DataType;
use qubit_value::Value;

/// Verifies that the installed datatype vocabulary is the value crate's vocabulary.
fn main() {
    let unset = Value::new_unset(DataType::UInt16);
    assert_eq!(unset.data_type(), DataType::UInt16);
    let port = Value::from("8080").to_with::<u16>(
        &ConversionPolicy::default(), &ConversionLimits::default()
    ).expect("documented conversion types must interoperate");
    assert_eq!(port, 8080);
}
