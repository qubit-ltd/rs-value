use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use url::Url;

use crate::Value;
use crate::error::{ValueError, ValueResult};

use super::multi_values::MultiValues;

// ============================================================================
// Internal generic conversion traits (private, not exported, to avoid polluting
// the standard type namespace).
// ============================================================================

/// Internal trait: used to extract multiple values from MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesGetter<T> {
    fn get_values(&self) -> ValueResult<Vec<T>>;
}

/// Internal trait: used to extract the first value from MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesFirstGetter<T> {
    fn get_first_value(&self) -> ValueResult<T>;
}

/// Internal trait: used to set specific types in MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSetter<T> {
    fn set_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}

/// Internal dispatch trait: dispatches Vec<T>, &[T], T to optimal set path
#[doc(hidden)]
pub trait MultiValuesSetArg<'a> {
    /// Element type
    type Item: 'a + Clone;

    fn apply(self, target: &mut MultiValues) -> ValueResult<()>;
}

/// Internal trait: used to set specific types in MultiValues via slice
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSetterSlice<T> {
    fn set_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

/// Internal trait: used to set a single value in MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesSingleSetter<T> {
    fn set_single_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to add a single value to MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesAdder<T> {
    fn add_value(&mut self, value: T) -> ValueResult<()>;
}

/// Internal trait: used to add multiple values to MultiValues
///
/// This trait is used for internal implementation and cross-crate usage
#[doc(hidden)]
pub trait MultiValuesMultiAdder<T> {
    fn add_values(&mut self, values: Vec<T>) -> ValueResult<()>;
}

/// Internal dispatch trait: dispatches T / Vec<T> / &[T] to optimal add path
#[doc(hidden)]
pub trait MultiValuesAddArg<'a> {
    /// Element type
    type Item: 'a + Clone;

    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()>;
}

/// Internal trait: used to append multiple values to MultiValues via slice
/// (calls add_[xxx]s_slice by type)
#[doc(hidden)]
pub(crate) trait MultiValuesMultiAdderSlice<T> {
    fn add_values_slice(&mut self, values: &[T]) -> ValueResult<()>;
}

/// Internal trait: used to create MultiValues from Vec<T>
///
/// This trait is not exported in mod.rs, only used for internal implementation,
/// to avoid polluting the standard type namespace
#[doc(hidden)]
pub trait MultiValuesConstructor<T> {
    fn from_vec(values: Vec<T>) -> Self;
}

// ============================================================================
// Internal trait implementations (simplified using macros)
// ============================================================================

macro_rules! impl_multi_value_traits {
    ($type:ty, $variant:ident, $data_type:expr) => {
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

        impl MultiValuesSetter<$type> for MultiValues {
            #[inline]
            fn set_values(&mut self, values: Vec<$type>) -> ValueResult<()> {
                *self = MultiValues::$variant(values);
                Ok(())
            }
        }

        // Generic From implementation for SetParam is at the top level, not
        // repeated here for specific types.

        impl MultiValuesSetterSlice<$type> for MultiValues {
            #[inline]
            fn set_values_slice(&mut self, values: &[$type]) -> ValueResult<()> {
                // Equivalent to set_[xxx]s_slice: replace entire list with slice
                *self = MultiValues::$variant(values.to_vec());
                Ok(())
            }
        }

        impl MultiValuesSingleSetter<$type> for MultiValues {
            #[inline]
            fn set_single_value(&mut self, value: $type) -> ValueResult<()> {
                *self = MultiValues::$variant(vec![value]);
                Ok(())
            }
        }

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

        // Three types of implementations for local dispatch trait
        impl<'a> MultiValuesSetArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetter<$type>>::set_values(target, self)
            }
        }

        impl<'a> MultiValuesSetArg<'a> for &'a [$type]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetterSlice<$type>>::set_values_slice(target, self)
            }
        }

        impl<'a> MultiValuesSetArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSingleSetter<$type>>::set_single_value(target, self)
            }
        }

        impl MultiValuesMultiAdder<$type> for MultiValues {
            #[inline]
            fn add_values(&mut self, values: Vec<$type>) -> ValueResult<()> {
                match self {
                    MultiValues::$variant(v) => {
                        v.extend(values);
                        Ok(())
                    }
                    MultiValues::Empty(dt) if *dt == $data_type => {
                        *self = MultiValues::$variant(values);
                        Ok(())
                    }
                    _ => Err(ValueError::TypeMismatch {
                        expected: $data_type,
                        actual: self.data_type(),
                    }),
                }
            }
        }

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

        // add dispatch: T / Vec<T> / &[T]
        impl<'a> MultiValuesAddArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesAdder<$type>>::add_value(target, self)
            }
        }

        impl<'a> MultiValuesAddArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdder<$type>>::add_values(target, self)
            }
        }

        impl<'a> MultiValuesAddArg<'a> for &'a [$type]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdderSlice<$type>>::add_values_slice(target, self)
            }
        }

        impl MultiValuesConstructor<$type> for MultiValues {
            #[inline]
            fn from_vec(values: Vec<$type>) -> Self {
                MultiValues::$variant(values)
            }
        }
    };
}

// Implementation for Copy types
impl_multi_value_traits!(bool, Bool, DataType::Bool);
impl_multi_value_traits!(char, Char, DataType::Char);
impl_multi_value_traits!(i8, Int8, DataType::Int8);
impl_multi_value_traits!(i16, Int16, DataType::Int16);
impl_multi_value_traits!(i32, Int32, DataType::Int32);
impl_multi_value_traits!(i64, Int64, DataType::Int64);
impl_multi_value_traits!(i128, Int128, DataType::Int128);
impl_multi_value_traits!(u8, UInt8, DataType::UInt8);
impl_multi_value_traits!(u16, UInt16, DataType::UInt16);
impl_multi_value_traits!(u32, UInt32, DataType::UInt32);
impl_multi_value_traits!(u64, UInt64, DataType::UInt64);
impl_multi_value_traits!(u128, UInt128, DataType::UInt128);
impl_multi_value_traits!(f32, Float32, DataType::Float32);
impl_multi_value_traits!(f64, Float64, DataType::Float64);
impl_multi_value_traits!(String, String, DataType::String);
impl_multi_value_traits!(NaiveDate, Date, DataType::Date);
impl_multi_value_traits!(NaiveTime, Time, DataType::Time);
impl_multi_value_traits!(NaiveDateTime, DateTime, DataType::DateTime);
impl_multi_value_traits!(DateTime<Utc>, Instant, DataType::Instant);
impl_multi_value_traits!(BigInt, BigInteger, DataType::BigInteger);
impl_multi_value_traits!(BigDecimal, BigDecimal, DataType::BigDecimal);
impl_multi_value_traits!(isize, IntSize, DataType::IntSize);
impl_multi_value_traits!(usize, UIntSize, DataType::UIntSize);
impl_multi_value_traits!(Duration, Duration, DataType::Duration);
impl_multi_value_traits!(Url, Url, DataType::Url);
impl_multi_value_traits!(HashMap<String, String>, StringMap, DataType::StringMap);
impl_multi_value_traits!(serde_json::Value, Json, DataType::Json);

// Convenience adaptation: &str supported as input type for String
impl MultiValuesSetArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesSingleSetter<String>>::set_single_value(target, self.to_string())
    }
}

impl MultiValuesSetArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl<'b> MultiValuesSetArg<'_> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl MultiValuesAddArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesAdder<String>>::add_value(target, self.to_string())
    }
}

impl MultiValuesAddArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl<'b> MultiValuesAddArg<'_> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

// ============================================================================
// Inherent conversion APIs and `Value` interop
// ============================================================================

impl MultiValues {
    /// Convert to a single [`Value`] by taking the first element.
    ///
    /// If there is no element, returns `Value::Empty(self.data_type())`.
    ///
    /// # Returns
    ///
    /// Returns the first element wrapped as [`Value`], or an empty value
    /// preserving the current data type.
    pub fn to_value(&self) -> Value {
        match self {
            MultiValues::Empty(dt) => Value::Empty(*dt),
            MultiValues::Bool(v) => v
                .first()
                .copied()
                .map(Value::Bool)
                .unwrap_or(Value::Empty(DataType::Bool)),
            MultiValues::Char(v) => v
                .first()
                .copied()
                .map(Value::Char)
                .unwrap_or(Value::Empty(DataType::Char)),
            MultiValues::Int8(v) => v
                .first()
                .copied()
                .map(Value::Int8)
                .unwrap_or(Value::Empty(DataType::Int8)),
            MultiValues::Int16(v) => v
                .first()
                .copied()
                .map(Value::Int16)
                .unwrap_or(Value::Empty(DataType::Int16)),
            MultiValues::Int32(v) => v
                .first()
                .copied()
                .map(Value::Int32)
                .unwrap_or(Value::Empty(DataType::Int32)),
            MultiValues::Int64(v) => v
                .first()
                .copied()
                .map(Value::Int64)
                .unwrap_or(Value::Empty(DataType::Int64)),
            MultiValues::Int128(v) => v
                .first()
                .copied()
                .map(Value::Int128)
                .unwrap_or(Value::Empty(DataType::Int128)),
            MultiValues::UInt8(v) => v
                .first()
                .copied()
                .map(Value::UInt8)
                .unwrap_or(Value::Empty(DataType::UInt8)),
            MultiValues::UInt16(v) => v
                .first()
                .copied()
                .map(Value::UInt16)
                .unwrap_or(Value::Empty(DataType::UInt16)),
            MultiValues::UInt32(v) => v
                .first()
                .copied()
                .map(Value::UInt32)
                .unwrap_or(Value::Empty(DataType::UInt32)),
            MultiValues::UInt64(v) => v
                .first()
                .copied()
                .map(Value::UInt64)
                .unwrap_or(Value::Empty(DataType::UInt64)),
            MultiValues::UInt128(v) => v
                .first()
                .copied()
                .map(Value::UInt128)
                .unwrap_or(Value::Empty(DataType::UInt128)),
            MultiValues::IntSize(v) => v
                .first()
                .copied()
                .map(Value::IntSize)
                .unwrap_or(Value::Empty(DataType::IntSize)),
            MultiValues::UIntSize(v) => v
                .first()
                .copied()
                .map(Value::UIntSize)
                .unwrap_or(Value::Empty(DataType::UIntSize)),
            MultiValues::Float32(v) => v
                .first()
                .copied()
                .map(Value::Float32)
                .unwrap_or(Value::Empty(DataType::Float32)),
            MultiValues::Float64(v) => v
                .first()
                .copied()
                .map(Value::Float64)
                .unwrap_or(Value::Empty(DataType::Float64)),
            MultiValues::BigInteger(v) => v
                .first()
                .cloned()
                .map(Value::BigInteger)
                .unwrap_or(Value::Empty(DataType::BigInteger)),
            MultiValues::BigDecimal(v) => v
                .first()
                .cloned()
                .map(Value::BigDecimal)
                .unwrap_or(Value::Empty(DataType::BigDecimal)),
            MultiValues::String(v) => v
                .first()
                .cloned()
                .map(Value::String)
                .unwrap_or(Value::Empty(DataType::String)),
            MultiValues::Date(v) => v
                .first()
                .copied()
                .map(Value::Date)
                .unwrap_or(Value::Empty(DataType::Date)),
            MultiValues::Time(v) => v
                .first()
                .copied()
                .map(Value::Time)
                .unwrap_or(Value::Empty(DataType::Time)),
            MultiValues::DateTime(v) => v
                .first()
                .copied()
                .map(Value::DateTime)
                .unwrap_or(Value::Empty(DataType::DateTime)),
            MultiValues::Instant(v) => v
                .first()
                .copied()
                .map(Value::Instant)
                .unwrap_or(Value::Empty(DataType::Instant)),
            MultiValues::Duration(v) => v
                .first()
                .copied()
                .map(Value::Duration)
                .unwrap_or(Value::Empty(DataType::Duration)),
            MultiValues::Url(v) => v
                .first()
                .cloned()
                .map(Value::Url)
                .unwrap_or(Value::Empty(DataType::Url)),
            MultiValues::StringMap(v) => v
                .first()
                .cloned()
                .map(Value::StringMap)
                .unwrap_or(Value::Empty(DataType::StringMap)),
            MultiValues::Json(v) => v
                .first()
                .cloned()
                .map(Value::Json)
                .unwrap_or(Value::Empty(DataType::Json)),
        }
    }

    /// Merge another multiple values
    ///
    /// Append all values from another multiple values to the current multiple values
    ///
    /// # Parameters
    ///
    /// * `other` - The multiple values to merge
    ///
    /// # Returns
    ///
    /// If types match, returns `Ok(())`; otherwise returns an error
    ///
    /// # Example
    ///
    /// ```rust
    /// use qubit_value::MultiValues;
    ///
    /// let mut a = MultiValues::Int32(vec![1, 2]);
    /// let b = MultiValues::Int32(vec![3, 4]);
    /// a.merge(&b).unwrap();
    /// assert_eq!(a.get_int32s().unwrap(), &[1, 2, 3, 4]);
    /// ```
    pub fn merge(&mut self, other: &MultiValues) -> ValueResult<()> {
        if self.data_type() != other.data_type() {
            return Err(ValueError::TypeMismatch {
                expected: self.data_type(),
                actual: other.data_type(),
            });
        }

        match (self, other) {
            (MultiValues::Bool(v), MultiValues::Bool(o)) => v.extend_from_slice(o),
            (MultiValues::Char(v), MultiValues::Char(o)) => v.extend_from_slice(o),
            (MultiValues::Int8(v), MultiValues::Int8(o)) => v.extend_from_slice(o),
            (MultiValues::Int16(v), MultiValues::Int16(o)) => v.extend_from_slice(o),
            (MultiValues::Int32(v), MultiValues::Int32(o)) => v.extend_from_slice(o),
            (MultiValues::Int64(v), MultiValues::Int64(o)) => v.extend_from_slice(o),
            (MultiValues::Int128(v), MultiValues::Int128(o)) => v.extend_from_slice(o),
            (MultiValues::UInt8(v), MultiValues::UInt8(o)) => v.extend_from_slice(o),
            (MultiValues::UInt16(v), MultiValues::UInt16(o)) => v.extend_from_slice(o),
            (MultiValues::UInt32(v), MultiValues::UInt32(o)) => v.extend_from_slice(o),
            (MultiValues::UInt64(v), MultiValues::UInt64(o)) => v.extend_from_slice(o),
            (MultiValues::UInt128(v), MultiValues::UInt128(o)) => v.extend_from_slice(o),
            (MultiValues::Float32(v), MultiValues::Float32(o)) => v.extend_from_slice(o),
            (MultiValues::Float64(v), MultiValues::Float64(o)) => v.extend_from_slice(o),
            (MultiValues::String(v), MultiValues::String(o)) => v.extend_from_slice(o),
            (MultiValues::Date(v), MultiValues::Date(o)) => v.extend_from_slice(o),
            (MultiValues::Time(v), MultiValues::Time(o)) => v.extend_from_slice(o),
            (MultiValues::DateTime(v), MultiValues::DateTime(o)) => v.extend_from_slice(o),
            (MultiValues::Instant(v), MultiValues::Instant(o)) => v.extend_from_slice(o),
            (MultiValues::BigInteger(v), MultiValues::BigInteger(o)) => v.extend_from_slice(o),
            (MultiValues::BigDecimal(v), MultiValues::BigDecimal(o)) => v.extend_from_slice(o),
            (MultiValues::IntSize(v), MultiValues::IntSize(o)) => v.extend_from_slice(o),
            (MultiValues::UIntSize(v), MultiValues::UIntSize(o)) => v.extend_from_slice(o),
            (MultiValues::Duration(v), MultiValues::Duration(o)) => v.extend_from_slice(o),
            (MultiValues::Url(v), MultiValues::Url(o)) => v.extend_from_slice(o),
            (MultiValues::StringMap(v), MultiValues::StringMap(o)) => v.extend(o.iter().cloned()),
            (MultiValues::Json(v), MultiValues::Json(o)) => v.extend(o.iter().cloned()),
            (slot @ MultiValues::Empty(_), other_values) => *slot = other_values.clone(),
            _ => unreachable!(),
        }

        Ok(())
    }
}

impl Default for MultiValues {
    #[inline]
    fn default() -> Self {
        MultiValues::Empty(DataType::String)
    }
}

impl From<Value> for MultiValues {
    fn from(value: Value) -> Self {
        match value {
            Value::Empty(dt) => MultiValues::Empty(dt),
            Value::Bool(v) => MultiValues::Bool(vec![v]),
            Value::Char(v) => MultiValues::Char(vec![v]),
            Value::Int8(v) => MultiValues::Int8(vec![v]),
            Value::Int16(v) => MultiValues::Int16(vec![v]),
            Value::Int32(v) => MultiValues::Int32(vec![v]),
            Value::Int64(v) => MultiValues::Int64(vec![v]),
            Value::Int128(v) => MultiValues::Int128(vec![v]),
            Value::UInt8(v) => MultiValues::UInt8(vec![v]),
            Value::UInt16(v) => MultiValues::UInt16(vec![v]),
            Value::UInt32(v) => MultiValues::UInt32(vec![v]),
            Value::UInt64(v) => MultiValues::UInt64(vec![v]),
            Value::UInt128(v) => MultiValues::UInt128(vec![v]),
            Value::Float32(v) => MultiValues::Float32(vec![v]),
            Value::Float64(v) => MultiValues::Float64(vec![v]),
            Value::String(v) => MultiValues::String(vec![v]),
            Value::Date(v) => MultiValues::Date(vec![v]),
            Value::Time(v) => MultiValues::Time(vec![v]),
            Value::DateTime(v) => MultiValues::DateTime(vec![v]),
            Value::Instant(v) => MultiValues::Instant(vec![v]),
            Value::BigInteger(v) => MultiValues::BigInteger(vec![v]),
            Value::BigDecimal(v) => MultiValues::BigDecimal(vec![v]),
            Value::IntSize(v) => MultiValues::IntSize(vec![v]),
            Value::UIntSize(v) => MultiValues::UIntSize(vec![v]),
            Value::Duration(v) => MultiValues::Duration(vec![v]),
            Value::Url(v) => MultiValues::Url(vec![v]),
            Value::StringMap(v) => MultiValues::StringMap(vec![v]),
            Value::Json(v) => MultiValues::Json(vec![v]),
        }
    }
}
