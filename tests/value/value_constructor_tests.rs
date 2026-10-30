// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

#[cfg(feature = "big-decimal")]
use std::str::FromStr;

#[cfg(feature = "big-decimal")]
use bigdecimal::BigDecimal;
#[cfg(feature = "big-integer")]
use num_bigint::BigInt;
use qubit_value::Value;
#[cfg(feature = "url")]
use url::Url;

#[test]
fn test_value_constructor_handles_owned_reference_and_special_types() {
    assert_eq!(
        Value::new("name")
            .get_string()
            .expect("constructed string should be readable"),
        "name"
    );
    #[cfg(feature = "big-integer")]
    assert_eq!(
        Value::new(BigInt::from(123))
            .get_biginteger_ref()
            .expect("constructed big integer should be readable"),
        &BigInt::from(123)
    );
    #[cfg(feature = "big-decimal")]
    assert_eq!(
        Value::new(BigDecimal::from_str("12.50").expect("test decimal should parse"))
            .get_bigdecimal_ref()
            .expect("constructed big decimal should be readable"),
        &BigDecimal::from_str("12.50").expect("test decimal should parse")
    );

    #[cfg(feature = "url")]
    {
        use qubit_datatype::DataType;

        let url = Url::parse("https://example.com/a").expect("test URL should parse");
        assert_eq!(
            Value::new(url.clone())
                .get_url_ref()
                .expect("constructed URL should be readable"),
            &url
        );
        assert_eq!(Value::new(url).data_type(), DataType::Url);
    }
}
