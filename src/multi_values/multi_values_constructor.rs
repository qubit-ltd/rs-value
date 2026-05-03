/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations for constructing `MultiValues` from `Vec<T>`.

/// Private marker trait used to prevent downstream implementations.
trait MultiValuesConstructorSealed<T> {}

/// Internal trait used to create `MultiValues` from `Vec<T>`.
///
/// This trait backs `MultiValues::new<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[allow(private_bounds)]
pub trait MultiValuesConstructor<T>: MultiValuesConstructorSealed<T> {
    /// Builds a `MultiValues` instance from `values`.
    ///
    /// # Returns
    ///
    /// Returns the enum variant corresponding to `T`.
    fn from_vec(values: Vec<T>) -> Self;
}

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

use super::multi_values::MultiValues;

macro_rules! impl_multi_values_constructor {
    ($type:ty, $variant:ident) => {
        impl MultiValuesConstructorSealed<$type> for MultiValues {}

        impl MultiValuesConstructor<$type> for MultiValues {
            #[inline]
            fn from_vec(values: Vec<$type>) -> Self {
                MultiValues::$variant(values)
            }
        }
    };
}

impl_multi_values_constructor!(bool, Bool);
impl_multi_values_constructor!(char, Char);
impl_multi_values_constructor!(i8, Int8);
impl_multi_values_constructor!(i16, Int16);
impl_multi_values_constructor!(i32, Int32);
impl_multi_values_constructor!(i64, Int64);
impl_multi_values_constructor!(i128, Int128);
impl_multi_values_constructor!(u8, UInt8);
impl_multi_values_constructor!(u16, UInt16);
impl_multi_values_constructor!(u32, UInt32);
impl_multi_values_constructor!(u64, UInt64);
impl_multi_values_constructor!(u128, UInt128);
impl_multi_values_constructor!(isize, IntSize);
impl_multi_values_constructor!(usize, UIntSize);
impl_multi_values_constructor!(f32, Float32);
impl_multi_values_constructor!(f64, Float64);
impl_multi_values_constructor!(String, String);
impl_multi_values_constructor!(NaiveDate, Date);
impl_multi_values_constructor!(NaiveTime, Time);
impl_multi_values_constructor!(NaiveDateTime, DateTime);
impl_multi_values_constructor!(DateTime<Utc>, Instant);
impl_multi_values_constructor!(BigInt, BigInteger);
impl_multi_values_constructor!(BigDecimal, BigDecimal);
impl_multi_values_constructor!(Duration, Duration);
impl_multi_values_constructor!(Url, Url);
impl_multi_values_constructor!(HashMap<String, String>, StringMap);
impl_multi_values_constructor!(serde_json::Value, Json);
