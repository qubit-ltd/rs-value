/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations for setting a single `MultiValues` element.

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

/// Internal trait used to set a single value in `MultiValues`.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
pub trait MultiValuesSingleSetter<T>: super::sealed::MultiValuesSingleSetterSealed<T> {
    /// Replaces the stored values with one `value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_single_value(&mut self, value: T) -> ValueResult<()>;
}

macro_rules! impl_multi_values_single_setter {
    ($type:ty, $variant:ident) => {
        impl super::sealed::MultiValuesSingleSetterSealed<$type> for MultiValues {}

        impl MultiValuesSingleSetter<$type> for MultiValues {
            #[inline]
            fn set_single_value(&mut self, value: $type) -> ValueResult<()> {
                *self = MultiValues::$variant(vec![value]);
                Ok(())
            }
        }
    };
}

impl_multi_values_single_setter!(bool, Bool);
impl_multi_values_single_setter!(char, Char);
impl_multi_values_single_setter!(i8, Int8);
impl_multi_values_single_setter!(i16, Int16);
impl_multi_values_single_setter!(i32, Int32);
impl_multi_values_single_setter!(i64, Int64);
impl_multi_values_single_setter!(i128, Int128);
impl_multi_values_single_setter!(u8, UInt8);
impl_multi_values_single_setter!(u16, UInt16);
impl_multi_values_single_setter!(u32, UInt32);
impl_multi_values_single_setter!(u64, UInt64);
impl_multi_values_single_setter!(u128, UInt128);
impl_multi_values_single_setter!(isize, IntSize);
impl_multi_values_single_setter!(usize, UIntSize);
impl_multi_values_single_setter!(f32, Float32);
impl_multi_values_single_setter!(f64, Float64);
impl_multi_values_single_setter!(String, String);
impl_multi_values_single_setter!(NaiveDate, Date);
impl_multi_values_single_setter!(NaiveTime, Time);
impl_multi_values_single_setter!(NaiveDateTime, DateTime);
impl_multi_values_single_setter!(DateTime<Utc>, Instant);
impl_multi_values_single_setter!(BigInt, BigInteger);
impl_multi_values_single_setter!(BigDecimal, BigDecimal);
impl_multi_values_single_setter!(Duration, Duration);
impl_multi_values_single_setter!(Url, Url);
impl_multi_values_single_setter!(HashMap<String, String>, StringMap);
impl_multi_values_single_setter!(serde_json::Value, Json);
