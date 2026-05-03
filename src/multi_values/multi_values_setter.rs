/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/
//! Internal implementations for replacing a full `MultiValues` vector.

use crate::value_error::ValueResult;
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

/// Private marker trait used to prevent downstream implementations.
trait MultiValuesSetterSealed<T> {}

/// Internal trait used to set specific value lists in `MultiValues`.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[allow(private_bounds)]
pub trait MultiValuesSetter<T>: MultiValuesSetterSealed<T> {
    /// Replaces the stored values with `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}

macro_rules! impl_multi_values_setter {
    ($type:ty, $variant:ident) => {
        impl MultiValuesSetterSealed<$type> for MultiValues {}

        impl MultiValuesSetter<$type> for MultiValues {
            #[inline]
            fn set_values(&mut self, values: Vec<$type>) -> ValueResult<()> {
                *self = MultiValues::$variant(values);
                Ok(())
            }
        }
    };
}

impl_multi_values_setter!(bool, Bool);
impl_multi_values_setter!(char, Char);
impl_multi_values_setter!(i8, Int8);
impl_multi_values_setter!(i16, Int16);
impl_multi_values_setter!(i32, Int32);
impl_multi_values_setter!(i64, Int64);
impl_multi_values_setter!(i128, Int128);
impl_multi_values_setter!(u8, UInt8);
impl_multi_values_setter!(u16, UInt16);
impl_multi_values_setter!(u32, UInt32);
impl_multi_values_setter!(u64, UInt64);
impl_multi_values_setter!(u128, UInt128);
impl_multi_values_setter!(isize, IntSize);
impl_multi_values_setter!(usize, UIntSize);
impl_multi_values_setter!(f32, Float32);
impl_multi_values_setter!(f64, Float64);
impl_multi_values_setter!(String, String);
impl_multi_values_setter!(NaiveDate, Date);
impl_multi_values_setter!(NaiveTime, Time);
impl_multi_values_setter!(NaiveDateTime, DateTime);
impl_multi_values_setter!(DateTime<Utc>, Instant);
impl_multi_values_setter!(BigInt, BigInteger);
impl_multi_values_setter!(BigDecimal, BigDecimal);
impl_multi_values_setter!(Duration, Duration);
impl_multi_values_setter!(Url, Url);
impl_multi_values_setter!(HashMap<String, String>, StringMap);
impl_multi_values_setter!(serde_json::Value, Json);
