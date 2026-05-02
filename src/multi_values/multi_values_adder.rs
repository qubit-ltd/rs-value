/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Internal implementations for appending one `MultiValues` element.

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

/// Internal trait used to add a single value to `MultiValues`.
///
/// This trait backs `MultiValues::add<S>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait MultiValuesAdder<T> {
    /// Appends `value` to the stored values.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be appended, or a `ValueError`
    /// describing the type mismatch.
    fn add_value(&mut self, value: T) -> ValueResult<()>;
}

macro_rules! impl_multi_values_adder {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl MultiValuesAdder<$type> for MultiValues {
            #[inline]
            fn add_value(&mut self, value: $type) -> ValueResult<()> {
                match self {
                    MultiValues::$variant(v) => {
                        v.push(value);
                        Ok(())
                    }
                    MultiValues::Empty(dt) if *dt == $data_type => {
                        *self = MultiValues::$variant(vec![value]);
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

impl_multi_values_adder!(bool, Bool, DataType::Bool);
impl_multi_values_adder!(char, Char, DataType::Char);
impl_multi_values_adder!(i8, Int8, DataType::Int8);
impl_multi_values_adder!(i16, Int16, DataType::Int16);
impl_multi_values_adder!(i32, Int32, DataType::Int32);
impl_multi_values_adder!(i64, Int64, DataType::Int64);
impl_multi_values_adder!(i128, Int128, DataType::Int128);
impl_multi_values_adder!(u8, UInt8, DataType::UInt8);
impl_multi_values_adder!(u16, UInt16, DataType::UInt16);
impl_multi_values_adder!(u32, UInt32, DataType::UInt32);
impl_multi_values_adder!(u64, UInt64, DataType::UInt64);
impl_multi_values_adder!(u128, UInt128, DataType::UInt128);
impl_multi_values_adder!(isize, IntSize, DataType::IntSize);
impl_multi_values_adder!(usize, UIntSize, DataType::UIntSize);
impl_multi_values_adder!(f32, Float32, DataType::Float32);
impl_multi_values_adder!(f64, Float64, DataType::Float64);
impl_multi_values_adder!(String, String, DataType::String);
impl_multi_values_adder!(NaiveDate, Date, DataType::Date);
impl_multi_values_adder!(NaiveTime, Time, DataType::Time);
impl_multi_values_adder!(NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_values_adder!(DateTime<Utc>, Instant, DataType::Instant);
impl_multi_values_adder!(BigInt, BigInteger, DataType::BigInteger);
impl_multi_values_adder!(BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_values_adder!(Duration, Duration, DataType::Duration);
impl_multi_values_adder!(Url, Url, DataType::Url);
impl_multi_values_adder!(HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_values_adder!(serde_json::Value, Json, DataType::Json);
