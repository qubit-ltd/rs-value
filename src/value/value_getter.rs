/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! `TryFrom<&Value>` implementations for strict typed reads.

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
use crate::value_error::{
    ValueError,
    ValueResult,
};

macro_rules! impl_value_try_from_ref {
    ($type:ty, $method:ident) => {
        impl TryFrom<&Value> for $type {
            type Error = ValueError;

            #[inline]
            fn try_from(value: &Value) -> ValueResult<$type> {
                value.$method()
            }
        }
    };
}

// Primitive and common value types
impl_value_try_from_ref!(bool, get_bool);
impl_value_try_from_ref!(char, get_char);
impl_value_try_from_ref!(i8, get_int8);
impl_value_try_from_ref!(i16, get_int16);
impl_value_try_from_ref!(i32, get_int32);
impl_value_try_from_ref!(i64, get_int64);
impl_value_try_from_ref!(i128, get_int128);
impl_value_try_from_ref!(u8, get_uint8);
impl_value_try_from_ref!(u16, get_uint16);
impl_value_try_from_ref!(u32, get_uint32);
impl_value_try_from_ref!(u64, get_uint64);
impl_value_try_from_ref!(u128, get_uint128);
impl_value_try_from_ref!(f32, get_float32);
impl_value_try_from_ref!(f64, get_float64);
impl_value_try_from_ref!(NaiveDate, get_date);
impl_value_try_from_ref!(NaiveTime, get_time);
impl_value_try_from_ref!(NaiveDateTime, get_datetime);
impl_value_try_from_ref!(DateTime<Utc>, get_instant);
impl_value_try_from_ref!(BigInt, get_biginteger);
impl_value_try_from_ref!(BigDecimal, get_bigdecimal);
impl_value_try_from_ref!(isize, get_intsize);
impl_value_try_from_ref!(usize, get_uintsize);
impl_value_try_from_ref!(Duration, get_duration);
impl_value_try_from_ref!(Url, get_url);
impl_value_try_from_ref!(HashMap<String, String>, get_string_map);
impl_value_try_from_ref!(serde_json::Value, get_json);

/// String specialization because `Value::get_string()` returns `&str`.
impl TryFrom<&Value> for String {
    type Error = ValueError;

    #[inline]
    fn try_from(value: &Value) -> ValueResult<String> {
        value.get_string().map(|s| s.to_string())
    }
}
