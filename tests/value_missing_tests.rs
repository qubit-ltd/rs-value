// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_datatype::BlankStringPolicy;
use qubit_datatype::ConversionLimits;
use qubit_datatype::ConversionOperationLimits;
use qubit_datatype::ConversionPolicy;
use qubit_datatype::ConversionSession;
use qubit_datatype::DataConversionError;
use qubit_datatype::DataConverters;
use qubit_datatype::DataListConversionError;
use qubit_datatype::DataType;
use qubit_datatype::StringConversionPolicy;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueError;
use qubit_value::ValueMissing;
use qubit_value::ValueMissingReason;

/// Builds a policy under which whitespace is a missing concrete string.
fn missing_policy() -> ConversionPolicy {
    ConversionPolicy::builder()
        .string_policy(
            StringConversionPolicy::builder()
                .trim(true)
                .blank_string_policy(BlankStringPolicy::TreatAsMissing)
                .build(),
        )
        .build()
}

#[test]
fn test_missing_strict_read_retains_requested_target() {
    let error = Value::new_unset(DataType::Int32).get::<i32>().unwrap_err();
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::UnsetScalar);
    assert_eq!(missing.source_type(), Some(DataType::Int32));
    assert_eq!(missing.target_type(), Some(DataType::Int32));
    assert_eq!(missing.source_index(), None);
    assert!(missing.is_defaultable_for_strict_read());
    assert!(!missing.is_conversion());
}

#[test]
fn test_missing_conversion_retains_unset_scalar_fact() {
    let error = Value::new_unset(DataType::String).to::<i32>().unwrap_err();
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::UnsetScalar);
    assert!(missing.is_unset());
    assert!(missing.is_conversion());
    assert_eq!(missing.source_type(), Some(DataType::String));
    assert_eq!(missing.target_type(), Some(DataType::Int32));
}

#[test]
fn test_missing_collection_conversion_retains_unset_fact() {
    let values = MultiValues::Unset(DataType::Int32);
    let error = values.to_first::<i64>().unwrap_err();
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::UnsetCollection);
    assert_eq!(missing.source_type(), Some(DataType::Int32));
    assert_eq!(missing.target_type(), Some(DataType::Int64));
    assert_eq!(missing.source_index(), None);
    assert_eq!(values.to_first_or::<i64>(7), Ok(7));
}

#[test]
fn test_missing_empty_conversion_retains_element_type() {
    let values = MultiValues::Int32(Vec::new());
    let error = values.to_first::<i64>().unwrap_err();
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::EmptyCollection);
    assert_eq!(missing.source_type(), Some(DataType::Int32));
    assert_eq!(missing.target_type(), Some(DataType::Int64));
    assert!(missing.is_conversion());
    assert!(!missing.is_defaultable_for_strict_read());
    assert!(!missing.is_defaultable_for_conversion());
    assert!(values.get::<i32>().unwrap().is_empty());
    assert!(values.to_list::<i64>().unwrap().is_empty());
    assert!(values.get_first_or::<i32>(7).is_err());
    assert!(values.to_first_or::<i64>(7).is_err());
}

#[test]
fn test_missing_policy_scalar_defaults_but_collection_item_does_not() {
    let policy = missing_policy();
    let limits = ConversionLimits::default_ref();
    let value = Value::from(" ");
    let error = value.to_with::<i32>(&policy, limits).unwrap_err();
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::Conversion);
    assert!(!missing.is_unset());
    assert!(!missing.is_defaultable_for_strict_read());
    assert!(missing.is_defaultable_for_conversion());
    assert_eq!(value.to_or_with::<i32>(7, &policy, limits), Ok(7));
    assert_eq!(value.get::<String>().unwrap(), " ");
    for (texts, index) in [(vec!["1".to_owned(), " ".to_owned()], 1), (vec![" ".to_owned()], 0)] {
        let values = MultiValues::String(texts);
        let error = values.to_list_or_with::<i32>(vec![7], &policy, limits).unwrap_err();
        let missing = error.missing().unwrap();
        assert_eq!(missing.source_index(), Some(index));
        assert_eq!(missing.reason(), ValueMissingReason::Conversion);
        assert!(!missing.is_defaultable_for_conversion());
    }
    let first = MultiValues::String(vec![" ".to_owned()]);
    let error = first.to_first_or_with::<i32>(7, &policy, limits).unwrap_err();
    assert_eq!(error.missing().unwrap().source_index(), Some(0));
}

#[test]
fn test_missing_conversion_source_chain_is_preserved() {
    let original = DataConversionError::missing(DataType::String, DataType::Int32);
    let error = ValueError::from(original.clone());
    let missing = error.source().expect("missing descriptor source");
    assert!(missing.downcast_ref::<ValueMissing>().is_some());
    assert_eq!(
        missing.source().and_then(|source| source.downcast_ref()),
        Some(&original)
    );
    assert_eq!(error.missing().unwrap().conversion_error(), Some(&original));
    let strict = ValueError::Missing(ValueMissing::unset_scalar(DataType::String, DataType::String));
    assert!(strict.source().unwrap().source().is_none());
    let indexed = ValueError::from(DataListConversionError::new(3, original.clone()));
    assert_eq!(indexed.missing().unwrap().conversion_error(), Some(&original));
    assert_eq!(indexed.missing().unwrap().source_index(), Some(3));
    assert_eq!(indexed.clone(), indexed);
}

#[test]
fn test_missing_generic_empty_conversion_does_not_invent_source() {
    let empty = Vec::<i32>::new();
    let error = ValueError::from(DataConverters::from(&empty).to_first::<i64>().unwrap_err());
    let missing = error.missing().unwrap();
    assert_eq!(missing.reason(), ValueMissingReason::EmptyCollection);
    assert_eq!(missing.source_type(), None);
    assert_eq!(missing.target_type(), Some(DataType::Int64));
    assert!(missing.is_empty_collection());
    assert!(!missing.is_defaultable_for_conversion());
}

#[test]
fn test_missing_session_reads_preserve_storage_and_item_context() {
    let policy = missing_policy();
    let limits = ConversionLimits::default();
    let mut session = ConversionSession::new(&policy, &limits);
    let error = Value::Unset(DataType::String).to_in::<i32>(&mut session).unwrap_err();
    assert_eq!(error.missing().unwrap().reason(), ValueMissingReason::UnsetScalar);
    let unset = MultiValues::Unset(DataType::String);
    for error in [
        unset.to_first_in::<i64>(&mut session).unwrap_err(),
        unset.to_list_in::<i64>(&mut session).unwrap_err(),
    ] {
        let missing = error.missing().unwrap();
        assert_eq!(missing.reason(), ValueMissingReason::UnsetCollection);
        assert_eq!(missing.source_type(), Some(DataType::String));
        assert_eq!(missing.target_type(), Some(DataType::Int64));
        assert!(missing.conversion_error().is_some());
    }
    let empty = MultiValues::Int32(Vec::new());
    let error = empty.to_first_in::<i64>(&mut session).unwrap_err();
    assert_eq!(error.missing().unwrap().reason(), ValueMissingReason::EmptyCollection);
    assert_eq!(error.missing().unwrap().source_type(), Some(DataType::Int32));
    let values = MultiValues::String(vec![" ".to_owned()]);
    for error in [
        values.to_first_in::<i64>(&mut session).unwrap_err(),
        values.to_list_in::<i64>(&mut session).unwrap_err(),
    ] {
        assert_eq!(error.missing().unwrap().source_index(), Some(0));
        assert!(!error.missing().unwrap().is_defaultable_for_conversion());
    }
}

#[test]
fn test_missing_strict_mismatch_and_invalid_conversion_never_default() {
    let unset = Value::Unset(DataType::String);
    assert!(matches!(unset.get_or::<i32>(7), Err(ValueError::TypeMismatch { .. })));
    for value in [Value::from("invalid"), Value::from("999999999999999999999999999999")] {
        let mut called = false;
        assert!(
            value
                .to_or_else::<i32, _>(|| {
                    called = true;
                    7
                })
                .is_err()
        );
        assert!(!called);
    }
}

#[test]
fn test_missing_enrichment_preserves_admission_error_priority() {
    let policy = missing_policy();
    for (limit, missing) in [(0, false), (1, true), (2, true)] {
        let limits = ConversionLimits::builder()
            .operation_limits(ConversionOperationLimits::builder().max_input_bytes(limit).build())
            .build();
        let error = Value::from(" ").to_with::<i32>(&policy, &limits).unwrap_err();
        assert_eq!(error.is_missing(), missing);
        let mut called = false;
        let result = Value::from(" ").to_or_else_with::<i32, _>(
            || {
                called = true;
                7
            },
            &policy,
            &limits,
        );
        assert_eq!(called, missing);
        assert_eq!(result.is_ok(), missing);
    }
}

#[test]
fn test_missing_storage_constructors_and_display_preserve_facts() {
    for (missing, reason) in [
        (
            ValueMissing::unset_scalar(DataType::Bool, DataType::Bool),
            ValueMissingReason::UnsetScalar,
        ),
        (
            ValueMissing::unset_collection(DataType::String, DataType::String),
            ValueMissingReason::UnsetCollection,
        ),
        (
            ValueMissing::empty_collection(DataType::Int32, DataType::Int64),
            ValueMissingReason::EmptyCollection,
        ),
    ] {
        assert_eq!(missing.reason(), reason);
        assert!(missing.source_type().is_some());
        assert!(missing.target_type().is_some());
        assert_eq!(missing.source_index(), None);
        assert!(!missing.is_conversion());
        assert!(!missing.to_string().is_empty());
        assert_eq!(missing.clone(), missing);
    }
}
