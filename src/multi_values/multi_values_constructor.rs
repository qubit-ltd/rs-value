/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! `From<T>` implementations for supported `MultiValues` input forms.

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

macro_rules! impl_multi_values_from {
    ($type:ty, $variant:ident) => {
        impl From<$type> for MultiValues {
            #[inline]
            fn from(value: $type) -> Self {
                MultiValues::$variant(vec![value])
            }
        }

        impl From<Vec<$type>> for MultiValues {
            #[inline]
            fn from(values: Vec<$type>) -> Self {
                MultiValues::$variant(values)
            }
        }

        impl From<&[$type]> for MultiValues
        where
            $type: Clone,
        {
            #[inline]
            fn from(values: &[$type]) -> Self {
                MultiValues::$variant(values.to_vec())
            }
        }

        impl From<&Vec<$type>> for MultiValues
        where
            $type: Clone,
        {
            #[inline]
            fn from(values: &Vec<$type>) -> Self {
                MultiValues::$variant(values.clone())
            }
        }

        impl<const N: usize> From<[$type; N]> for MultiValues {
            #[inline]
            fn from(values: [$type; N]) -> Self {
                MultiValues::$variant(Vec::from(values))
            }
        }

        impl<const N: usize> From<&[$type; N]> for MultiValues
        where
            $type: Clone,
        {
            #[inline]
            fn from(values: &[$type; N]) -> Self {
                MultiValues::$variant(values.to_vec())
            }
        }
    };
}

impl_multi_values_from!(bool, Bool);
impl_multi_values_from!(char, Char);
impl_multi_values_from!(i8, Int8);
impl_multi_values_from!(i16, Int16);
impl_multi_values_from!(i32, Int32);
impl_multi_values_from!(i64, Int64);
impl_multi_values_from!(i128, Int128);
impl_multi_values_from!(u8, UInt8);
impl_multi_values_from!(u16, UInt16);
impl_multi_values_from!(u32, UInt32);
impl_multi_values_from!(u64, UInt64);
impl_multi_values_from!(u128, UInt128);
impl_multi_values_from!(isize, IntSize);
impl_multi_values_from!(usize, UIntSize);
impl_multi_values_from!(f32, Float32);
impl_multi_values_from!(f64, Float64);
impl_multi_values_from!(String, String);
impl_multi_values_from!(NaiveDate, Date);
impl_multi_values_from!(NaiveTime, Time);
impl_multi_values_from!(NaiveDateTime, DateTime);
impl_multi_values_from!(DateTime<Utc>, Instant);
impl_multi_values_from!(BigInt, BigInteger);
impl_multi_values_from!(BigDecimal, BigDecimal);
impl_multi_values_from!(Duration, Duration);
impl_multi_values_from!(Url, Url);
impl_multi_values_from!(HashMap<String, String>, StringMap);
impl_multi_values_from!(serde_json::Value, Json);

impl From<&str> for MultiValues {
    #[inline]
    fn from(value: &str) -> Self {
        MultiValues::String(vec![value.to_string()])
    }
}

impl<'a> From<Vec<&'a str>> for MultiValues {
    #[inline]
    fn from(values: Vec<&'a str>) -> Self {
        MultiValues::String(collect_strings(values))
    }
}

impl<'a, 'b> From<&'a [&'b str]> for MultiValues {
    #[inline]
    fn from(values: &'a [&'b str]) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}

impl<'a, 'b> From<&'a Vec<&'b str>> for MultiValues {
    #[inline]
    fn from(values: &'a Vec<&'b str>) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for MultiValues {
    #[inline]
    fn from(values: [&'a str; N]) -> Self {
        MultiValues::String(collect_strings(values))
    }
}

impl<'a, 'b, const N: usize> From<&'a [&'b str; N]> for MultiValues {
    #[inline]
    fn from(values: &'a [&'b str; N]) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}
