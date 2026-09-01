// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests for scalar/collection shape preservation.

use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;

#[test]
fn adding_same_type_scalars_promotes_to_an_ordered_collection() {
    let mut container = ValueContainer::Scalar(Value::Int32(7));

    container.add(ValueContainer::Scalar(Value::Int32(8))).unwrap();

    assert_eq!(container, ValueContainer::Collection(MultiValues::Int32(vec![7, 8])));
}

#[test]
fn adding_different_types_rejects_without_changing_shape() {
    let mut container = ValueContainer::Scalar(Value::Int32(7));

    assert!(
        container
            .add(ValueContainer::Scalar(Value::String("8".to_owned())))
            .is_err()
    );
    assert_eq!(container, ValueContainer::Scalar(Value::Int32(7)));
}
