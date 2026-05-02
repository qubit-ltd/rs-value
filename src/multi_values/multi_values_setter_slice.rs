/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal implementations for replacing a `MultiValues` sequence from a slice.

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

/// Internal trait used to set `MultiValues` from a slice.
///
/// This trait backs `MultiValues::set<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesSetterSlice<T>: super::sealed::MultiValuesSetterSliceSealed<T> {
    /// Replaces the stored values with a clone of `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be stored, or a `ValueError`
    /// describing why the update failed.
    fn set_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

macro_rules! impl_multi_values_setter_slice {
    ($type:ty, $variant:ident) => {
        impl super::sealed::MultiValuesSetterSliceSealed<$type> for MultiValues {}

        impl MultiValuesSetterSlice<$type> for MultiValues {
            #[inline]
            fn set_values_slice(&mut self, values: &[$type]) -> ValueResult<()> {
                *self = MultiValues::$variant(values.to_vec());
                Ok(())
            }
        }
    };
}

impl_multi_values_setter_slice!(bool, Bool);
impl_multi_values_setter_slice!(char, Char);
impl_multi_values_setter_slice!(i8, Int8);
impl_multi_values_setter_slice!(i16, Int16);
impl_multi_values_setter_slice!(i32, Int32);
impl_multi_values_setter_slice!(i64, Int64);
impl_multi_values_setter_slice!(i128, Int128);
impl_multi_values_setter_slice!(u8, UInt8);
impl_multi_values_setter_slice!(u16, UInt16);
impl_multi_values_setter_slice!(u32, UInt32);
impl_multi_values_setter_slice!(u64, UInt64);
impl_multi_values_setter_slice!(u128, UInt128);
impl_multi_values_setter_slice!(isize, IntSize);
impl_multi_values_setter_slice!(usize, UIntSize);
impl_multi_values_setter_slice!(f32, Float32);
impl_multi_values_setter_slice!(f64, Float64);
impl_multi_values_setter_slice!(String, String);
impl_multi_values_setter_slice!(NaiveDate, Date);
impl_multi_values_setter_slice!(NaiveTime, Time);
impl_multi_values_setter_slice!(NaiveDateTime, DateTime);
impl_multi_values_setter_slice!(DateTime<Utc>, Instant);
impl_multi_values_setter_slice!(BigInt, BigInteger);
impl_multi_values_setter_slice!(BigDecimal, BigDecimal);
impl_multi_values_setter_slice!(Duration, Duration);
impl_multi_values_setter_slice!(Url, Url);
impl_multi_values_setter_slice!(HashMap<String, String>, StringMap);
impl_multi_values_setter_slice!(serde_json::Value, Json);
