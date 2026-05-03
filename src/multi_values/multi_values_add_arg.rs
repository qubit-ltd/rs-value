/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Internal dispatch implementations for `MultiValues::add<S>()` arguments.

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

use super::multi_values_adder::MultiValuesAdder;
use super::multi_values_multi_adder::MultiValuesMultiAdder;
use super::multi_values_multi_adder_slice::MultiValuesMultiAdderSlice;

/// Private marker trait used to prevent downstream implementations.
trait MultiValuesAddArgSealed {}

/// Internal dispatch trait for `MultiValues::add<S>()`.
///
/// Implementations route `T`, `Vec<T>`, and `&[T]` to the matching add path.
#[allow(private_bounds)]
pub trait MultiValuesAddArg<'a>: MultiValuesAddArgSealed {
    /// Element type being added.
    type Item: 'a + Clone;

    /// Applies this add argument to `target`.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` when the target is updated, or a `ValueError` from the
    /// selected add path.
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()>;
}

macro_rules! impl_multi_values_add_arg {
    ($type:ty) => {
        impl MultiValuesAddArgSealed for $type {}

        impl<'a> MultiValuesAddArg<'a> for $type {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesAdder<$type>>::add_value(target, self)
            }
        }

        impl MultiValuesAddArgSealed for Vec<$type> {}

        impl<'a> MultiValuesAddArg<'a> for Vec<$type> {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdder<$type>>::add_values(target, self)
            }
        }

        impl<'a> MultiValuesAddArgSealed for &'a [$type] {}

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

        impl<'a> MultiValuesAddArgSealed for &'a Vec<$type> {}

        impl<'a> MultiValuesAddArg<'a> for &'a Vec<$type>
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdderSlice<$type>>::add_values_slice(
                    target,
                    self.as_slice(),
                )
            }
        }

        impl<const N: usize> MultiValuesAddArgSealed for [$type; N] {}

        impl<'a, const N: usize> MultiValuesAddArg<'a> for [$type; N] {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdder<$type>>::add_values(target, Vec::from(self))
            }
        }

        impl<'a, const N: usize> MultiValuesAddArgSealed for &'a [$type; N] {}

        impl<'a, const N: usize> MultiValuesAddArg<'a> for &'a [$type; N]
        where
            $type: Clone,
        {
            type Item = $type;

            #[inline]
            fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
                <MultiValues as MultiValuesMultiAdderSlice<$type>>::add_values_slice(
                    target,
                    self.as_slice(),
                )
            }
        }
    };
}

impl_multi_values_add_arg!(bool);
impl_multi_values_add_arg!(char);
impl_multi_values_add_arg!(i8);
impl_multi_values_add_arg!(i16);
impl_multi_values_add_arg!(i32);
impl_multi_values_add_arg!(i64);
impl_multi_values_add_arg!(i128);
impl_multi_values_add_arg!(u8);
impl_multi_values_add_arg!(u16);
impl_multi_values_add_arg!(u32);
impl_multi_values_add_arg!(u64);
impl_multi_values_add_arg!(u128);
impl_multi_values_add_arg!(isize);
impl_multi_values_add_arg!(usize);
impl_multi_values_add_arg!(f32);
impl_multi_values_add_arg!(f64);
impl_multi_values_add_arg!(String);
impl_multi_values_add_arg!(NaiveDate);
impl_multi_values_add_arg!(NaiveTime);
impl_multi_values_add_arg!(NaiveDateTime);
impl_multi_values_add_arg!(DateTime<Utc>);
impl_multi_values_add_arg!(BigInt);
impl_multi_values_add_arg!(BigDecimal);
impl_multi_values_add_arg!(Duration);
impl_multi_values_add_arg!(Url);
impl_multi_values_add_arg!(HashMap<String, String>);
impl_multi_values_add_arg!(serde_json::Value);

impl MultiValuesAddArgSealed for &str {}

impl MultiValuesAddArg<'_> for &str {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        <MultiValues as MultiValuesAdder<String>>::add_value(target, self.to_string())
    }
}

impl MultiValuesAddArgSealed for Vec<&str> {}

impl MultiValuesAddArg<'_> for Vec<&str> {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(|s| s.to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl<'b> MultiValuesAddArgSealed for &'b [&'b str] {}

impl<'b> MultiValuesAddArg<'b> for &'b [&'b str] {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl MultiValuesAddArgSealed for &Vec<&str> {}

impl MultiValuesAddArg<'_> for &Vec<&str> {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl<const N: usize> MultiValuesAddArgSealed for [&str; N] {}

impl<const N: usize> MultiValuesAddArg<'_> for [&str; N] {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.into_iter().map(str::to_string).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}

impl<const N: usize> MultiValuesAddArgSealed for &[&str; N] {}

impl<const N: usize> MultiValuesAddArg<'_> for &[&str; N] {
    type Item = String;

    #[inline]
    fn apply_add(self, target: &mut MultiValues) -> ValueResult<()> {
        let owned: Vec<String> = self.iter().map(|s| (*s).to_string()).collect();
        <MultiValues as MultiValuesMultiAdder<String>>::add_values(target, owned)
    }
}
