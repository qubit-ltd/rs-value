// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Boundary and error-path tests for the public [`Value`] contract.

use std::collections::HashMap;

use qubit_datatype::DataType;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueMissing;
use qubit_value::ValueRef;

#[test]
fn test_value_get_or_only_falls_back_for_unset_storage() {
    assert_eq!(
        Value::new_unset(DataType::String)
            .get_or::<String>("fallback")
            .expect("unset values should use the fallback"),
        "fallback"
    );
    assert_eq!(
        Value::new_unset(DataType::String)
            .get_or_else::<String, _>(|| "lazy-fallback".to_owned())
            .expect("unset values should invoke the lazy fallback"),
        "lazy-fallback"
    );
}

#[test]
fn test_value_get_or_preserves_type_and_empty_errors() {
    assert!(matches!(
        Value::Int32(7).get_or::<String>("fallback"),
        Err(ValueError::TypeMismatch {
            expected: DataType::String,
            actual: DataType::Int32,
        })
    ));
    assert!(matches!(
        Value::String(String::new()).get_or::<String>("fallback"),
        Ok(value) if value.is_empty()
    ));
}

#[test]
fn test_value_get_ref_matches_owned_strict_reads() {
    let value = Value::String("borrowed".to_owned());
    assert_eq!(value.get_ref::<str>().expect("string should borrow"), "borrowed");
    assert!(matches!(
        Value::new_unset(DataType::Int32).get_ref::<i32>(),
        Err(ValueError::Missing(ref missing)) if *missing == ValueMissing::unset_scalar(DataType::Int32, DataType::Int32)
    ));
}

#[test]
fn test_value_view_preserves_scalar_and_collection_semantics() {
    let map = HashMap::from([(String::from("key"), String::from("value"))]);
    let duration = std::time::Duration::from_secs(3);
    assert!(matches!(
        Value::new_unset(DataType::Bool).view(),
        ValueRef::Unset(DataType::Bool)
    ));
    assert!(matches!(Value::Bool(true).view(), ValueRef::Bool(true)));
    assert!(matches!(Value::Char('x').view(), ValueRef::Char('x')));
    assert!(matches!(Value::Int8(-1).view(), ValueRef::Int8(-1)));
    assert!(matches!(Value::Int16(-2).view(), ValueRef::Int16(-2)));
    assert!(matches!(Value::Int32(-3).view(), ValueRef::Int32(-3)));
    assert!(matches!(Value::Int64(-4).view(), ValueRef::Int64(-4)));
    assert!(matches!(Value::Int128(-5).view(), ValueRef::Int128(-5)));
    assert!(matches!(Value::UInt8(1).view(), ValueRef::UInt8(1)));
    assert!(matches!(Value::UInt16(2).view(), ValueRef::UInt16(2)));
    assert!(matches!(Value::UInt32(3).view(), ValueRef::UInt32(3)));
    assert!(matches!(Value::UInt64(4).view(), ValueRef::UInt64(4)));
    assert!(matches!(Value::UInt128(5).view(), ValueRef::UInt128(5)));
    assert!(matches!(Value::Float32(1.5).view(), ValueRef::Float32(value) if value == 1.5));
    assert!(matches!(Value::Float64(2.5).view(), ValueRef::Float64(value) if value == 2.5));
    assert!(matches!(
        Value::String("text".to_owned()).view(),
        ValueRef::String("text")
    ));
    assert!(matches!(Value::Duration(duration).view(), ValueRef::Duration(value) if value == &duration));
    assert!(matches!(Value::StringMap(map.clone()).view(), ValueRef::StringMap(value) if value == &map));
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_value_json_projection_and_serialization_errors_are_structured() {
    use qubit_datatype::DataConversionErrorKind;

    assert_eq!(
        Value::from_json_value(serde_json::json!({"answer": 42}))
            .to_json_value()
            .expect("JSON values should project unchanged"),
        serde_json::json!({"answer": 42})
    );
    assert!(matches!(
        Value::Float64(f64::NAN).to_json_value(),
        Err(ValueError::Conversion(error))
            if error.kind() == DataConversionErrorKind::InvalidValue
    ));
    assert!(matches!(
        Value::from_serializable(&f64::NAN),
        Err(ValueError::Conversion(error))
            if error.kind() == DataConversionErrorKind::InvalidValue
    ));
}
