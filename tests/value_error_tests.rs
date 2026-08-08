// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use std::error::Error;

use qubit_datatype::DataConversionError;
use qubit_datatype::DataListConversionError;
use qubit_datatype::DataType;
use qubit_datatype::InvalidValueReason;
use qubit_value::ValueError;
use qubit_value::ValueMissing;

#[test]
fn test_missing_error_exposes_default_semantics() {
    let error = ValueError::Missing(ValueMissing::Conversion {
        from: DataType::String,
        to: DataType::Int32,
    });

    assert!(error.is_missing());
    assert_eq!(
        error.missing(),
        Some(&ValueMissing::Conversion {
            from: DataType::String,
            to: DataType::Int32,
        })
    );
    assert!(error.missing().is_some_and(|missing| {
        missing.is_unset() || matches!(missing, ValueMissing::Conversion { .. })
    }));
}

#[test]
fn test_value_error_display_includes_context() {
    let mismatch = ValueError::TypeMismatch {
        expected: DataType::String,
        actual: DataType::Int32,
    };
    assert_eq!(
        mismatch.to_string(),
        "Type mismatch: expected string, actual int32"
    );
}

#[test]
fn test_value_error_variants_compare_by_payload() {
    assert_eq!(
        ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::String,
        }),
        ValueError::Missing(ValueMissing::UnsetScalar {
            data_type: DataType::String,
        }),
    );
    let source = DataConversionError::invalid(
        DataType::String,
        DataType::Int32,
        InvalidValueReason::OutOfRange,
    );
    let single = ValueError::Conversion(source.clone());
    assert_eq!(
        single.source().and_then(|error| error.downcast_ref()),
        Some(&source),
    );

    let list_source = DataListConversionError::new(2, source);
    let list = ValueError::ListConversion(list_source.clone());
    assert_eq!(
        list.source().and_then(|error| error.downcast_ref()),
        Some(&list_source),
    );
}

#[test]
fn test_value_missing_preserves_shape_state_and_declared_type() {
    let scalar = ValueMissing::UnsetScalar {
        data_type: DataType::Int32,
    };
    let collection = ValueMissing::UnsetCollection {
        data_type: DataType::String,
    };
    let empty = ValueMissing::EmptyCollection {
        data_type: DataType::UInt64,
    };

    assert_eq!(scalar.source_type(), Some(DataType::Int32));
    assert!(scalar.is_unset());
    assert!(!scalar.is_empty_collection());
    assert_eq!(collection.source_type(), Some(DataType::String));
    assert!(collection.is_unset());
    assert!(!collection.is_empty_collection());
    assert_eq!(empty.source_type(), Some(DataType::UInt64));
    assert!(!empty.is_unset());
    assert!(empty.is_empty_collection());
}

#[test]
fn test_value_missing_preserves_collection_item_context() {
    let missing = ValueMissing::CollectionItem {
        source_index: 4,
        from: DataType::String,
        to: DataType::Int32,
    };

    assert_eq!(missing.source_index(), Some(4));
    assert_eq!(missing.source_type(), Some(DataType::String));
    assert_eq!(missing.target_type(), Some(DataType::Int32));
    assert!(missing.is_conversion());
}

#[test]
fn test_conversion_missing_errors_are_promoted_to_value_missing() {
    let scalar = ValueError::from(DataConversionError::missing(
        DataType::String,
        DataType::Int32,
    ));
    assert!(matches!(
        scalar,
        ValueError::Missing(ValueMissing::Conversion {
            from: DataType::String,
            to: DataType::Int32,
        })
    ));

    let empty = ValueError::from(DataConversionError::empty_collection(
        DataType::Int32,
    ));
    assert!(matches!(
        empty,
        ValueError::Missing(ValueMissing::EmptyCollectionConversion {
            to: DataType::Int32,
        })
    ));

    let list = ValueError::from(DataListConversionError::new(
        3,
        DataConversionError::missing(DataType::String, DataType::Int32),
    ));
    assert!(matches!(
        list,
        ValueError::Missing(ValueMissing::CollectionItem {
            source_index: 3,
            from: DataType::String,
            to: DataType::Int32,
        })
    ));
}

#[test]
fn test_value_error_clone_preserves_structured_source() {
    let source = DataConversionError::invalid(
        DataType::String,
        DataType::Int32,
        InvalidValueReason::OutOfRange,
    );
    let error =
        ValueError::ListConversion(DataListConversionError::new(3, source));

    assert_eq!(error.clone(), error);
}

#[test]
fn test_value_error_accessors_cover_non_missing_variants() {
    let mismatch = ValueError::TypeMismatch {
        expected: DataType::String,
        actual: DataType::Int32,
    };
    assert!(!mismatch.is_missing());
    assert_eq!(mismatch.missing(), None);

    let conversion = ValueError::Conversion(DataConversionError::invalid(
        DataType::String,
        DataType::Int32,
        InvalidValueReason::OutOfRange,
    ));
    assert!(!conversion.is_missing());
    assert_eq!(conversion.missing(), None);

    let list = ValueError::ListConversion(DataListConversionError::new(
        0,
        DataConversionError::invalid(
            DataType::String,
            DataType::Int32,
            InvalidValueReason::OutOfRange,
        ),
    ));
    assert!(!list.is_missing());
    assert_eq!(list.missing(), None);
}
