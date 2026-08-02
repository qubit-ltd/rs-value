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

use std::collections::HashMap;

use qubit_redact::{
    Redact as _,
    RedactionPolicy,
    Sensitivity,
};
use qubit_value::Value;

#[test]
fn test_rs_metadata_feature_profile_masks_sensitive_map_values() {
    let value = Value::StringMap(HashMap::from([
        ("api_key".to_owned(), "raw-secret".to_owned()),
        ("label".to_owned(), "visible".to_owned()),
    ]));
    let policy = RedactionPolicy::builder()
        .raise("api_key", Sensitivity::Secret)
        .expect("the test builder input should be valid")
        .build()
        .expect("policy should build");

    let output = format!("{:?}", value.redacted_with(&policy));

    assert!(!output.contains("raw-secret"));
    assert!(output.contains("visible"));
}

#[test]
fn test_rs_metadata_feature_profile_keeps_converter_api_available() {
    assert_eq!(
        Value::from("42")
            .to::<i32>()
            .expect("convert metadata value"),
        42
    );
}
