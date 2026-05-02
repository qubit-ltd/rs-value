/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Internal first-value getter trait implementations for `MultiValues`.

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

use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;
use qubit_common::lang::DataType;

/// Internal trait used to extract the first value from `MultiValues`.
///
/// This trait backs `MultiValues::get_first<T>()`; downstream code should call
/// the inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesFirstGetter<T> {
    /// Gets the first stored value as `T`.
    ///
    /// # Returns
    ///
    /// Returns the first value when present and typed as `T`, or a `ValueError`
    /// describing the missing value or mismatch.
    fn get_first_value(&self) -> ValueResult<T>;
}

macro_rules! impl_multi_values_first_getter {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl MultiValuesFirstGetter<$type> for MultiValues {
            #[inline]
            fn get_first_value(&self) -> ValueResult<$type> {
                match self {
                    MultiValues::$variant(v) if !v.is_empty() => Ok(v[0].clone()),
                    MultiValues::$variant(_) => Err(ValueError::NoValue),
                    MultiValues::Empty(dt) if *dt == $data_type => Err(ValueError::NoValue),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }
    };
}

impl_multi_values_first_getter!(bool, Bool, DataType::Bool);
impl_multi_values_first_getter!(char, Char, DataType::Char);
impl_multi_values_first_getter!(i8, Int8, DataType::Int8);
impl_multi_values_first_getter!(i16, Int16, DataType::Int16);
impl_multi_values_first_getter!(i32, Int32, DataType::Int32);
impl_multi_values_first_getter!(i64, Int64, DataType::Int64);
impl_multi_values_first_getter!(i128, Int128, DataType::Int128);
impl_multi_values_first_getter!(u8, UInt8, DataType::UInt8);
impl_multi_values_first_getter!(u16, UInt16, DataType::UInt16);
impl_multi_values_first_getter!(u32, UInt32, DataType::UInt32);
impl_multi_values_first_getter!(u64, UInt64, DataType::UInt64);
impl_multi_values_first_getter!(u128, UInt128, DataType::UInt128);
impl_multi_values_first_getter!(f32, Float32, DataType::Float32);
impl_multi_values_first_getter!(f64, Float64, DataType::Float64);
impl_multi_values_first_getter!(String, String, DataType::String);
impl_multi_values_first_getter!(NaiveDate, Date, DataType::Date);
impl_multi_values_first_getter!(NaiveTime, Time, DataType::Time);
impl_multi_values_first_getter!(NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_values_first_getter!(DateTime<Utc>, Instant, DataType::Instant);
impl_multi_values_first_getter!(BigInt, BigInteger, DataType::BigInteger);
impl_multi_values_first_getter!(BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_values_first_getter!(isize, IntSize, DataType::IntSize);
impl_multi_values_first_getter!(usize, UIntSize, DataType::UIntSize);
impl_multi_values_first_getter!(Duration, Duration, DataType::Duration);
impl_multi_values_first_getter!(Url, Url, DataType::Url);
impl_multi_values_first_getter!(HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_values_first_getter!(serde_json::Value, Json, DataType::Json);
