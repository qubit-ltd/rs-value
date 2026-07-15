// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
// =============================================================================

//! Tests for the explicit scalar-or-collection value container.

use qubit_datatype::{
    CollectionConversionOptions,
    DataConversionOptions,
    DataType,
};
use qubit_value::{
    MultiValues,
    Value,
    ValueContainer,
    ValueError,
};
use serde_json::json;

#[test]
fn test_value_container_preserves_scalar_and_collection_shapes() {
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![42_i32]);

    assert!(scalar.is_scalar());
    assert!(!scalar.is_collection());
    assert!(collection.is_collection());
    assert!(!collection.is_scalar());
    assert_eq!(scalar.count(), 1);
    assert_eq!(collection.count(), 1);
    assert_eq!(scalar.to_json_value().expect("scalar JSON"), json!(42));
    assert_eq!(
        collection.to_json_value().expect("collection JSON"),
        json!([42])
    );
}

#[test]
fn test_value_container_string_splitting_depends_on_explicit_shape() {
    let options = DataConversionOptions::default().with_collection_options(
        CollectionConversionOptions::default()
            .with_split_scalar_strings(true)
            .with_delimiters([',']),
    );
    let scalar = ValueContainer::from("a,b");
    let collection = ValueContainer::from(vec!["a,b"]);

    assert_eq!(
        scalar
            .to_list_with::<String>(&options)
            .expect("split scalar string"),
        vec!["a".to_string(), "b".to_string()]
    );
    assert_eq!(
        collection
            .to_list_with::<String>(&options)
            .expect("preserve collection string"),
        vec!["a,b".to_string()]
    );
}

#[test]
fn test_value_container_add_promotes_scalar_and_checks_type() {
    let mut container = ValueContainer::from(1_i32);
    container.add(2_i32).expect("append compatible value");

    assert_eq!(
        container,
        ValueContainer::Collection(MultiValues::Int32(vec![1, 2]))
    );
    assert!(matches!(
        container.add("wrong type"),
        Err(ValueError::TypeMismatch {
            expected: DataType::Int32,
            actual: DataType::String,
        })
    ));
}

#[test]
fn test_value_container_add_initializes_typed_unset_scalar() {
    let mut container = ValueContainer::Scalar(Value::Unset(DataType::Int32));
    container.add(1_i32).expect("initialize typed unset");

    assert_eq!(
        container,
        ValueContainer::Collection(MultiValues::Int32(vec![1]))
    );
}

#[test]
fn test_value_container_clear_and_unset_preserve_shape() {
    let mut scalar = ValueContainer::from(42_i32);
    scalar.clear();
    assert_eq!(
        scalar,
        ValueContainer::Scalar(Value::Unset(DataType::Int32))
    );

    let mut collection = ValueContainer::from(vec![42_i32]);
    collection.clear();
    assert_eq!(
        collection,
        ValueContainer::Collection(MultiValues::Int32(vec![]))
    );
    collection.unset();
    assert_eq!(
        collection,
        ValueContainer::Collection(MultiValues::Unset(DataType::Int32))
    );
}

#[test]
fn test_value_container_tagged_wire_preserves_shape() {
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![42_i32]);

    assert_eq!(
        serde_json::to_value(&scalar).expect("serialize scalar"),
        json!({"version": 1, "value": {"scalar": {"int32": 42}}})
    );
    assert_eq!(
        serde_json::to_value(&collection).expect("serialize collection"),
        json!({"version": 1, "value": {"collection": {"int32": [42]}}})
    );
}

#[test]
fn test_value_container_constructors_cover_borrowed_and_owned_collections() {
    let values = vec![1_i32, 2];
    let array = [3_i32, 4];
    assert_eq!(
        ValueContainer::from(values.as_slice()),
        ValueContainer::Collection(MultiValues::Int32(vec![1, 2]))
    );
    assert_eq!(
        ValueContainer::from(&values),
        ValueContainer::Collection(MultiValues::Int32(vec![1, 2]))
    );
    assert_eq!(
        ValueContainer::from(array),
        ValueContainer::Collection(MultiValues::Int32(vec![3, 4]))
    );
    assert_eq!(
        ValueContainer::from(&array),
        ValueContainer::Collection(MultiValues::Int32(vec![3, 4]))
    );

    let strings = vec!["a", "b"];
    let string_array = ["c", "d"];
    assert_eq!(
        ValueContainer::from(strings.clone()),
        ValueContainer::Collection(MultiValues::String(vec![
            "a".into(),
            "b".into()
        ]))
    );
    assert_eq!(
        ValueContainer::from(strings.as_slice()),
        ValueContainer::Collection(MultiValues::String(vec![
            "a".into(),
            "b".into()
        ]))
    );
    assert_eq!(
        ValueContainer::from(&strings),
        ValueContainer::Collection(MultiValues::String(vec![
            "a".into(),
            "b".into()
        ]))
    );
    assert_eq!(
        ValueContainer::from(string_array),
        ValueContainer::Collection(MultiValues::String(vec![
            "c".into(),
            "d".into()
        ]))
    );
    assert_eq!(
        ValueContainer::from(&string_array),
        ValueContainer::Collection(MultiValues::String(vec![
            "c".into(),
            "d".into()
        ]))
    );

    assert_eq!(
        ValueContainer::from(Value::Int32(5)),
        ValueContainer::Scalar(Value::Int32(5))
    );
    assert_eq!(
        ValueContainer::from(MultiValues::Int32(vec![6])),
        ValueContainer::Collection(MultiValues::Int32(vec![6]))
    );
}

#[test]
fn test_value_container_strict_access_mutation_and_state_cover_both_shapes() {
    let mut scalar = ValueContainer::from(1_i32);
    let mut collection = ValueContainer::from(vec![2_i32, 3]);

    assert_eq!(scalar.data_type(), DataType::Int32);
    assert_eq!(collection.data_type(), DataType::Int32);
    assert!(!scalar.is_unset());
    assert!(!collection.is_unset());
    assert_eq!(scalar.get::<i32>().unwrap(), 1);
    assert_eq!(collection.get::<i32>().unwrap(), 2);
    assert_eq!(scalar.get_list::<i32>().unwrap(), vec![1]);
    assert_eq!(collection.get_list::<i32>().unwrap(), vec![2, 3]);

    scalar.set("replacement");
    assert_eq!(scalar, ValueContainer::from("replacement"));

    collection
        .add(vec![4_i32, 5])
        .expect("append explicit collection");
    collection.add(6_i32).expect("append scalar to collection");
    assert_eq!(
        collection,
        ValueContainer::Collection(MultiValues::Int32(vec![2, 3, 4, 5, 6]))
    );

    scalar.unset();
    collection.unset();
    assert!(scalar.is_unset());
    assert!(collection.is_unset());
    assert_eq!(scalar.count(), 0);
    assert_eq!(collection.count(), 0);
}

#[test]
fn test_value_container_conversion_covers_scalar_and_collection_dispatch() {
    let options = DataConversionOptions::default();
    let scalar = ValueContainer::from(42_i32);
    let collection = ValueContainer::from(vec![43_i32, 44]);

    assert_eq!(scalar.to::<i64>().unwrap(), 42);
    assert_eq!(scalar.to_with::<i64>(&options).unwrap(), 42);
    assert_eq!(collection.to_with::<i64>(&options).unwrap(), 43);
    assert_eq!(scalar.to_list::<i64>().unwrap(), vec![42]);
    assert_eq!(collection.to_list::<i64>().unwrap(), vec![43, 44]);
    assert_eq!(scalar.to_list_with::<i64>(&options).unwrap(), vec![42]);
    assert_eq!(
        collection.to_list_with::<i64>(&options).unwrap(),
        vec![43, 44]
    );
}
