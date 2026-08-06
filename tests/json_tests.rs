// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection behavior.

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_scalar() {
    use qubit_value::Value;

    assert_eq!(
        Value::Int32(42).to_json_value().expect("project scalar"),
        serde_json::json!(42),
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_every_scalar_variant() {
    use qubit_value::Value;

    macro_rules! assert_scalar {
        ($value:expr, $expected:expr) => {
            assert_eq!(
                $value.to_json_value().expect("project scalar"),
                $expected
            );
        };
    }

    assert_scalar!(Value::Bool(true), serde_json::json!(true));
    assert_scalar!(Value::Char('a'), serde_json::json!("a"));
    assert_scalar!(Value::Int8(-1), serde_json::json!(-1));
    assert_scalar!(Value::Int16(-1), serde_json::json!(-1));
    assert_scalar!(Value::Int32(-1), serde_json::json!(-1));
    assert_scalar!(Value::Int64(-1), serde_json::json!(-1));
    assert_scalar!(Value::Int128(-1), serde_json::json!("-1"));
    assert_scalar!(Value::UInt8(1), serde_json::json!(1));
    assert_scalar!(Value::UInt16(1), serde_json::json!(1));
    assert_scalar!(Value::UInt32(1), serde_json::json!(1));
    assert_scalar!(Value::UInt64(1), serde_json::json!(1));
    assert_scalar!(Value::UInt128(1), serde_json::json!("1"));
    assert_scalar!(Value::Float64(1.5), serde_json::json!(1.5));
    assert!(Value::Float32(f32::NAN).to_json_value().is_err());
    assert_scalar!(
        Value::String("text".to_string()),
        serde_json::json!("text")
    );
    assert_scalar!(
        Value::Duration(std::time::Duration::from_secs(1)),
        serde_json::json!("1000ms")
    );
    assert_scalar!(
        Value::Date(chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        serde_json::json!("2025-01-01")
    );
    assert_scalar!(
        Value::Time(chrono::NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
        serde_json::json!("01:02:03")
    );
    let datetime = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    assert_scalar!(
        Value::DateTime(datetime),
        serde_json::json!("2025-01-01 01:02:03")
    );
    let instant = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        datetime,
        chrono::Utc,
    );
    assert_scalar!(
        Value::Instant(instant),
        serde_json::json!("2025-01-01 01:02:03 UTC")
    );
    assert_scalar!(
        Value::BigInteger(num_bigint::BigInt::from(7)),
        serde_json::json!("7")
    );
    assert_scalar!(
        Value::BigDecimal("7.5".parse::<bigdecimal::BigDecimal>().unwrap()),
        serde_json::json!("7.5")
    );
    assert_scalar!(
        Value::Url(url::Url::parse("https://example.com").unwrap()),
        serde_json::json!("https://example.com/")
    );
    assert_scalar!(
        Value::Json(serde_json::json!({"z": 1, "a": 2})),
        serde_json::json!({"a": 2, "z": 1})
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_reports_collection_and_float_projection_errors() {
    use qubit_value::{
        MultiValues,
        Value,
        ValueContainer,
    };

    assert!(matches!(
        MultiValues::Float32(vec![f32::NAN]).to_json_value(),
        Err(qubit_value::ValueError::ListConversion(_))
    ));
    assert!(matches!(
        MultiValues::Float64(vec![f64::INFINITY]).to_json_value(),
        Err(qubit_value::ValueError::ListConversion(_))
    ));
    assert_eq!(
        ValueContainer::Collection(MultiValues::Int32(vec![1, 2]))
            .to_json_value()
            .expect("project collection container"),
        serde_json::json!([1, 2])
    );
    assert_eq!(
        ValueContainer::Scalar(Value::Unset(qubit_datatype::DataType::Json))
            .to_json_value()
            .expect("project unset container"),
        serde_json::Value::Null
    );

    let value_project: fn(&Value) -> _ = Value::to_json_value;
    assert_eq!(
        value_project(&Value::Int32(3)).unwrap(),
        serde_json::json!(3)
    );
    let container_project: fn(&ValueContainer) -> _ =
        ValueContainer::to_json_value;
    assert_eq!(
        container_project(&ValueContainer::Scalar(Value::Int32(4))).unwrap(),
        serde_json::json!(4)
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_float32_with_display_roundtrip() {
    use qubit_value::Value;
    use serde_json::Number;

    for bits in [
        0xC65B_9806_u32, // -14054.006
        0x4823_0AF3_u32, // 166955.8
        0x9CA9_7CE0_u32, // 0.000000000000000000000000000004358592
        0x4078_7ACD_u32, // 3.8824952
        0x2696_F5F4_u32, // 0.000000000000001047500658
    ] {
        let value = f32::from_bits(bits);
        let projected = Value::Float32(value)
            .to_json_value()
            .expect("project float32");
        let projected_text =
            serde_json::to_string(&projected).expect("serialize json");

        let legacy_text = serde_json::to_string(&serde_json::Value::Number(
            Number::from_f64(f64::from(value)).expect("finite f64"),
        ))
        .expect("legacy serialize json");

        assert_eq!(
            projected_text,
            value.to_string(),
            "natural json should preserve f32 display text",
        );
        assert_ne!(
            projected_text, legacy_text,
            "this sample should differ from f32->f64 cast path",
        );
    }
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_string_map_keys_in_dictionary_order() {
    use std::collections::HashMap;

    use qubit_value::Value;

    let map = HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
        ("m".to_owned(), "13".to_owned()),
    ]);
    let projected = Value::StringMap(map)
        .to_json_value()
        .expect("project string map");

    assert_eq!(
        serde_json::to_string(&projected).expect("serialize projected map"),
        r#"{"a":"1","m":"13","z":"26"}"#,
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_canonicalizes_nested_json_object_keys() {
    use qubit_value::Value;

    let value = Value::Json(
        serde_json::from_str(r#"{"z":{"b":1,"a":2},"a":0}"#)
            .expect("parse JSON value"),
    );
    let projected = value.to_json_value().expect("project JSON value");

    assert_eq!(
        serde_json::to_string(&projected).expect("serialize projected JSON"),
        r#"{"a":0,"z":{"a":2,"b":1}}"#,
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_every_collection_variant() {
    use qubit_value::MultiValues;

    macro_rules! assert_collection {
        ($values:expr, $expected:expr) => {
            assert_eq!(
                $values.to_json_value().expect("project collection"),
                $expected
            );
        };
    }

    assert_collection!(
        MultiValues::Bool(vec![true, false]),
        serde_json::json!([true, false])
    );
    assert_collection!(
        MultiValues::Char(vec!['a', 'b']),
        serde_json::json!(["a", "b"])
    );
    assert_collection!(
        MultiValues::Int8(vec![-1, 2]),
        serde_json::json!([-1, 2])
    );
    assert_collection!(
        MultiValues::Int16(vec![-1, 2]),
        serde_json::json!([-1, 2])
    );
    assert_collection!(
        MultiValues::Int32(vec![-1, 2]),
        serde_json::json!([-1, 2])
    );
    assert_collection!(
        MultiValues::Int64(vec![-1, 2]),
        serde_json::json!([-1, 2])
    );
    assert_collection!(
        MultiValues::Int128(vec![-1, 2]),
        serde_json::json!(["-1", "2"])
    );
    assert_collection!(
        MultiValues::UInt8(vec![1, 2]),
        serde_json::json!([1, 2])
    );
    assert_collection!(
        MultiValues::UInt16(vec![1, 2]),
        serde_json::json!([1, 2])
    );
    assert_collection!(
        MultiValues::UInt32(vec![1, 2]),
        serde_json::json!([1, 2])
    );
    assert_collection!(
        MultiValues::UInt64(vec![1, 2]),
        serde_json::json!([1, 2])
    );
    assert_collection!(
        MultiValues::UInt128(vec![1, 2]),
        serde_json::json!(["1", "2"])
    );
    assert_collection!(
        MultiValues::Float32(vec![1.5, 2.5]),
        serde_json::json!([1.5, 2.5])
    );
    assert_collection!(
        MultiValues::Float64(vec![1.5, 2.5]),
        serde_json::json!([1.5, 2.5])
    );
    assert_collection!(
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
        serde_json::json!(["a", "b"])
    );
    assert_collection!(
        MultiValues::Date(vec![
            chrono::NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()
        ]),
        serde_json::json!(["2025-01-01"])
    );
    assert_collection!(
        MultiValues::Time(vec![
            chrono::NaiveTime::from_hms_opt(1, 2, 3).unwrap()
        ]),
        serde_json::json!(["01:02:03"])
    );
    let datetime = chrono::NaiveDate::from_ymd_opt(2025, 1, 1)
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    assert_collection!(
        MultiValues::DateTime(vec![datetime]),
        serde_json::json!(["2025-01-01 01:02:03"])
    );
    let instant = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
        datetime,
        chrono::Utc,
    );
    assert_collection!(
        MultiValues::Instant(vec![instant]),
        serde_json::json!(["2025-01-01 01:02:03 UTC"])
    );
    assert_collection!(
        MultiValues::BigInteger(vec![num_bigint::BigInt::from(7)]),
        serde_json::json!(["7"])
    );
    assert_collection!(
        MultiValues::BigDecimal(vec![
            "7.5".parse::<bigdecimal::BigDecimal>().unwrap()
        ]),
        serde_json::json!(["7.5"])
    );
    assert_collection!(
        MultiValues::Duration(vec![std::time::Duration::from_secs(1)]),
        serde_json::json!(["1000ms"])
    );
    assert_collection!(
        MultiValues::Url(vec![url::Url::parse("https://example.com").unwrap()]),
        serde_json::json!(["https://example.com/"])
    );
    assert_collection!(
        MultiValues::StringMap(vec![std::collections::HashMap::from([(
            "key".to_string(),
            "value".to_string()
        ),])]),
        serde_json::json!([{"key": "value"}])
    );
    assert_collection!(
        MultiValues::Json(vec![serde_json::json!({"z": 1, "a": 2})]),
        serde_json::json!([{ "a": 2, "z": 1 }])
    );
}
