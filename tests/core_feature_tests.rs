// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::{MultiValues, NamedMultiValues, NamedValue, Value, ValueContainer, ValueError};

#[test]
fn unset_is_distinct_from_concrete_empty_values_without_optional_features() {
    let unset_value = Value::Unset(DataType::String);
    let empty_string = Value::String(String::new());
    assert!(unset_value.is_unset());
    assert!(!empty_string.is_unset());

    let unset_values = MultiValues::Unset(DataType::Int32);
    let empty_values = MultiValues::Int32(Vec::new());
    assert!(unset_values.is_unset());
    assert!(!empty_values.is_unset());
    assert!(empty_values.get_int32s().unwrap().is_empty());
}

#[test]
fn generic_mutation_is_available_without_optional_features() {
    let mut value = Value::Unset(DataType::Int32);
    value.set(42_i32);
    assert_eq!(value.get::<i32>().unwrap(), 42);

    let mut values = MultiValues::Unset(DataType::Int32);
    values.set([1_i32, 2]);
    values.add(&[3_i32, 4][..]).unwrap();
    assert_eq!(values.get_int32s().unwrap(), &[1, 2, 3, 4]);

    assert!(matches!(
        values.add(true),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::Bool,
        })
    ));
}

#[test]
fn lazy_scalar_default_is_only_evaluated_for_unset_values() {
    let value = Value::Int32(42);
    let mut evaluated = false;

    let result = value.get_or_else(|| {
        evaluated = true;
        0_i32
    });

    assert_eq!(result.expect("read concrete value"), 42);
    assert!(!evaluated, "concrete reads must not evaluate the fallback");
}

#[test]
fn test_add_empty_collection_preserves_scalar_shape() {
    let mut container = ValueContainer::from(42_i32);

    container
        .add(Vec::<i32>::new())
        .expect("empty collection has the same data type");

    assert_eq!(container, ValueContainer::Scalar(Value::Int32(42)));
}

#[test]
fn test_add_unset_value_preserves_scalar_shape() {
    let mut container = ValueContainer::from(42_i32);

    container
        .add(Value::Unset(DataType::Int32))
        .expect("unset value has the same data type");

    assert_eq!(container, ValueContainer::Scalar(Value::Int32(42)));
}

#[test]
fn test_add_moves_owned_strings_when_promoting_scalar() {
    let mut container = ValueContainer::from("existing");
    let appended = vec!["appended".to_string()];
    let appended_ptr = appended[0].as_ptr();

    container
        .add(appended)
        .expect("owned strings have the same data type");

    let ValueContainer::Collection(MultiValues::String(values)) = container else {
        panic!("expected a string collection");
    };
    assert_eq!(values[1].as_ptr(), appended_ptr);
}

#[test]
fn test_add_moves_owned_strings_into_collection() {
    let mut container = ValueContainer::from(vec!["existing".to_string()]);
    let appended = vec!["appended".to_string()];
    let appended_ptr = appended[0].as_ptr();

    container
        .add(appended)
        .expect("owned strings have the same data type");

    let ValueContainer::Collection(MultiValues::String(values)) = container else {
        panic!("expected a string collection");
    };
    assert_eq!(values[1].as_ptr(), appended_ptr);
}

#[test]
fn test_add_rejects_mismatched_empty_collection() {
    let mut container = ValueContainer::from(42_i32);

    assert!(matches!(
        container.add(Vec::<bool>::new()),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::Bool,
        })
    ));
    assert_eq!(container, ValueContainer::Scalar(Value::Int32(42)));
}

#[test]
fn test_value_container_preserves_explicit_shapes() {
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![42_i32]);

    assert!(scalar.is_scalar());
    assert!(!scalar.is_collection());
    assert_eq!(scalar.len(), 1);
    assert!(collection.is_collection());
    assert!(!collection.is_scalar());
    assert_eq!(collection.len(), 1);
}

#[test]
fn test_value_container_generic_api_uses_public_bounds() {
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![43_i32, 44]);

    let scalar_value: i32 = scalar.get_first().expect("strict scalar access");
    let collection_first: i32 = collection.get_first().expect("strict first access");
    let scalar_values: Vec<i32> = scalar.get_list().expect("strict scalar list access");
    let collection_values: Vec<i32> = collection.get_list().expect("strict collection access");

    assert_eq!(scalar_value, 42);
    assert_eq!(collection_first, 43);
    assert_eq!(scalar_values, vec![42]);
    assert_eq!(collection_values, vec![43, 44]);
}

#[test]
fn test_value_container_mutation_preserves_shape() {
    let mut scalar = ValueContainer::from(42_i32);
    let mut collection = ValueContainer::from(vec![43_i32]);

    scalar.set("replacement");
    collection.add(44_i32).expect("append compatible scalar");
    assert_eq!(scalar, ValueContainer::from("replacement"));
    assert_eq!(
        collection,
        ValueContainer::Collection(MultiValues::Int32(vec![43, 44]))
    );

    scalar.clear();
    collection.unset();
    assert_eq!(
        scalar,
        ValueContainer::Scalar(Value::Unset(DataType::String))
    );
    assert_eq!(
        collection,
        ValueContainer::Collection(MultiValues::Unset(DataType::Int32))
    );
}

#[test]
fn test_value_container_rejects_mismatched_add() {
    let mut container = ValueContainer::from(42_i32);

    assert!(matches!(
        container.add(true),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::Bool,
        })
    ));
    assert_eq!(container, ValueContainer::Scalar(Value::Int32(42)));
}

#[test]
fn numeric_classification_depends_on_concrete_state() {
    assert!(!Value::Unset(DataType::Int128).is_numeric());
    assert!(Value::Int128(i128::MIN).is_numeric());
    assert!(!MultiValues::Unset(DataType::UInt128).is_numeric());
    assert!(MultiValues::UInt128(Vec::new()).is_numeric());
    assert!(!Value::String("1".to_string()).is_numeric());
}

#[test]
fn named_wrappers_retain_generic_core_access() {
    let mut named = NamedValue::new("port", Value::Int32(8080));
    named.value_mut().set(9090_i32);
    assert_eq!(named.value().get_int32().unwrap(), 9090);

    let mut named_values = NamedMultiValues::new("ports", MultiValues::Int32(vec![8080]));
    named_values.values_mut().add(9090_i32).unwrap();
    assert_eq!(named_values.values().get_int32s().unwrap(), &[8080, 9090]);
}
