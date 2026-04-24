use std::collections::HashMap;
use std::time::Duration;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use num_bigint::BigInt;
use qubit_common::lang::DataType;
use url::Url;

use crate::error::{ValueError, ValueResult};

use super::multi_values_core::MultiValues;

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
pub(crate) trait MultiValuesConstructor<T> {
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
