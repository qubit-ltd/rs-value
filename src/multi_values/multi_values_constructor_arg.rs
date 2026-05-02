/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal dispatch implementations for `MultiValues::new<S>()` arguments.

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
use super::multi_values_constructor::MultiValuesConstructor;

/// Collects borrowed string values into owned strings.
#[inline]
fn collect_strings<'a, I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let mut result = Vec::new();
    for value in values {
        result.push(value.to_string());
    }
    result
}

/// Internal dispatch trait for `MultiValues::new<S>()` arguments.
#[doc(hidden)]
pub trait MultiValuesConstructorArg<'a> {
    /// Builds a `MultiValues` instance from this argument.
    fn into_multi_values(self) -> MultiValues;
}

macro_rules! impl_multi_values_constructor_arg {
    ($type:ty) => {
        impl<'a> MultiValuesConstructorArg<'a> for Vec<$type> {
            #[inline]
            fn into_multi_values(self) -> MultiValues {
                <MultiValues as MultiValuesConstructor<$type>>::from_vec(self)
            }
        }

        impl<'a> MultiValuesConstructorArg<'a> for &'a [$type]
        where
            $type: Clone,
        {
            #[inline]
            fn into_multi_values(self) -> MultiValues {
                <MultiValues as MultiValuesConstructor<$type>>::from_vec(self.to_vec())
            }
        }

        impl<'a> MultiValuesConstructorArg<'a> for &'a Vec<$type>
        where
            $type: Clone,
        {
            #[inline]
            fn into_multi_values(self) -> MultiValues {
                <MultiValues as MultiValuesConstructor<$type>>::from_vec(self.clone())
            }
        }

        impl<'a, const N: usize> MultiValuesConstructorArg<'a> for [$type; N] {
            #[inline]
            fn into_multi_values(self) -> MultiValues {
                <MultiValues as MultiValuesConstructor<$type>>::from_vec(Vec::from(self))
            }
        }

        impl<'a, const N: usize> MultiValuesConstructorArg<'a> for &'a [$type; N]
        where
            $type: Clone,
        {
            #[inline]
            fn into_multi_values(self) -> MultiValues {
                <MultiValues as MultiValuesConstructor<$type>>::from_vec(self.to_vec())
            }
        }
    };
}

impl_multi_values_constructor_arg!(bool);
impl_multi_values_constructor_arg!(char);
impl_multi_values_constructor_arg!(i8);
impl_multi_values_constructor_arg!(i16);
impl_multi_values_constructor_arg!(i32);
impl_multi_values_constructor_arg!(i64);
impl_multi_values_constructor_arg!(i128);
impl_multi_values_constructor_arg!(u8);
impl_multi_values_constructor_arg!(u16);
impl_multi_values_constructor_arg!(u32);
impl_multi_values_constructor_arg!(u64);
impl_multi_values_constructor_arg!(u128);
impl_multi_values_constructor_arg!(isize);
impl_multi_values_constructor_arg!(usize);
impl_multi_values_constructor_arg!(f32);
impl_multi_values_constructor_arg!(f64);
impl_multi_values_constructor_arg!(String);
impl_multi_values_constructor_arg!(NaiveDate);
impl_multi_values_constructor_arg!(NaiveTime);
impl_multi_values_constructor_arg!(NaiveDateTime);
impl_multi_values_constructor_arg!(DateTime<Utc>);
impl_multi_values_constructor_arg!(BigInt);
impl_multi_values_constructor_arg!(BigDecimal);
impl_multi_values_constructor_arg!(Duration);
impl_multi_values_constructor_arg!(Url);
impl_multi_values_constructor_arg!(HashMap<String, String>);
impl_multi_values_constructor_arg!(serde_json::Value);

impl MultiValuesConstructorArg<'_> for Vec<&str> {
    #[inline]
    fn into_multi_values(self) -> MultiValues {
        MultiValues::String(collect_strings(self))
    }
}

impl MultiValuesConstructorArg<'_> for &[&str] {
    #[inline]
    fn into_multi_values(self) -> MultiValues {
        MultiValues::String(collect_strings(self.iter().copied()))
    }
}

impl MultiValuesConstructorArg<'_> for &Vec<&str> {
    #[inline]
    fn into_multi_values(self) -> MultiValues {
        MultiValues::String(collect_strings(self.iter().copied()))
    }
}

impl<const N: usize> MultiValuesConstructorArg<'_> for [&str; N] {
    #[inline]
    fn into_multi_values(self) -> MultiValues {
        MultiValues::String(collect_strings(self))
    }
}

impl<const N: usize> MultiValuesConstructorArg<'_> for &[&str; N] {
    #[inline]
    fn into_multi_values(self) -> MultiValues {
        MultiValues::String(collect_strings(self.iter().copied()))
    }
}
