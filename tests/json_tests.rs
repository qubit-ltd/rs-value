// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Natural JSON projection behavior.

#[cfg(all(feature = "converter", feature = "json"))]
use std::collections::HashMap;
#[cfg(all(feature = "converter", feature = "json"))]
use std::time::Duration;

#[cfg(all(feature = "converter", feature = "json"))]
use bigdecimal::BigDecimal;
#[cfg(all(feature = "converter", feature = "json"))]
use chrono::DateTime;
#[cfg(all(feature = "converter", feature = "json"))]
use chrono::NaiveDate;
#[cfg(all(feature = "converter", feature = "json"))]
use chrono::NaiveTime;
#[cfg(all(feature = "converter", feature = "json"))]
use chrono::Utc;
#[cfg(all(feature = "converter", feature = "json"))]
use num_bigint::BigInt;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_datatype::DataType;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::MultiValues;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::Value;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::ValueContainer;
#[cfg(all(feature = "converter", feature = "json"))]
use qubit_value::ValueError;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::Number;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::Value as JsonValue;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::from_str;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::json;
#[cfg(all(feature = "converter", feature = "json"))]
use serde_json::to_string;

/// Projection budgets cover the whole collection, including cheap scalar paths.
#[test]
fn test_natural_json_enforces_cumulative_projection_limits() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_output_bytes(5).build())
        .build();
    let values = MultiValues::String(vec!["abc".into(), "def".into()]);
    let error = values
        .to_json_value_with(&policy, &limits)
        .expect_err("cumulative output exceeds five bytes");
    assert!(matches!(
        error,
        ValueError::JsonProjectionLimit {
            source_index: Some(1),
            ..
        }
    ));
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_items(0).build())
        .build();
    assert!(Value::Int32(1).to_json_value_with(&policy, &limits).is_err());
}

/// Float budgets measure the projected JSON representation at exact boundaries.
#[test]
fn test_natural_json_float_budget_matches_projected_number() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    let policy = ConversionPolicy::default();
    for value in [
        Value::Float64(1.0),
        Value::Float64(-0.0),
        Value::Float64(1e-30),
        Value::Float32(1.0),
        Value::Float32(1e-30),
    ] {
        let projected = value.to_json_value().expect("finite float projection");
        let bytes = projected.to_string().len() as u64;
        let limits = ConversionLimits::builder()
            .operation_limits(ConversionOperationLimits::builder().max_output_bytes(bytes).build())
            .build();
        assert_eq!(value.to_json_value_with(&policy, &limits).unwrap(), projected);
        let limits = ConversionLimits::builder()
            .operation_limits(ConversionOperationLimits::builder().max_output_bytes(bytes - 1).build())
            .build();
        assert!(value.to_json_value_with(&policy, &limits).is_err(), "{projected}");
    }
}

/// JSON and map payloads obey the same structural limits as projected lists.
#[test]
fn test_natural_json_enforces_structure_before_materializing() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::StructuredConversionLimits;
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::builder()
        .structured_limits(
            StructuredConversionLimits::builder()
                .max_depth(2)
                .max_map_entries(1)
                .max_sequence_items(1)
                .build(),
        )
        .build();
    assert!(Value::Json(json!([[0]])).to_json_value_with(&policy, &limits).is_err());
    assert!(
        Value::StringMap(HashMap::from([("a".into(), "1".into()), ("b".into(), "2".into())]))
            .to_json_value_with(&policy, &limits)
            .is_err()
    );
    assert!(
        MultiValues::Int32(vec![1, 2])
            .to_json_value_with(&policy, &limits)
            .is_err()
    );
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_structured_nodes(1).build())
        .build();
    assert!(
        MultiValues::Int32(vec![1])
            .to_json_value_with(&policy, &limits)
            .is_err()
    );
}

/// Duration formatting shares cumulative output capacity with every list item.
#[test]
fn test_natural_json_bounds_duration_and_wide_number_text() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_output_bytes(6).build())
        .build();
    assert!(
        MultiValues::Duration(vec![Duration::from_secs(1); 2])
            .to_json_value_with(&policy, &limits)
            .is_err()
    );
    assert!(Value::Int128(i128::MAX).to_json_value_with(&policy, &limits).is_err());
    assert!(
        Value::BigDecimal("123456789.123".parse().expect("decimal"))
            .to_json_value_with(&policy, &limits)
            .is_err()
    );
}
#[cfg(all(feature = "converter", feature = "json"))]
use url::Url;

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_scalar() {
    assert_eq!(Value::Int32(42).to_json_value().expect("project scalar"), json!(42),);
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_every_scalar_variant() {
    macro_rules! assert_scalar {
        ($value:expr, $expected:expr) => {
            assert_eq!($value.to_json_value().expect("project scalar"), $expected);
        };
    }

    assert_scalar!(Value::Bool(true), json!(true));
    assert_scalar!(Value::Char('a'), json!("a"));
    assert_scalar!(Value::Int8(-1), json!(-1));
    assert_scalar!(Value::Int16(-1), json!(-1));
    assert_scalar!(Value::Int32(-1), json!(-1));
    assert_scalar!(Value::Int64(-1), json!(-1));
    assert_scalar!(Value::Int128(-1), json!("-1"));
    assert_scalar!(Value::UInt8(1), json!(1));
    assert_scalar!(Value::UInt16(1), json!(1));
    assert_scalar!(Value::UInt32(1), json!(1));
    assert_scalar!(Value::UInt64(1), json!(1));
    assert_scalar!(Value::UInt128(1), json!("1"));
    assert_scalar!(Value::Float64(1.5), json!(1.5));
    assert!(Value::Float32(f32::NAN).to_json_value().is_err());
    assert_scalar!(Value::String("text".to_string()), json!("text"));
    assert_scalar!(Value::Duration(Duration::from_secs(1)), json!("1000ms"));
    assert_scalar!(
        Value::Date(NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()),
        json!("2025-01-01")
    );
    assert_scalar!(
        Value::Time(NaiveTime::from_hms_opt(1, 2, 3).unwrap()),
        json!("01:02:03")
    );
    let datetime = NaiveDate::from_ymd_opt(2025, 1, 1)
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    assert_scalar!(Value::DateTime(datetime), json!("2025-01-01 01:02:03"));
    let instant = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
    assert_scalar!(Value::Instant(instant), json!("2025-01-01 01:02:03 UTC"));
    assert_scalar!(Value::BigInteger(BigInt::from(7)), json!("7"));
    assert_scalar!(Value::BigDecimal("7.5".parse::<BigDecimal>().unwrap()), json!("7.5"));
    assert_scalar!(
        Value::Url(Url::parse("https://example.com").unwrap()),
        json!("https://example.com/")
    );
    assert_scalar!(Value::Json(json!({"z": 1, "a": 2})), json!({"a": 2, "z": 1}));
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_reports_collection_and_float_projection_errors() {
    assert!(matches!(
        MultiValues::Float32(vec![f32::NAN]).to_json_value(),
        Err(ValueError::ListConversion(_))
    ));
    assert!(matches!(
        MultiValues::Float64(vec![f64::INFINITY]).to_json_value(),
        Err(ValueError::ListConversion(_))
    ));
    assert_eq!(
        ValueContainer::Collection(MultiValues::Int32(vec![1, 2]))
            .to_json_value()
            .expect("project collection container"),
        json!([1, 2])
    );
    assert_eq!(
        ValueContainer::Scalar(Value::Unset(DataType::Json))
            .to_json_value()
            .expect("project unset container"),
        JsonValue::Null
    );

    let value_project: fn(&Value) -> _ = Value::to_json_value;
    assert_eq!(value_project(&Value::Int32(3)).unwrap(), json!(3));
    let container_project: fn(&ValueContainer) -> _ = ValueContainer::to_json_value;
    assert_eq!(
        container_project(&ValueContainer::Scalar(Value::Int32(4))).unwrap(),
        json!(4)
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_float32_without_widening() {
    for bits in [
        0xC65B_9806_u32, // -14054.006
        0x4823_0AF3_u32, // 166955.8
        0x9CA9_7CE0_u32, // 0.000000000000000000000000000004358592
        0x4078_7ACD_u32, // 3.8824952
        0x2696_F5F4_u32, // 0.000000000000001047500658
    ] {
        let value = f32::from_bits(bits);
        let projected = Value::Float32(value).to_json_value().expect("project float32");
        let projected_text = to_string(&projected).expect("serialize json");

        let legacy_text = to_string(&JsonValue::Number(
            Number::from_f64(f64::from(value)).expect("finite f64"),
        ))
        .expect("legacy serialize json");

        assert_eq!(
            from_str::<f32>(&projected_text).expect("decode projected float32"),
            value,
            "natural JSON should round-trip the original f32 value",
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
    let map = HashMap::from([
        ("z".to_owned(), "26".to_owned()),
        ("a".to_owned(), "1".to_owned()),
        ("m".to_owned(), "13".to_owned()),
    ]);
    let projected = Value::StringMap(map).to_json_value().expect("project string map");

    assert_eq!(
        to_string(&projected).expect("serialize projected map"),
        r#"{"a":"1","m":"13","z":"26"}"#,
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_canonicalizes_nested_json_object_keys() {
    let value = Value::Json(from_str(r#"{"z":{"b":1,"a":2},"a":0}"#).expect("parse JSON value"));
    let projected = value.to_json_value().expect("project JSON value");

    assert_eq!(
        to_string(&projected).expect("serialize projected JSON"),
        r#"{"a":0,"z":{"a":2,"b":1}}"#,
    );
}

#[cfg(all(feature = "converter", feature = "json"))]
#[test]
fn test_natural_json_projects_every_collection_variant() {
    macro_rules! assert_collection {
        ($values:expr, $expected:expr) => {
            assert_eq!($values.to_json_value().expect("project collection"), $expected);
        };
    }

    assert_collection!(MultiValues::Bool(vec![true, false]), json!([true, false]));
    assert_collection!(MultiValues::Char(vec!['a', 'b']), json!(["a", "b"]));
    assert_collection!(MultiValues::Int8(vec![-1, 2]), json!([-1, 2]));
    assert_collection!(MultiValues::Int16(vec![-1, 2]), json!([-1, 2]));
    assert_collection!(MultiValues::Int32(vec![-1, 2]), json!([-1, 2]));
    assert_collection!(MultiValues::Int64(vec![-1, 2]), json!([-1, 2]));
    assert_collection!(MultiValues::Int128(vec![-1, 2]), json!(["-1", "2"]));
    assert_collection!(MultiValues::UInt8(vec![1, 2]), json!([1, 2]));
    assert_collection!(MultiValues::UInt16(vec![1, 2]), json!([1, 2]));
    assert_collection!(MultiValues::UInt32(vec![1, 2]), json!([1, 2]));
    assert_collection!(MultiValues::UInt64(vec![1, 2]), json!([1, 2]));
    assert_collection!(MultiValues::UInt128(vec![1, 2]), json!(["1", "2"]));
    assert_collection!(MultiValues::Float32(vec![1.5, 2.5]), json!([1.5, 2.5]));
    assert_collection!(MultiValues::Float64(vec![1.5, 2.5]), json!([1.5, 2.5]));
    assert_collection!(
        MultiValues::String(vec!["a".to_string(), "b".to_string()]),
        json!(["a", "b"])
    );
    assert_collection!(
        MultiValues::Date(vec![NaiveDate::from_ymd_opt(2025, 1, 1).unwrap()]),
        json!(["2025-01-01"])
    );
    assert_collection!(
        MultiValues::Time(vec![NaiveTime::from_hms_opt(1, 2, 3).unwrap()]),
        json!(["01:02:03"])
    );
    let datetime = NaiveDate::from_ymd_opt(2025, 1, 1)
        .unwrap()
        .and_hms_opt(1, 2, 3)
        .unwrap();
    assert_collection!(MultiValues::DateTime(vec![datetime]), json!(["2025-01-01 01:02:03"]));
    let instant = DateTime::<Utc>::from_naive_utc_and_offset(datetime, Utc);
    assert_collection!(MultiValues::Instant(vec![instant]), json!(["2025-01-01 01:02:03 UTC"]));
    assert_collection!(MultiValues::BigInteger(vec![BigInt::from(7)]), json!(["7"]));
    assert_collection!(
        MultiValues::BigDecimal(vec!["7.5".parse::<BigDecimal>().unwrap()]),
        json!(["7.5"])
    );
    assert_collection!(MultiValues::Duration(vec![Duration::from_secs(1)]), json!(["1000ms"]));
    assert_collection!(
        MultiValues::Url(vec![Url::parse("https://example.com").unwrap()]),
        json!(["https://example.com/"])
    );
    assert_collection!(
        MultiValues::StringMap(vec![HashMap::from([("key".to_string(), "value".to_string()),])]),
        json!([{"key": "value"}])
    );
    assert_collection!(
        MultiValues::Json(vec![json!({"z": 1, "a": 2})]),
        json!([{ "a": 2, "z": 1 }])
    );
}

/// Cumulative failures retain the original maximum and previously consumed
/// bytes.
#[test]
fn test_natural_json_limit_error_preserves_budget_facts() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::ConversionResource;
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_output_bytes(5).build())
        .build();
    let error = MultiValues::String(vec!["abc".into(), "def".into()])
        .to_json_value_with(&policy, &limits)
        .expect_err("second item exceeds output budget");
    assert_eq!(error, error.clone());
    assert!(!error.is_missing());
    assert!(error.missing().is_none());
    let ValueError::JsonProjectionLimit {
        data_type,
        source_index,
        source,
    } = error
    else {
        panic!("expected structured projection error");
    };
    assert_eq!(data_type, DataType::String);
    assert_eq!(source_index, Some(1));
    assert_eq!(*source.resource(), ConversionResource::OutputBytes);
    let budget = source.budget_error().expect("representable budget failure");
    assert_eq!(budget.configured_limit(), 5);
    assert_eq!(budget.used(), Some(3));
    assert_eq!(budget.remaining(), Some(2));
}

/// Input, keys, payload, big-number guards and empty/unset shapes remain
/// distinct.
#[test]
fn test_natural_json_projection_boundary_matrix() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::NumericConversionLimits;
    let policy = ConversionPolicy::default();
    let input = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_input_bytes(2).build())
        .build();
    for value in [
        Value::from("abc"),
        Value::Json(json!({"abc": 0})),
        Value::StringMap(HashMap::from([("a".into(), "bc".into())])),
    ] {
        assert!(matches!(
            value.to_json_value_with(&policy, &input),
            Err(ValueError::JsonProjectionLimit { .. })
        ));
    }
    let payload = ConversionLimits::builder()
        .operation_limits(
            ConversionOperationLimits::builder()
                .max_structured_payload_bytes(2)
                .build(),
        )
        .build();
    assert!(
        Value::Json(json!(["ab", 3]))
            .to_json_value_with(&policy, &payload)
            .is_err()
    );
    let numeric = ConversionLimits::builder()
        .numeric_limits(
            NumericConversionLimits::builder()
                .max_big_integer_digits(2)
                .max_big_decimal_scale_magnitude(2)
                .build(),
        )
        .build();
    assert!(
        Value::BigInteger(BigInt::from(1000))
            .to_json_value_with(&policy, &numeric)
            .is_err()
    );
    assert!(
        Value::BigDecimal("1e100".parse().expect("decimal"))
            .to_json_value_with(&policy, &numeric)
            .is_err()
    );
    let zero_items = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_items(0).build())
        .build();
    assert_eq!(
        MultiValues::Int32(vec![])
            .to_json_value_with(&policy, &zero_items)
            .expect("no scalar items"),
        json!([])
    );
    assert!(
        MultiValues::new_unset(DataType::Int32)
            .to_json_value_with(&policy, &zero_items)
            .is_err()
    );
}

/// Nested keys and string values consume the same operation-wide output budget.
#[test]
fn test_natural_json_keys_and_values_share_output_budget() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionOperationLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::ConversionResource;
    let policy = ConversionPolicy::default();
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_output_bytes(1).build())
        .build();
    for value in [
        Value::Json(json!({"ab": null})),
        Value::StringMap(HashMap::from([("ab".into(), "v".into())])),
    ] {
        let error = value
            .to_json_value_with(&policy, &limits)
            .expect_err("keys consume output bytes");
        let ValueError::JsonProjectionLimit { source, .. } = error else {
            panic!("expected a projection budget failure");
        };
        assert_eq!(*source.resource(), ConversionResource::OutputBytes);
        assert_eq!(source.budget_error().unwrap().configured_limit(), 1);
    }
    let limits = ConversionLimits::builder()
        .operation_limits(ConversionOperationLimits::builder().max_output_bytes(3).build())
        .build();
    let values = MultiValues::Json(vec![json!({"a": "b"}), json!({"c": "d"})]);
    let error = values
        .to_json_value_with(&policy, &limits)
        .expect_err("four bytes across two objects");
    let ValueError::JsonProjectionLimit {
        source_index, source, ..
    } = error
    else {
        panic!("expected a projection budget failure");
    };
    assert_eq!(source_index, Some(1));
    assert_eq!(*source.resource(), ConversionResource::OutputBytes);
    let budget = source.budget_error().unwrap();
    assert_eq!(budget.configured_limit(), 3);
    assert_eq!(budget.used(), Some(3));
    assert_eq!(budget.remaining(), Some(0));
}

/// A string's representation does not determine whether the text bound applies.
#[test]
fn test_natural_json_text_limit_applies_inside_json_and_maps() {
    use qubit_datatype::ConversionLimits;
    use qubit_datatype::ConversionPolicy;
    use qubit_datatype::StructuredConversionLimits;
    let limits = ConversionLimits::builder()
        .structured_limits(StructuredConversionLimits::builder().max_text_bytes(2).build())
        .build();
    for value in [
        Value::from("abc"),
        Value::Json(json!("abc")),
        Value::StringMap(HashMap::from([("a".into(), "abc".into())])),
    ] {
        assert!(value.to_json_value_with(&ConversionPolicy::default(), &limits).is_err());
    }
}
