/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal getter trait implementations for `MultiValues`.

use qubit_datatype::DataType;

use crate::value_error::{
    ValueError,
    ValueResult,
};

use super::multi_values::MultiValues;

/// Private marker trait used to prevent downstream implementations.
trait MultiValuesGetterSealed<T> {}

/// Internal trait used to extract multiple values from `MultiValues`.
///
/// This trait backs `MultiValues::get<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[allow(private_bounds)]
pub trait MultiValuesGetter<T>: MultiValuesGetterSealed<T> {
    /// Gets all stored values as `Vec<T>`.
    ///
    /// # Returns
    ///
    /// Returns a cloned vector when the stored variant matches `T`, or a
    /// `ValueError` describing the mismatch.
    fn get_values(&self) -> ValueResult<Vec<T>>;
}

macro_rules! impl_multi_values_getter {
    ($type:ty, $variant:ident, $data_type:expr) => {
        impl MultiValuesGetterSealed<$type> for MultiValues {}

        impl MultiValuesGetter<$type> for MultiValues {
            #[inline]
            fn get_values(&self) -> ValueResult<Vec<$type>> {
                match self {
                    MultiValues::$variant(v) => Ok(v.clone()),
                    MultiValues::Empty(dt) if *dt == $data_type => Ok(Vec::new()),
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }
    };
}

impl_multi_values_getter!(bool, Bool, DataType::Bool);
impl_multi_values_getter!(char, Char, DataType::Char);
impl_multi_values_getter!(i8, Int8, DataType::Int8);
impl_multi_values_getter!(i16, Int16, DataType::Int16);
impl_multi_values_getter!(i32, Int32, DataType::Int32);
impl_multi_values_getter!(i64, Int64, DataType::Int64);
impl_multi_values_getter!(i128, Int128, DataType::Int128);
impl_multi_values_getter!(u8, UInt8, DataType::UInt8);
impl_multi_values_getter!(u16, UInt16, DataType::UInt16);
impl_multi_values_getter!(u32, UInt32, DataType::UInt32);
impl_multi_values_getter!(u64, UInt64, DataType::UInt64);
impl_multi_values_getter!(u128, UInt128, DataType::UInt128);
impl_multi_values_getter!(f32, Float32, DataType::Float32);
impl_multi_values_getter!(f64, Float64, DataType::Float64);
impl_multi_values_getter!(String, String, DataType::String);
impl_multi_values_getter!(chrono::NaiveDate, Date, DataType::Date);
impl_multi_values_getter!(chrono::NaiveTime, Time, DataType::Time);
impl_multi_values_getter!(chrono::NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_values_getter!(chrono::DateTime<chrono::Utc>, Instant, DataType::Instant);
impl_multi_values_getter!(num_bigint::BigInt, BigInteger, DataType::BigInteger);
impl_multi_values_getter!(bigdecimal::BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_values_getter!(isize, IntSize, DataType::IntSize);
impl_multi_values_getter!(usize, UIntSize, DataType::UIntSize);
impl_multi_values_getter!(std::time::Duration, Duration, DataType::Duration);
impl_multi_values_getter!(url::Url, Url, DataType::Url);
impl_multi_values_getter!(std::collections::HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_values_getter!(serde_json::Value, Json, DataType::Json);
