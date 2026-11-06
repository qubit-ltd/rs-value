// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests that the complete runtime type set remains available.

use std::collections::HashMap;
#[cfg(feature = "all")]
use std::slice::from_ref;
use std::time::Duration;

#[cfg(feature = "all")]
use bigdecimal::BigDecimal;
#[cfg(feature = "all")]
use chrono::DateTime;
#[cfg(feature = "all")]
use chrono::NaiveDate;
#[cfg(feature = "all")]
use chrono::NaiveDateTime;
#[cfg(feature = "all")]
use chrono::NaiveTime;
#[cfg(feature = "all")]
use chrono::Utc;
#[cfg(feature = "all")]
use num_bigint::BigInt;
use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
#[cfg(feature = "all")]
use qubit_value::ValueContainer;
#[cfg(feature = "all")]
use serde_json::json;
#[cfg(feature = "all")]
use url::Url;

#[test]
fn test_all_core_types_have_scalar_and_collection_constructors() {
    let scalars = [
        Value::Bool(true).data_type(),
        Value::Char('x').data_type(),
        Value::Int8(1).data_type(),
        Value::Int16(1).data_type(),
        Value::Int32(1).data_type(),
        Value::Int64(1).data_type(),
        Value::Int128(1).data_type(),
        Value::UInt8(1).data_type(),
        Value::UInt16(1).data_type(),
        Value::UInt32(1).data_type(),
        Value::UInt64(1).data_type(),
        Value::UInt128(1).data_type(),
        Value::Float32(1.0).data_type(),
        Value::Float64(1.0).data_type(),
        Value::String(String::from("x")).data_type(),
        Value::Duration(Duration::from_secs(1)).data_type(),
        Value::StringMap(HashMap::new()).data_type(),
    ];
    let collections = [
        MultiValues::Bool(vec![true]).data_type(),
        MultiValues::Char(vec!['x']).data_type(),
        MultiValues::Int32(vec![1]).data_type(),
        MultiValues::Float64(vec![1.0]).data_type(),
        MultiValues::String(vec![String::from("x")]).data_type(),
        MultiValues::Duration(vec![Duration::from_secs(1)]).data_type(),
        MultiValues::StringMap(vec![HashMap::new()]).data_type(),
    ];

    assert!(
        scalars.contains(&DataType::Bool),
        "the scalar bool constructor should report DataType::Bool"
    );
    assert!(
        scalars.contains(&DataType::StringMap),
        "the scalar string-map constructor should report DataType::StringMap"
    );
    assert!(
        collections.contains(&DataType::Bool),
        "the collection bool constructor should report DataType::Bool"
    );
    assert!(
        collections.contains(&DataType::StringMap),
        "the collection string-map constructor should report DataType::StringMap"
    );
}

/// Exercises every type-table row through the public scalar and collection
/// constructor families used by downstream callers.
#[cfg(feature = "all")]
#[test]
fn test_all_type_table_rows_construct_value_containers_from_owned_and_borrowed_inputs() {
    macro_rules! assert_constructors {
        ($value:expr, $data_type:expr) => {{
            let value = $value;
            let scalar = ValueContainer::from(value.clone());
            let owned = ValueContainer::from(vec![value.clone()]);
            let borrowed_slice = ValueContainer::from([value.clone()].as_slice());
            let borrowed_vec = ValueContainer::from(&vec![value.clone()]);
            let array = ValueContainer::from([value.clone()]);
            let array_value = [value.clone()];
            let borrowed_array = ValueContainer::from(&array_value);

            assert_eq!(
                scalar.data_type(),
                $data_type,
                "scalar constructor must preserve type metadata"
            );
            for container in [owned, borrowed_slice, borrowed_vec, array, borrowed_array] {
                assert_eq!(
                    container.data_type(),
                    $data_type,
                    "collection constructor must preserve type metadata"
                );
                assert!(
                    container.is_collection(),
                    "collection input must retain collection shape"
                );
                assert_eq!(container.len(), 1, "collection constructor must retain its single item");
            }

            let mut promoted = ValueContainer::from(value.clone());
            promoted.add(value.clone()).expect("same-type scalar should promote");
            assert_eq!(
                promoted.data_type(),
                $data_type,
                "scalar promotion must preserve type metadata"
            );
            assert_eq!(
                promoted.len(),
                2,
                "scalar promotion must retain the expected item count"
            );

            let mut extended = ValueContainer::from(vec![value.clone()]);
            extended.add(value.clone()).expect("same-type item should append");
            assert_eq!(
                extended.data_type(),
                $data_type,
                "collection append must preserve type metadata"
            );
            assert_eq!(
                extended.len(),
                2,
                "collection append must retain the expected item count"
            );

            let mut replaced = ValueContainer::from(0_i32);
            replaced.set(vec![value.clone()]);
            assert_eq!(
                replaced.data_type(),
                $data_type,
                "collection replacement must preserve type metadata"
            );
            assert_eq!(
                replaced.len(),
                1,
                "collection replacement must retain the expected item count"
            );

            let mut multi_values = MultiValues::new(vec![value.clone()]);
            assert_eq!(
                multi_values.data_type(),
                $data_type,
                "construction must preserve type metadata"
            );
            assert_eq!(
                multi_values.get(),
                Ok(vec![value.clone()]),
                "owned access must return the original item"
            );
            assert_eq!(
                multi_values.get_first(),
                Ok(value.clone()),
                "first-item access must return the original item"
            );
            assert_eq!(
                multi_values.get_first_ref(),
                Ok(&value),
                "borrowed first-item access must return the original item"
            );
            assert_eq!(
                multi_values.get_slice(),
                Ok(from_ref(&value)),
                "slice access must return the original item"
            );
            multi_values.add(value.clone()).expect("same-type value should append");
            assert_eq!(multi_values.len(), 2, "append must retain both items");
            multi_values.set(vec![value]);
            assert_eq!(multi_values.len(), 1, "replacement must retain one item");
        }};
    }

    let date = NaiveDate::from_ymd_opt(2025, 1, 2).expect("valid date");
    let time = NaiveTime::from_hms_opt(3, 4, 5).expect("valid time");
    let datetime = NaiveDateTime::new(date, time);
    let instant = DateTime::<Utc>::from_timestamp(1_735_776_000, 0).expect("valid instant");

    assert_constructors!(true, DataType::Bool);
    assert_constructors!('x', DataType::Char);
    assert_constructors!(-8_i8, DataType::Int8);
    assert_constructors!(-16_i16, DataType::Int16);
    assert_constructors!(-32_i32, DataType::Int32);
    assert_constructors!(-64_i64, DataType::Int64);
    assert_constructors!(-128_i128, DataType::Int128);
    assert_constructors!(8_u8, DataType::UInt8);
    assert_constructors!(16_u16, DataType::UInt16);
    assert_constructors!(32_u32, DataType::UInt32);
    assert_constructors!(64_u64, DataType::UInt64);
    assert_constructors!(128_u128, DataType::UInt128);
    assert_constructors!(1.25_f32, DataType::Float32);
    assert_constructors!(2.5_f64, DataType::Float64);
    assert_constructors!(BigInt::from(123), DataType::BigInteger);
    assert_constructors!(BigDecimal::from(123), DataType::BigDecimal);
    assert_constructors!(String::from("text"), DataType::String);
    assert_constructors!(date, DataType::Date);
    assert_constructors!(time, DataType::Time);
    assert_constructors!(datetime, DataType::DateTime);
    assert_constructors!(instant, DataType::Instant);
    assert_constructors!(Duration::from_secs(1), DataType::Duration);
    assert_constructors!(Url::parse("https://example.com").expect("valid URL"), DataType::Url);
    assert_constructors!(
        HashMap::from([(String::from("key"), String::from("value"))]),
        DataType::StringMap
    );
    assert_constructors!(json!({"key": "value"}), DataType::Json);
}
