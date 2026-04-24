use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use num_traits::ToPrimitive;
use url::Url;

use qubit_common::lang::DataType;
use qubit_common::lang::argument::NumericArgument;

use super::value::Value;
use crate::error::{ValueError, ValueResult};

fn parse_duration_string(s: &str) -> ValueResult<Duration> {
    let trimmed = s.trim();
    let nanos_text = trimmed.strip_suffix("ns").ok_or_else(|| {
        ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: expected '<nanoseconds>ns'",
            s
        ))
    })?;
    let total_nanos = nanos_text.parse::<u128>().map_err(|_| {
        ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: invalid nanoseconds value",
            s
        ))
    })?;
    let secs = total_nanos / 1_000_000_000;
    if secs > u64::MAX as u128 {
        return Err(ValueError::ConversionError(format!(
            "Cannot convert '{}' to Duration: value out of range",
            s
        )));
    }
    let nanos = (total_nanos % 1_000_000_000) as u32;
    Ok(Duration::new(secs as u64, nanos))
}

fn range_check<T>(value: T, min: T, max: T, target: &str) -> ValueResult<T>
where
    T: NumericArgument + Copy,
{
    value
        .require_in_closed_range("value", min, max)
        .map_err(|e| {
            ValueError::ConversionError(format!("Cannot convert value to {}: {}", target, e))
        })
}

fn checked_float_to_i32(value: f64, source: &str) -> ValueResult<i32> {
    if !value.is_finite() {
        return Err(ValueError::ConversionError(format!(
            "Cannot convert non-finite {} value to i32",
            source
        )));
    }
    if value < i32::MIN as f64 || value > i32::MAX as f64 {
        return Err(ValueError::ConversionError(format!(
            "{} value out of i32 range",
            source
        )));
    }
    Ok(value as i32)
}

fn checked_float_to_i64(value: f64, source: &str) -> ValueResult<i64> {
    if !value.is_finite() {
        return Err(ValueError::ConversionError(format!(
            "Cannot convert non-finite {} value to i64",
            source
        )));
    }
    if value < i64::MIN as f64 || value > i64::MAX as f64 {
        return Err(ValueError::ConversionError(format!(
            "{} value out of i64 range",
            source
        )));
    }
    Ok(value as i64)
}

fn checked_bigint_to_f32(value: &BigInt) -> ValueResult<f32> {
    let converted = value.to_f32().ok_or(ValueError::ConversionError(
        "BigInteger value cannot be converted to f32".to_string(),
    ))?;
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(ValueError::ConversionError(
            "BigInteger value out of f32 range".to_string(),
        ))
    }
}

fn checked_bigdecimal_to_f32(value: &BigDecimal) -> ValueResult<f32> {
    let converted = value.to_f32().ok_or(ValueError::ConversionError(
        "BigDecimal value cannot be converted to f32".to_string(),
    ))?;
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(ValueError::ConversionError(
            "BigDecimal value out of f32 range".to_string(),
        ))
    }
}

fn checked_bigint_to_f64(value: &BigInt) -> ValueResult<f64> {
    let converted = value.to_f64().ok_or(ValueError::ConversionError(
        "BigInteger value cannot be converted to f64".to_string(),
    ))?;
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(ValueError::ConversionError(
            "BigInteger value out of f64 range".to_string(),
        ))
    }
}

fn checked_bigdecimal_to_f64(value: &BigDecimal) -> ValueResult<f64> {
    let converted = value.to_f64().ok_or(ValueError::ConversionError(
        "BigDecimal value cannot be converted to f64".to_string(),
    ))?;
    if converted.is_finite() {
        Ok(converted)
    } else {
        Err(ValueError::ConversionError(
            "BigDecimal value out of f64 range".to_string(),
        ))
    }
}

// ============================================================================
// Internal generic conversion traits (private, not exported, to avoid
// polluting the standard type namespace)
// ============================================================================

/// Internal trait: used to extract specific types from Value.
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace.
#[doc(hidden)]
pub trait ValueGetter<T> {
    fn get_value(&self) -> ValueResult<T>;
}

/// Internal trait: used to create Value from types.
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace.
#[doc(hidden)]
pub trait ValueConstructor<T> {
    fn from_type(value: T) -> Self;
}

/// Internal trait: used to set specific types in Value.
///
/// This trait is not exported in mod.rs, only used for internal
/// implementation, to avoid polluting the standard type namespace.
#[doc(hidden)]
pub trait ValueSetter<T> {
    fn set_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to convert Value to target types
///
/// This trait powers `Value::to<T>()`. Each implementation must clearly define
/// which source variants are accepted for the target type.
#[doc(hidden)]
pub trait ValueConverter<T> {
    fn convert(&self) -> ValueResult<T>;
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

macro_rules! impl_strict_value_converter {
    ($(#[$attr:meta])* $type:ty, $get_method:ident) => {
        $(#[$attr])*
        impl ValueConverter<$type> for Value {
            #[inline]
            fn convert(&self) -> ValueResult<$type> {
                self.$get_method()
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

/// Target type `String` supports conversion from:
///
/// - `Value::String`
/// - `Value::Bool`, `Value::Char`
/// - all integer and floating-point variants
/// - `Value::Date`, `Value::Time`, `Value::DateTime`, `Value::Instant`
/// - `Value::BigInteger`, `Value::BigDecimal`
/// - `Value::IntSize`, `Value::UIntSize`
/// - `Value::Duration`, formatted as `<nanoseconds>ns`
/// - `Value::Url`
/// - `Value::StringMap`, serialized as JSON text
/// - `Value::Json`, serialized as JSON text
impl ValueConverter<String> for Value {
    fn convert(&self) -> ValueResult<String> {
        match self {
            Value::String(v) => Ok(v.clone()),
            Value::Bool(v) => Ok(v.to_string()),
            Value::Char(v) => Ok(v.to_string()),
            Value::Int8(v) => Ok(v.to_string()),
            Value::Int16(v) => Ok(v.to_string()),
            Value::Int32(v) => Ok(v.to_string()),
            Value::Int64(v) => Ok(v.to_string()),
            Value::Int128(v) => Ok(v.to_string()),
            Value::UInt8(v) => Ok(v.to_string()),
            Value::UInt16(v) => Ok(v.to_string()),
            Value::UInt32(v) => Ok(v.to_string()),
            Value::UInt64(v) => Ok(v.to_string()),
            Value::UInt128(v) => Ok(v.to_string()),
            Value::Float32(v) => Ok(v.to_string()),
            Value::Float64(v) => Ok(v.to_string()),
            Value::Date(v) => Ok(v.to_string()),
            Value::Time(v) => Ok(v.to_string()),
            Value::DateTime(v) => Ok(v.to_string()),
            Value::Instant(v) => Ok(v.to_rfc3339()),
            Value::BigInteger(v) => Ok(v.to_string()),
            Value::BigDecimal(v) => Ok(v.to_string()),
            Value::IntSize(v) => Ok(v.to_string()),
            Value::UIntSize(v) => Ok(v.to_string()),
            Value::Duration(v) => Ok(format!("{}ns", v.as_nanos())),
            Value::Url(v) => Ok(v.to_string()),
            Value::StringMap(v) => Ok(serde_json::to_string(v)
                .expect("serializing HashMap<String, String> to JSON cannot fail")),
            Value::Json(v) => {
                Ok(serde_json::to_string(v).expect("serializing serde_json::Value cannot fail"))
            }
            Value::Empty(_) => Err(ValueError::NoValue),
        }
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

/// Target type `bool` supports conversion from:
///
/// - `Value::Bool`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::String`, parsed as `bool`
impl ValueConverter<bool> for Value {
    fn convert(&self) -> ValueResult<bool> {
        match self {
            Value::Bool(v) => Ok(*v),
            Value::Int8(v) => Ok(*v != 0),
            Value::Int16(v) => Ok(*v != 0),
            Value::Int32(v) => Ok(*v != 0),
            Value::Int64(v) => Ok(*v != 0),
            Value::Int128(v) => Ok(*v != 0),
            Value::UInt8(v) => Ok(*v != 0),
            Value::UInt16(v) => Ok(*v != 0),
            Value::UInt32(v) => Ok(*v != 0),
            Value::UInt64(v) => Ok(*v != 0),
            Value::UInt128(v) => Ok(*v != 0),
            Value::String(s) => s.parse::<bool>().map_err(|_| {
                ValueError::ConversionError(format!("Cannot convert '{}' to boolean", s))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Bool,
            }),
        }
    }
}

/// Target type `i32` supports conversion from:
///
/// - `Value::Int32`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int64`, `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`, `Value::Float64`
/// - `Value::String`, parsed as `i32`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<i32> for Value {
    fn convert(&self) -> ValueResult<i32> {
        match self {
            Value::Int32(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as i32),
            Value::Int8(v) => Ok(*v as i32),
            Value::Int16(v) => Ok(*v as i32),
            Value::Int64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("i64 value out of i32 range".to_string())),
            Value::Int128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("i128 value out of i32 range".to_string())
            }),
            Value::UInt8(v) => Ok(*v as i32),
            Value::UInt16(v) => Ok(*v as i32),
            Value::UInt32(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u32 value out of i32 range".to_string())),
            Value::UInt64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u64 value out of i32 range".to_string())),
            Value::UInt128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("u128 value out of i32 range".to_string())
            }),
            Value::Float32(v) => checked_float_to_i32(*v as f64, "f32"),
            Value::Float64(v) => checked_float_to_i32(*v, "f64"),
            Value::String(s) => s
                .parse::<i32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to i32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => v.to_i32().ok_or_else(|| {
                ValueError::ConversionError("BigInteger value out of i32 range".to_string())
            }),
            Value::BigDecimal(v) => v.to_i32().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigDecimal value cannot be converted to i32".to_string(),
                )
            }),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Int32,
            }),
        }
    }
}

/// Target type `i64` supports conversion from:
///
/// - `Value::Int64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`, `Value::Float64`
/// - `Value::String`, parsed as `i64`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<i64> for Value {
    fn convert(&self) -> ValueResult<i64> {
        match self {
            Value::Int64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as i64),
            Value::Int8(v) => Ok(*v as i64),
            Value::Int16(v) => Ok(*v as i64),
            Value::Int32(v) => Ok(*v as i64),
            Value::Int128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("i128 value out of i64 range".to_string())
            }),
            Value::UInt8(v) => Ok(*v as i64),
            Value::UInt16(v) => Ok(*v as i64),
            Value::UInt32(v) => Ok(*v as i64),
            Value::UInt64(v) => (*v)
                .try_into()
                .map_err(|_| ValueError::ConversionError("u64 value out of i64 range".to_string())),
            Value::UInt128(v) => (*v).try_into().map_err(|_| {
                ValueError::ConversionError("u128 value out of i64 range".to_string())
            }),
            Value::Float32(v) => checked_float_to_i64(*v as f64, "f32"),
            Value::Float64(v) => checked_float_to_i64(*v, "f64"),
            Value::String(s) => s
                .parse::<i64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to i64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => v.to_i64().ok_or_else(|| {
                ValueError::ConversionError("BigInteger value out of i64 range".to_string())
            }),
            Value::BigDecimal(v) => v.to_i64().ok_or_else(|| {
                ValueError::ConversionError(
                    "BigDecimal value cannot be converted to i64".to_string(),
                )
            }),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Int64,
            }),
        }
    }
}

/// Target type `f64` supports conversion from:
///
/// - `Value::Float64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Float32`
/// - `Value::String`, parsed as `f64`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<f64> for Value {
    fn convert(&self) -> ValueResult<f64> {
        match self {
            Value::Float64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Char(v) => Ok(*v as u32 as f64),
            Value::Float32(v) => Ok(*v as f64),
            Value::Int8(v) => Ok(*v as f64),
            Value::Int16(v) => Ok(*v as f64),
            Value::Int32(v) => Ok(*v as f64),
            Value::Int64(v) => Ok(*v as f64),
            Value::Int128(v) => Ok(*v as f64),
            Value::UInt8(v) => Ok(*v as f64),
            Value::UInt16(v) => Ok(*v as f64),
            Value::UInt32(v) => Ok(*v as f64),
            Value::UInt64(v) => Ok(*v as f64),
            Value::UInt128(v) => Ok(*v as f64),
            Value::String(s) => s
                .parse::<f64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to f64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => checked_bigint_to_f64(v),
            Value::BigDecimal(v) => checked_bigdecimal_to_f64(v),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Float64,
            }),
        }
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

/// Target type `Duration` supports conversion from:
///
/// - `Value::Duration`
/// - `Value::String`, parsed from `<nanoseconds>ns`
impl ValueConverter<Duration> for Value {
    fn convert(&self) -> ValueResult<Duration> {
        match self {
            Value::Duration(v) => Ok(*v),
            Value::String(s) => parse_duration_string(s),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Duration,
            }),
        }
    }
}

/// Target type `Url` supports conversion from:
///
/// - `Value::Url`
/// - `Value::String`, parsed as URL text
impl ValueConverter<Url> for Value {
    fn convert(&self) -> ValueResult<Url> {
        match self {
            Value::Url(v) => Ok(v.clone()),
            Value::String(s) => Url::parse(s).map_err(|e| {
                ValueError::ConversionError(format!("Cannot convert '{}' to Url: {}", s, e))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Url,
            }),
        }
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

/// Target type `serde_json::Value` supports conversion from:
///
/// - `Value::Json`
/// - `Value::String`, parsed as JSON text
/// - `Value::StringMap`, converted to a JSON object
impl ValueConverter<serde_json::Value> for Value {
    fn convert(&self) -> ValueResult<serde_json::Value> {
        match self {
            Value::Json(v) => Ok(v.clone()),
            Value::String(s) => serde_json::from_str(s)
                .map_err(|e| ValueError::JsonDeserializationError(e.to_string())),
            Value::StringMap(v) => Ok(serde_json::to_value(v)
                .expect("serializing HashMap<String, String> to JSON value cannot fail")),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Json,
            }),
        }
    }
}

impl_strict_value_converter!(
    /// Target type `char` supports conversion from:
    ///
    /// - `Value::Char`
    char,
    get_char
);
impl_strict_value_converter!(
    /// Target type `i8` supports conversion from:
    ///
    /// - `Value::Int8`
    i8,
    get_int8
);
impl_strict_value_converter!(
    /// Target type `i16` supports conversion from:
    ///
    /// - `Value::Int16`
    i16,
    get_int16
);
impl_strict_value_converter!(
    /// Target type `i128` supports conversion from:
    ///
    /// - `Value::Int128`
    i128,
    get_int128
);
/// Target type `u8` supports conversion from:
///
/// - `Value::UInt8`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt16`, `Value::UInt32`, `Value::UInt64`, `Value::UInt128`
/// - `Value::String`, parsed as `u8`
impl ValueConverter<u8> for Value {
    fn convert(&self) -> ValueResult<u8> {
        match self {
            Value::UInt8(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => {
                let code = range_check(*v as u32, u8::MIN as u32, u8::MAX as u32, "u8")?;
                Ok(code as u8)
            }
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u8")?;
                Ok(n as u8)
            }
            Value::Int16(v) => {
                let n = range_check(*v, u8::MIN as i16, u8::MAX as i16, "u8")?;
                Ok(n as u8)
            }
            Value::Int32(v) => {
                let n = range_check(*v, u8::MIN as i32, u8::MAX as i32, "u8")?;
                Ok(n as u8)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u8::MIN as i64, u8::MAX as i64, "u8")?;
                Ok(n as u8)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u8::MIN as i128, u8::MAX as i128, "u8")?;
                Ok(n as u8)
            }
            Value::UInt16(v) => {
                let n = range_check(*v, u8::MIN as u16, u8::MAX as u16, "u8")?;
                Ok(n as u8)
            }
            Value::UInt32(v) => {
                let n = range_check(*v, u8::MIN as u32, u8::MAX as u32, "u8")?;
                Ok(n as u8)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u8::MIN as u64, u8::MAX as u64, "u8")?;
                Ok(n as u8)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u8::MIN as u128, u8::MAX as u128, "u8")?;
                Ok(n as u8)
            }
            Value::String(s) => s
                .parse::<u8>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u8", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt8,
            }),
        }
    }
}

/// Target type `u16` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u16`
impl ValueConverter<u16> for Value {
    fn convert(&self) -> ValueResult<u16> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => {
                let code = range_check(*v as u32, u16::MIN as u32, u16::MAX as u32, "u16")?;
                Ok(code as u16)
            }
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u16")?;
                Ok(n as u16)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u16")?;
                Ok(n as u16)
            }
            Value::Int32(v) => {
                let n = range_check(*v, u16::MIN as i32, u16::MAX as i32, "u16")?;
                Ok(n as u16)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u16::MIN as i64, u16::MAX as i64, "u16")?;
                Ok(n as u16)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u16::MIN as i128, u16::MAX as i128, "u16")?;
                Ok(n as u16)
            }
            Value::UInt32(v) => {
                let n = range_check(*v, u16::MIN as u32, u16::MAX as u32, "u16")?;
                Ok(n as u16)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u16::MIN as u64, u16::MAX as u64, "u16")?;
                Ok(n as u16)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u16::MIN as u128, u16::MAX as u128, "u16")?;
                Ok(n as u16)
            }
            Value::String(s) => s
                .parse::<u16>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u16", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt16,
            }),
        }
    }
}

/// Target type `u32` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u32`
impl ValueConverter<u32> for Value {
    fn convert(&self) -> ValueResult<u32> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok(*v as u32),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u32")?;
                Ok(n as u32)
            }
            Value::Int64(v) => {
                let n = range_check(*v, u32::MIN as i64, u32::MAX as i64, "u32")?;
                Ok(n as u32)
            }
            Value::Int128(v) => {
                let n = range_check(*v, u32::MIN as i128, u32::MAX as i128, "u32")?;
                Ok(n as u32)
            }
            Value::UInt64(v) => {
                let n = range_check(*v, u32::MIN as u64, u32::MAX as u64, "u32")?;
                Ok(n as u32)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u32::MIN as u128, u32::MAX as u128, "u32")?;
                Ok(n as u32)
            }
            Value::String(s) => s
                .parse::<u32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt32,
            }),
        }
    }
}

/// Target type `u64` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u64`
impl ValueConverter<u64> for Value {
    fn convert(&self) -> ValueResult<u64> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok((*v).into()),
            Value::UInt64(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok((*v as u32).into()),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int64(v) => {
                let n = range_check(*v, 0i64, i64::MAX, "u64")?;
                Ok(n as u64)
            }
            Value::Int128(v) => {
                let n = range_check(*v, 0i128, u64::MAX as i128, "u64")?;
                Ok(n as u64)
            }
            Value::UInt128(v) => {
                let n = range_check(*v, u64::MIN as u128, u64::MAX as u128, "u64")?;
                Ok(n as u64)
            }
            Value::String(s) => s
                .parse::<u64>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to u64", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt64,
            }),
        }
    }
}

/// Target type `u128` supports conversion from:
///
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::String`, parsed as `u128`
impl ValueConverter<u128> for Value {
    fn convert(&self) -> ValueResult<u128> {
        match self {
            Value::UInt8(v) => Ok((*v).into()),
            Value::UInt16(v) => Ok((*v).into()),
            Value::UInt32(v) => Ok((*v).into()),
            Value::UInt64(v) => Ok((*v).into()),
            Value::UInt128(v) => Ok(*v),
            Value::Bool(v) => Ok(if *v { 1 } else { 0 }),
            Value::Char(v) => Ok((*v as u32).into()),
            Value::Int8(v) => {
                let n = range_check(*v, 0i8, i8::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int16(v) => {
                let n = range_check(*v, 0i16, i16::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int32(v) => {
                let n = range_check(*v, 0i32, i32::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int64(v) => {
                let n = range_check(*v, 0i64, i64::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::Int128(v) => {
                let n = range_check(*v, 0i128, i128::MAX, "u128")?;
                Ok(n as u128)
            }
            Value::String(s) => s.parse::<u128>().map_err(|_| {
                ValueError::ConversionError(format!("Cannot convert '{}' to u128", s))
            }),
            Value::Empty(_) => Err(ValueError::NoValue),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::UInt128,
            }),
        }
    }
}

/// Target type `f32` supports conversion from:
///
/// - `Value::Float32`, `Value::Float64`
/// - `Value::Bool`
/// - `Value::Char`
/// - `Value::Int8`, `Value::Int16`, `Value::Int32`, `Value::Int64`,
///   `Value::Int128`
/// - `Value::UInt8`, `Value::UInt16`, `Value::UInt32`, `Value::UInt64`,
///   `Value::UInt128`
/// - `Value::String`, parsed as `f32`
/// - `Value::BigInteger`, `Value::BigDecimal`
impl ValueConverter<f32> for Value {
    fn convert(&self) -> ValueResult<f32> {
        match self {
            Value::Float32(v) => Ok(*v),
            Value::Float64(v) => {
                if v.is_nan() || v.is_infinite() {
                    Ok(*v as f32)
                } else {
                    let n = range_check(*v, f32::MIN as f64, f32::MAX as f64, "f32")?;
                    Ok(n as f32)
                }
            }
            Value::Bool(v) => Ok(if *v { 1.0 } else { 0.0 }),
            Value::Char(v) => Ok(*v as u32 as f32),
            Value::Int8(v) => Ok(*v as f32),
            Value::Int16(v) => Ok(*v as f32),
            Value::Int32(v) => Ok(*v as f32),
            Value::Int64(v) => Ok(*v as f32),
            Value::Int128(v) => Ok(*v as f32),
            Value::UInt8(v) => Ok(*v as f32),
            Value::UInt16(v) => Ok(*v as f32),
            Value::UInt32(v) => Ok(*v as f32),
            Value::UInt64(v) => Ok(*v as f32),
            Value::UInt128(v) => Ok(*v as f32),
            Value::String(s) => s
                .parse::<f32>()
                .map_err(|_| ValueError::ConversionError(format!("Cannot convert '{}' to f32", s))),
            Value::Empty(_) => Err(ValueError::NoValue),
            Value::BigInteger(v) => checked_bigint_to_f32(v),
            Value::BigDecimal(v) => checked_bigdecimal_to_f32(v),
            _ => Err(ValueError::ConversionFailed {
                from: self.data_type(),
                to: DataType::Float32,
            }),
        }
    }
}
impl_strict_value_converter!(
    /// Target type `NaiveDate` supports conversion from:
    ///
    /// - `Value::Date`
    NaiveDate,
    get_date
);
impl_strict_value_converter!(
    /// Target type `NaiveTime` supports conversion from:
    ///
    /// - `Value::Time`
    NaiveTime,
    get_time
);
impl_strict_value_converter!(
    /// Target type `NaiveDateTime` supports conversion from:
    ///
    /// - `Value::DateTime`
    NaiveDateTime,
    get_datetime
);
impl_strict_value_converter!(
    /// Target type `DateTime<Utc>` supports conversion from:
    ///
    /// - `Value::Instant`
    DateTime<Utc>,
    get_instant
);
impl_strict_value_converter!(
    /// Target type `BigInt` supports conversion from:
    ///
    /// - `Value::BigInteger`
    BigInt,
    get_biginteger
);
impl_strict_value_converter!(
    /// Target type `BigDecimal` supports conversion from:
    ///
    /// - `Value::BigDecimal`
    BigDecimal,
    get_bigdecimal
);
impl_strict_value_converter!(
    /// Target type `isize` supports conversion from:
    ///
    /// - `Value::IntSize`
    isize,
    get_intsize
);
impl_strict_value_converter!(
    /// Target type `usize` supports conversion from:
    ///
    /// - `Value::UIntSize`
    usize,
    get_uintsize
);
impl_strict_value_converter!(
    /// Target type `HashMap<String, String>` supports conversion from:
    ///
    /// - `Value::StringMap`
    HashMap<String, String>,
    get_string_map
);
