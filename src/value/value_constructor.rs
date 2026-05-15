/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! `From<T>` implementations for all supported `Value` input types.

use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{
    DateTime,
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use num_bigint::BigInt;
use url::Url;

use super::value::Value;

macro_rules! impl_value_from {
    ($type:ty, $variant:expr) => {
        impl From<$type> for Value {
            #[inline]
            fn from(value: $type) -> Self {
                $variant(value)
            }
        }
    };
}

impl_value_from!(bool, Value::Bool);
impl_value_from!(char, Value::Char);
impl_value_from!(i8, Value::Int8);
impl_value_from!(i16, Value::Int16);
impl_value_from!(i32, Value::Int32);
impl_value_from!(i64, Value::Int64);
impl_value_from!(i128, Value::Int128);
impl_value_from!(u8, Value::UInt8);
impl_value_from!(u16, Value::UInt16);
impl_value_from!(u32, Value::UInt32);
impl_value_from!(u64, Value::UInt64);
impl_value_from!(u128, Value::UInt128);
impl_value_from!(f32, Value::Float32);
impl_value_from!(f64, Value::Float64);
impl_value_from!(NaiveDate, Value::Date);
impl_value_from!(NaiveTime, Value::Time);
impl_value_from!(NaiveDateTime, Value::DateTime);
impl_value_from!(DateTime<Utc>, Value::Instant);
impl_value_from!(BigInt, Value::BigInteger);
impl_value_from!(BigDecimal, Value::BigDecimal);
impl_value_from!(isize, Value::IntSize);
impl_value_from!(usize, Value::UIntSize);
impl_value_from!(Duration, Value::Duration);
impl_value_from!(Url, Value::Url);
impl_value_from!(HashMap<String, String>, Value::StringMap);
impl_value_from!(serde_json::Value, Value::Json);
impl_value_from!(String, Value::String);

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}
