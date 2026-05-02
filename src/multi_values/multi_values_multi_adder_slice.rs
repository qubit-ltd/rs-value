/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Internal trait for appending multiple `MultiValues` elements from slices.

use crate::value_error::{
    ValueError,
    ValueResult,
};
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

use qubit_common::lang::DataType;

use super::multi_values::MultiValues;

/// Internal trait used to append multiple values from a slice.
///
/// This trait backs slice arguments to `MultiValues::add<S>()`; downstream code
/// should call the inherent method instead of implementing or naming this trait
/// directly.
#[doc(hidden)]
pub(crate) trait MultiValuesMultiAdderSlice<T> {
    /// Appends all values from `values`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the values can be appended, or a `ValueError`
    /// describing the type mismatch.
    fn add_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

macro_rules! impl_multi_values_multi_adder_slice {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl MultiValuesMultiAdderSlice<$type> for MultiValues {
            #[inline]
            fn add_values_slice(&mut self, values: &[$type]) -> ValueResult<()> {
                match self {
                    MultiValues::$variant(v) => {
                        v.extend_from_slice(values);
                        Ok(())
                    }
                    MultiValues::Empty(dt) if *dt == $data_type => {
                        *self = MultiValues::$variant(values.to_vec());
                        Ok(())
                    }
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }
    };
}

impl_multi_values_multi_adder_slice!(bool, Bool, DataType::Bool);
impl_multi_values_multi_adder_slice!(char, Char, DataType::Char);
impl_multi_values_multi_adder_slice!(i8, Int8, DataType::Int8);
impl_multi_values_multi_adder_slice!(i16, Int16, DataType::Int16);
impl_multi_values_multi_adder_slice!(i32, Int32, DataType::Int32);
impl_multi_values_multi_adder_slice!(i64, Int64, DataType::Int64);
impl_multi_values_multi_adder_slice!(i128, Int128, DataType::Int128);
impl_multi_values_multi_adder_slice!(u8, UInt8, DataType::UInt8);
impl_multi_values_multi_adder_slice!(u16, UInt16, DataType::UInt16);
impl_multi_values_multi_adder_slice!(u32, UInt32, DataType::UInt32);
impl_multi_values_multi_adder_slice!(u64, UInt64, DataType::UInt64);
impl_multi_values_multi_adder_slice!(u128, UInt128, DataType::UInt128);
impl_multi_values_multi_adder_slice!(isize, IntSize, DataType::IntSize);
impl_multi_values_multi_adder_slice!(usize, UIntSize, DataType::UIntSize);
impl_multi_values_multi_adder_slice!(f32, Float32, DataType::Float32);
impl_multi_values_multi_adder_slice!(f64, Float64, DataType::Float64);
impl_multi_values_multi_adder_slice!(String, String, DataType::String);
impl_multi_values_multi_adder_slice!(NaiveDate, Date, DataType::Date);
impl_multi_values_multi_adder_slice!(NaiveTime, Time, DataType::Time);
impl_multi_values_multi_adder_slice!(NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_values_multi_adder_slice!(DateTime<Utc>, Instant, DataType::Instant);
impl_multi_values_multi_adder_slice!(BigInt, BigInteger, DataType::BigInteger);
impl_multi_values_multi_adder_slice!(BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_values_multi_adder_slice!(Duration, Duration, DataType::Duration);
impl_multi_values_multi_adder_slice!(Url, Url, DataType::Url);
impl_multi_values_multi_adder_slice!(HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_values_multi_adder_slice!(serde_json::Value, Json, DataType::Json);
