/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal dispatch implementations for `MultiValues::set<S>()` arguments.

use super::multi_values::MultiValues;
use crate::value_error::ValueResult;
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

use super::multi_values_setter::MultiValuesSetter;
use super::multi_values_setter_slice::MultiValuesSetterSlice;
use super::multi_values_single_setter::MultiValuesSingleSetter;

/// Private marker trait used to prevent downstream implementations.
trait MultiValuesSetArgSealed {}

/// Internal dispatch trait for `MultiValues::set<S>()`.
///
/// Implementations route `Vec<T>`, `&[T]`, and `T` to the matching set path.
#[allow(private_bounds)]
pub trait MultiValuesSetArg<'a>: MultiValuesSetArgSealed {
    /// Element type being set.
    type Item: 'a + Clone;

    /// Applies this argument to `target`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the target is updated, or a `ValueError` from the
    /// selected set path.
    fn apply(self, target: &mut MultiValues) -> ValueResult<()>;
}

macro_rules! impl_multi_values_set_arg {
    ($type:ty) => {
        impl MultiValuesSetArgSealed for Vec<$type> {}

        impl<'a> MultiValuesSetArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetter<$type>>::set_values(target, self)
            }
        }

        impl<'a> MultiValuesSetArgSealed for &'a [$type] {}

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

        impl<'a> MultiValuesSetArgSealed for &'a Vec<$type> {}

        impl<'a> MultiValuesSetArg<'a> for &'a Vec<$type>
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetterSlice<$type>>::set_values_slice(
                    target,
                    self.as_slice(),
                )
            }
        }

        impl<const N: usize> MultiValuesSetArgSealed for [$type; N] {}

        impl<'a, const N: usize> MultiValuesSetArg<'a> for [$type; N] {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetter<$type>>::set_values(target, Vec::from(self))
            }
        }

        impl<'a, const N: usize> MultiValuesSetArgSealed for &'a [$type; N] {}

        impl<'a, const N: usize> MultiValuesSetArg<'a> for &'a [$type; N]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSetterSlice<$type>>::set_values_slice(
                    target,
                    self.as_slice(),
                )
            }
        }

        impl MultiValuesSetArgSealed for $type {}

        impl<'a> MultiValuesSetArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesSingleSetter<$type>>::set_single_value(target, self)
            }
        }
    };
}

impl_multi_values_set_arg!(bool);
impl_multi_values_set_arg!(char);
impl_multi_values_set_arg!(i8);
impl_multi_values_set_arg!(i16);
impl_multi_values_set_arg!(i32);
impl_multi_values_set_arg!(i64);
impl_multi_values_set_arg!(i128);
impl_multi_values_set_arg!(u8);
impl_multi_values_set_arg!(u16);
impl_multi_values_set_arg!(u32);
impl_multi_values_set_arg!(u64);
impl_multi_values_set_arg!(u128);
impl_multi_values_set_arg!(isize);
impl_multi_values_set_arg!(usize);
impl_multi_values_set_arg!(f32);
impl_multi_values_set_arg!(f64);
impl_multi_values_set_arg!(String);
impl_multi_values_set_arg!(NaiveDate);
impl_multi_values_set_arg!(NaiveTime);
impl_multi_values_set_arg!(NaiveDateTime);
impl_multi_values_set_arg!(DateTime<Utc>);
impl_multi_values_set_arg!(BigInt);
impl_multi_values_set_arg!(BigDecimal);
impl_multi_values_set_arg!(Duration);
impl_multi_values_set_arg!(Url);
impl_multi_values_set_arg!(HashMap<String, String>);
impl_multi_values_set_arg!(serde_json::Value);

impl MultiValuesSetArgSealed for &str {}

impl MultiValuesSetArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesSingleSetter<String>>::set_single_value(target, self.to_string())
    }
}

impl MultiValuesSetArgSealed for Vec<&str> {}

impl MultiValuesSetArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl<'b> MultiValuesSetArgSealed for &'b [&'b str] {}

impl<'b> MultiValuesSetArg<'b> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl MultiValuesSetArgSealed for &Vec<&str> {}

impl MultiValuesSetArg<'_> for &Vec<&str> {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl<const N: usize> MultiValuesSetArgSealed for [&str; N] {}

impl<const N: usize> MultiValuesSetArg<'_> for [&str; N] {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(str::to_string).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}

impl<const N: usize> MultiValuesSetArgSealed for &[&str; N] {}

impl<const N: usize> MultiValuesSetArg<'_> for &[&str; N] {
    type Item = String;

    #[inline]
    fn apply(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesSetter<String>>::set_values(target, owned)
    }
}
