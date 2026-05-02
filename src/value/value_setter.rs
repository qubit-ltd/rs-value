/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026.
 *    Haixing Hu, Qubit Co. Ltd.
 *
 *    All rights reserved.
 *
 ******************************************************************************/

//! Internal implementations of `ValueSetter<T>` for all supported `Value` types.

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

/// Internal trait used to set specific types in `Value`.
///
/// This trait backs `Value::set<T>()`; downstream code should call the
/// inherent method instead of implementing or naming this trait directly.
#[doc(hidden)]
pub trait ValueSetter<T> {
    /// Replaces the stored value with `value`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the value can be stored, or a `ValueError` when
    /// the implementation rejects the value.
    fn set_value(&mut self, value: T) -> ValueResult<()>;
}

macro_rules! impl_value_setter {
    ($type:ty, $method:ident) => {
        impl ValueSetter<$type> for Value {
            #[inline]
            fn set_value(&mut self, value: $type) -> ValueResult<()> {
                self.$method(value)
            }
        }
    };
}

impl_value_setter!(bool, set_bool);
impl_value_setter!(char, set_char);
impl_value_setter!(i8, set_int8);
impl_value_setter!(i16, set_int16);
impl_value_setter!(i32, set_int32);
impl_value_setter!(i64, set_int64);
impl_value_setter!(i128, set_int128);
impl_value_setter!(u8, set_uint8);
impl_value_setter!(u16, set_uint16);
impl_value_setter!(u32, set_uint32);
impl_value_setter!(u64, set_uint64);
impl_value_setter!(u128, set_uint128);
impl_value_setter!(f32, set_float32);
impl_value_setter!(f64, set_float64);
impl_value_setter!(DateTime<Utc>, set_instant);
impl_value_setter!(NaiveDate, set_date);
impl_value_setter!(NaiveTime, set_time);
impl_value_setter!(NaiveDateTime, set_datetime);
impl_value_setter!(BigInt, set_biginteger);
impl_value_setter!(BigDecimal, set_bigdecimal);
impl_value_setter!(isize, set_intsize);
impl_value_setter!(usize, set_uintsize);
impl_value_setter!(Duration, set_duration);
impl_value_setter!(Url, set_url);
impl_value_setter!(HashMap<String, String>, set_string_map);
impl_value_setter!(serde_json::Value, set_json);

impl ValueSetter<String> for Value {
    #[inline]
    fn set_value(&mut self, value: String) -> ValueResult<()> {
        self.set_string(value)
    }
}

/// Special handling for `&str`, converted into owned `String`.
impl ValueSetter<&str> for Value {
    #[inline]
    fn set_value(&mut self, value: &str) -> ValueResult<()> {
        self.set_string(value.to_string())
    }
}
