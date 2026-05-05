/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

use bigdecimal::BigDecimal;
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::Value;
use std::str::FromStr;
use url::Url;

#[test]
fn test_value_constructor_handles_owned_reference_and_special_types() {
    assert_eq!(Value::new("name").get_string().unwrap(), "name");
    assert_eq!(
        Value::new(BigInt::from(123)).get_biginteger_ref().unwrap(),
        &BigInt::from(123)
    );
    assert_eq!(
        Value::new(BigDecimal::from_str("12.50").unwrap())
            .get_bigdecimal_ref()
            .unwrap(),
        &BigDecimal::from_str("12.50").unwrap()
    );

    let url = Url::parse("https://example.com/a").unwrap();
    assert_eq!(Value::new(url.clone()).get_url_ref().unwrap(), &url);
    assert_eq!(Value::new(url).data_type(), DataType::Url);
}
