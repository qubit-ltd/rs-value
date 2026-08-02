// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
//
//        https://www.apache.org/licenses/LICENSE-2.0
//
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
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
