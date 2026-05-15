/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! `TryFrom<&MultiValues>` implementations for strict first-value and list reads.

use qubit_datatype::DataType;

use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;

macro_rules! impl_multi_value_first_try_from_ref {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl TryFrom<&MultiValues> for $type {
            type Error = ValueError;

            #[inline]
            fn try_from(values: &MultiValues) -> ValueResult<$type> {
                match values {
                    MultiValues::$variant(v) => v.first().cloned().ok_or(ValueError::NoValue),
                    MultiValues::Empty(dt) if *dt == $data_type => Err(ValueError::NoValue),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: values.data_type(),
                    }),
                }
            }
        }
    };
}

macro_rules! impl_multi_values_try_from_ref {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl TryFrom<&MultiValues> for Vec<$type> {
            type Error = ValueError;

            #[inline]
            fn try_from(values: &MultiValues) -> ValueResult<Vec<$type>> {
                match values {
                    MultiValues::$variant(v) => Ok(v.clone()),
                    MultiValues::Empty(dt) if *dt == $data_type => Ok(Vec::new()),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: values.data_type(),
                    }),
                }
            }
        }
    };
}

impl_multi_value_first_try_from_ref!(bool, Bool, DataType::Bool);
impl_multi_value_first_try_from_ref!(char, Char, DataType::Char);
impl_multi_value_first_try_from_ref!(i8, Int8, DataType::Int8);
impl_multi_value_first_try_from_ref!(i16, Int16, DataType::Int16);
impl_multi_value_first_try_from_ref!(i32, Int32, DataType::Int32);
impl_multi_value_first_try_from_ref!(i64, Int64, DataType::Int64);
impl_multi_value_first_try_from_ref!(i128, Int128, DataType::Int128);
impl_multi_value_first_try_from_ref!(u8, UInt8, DataType::UInt8);
impl_multi_value_first_try_from_ref!(u16, UInt16, DataType::UInt16);
impl_multi_value_first_try_from_ref!(u32, UInt32, DataType::UInt32);
impl_multi_value_first_try_from_ref!(u64, UInt64, DataType::UInt64);
impl_multi_value_first_try_from_ref!(u128, UInt128, DataType::UInt128);
impl_multi_value_first_try_from_ref!(f32, Float32, DataType::Float32);
impl_multi_value_first_try_from_ref!(f64, Float64, DataType::Float64);
impl_multi_value_first_try_from_ref!(String, String, DataType::String);
impl_multi_value_first_try_from_ref!(chrono::NaiveDate, Date, DataType::Date);
impl_multi_value_first_try_from_ref!(chrono::NaiveTime, Time, DataType::Time);
impl_multi_value_first_try_from_ref!(chrono::NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_value_first_try_from_ref!(chrono::DateTime<chrono::Utc>, Instant, DataType::Instant);
impl_multi_value_first_try_from_ref!(num_bigint::BigInt, BigInteger, DataType::BigInteger);
impl_multi_value_first_try_from_ref!(bigdecimal::BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_value_first_try_from_ref!(isize, IntSize, DataType::IntSize);
impl_multi_value_first_try_from_ref!(usize, UIntSize, DataType::UIntSize);
impl_multi_value_first_try_from_ref!(std::time::Duration, Duration, DataType::Duration);
impl_multi_value_first_try_from_ref!(url::Url, Url, DataType::Url);
impl_multi_value_first_try_from_ref!(
    std::collections::HashMap<String, String>,
    StringMap,
    DataType::StringMap
);
impl_multi_value_first_try_from_ref!(serde_json::Value, Json, DataType::Json);

impl_multi_values_try_from_ref!(bool, Bool, DataType::Bool);
impl_multi_values_try_from_ref!(char, Char, DataType::Char);
impl_multi_values_try_from_ref!(i8, Int8, DataType::Int8);
impl_multi_values_try_from_ref!(i16, Int16, DataType::Int16);
impl_multi_values_try_from_ref!(i32, Int32, DataType::Int32);
impl_multi_values_try_from_ref!(i64, Int64, DataType::Int64);
impl_multi_values_try_from_ref!(i128, Int128, DataType::Int128);
impl_multi_values_try_from_ref!(u8, UInt8, DataType::UInt8);
impl_multi_values_try_from_ref!(u16, UInt16, DataType::UInt16);
impl_multi_values_try_from_ref!(u32, UInt32, DataType::UInt32);
impl_multi_values_try_from_ref!(u64, UInt64, DataType::UInt64);
impl_multi_values_try_from_ref!(u128, UInt128, DataType::UInt128);
impl_multi_values_try_from_ref!(f32, Float32, DataType::Float32);
impl_multi_values_try_from_ref!(f64, Float64, DataType::Float64);
impl_multi_values_try_from_ref!(String, String, DataType::String);
impl_multi_values_try_from_ref!(chrono::NaiveDate, Date, DataType::Date);
impl_multi_values_try_from_ref!(chrono::NaiveTime, Time, DataType::Time);
impl_multi_values_try_from_ref!(chrono::NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_values_try_from_ref!(chrono::DateTime<chrono::Utc>, Instant, DataType::Instant);
impl_multi_values_try_from_ref!(num_bigint::BigInt, BigInteger, DataType::BigInteger);
impl_multi_values_try_from_ref!(bigdecimal::BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_values_try_from_ref!(isize, IntSize, DataType::IntSize);
impl_multi_values_try_from_ref!(usize, UIntSize, DataType::UIntSize);
impl_multi_values_try_from_ref!(std::time::Duration, Duration, DataType::Duration);
impl_multi_values_try_from_ref!(url::Url, Url, DataType::Url);
impl_multi_values_try_from_ref!(
    std::collections::HashMap<String, String>,
    StringMap,
    DataType::StringMap
);
impl_multi_values_try_from_ref!(serde_json::Value, Json, DataType::Json);
