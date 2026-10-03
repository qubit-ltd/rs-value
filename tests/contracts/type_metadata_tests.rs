// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Tests that the complete runtime type set remains available.

use qubit_datatype::DataType;
use qubit_value::MultiValues;
use qubit_value::Value;
use qubit_value::ValueContainer;

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
        Value::Duration(std::time::Duration::from_secs(1)).data_type(),
        Value::StringMap(std::collections::HashMap::new()).data_type(),
    ];
    let collections = [
        MultiValues::Bool(vec![true]).data_type(),
        MultiValues::Char(vec!['x']).data_type(),
        MultiValues::Int32(vec![1]).data_type(),
        MultiValues::Float64(vec![1.0]).data_type(),
        MultiValues::String(vec![String::from("x")]).data_type(),
        MultiValues::Duration(vec![std::time::Duration::from_secs(1)]).data_type(),
        MultiValues::StringMap(vec![std::collections::HashMap::new()]).data_type(),
    ];

    assert!(scalars.contains(&DataType::Bool));
    assert!(scalars.contains(&DataType::StringMap));
    assert!(collections.contains(&DataType::Bool));
    assert!(collections.contains(&DataType::StringMap));
}

/// Exercises every type-table row through the public scalar and collection
/// constructor families used by downstream callers.
#[cfg(feature = "all")]
#[test]
fn test_all_type_table_rows_construct_value_containers_from_owned_and_borrowed_inputs() {
    use std::collections::HashMap;
    use std::time::Duration;

    use bigdecimal::BigDecimal;
    use chrono::DateTime;
    use chrono::NaiveDate;
    use chrono::NaiveDateTime;
    use chrono::NaiveTime;
    use chrono::Utc;
    use num_bigint::BigInt;

    macro_rules! assert_constructors {
        ($value:expr, $data_type:expr) => {{
            let value = $value;
            let scalar = ValueContainer::from(value.clone());
            let owned = ValueContainer::from(vec![value.clone()]);
            let borrowed_slice = ValueContainer::from([value.clone()].as_slice());
            let borrowed_vec = ValueContainer::from(&vec![value.clone()]);
            let array = ValueContainer::from([value.clone()]);
            let borrowed_array = ValueContainer::from(&[value.clone()]);

            assert_eq!(scalar.data_type(), $data_type);
            for container in [owned, borrowed_slice, borrowed_vec, array, borrowed_array] {
                assert_eq!(container.data_type(), $data_type);
                assert!(container.is_collection());
                assert_eq!(container.len(), 1);
            }
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
    assert_constructors!(url::Url::parse("https://example.com").expect("valid URL"), DataType::Url);
    assert_constructors!(HashMap::from([(String::from("key"), String::from("value"))]), DataType::StringMap);
    assert_constructors!(serde_json::json!({"key": "value"}), DataType::Json);
}
