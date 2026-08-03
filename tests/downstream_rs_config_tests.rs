// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use qubit_datatype::{
    DataConversionError,
    DataConversionOptions,
    DataConversionTarget,
    DataType,
    DataTypeOf,
};
use qubit_value::{
    MultiValues,
    Value,
    ValueWireV1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Port(u16);

impl DataTypeOf for Port {
    const DATA_TYPE: DataType = DataType::UInt16;
}

impl DataConversionTarget for Port {
    fn convert_from(
        source: &qubit_datatype::DataConverter<'_>,
        options: &DataConversionOptions,
    ) -> Result<Self, DataConversionError> {
        u16::convert_from(source, options).map(Self)
    }
}

#[test]
fn test_rs_config_feature_profile_accepts_downstream_converter_targets() {
    assert_eq!(
        Value::from("8080").to::<Port>().expect("convert port"),
        Port(8080)
    );
    assert_eq!(
        MultiValues::from(vec!["8080", "8081"])
            .to_list::<Port>()
            .expect("convert ports"),
        vec![Port(8080), Port(8081)]
    );
}

#[test]
fn test_rs_config_feature_profile_preserves_json_values() {
    let value = Value::Json(serde_json::json!({"answer": 42}));
    let wire = ValueWireV1::try_from(value).expect("construct JSON wire value");
    let encoded = serde_json::to_string(&wire).expect("serialize JSON value");

    assert!(encoded.contains("answer"));
    assert!(encoded.contains("42"));
}
