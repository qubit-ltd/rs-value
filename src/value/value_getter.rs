/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations of `ValueGetter<T>` for all supported `Value` types.
//!
//! These impls are intentionally colocated with the trait definition to keep the
//! generic read path implementation and trait contract together.

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
use crate::value_error::ValueResult;

/// Internal trait used to extract specific types from `Value`.
///
/// This trait backs `Value::get<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueGetter<T>: super::sealed::ValueGetterSealed<T> {
    /// Gets the value as `T`.
    ///
    /// # Returns
    ///
    /// Returns the typed value when the stored variant matches `T`, or a
    /// `ValueError` describing the mismatch or missing value.
    fn get_value(&self) -> ValueResult<T>;
}

macro_rules! impl_value_getter_copy {
    ($type:ty, $method:ident) => {
        impl super::sealed::ValueGetterSealed<$type> for Value {}

        impl ValueGetter<$type> for Value {
            #[inline]
            fn get_value(&self) -> ValueResult<$type> {
                self.$method()
            }
        }
    };
}

// Primitive and common value types
impl_value_getter_copy!(bool, get_bool);
impl_value_getter_copy!(char, get_char);
impl_value_getter_copy!(i8, get_int8);
impl_value_getter_copy!(i16, get_int16);
impl_value_getter_copy!(i32, get_int32);
impl_value_getter_copy!(i64, get_int64);
impl_value_getter_copy!(i128, get_int128);
impl_value_getter_copy!(u8, get_uint8);
impl_value_getter_copy!(u16, get_uint16);
impl_value_getter_copy!(u32, get_uint32);
impl_value_getter_copy!(u64, get_uint64);
impl_value_getter_copy!(u128, get_uint128);
impl_value_getter_copy!(f32, get_float32);
impl_value_getter_copy!(f64, get_float64);
impl_value_getter_copy!(NaiveDate, get_date);
impl_value_getter_copy!(NaiveTime, get_time);
impl_value_getter_copy!(NaiveDateTime, get_datetime);
impl_value_getter_copy!(DateTime<Utc>, get_instant);
impl_value_getter_copy!(BigInt, get_biginteger);
impl_value_getter_copy!(BigDecimal, get_bigdecimal);
impl_value_getter_copy!(isize, get_intsize);
impl_value_getter_copy!(usize, get_uintsize);
impl_value_getter_copy!(Duration, get_duration);
impl_value_getter_copy!(Url, get_url);
impl_value_getter_copy!(HashMap<String, String>, get_string_map);
impl_value_getter_copy!(serde_json::Value, get_json);

/// String specialization because `Value::get_string()` returns `&str`.
impl super::sealed::ValueGetterSealed<String> for Value {}

impl ValueGetter<String> for Value {
    #[inline]
    fn get_value(&self) -> ValueResult<String> {
        self.get_string().map(|s| s.to_string())
    }
}
