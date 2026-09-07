// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::DataType;
use qubit_value::ValueMissing;

#[test]
fn test_value_missing_predicates_distinguish_storage_empty_and_conversion() {
    let cases = [
        (
            ValueMissing::UnsetScalar {
                data_type: DataType::Bool,
            },
            true,
            false,
            false,
        ),
        (
            ValueMissing::UnsetCollection {
                data_type: DataType::String,
            },
            true,
            false,
            false,
        ),
        (
            ValueMissing::EmptyCollection {
                data_type: DataType::Int32,
            },
            false,
            true,
            false,
        ),
        (
            ValueMissing::EmptyCollectionConversion { to: DataType::UInt64 },
            false,
            true,
            true,
        ),
        (
            ValueMissing::Conversion {
                from: DataType::String,
                to: DataType::Int32,
            },
            false,
            false,
            true,
        ),
        (
            ValueMissing::CollectionItem {
                source_index: 3,
                from: DataType::String,
                to: DataType::Int32,
            },
            false,
            false,
            true,
        ),
    ];

    for (missing, is_unset, is_empty_collection, is_conversion) in cases {
        assert_eq!(missing.is_unset(), is_unset);
        assert_eq!(missing.is_empty_collection(), is_empty_collection);
        assert_eq!(missing.is_conversion(), is_conversion);
    }
}

#[test]
fn test_value_missing_display_includes_structured_context() {
    assert_eq!(
        ValueMissing::UnsetScalar {
            data_type: DataType::Bool,
        }
        .to_string(),
        "unset scalar with declared type bool"
    );
    assert_eq!(
        ValueMissing::UnsetCollection {
            data_type: DataType::String,
        }
        .to_string(),
        "unset collection with declared type string"
    );
    assert_eq!(
        ValueMissing::EmptyCollection {
            data_type: DataType::Int32,
        }
        .to_string(),
        "empty collection with element type int32"
    );
    assert_eq!(
        ValueMissing::EmptyCollectionConversion { to: DataType::UInt64 }.to_string(),
        "empty collection conversion to uint64 produced no value"
    );
    assert_eq!(
        ValueMissing::Conversion {
            from: DataType::String,
            to: DataType::Int32,
        }
        .to_string(),
        "conversion from string to int32 produced no value"
    );
    assert_eq!(
        ValueMissing::CollectionItem {
            source_index: 3,
            from: DataType::String,
            to: DataType::Int32,
        }
        .to_string(),
        "collection item at index 3 conversion from string to int32 produced no value"
    );
}

#[test]
fn test_value_missing_accessors_preserve_conversion_context() {
    let missing = ValueMissing::CollectionItem {
        source_index: 2,
        from: DataType::String,
        to: DataType::Int64,
    };

    assert_eq!(missing.source_type(), Some(DataType::String));
    assert_eq!(missing.target_type(), Some(DataType::Int64));
    assert_eq!(missing.source_index(), Some(2));
    assert!(missing.is_conversion());
}

#[test]
fn test_empty_collection_conversion_exposes_target_context() {
    let missing = ValueMissing::EmptyCollectionConversion { to: DataType::Int32 };

    assert_eq!(missing.source_type(), None);
    assert_eq!(missing.target_type(), Some(DataType::Int32));
    assert!(missing.is_empty_collection());
    assert!(missing.is_conversion());
}

#[test]
fn test_value_missing_accessors_cover_all_variants_and_display() {
    let variants = [
        ValueMissing::UnsetScalar {
            data_type: DataType::Bool,
        },
        ValueMissing::UnsetCollection {
            data_type: DataType::String,
        },
        ValueMissing::EmptyCollection {
            data_type: DataType::Int32,
        },
        ValueMissing::EmptyCollectionConversion { to: DataType::UInt64 },
        ValueMissing::Conversion {
            from: DataType::String,
            to: DataType::Int32,
        },
        ValueMissing::CollectionItem {
            source_index: 3,
            from: DataType::String,
            to: DataType::Int32,
        },
    ];

    for missing in variants {
        let _ = missing.to_string();
        let _ = missing.source_type();
        let _ = missing.target_type();
        let _ = missing.source_index();
        let _ = missing.is_unset();
        let _ = missing.is_empty_collection();
        let _ = missing.is_conversion();
    }
}
