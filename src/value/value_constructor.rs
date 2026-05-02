/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations of `ValueConstructor<T>` for all supported `Value`
//! types.

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

/// Internal trait used to create `Value` from supported Rust types.
///
/// This trait backs `Value::new<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueConstructor<T> {
    /// Builds a `Value` that wraps `value`.
    ///
    /// # Returns
    ///
    /// Returns the enum variant corresponding to `T`.
    fn from_type(value: T) -> Self;
}

macro_rules! impl_value_constructor {
    ($type:ty, $variant:expr) => {
        impl ValueConstructor<$type> for Value {
            #[inline]
            fn from_type(value: $type) -> Self {
                $variant(value)
            }
        }
    };
}

impl_value_constructor!(bool, Value::Bool);
impl_value_constructor!(char, Value::Char);
impl_value_constructor!(i8, Value::Int8);
impl_value_constructor!(i16, Value::Int16);
impl_value_constructor!(i32, Value::Int32);
impl_value_constructor!(i64, Value::Int64);
impl_value_constructor!(i128, Value::Int128);
impl_value_constructor!(u8, Value::UInt8);
impl_value_constructor!(u16, Value::UInt16);
impl_value_constructor!(u32, Value::UInt32);
impl_value_constructor!(u64, Value::UInt64);
impl_value_constructor!(u128, Value::UInt128);
impl_value_constructor!(f32, Value::Float32);
impl_value_constructor!(f64, Value::Float64);
impl_value_constructor!(NaiveDate, Value::Date);
impl_value_constructor!(NaiveTime, Value::Time);
impl_value_constructor!(NaiveDateTime, Value::DateTime);
impl_value_constructor!(DateTime<Utc>, Value::Instant);
impl_value_constructor!(BigInt, Value::BigInteger);
impl_value_constructor!(BigDecimal, Value::BigDecimal);
impl_value_constructor!(isize, Value::IntSize);
impl_value_constructor!(usize, Value::UIntSize);
impl_value_constructor!(Duration, Value::Duration);
impl_value_constructor!(Url, Value::Url);
impl_value_constructor!(HashMap<String, String>, Value::StringMap);
impl_value_constructor!(serde_json::Value, Value::Json);

impl_value_constructor!(String, Value::String);

impl ValueConstructor<&str> for Value {
    #[inline]
    fn from_type(value: &str) -> Self {
        Value::String(value.to_string())
    }
}
