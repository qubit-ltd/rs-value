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

    assert_eq!(missing.data_type(), DataType::String);
    assert_eq!(missing.target_type(), Some(DataType::Int64));
    assert_eq!(missing.source_index(), Some(2));
    assert!(missing.is_conversion());
    assert!(!missing.uses_default());
}

#[test]
fn empty_collection_conversion_exposes_target_context() {
    let missing = ValueMissing::EmptyCollectionConversion {
        to: DataType::Int32,
    };

    assert_eq!(missing.data_type(), DataType::Int32);
    assert_eq!(missing.target_type(), Some(DataType::Int32));
    assert!(missing.is_empty_collection());
    assert!(missing.is_conversion());
    assert!(!missing.uses_default());
}
