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
fn value_missing_accessors_preserve_conversion_context() {
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
fn empty_collection_conversion_exposes_target_context() {
    let missing = ValueMissing::EmptyCollectionConversion {
        to: DataType::Int32,
    };

    assert_eq!(missing.source_type(), None);
    assert_eq!(missing.target_type(), Some(DataType::Int32));
    assert!(missing.is_empty_collection());
    assert!(missing.is_conversion());
}

#[test]
fn value_missing_accessors_cover_all_variants_and_display() {
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
        ValueMissing::EmptyCollectionConversion {
            to: DataType::UInt64,
        },
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
