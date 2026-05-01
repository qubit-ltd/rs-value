use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use url::Url;

use qubit_common::lang::{DataConversionError, DataConvertTo, DataConverter};

use super::value::Value;
use super::value_constructor::ValueConstructor;
use super::value_converter::ValueConverter;
use super::value_getter::ValueGetter;
use super::value_setter::ValueSetter;
use crate::value_error::{ValueError, ValueResult};

fn map_data_conversion_error(error: DataConversionError) -> ValueError {
    match error {
        DataConversionError::NoValue => ValueError::NoValue,
        DataConversionError::ConversionFailed { from, to } => {
            ValueError::ConversionFailed { from, to }
        }
        DataConversionError::ConversionError(message) => ValueError::ConversionError(message),
        DataConversionError::JsonSerializationError(message) => {
            ValueError::JsonSerializationError(message)
        }
        DataConversionError::JsonDeserializationError(message) => {
            ValueError::JsonDeserializationError(message)
        }
    }
}

fn data_converter_from_value(value: &Value) -> DataConverter<'_> {
    match value {
        Value::Empty(data_type) => DataConverter::Empty(*data_type),
        Value::Bool(value) => DataConverter::from(value),
        Value::Char(value) => DataConverter::from(value),
        Value::Int8(value) => DataConverter::from(value),
        Value::Int16(value) => DataConverter::from(value),
        Value::Int32(value) => DataConverter::from(value),
        Value::Int64(value) => DataConverter::from(value),
        Value::Int128(value) => DataConverter::from(value),
        Value::UInt8(value) => DataConverter::from(value),
        Value::UInt16(value) => DataConverter::from(value),
        Value::UInt32(value) => DataConverter::from(value),
        Value::UInt64(value) => DataConverter::from(value),
        Value::UInt128(value) => DataConverter::from(value),
        Value::IntSize(value) => DataConverter::from(value),
        Value::UIntSize(value) => DataConverter::from(value),
        Value::Float32(value) => DataConverter::from(value),
        Value::Float64(value) => DataConverter::from(value),
        Value::BigInteger(value) => DataConverter::from(value),
        Value::BigDecimal(value) => DataConverter::from(value),
        Value::String(value) => DataConverter::from(value),
        Value::Date(value) => DataConverter::from(value),
        Value::Time(value) => DataConverter::from(value),
        Value::DateTime(value) => DataConverter::from(value),
        Value::Instant(value) => DataConverter::from(value),
        Value::Duration(value) => DataConverter::from(value),
        Value::Url(value) => DataConverter::from(value),
        Value::StringMap(value) => DataConverter::from(value),
        Value::Json(value) => DataConverter::from(value),
    }
}

fn convert_with_data_converter<T>(value: &Value) -> ValueResult<T>
where
    for<'a> DataConverter<'a>: DataConvertTo<T>,
{
    data_converter_from_value(value)
        .to::<T>()
        .map_err(map_data_conversion_error)
}

// ============================================================================
// Implementation of internal traits (simplified using macros)
// ============================================================================

macro_rules! impl_value_traits {
    ($type:ty, $variant:ident, $get_method:ident, $set_method:ident) => {
        impl ValueGetter<$type> for Value {
            #[inline]
            fn get_value(&self) -> ValueResult<$type> {
                self.$get_method()
            }
        }

        impl ValueSetter<$type> for Value {
            #[inline]
            fn set_value(&mut self, value: $type) -> ValueResult<()> {
                self.$set_method(value)
            }
        }

        impl ValueConstructor<$type> for Value {
            #[inline]
            fn from_type(value: $type) -> Self {
                Value::$variant(value)
            }
        }
    };
}

macro_rules! impl_data_value_converter {
    ($type:ty) => {
        impl ValueConverter<$type> for Value {
            #[inline]
            fn convert(&self) -> ValueResult<$type> {
                convert_with_data_converter(self)
            }
        }
    };
}

// Implementation for Copy types
impl_value_traits!(bool, Bool, get_bool, set_bool);
impl_value_traits!(char, Char, get_char, set_char);
impl_value_traits!(i8, Int8, get_int8, set_int8);
impl_value_traits!(i16, Int16, get_int16, set_int16);
impl_value_traits!(i32, Int32, get_int32, set_int32);
impl_value_traits!(i64, Int64, get_int64, set_int64);
impl_value_traits!(i128, Int128, get_int128, set_int128);
impl_value_traits!(u8, UInt8, get_uint8, set_uint8);
impl_value_traits!(u16, UInt16, get_uint16, set_uint16);
impl_value_traits!(u32, UInt32, get_uint32, set_uint32);
impl_value_traits!(u64, UInt64, get_uint64, set_uint64);
impl_value_traits!(u128, UInt128, get_uint128, set_uint128);
impl_value_traits!(f32, Float32, get_float32, set_float32);
impl_value_traits!(f64, Float64, get_float64, set_float64);
impl_value_traits!(NaiveDate, Date, get_date, set_date);
impl_value_traits!(NaiveTime, Time, get_time, set_time);
impl_value_traits!(NaiveDateTime, DateTime, get_datetime, set_datetime);
impl_value_traits!(DateTime<Utc>, Instant, get_instant, set_instant);
impl_value_traits!(BigInt, BigInteger, get_biginteger, set_biginteger);
impl_value_traits!(BigDecimal, BigDecimal, get_bigdecimal, set_bigdecimal);
impl_value_traits!(isize, IntSize, get_intsize, set_intsize);
impl_value_traits!(usize, UIntSize, get_uintsize, set_uintsize);
impl_value_traits!(Duration, Duration, get_duration, set_duration);

// String needs cloning
impl ValueGetter<String> for Value {
    #[inline]
    fn get_value(&self) -> ValueResult<String> {
        self.get_string().map(|s| s.to_string())
    }
}

impl ValueSetter<String> for Value {
    #[inline]
    fn set_value(&mut self, value: String) -> ValueResult<()> {
        self.set_string(value)
    }
}

impl ValueConstructor<String> for Value {
    #[inline]
    fn from_type(value: String) -> Self {
        Value::String(value)
    }
}

// Special handling for &str - convert to String
impl ValueSetter<&str> for Value {
    #[inline]
    fn set_value(&mut self, value: &str) -> ValueResult<()> {
        self.set_string(value.to_string())
    }
}

impl ValueConstructor<&str> for Value {
    #[inline]
    fn from_type(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

// Url
impl ValueGetter<Url> for Value {
    #[inline]
    fn get_value(&self) -> ValueResult<Url> {
        self.get_url()
    }
}

impl ValueSetter<Url> for Value {
    #[inline]
    fn set_value(&mut self, value: Url) -> ValueResult<()> {
        self.set_url(value)
    }
}

impl ValueConstructor<Url> for Value {
    #[inline]
    fn from_type(value: Url) -> Self {
        Value::Url(value)
    }
}

// HashMap<String, String>
impl ValueGetter<HashMap<String, String>> for Value {
    #[inline]
    fn get_value(&self) -> ValueResult<HashMap<String, String>> {
        self.get_string_map()
    }
}

impl ValueSetter<HashMap<String, String>> for Value {
    #[inline]
    fn set_value(&mut self, value: HashMap<String, String>) -> ValueResult<()> {
        self.set_string_map(value)
    }
}

impl ValueConstructor<HashMap<String, String>> for Value {
    #[inline]
    fn from_type(value: HashMap<String, String>) -> Self {
        Value::StringMap(value)
    }
}

// serde_json::Value
impl ValueGetter<serde_json::Value> for Value {
    #[inline]
    fn get_value(&self) -> ValueResult<serde_json::Value> {
        self.get_json()
    }
}

impl ValueSetter<serde_json::Value> for Value {
    #[inline]
    fn set_value(&mut self, value: serde_json::Value) -> ValueResult<()> {
        self.set_json(value)
    }
}

impl ValueConstructor<serde_json::Value> for Value {
    #[inline]
    fn from_type(value: serde_json::Value) -> Self {
        Value::Json(value)
    }
}

impl_data_value_converter!(String);
impl_data_value_converter!(bool);
impl_data_value_converter!(char);
impl_data_value_converter!(i8);
impl_data_value_converter!(i16);
impl_data_value_converter!(i32);
impl_data_value_converter!(i64);
impl_data_value_converter!(i128);
impl_data_value_converter!(u8);
impl_data_value_converter!(u16);
impl_data_value_converter!(u32);
impl_data_value_converter!(u64);
impl_data_value_converter!(u128);
impl_data_value_converter!(isize);
impl_data_value_converter!(usize);
impl_data_value_converter!(f32);
impl_data_value_converter!(f64);
impl_data_value_converter!(NaiveDate);
impl_data_value_converter!(NaiveTime);
impl_data_value_converter!(NaiveDateTime);
impl_data_value_converter!(DateTime<Utc>);
impl_data_value_converter!(BigInt);
impl_data_value_converter!(BigDecimal);
impl_data_value_converter!(Duration);
impl_data_value_converter!(Url);
impl_data_value_converter!(HashMap<String, String>);
impl_data_value_converter!(serde_json::Value);
