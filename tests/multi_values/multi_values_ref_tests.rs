// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_value::{
    MultiValues,
    MultiValuesRef,
};

#[test]
fn test_multi_values_ref_borrows_collection_payload_without_changing_it() {
    let values = MultiValues::Int32(vec![1, 2, 3]);

    assert!(matches!(
        values.view(),
        MultiValuesRef::Int32(items) if items == [1, 2, 3]
    ));
    assert_eq!(values.get_int32s().expect("read integers"), &[1, 2, 3]);
}
