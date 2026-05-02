/*******************************************************************************
 *
 *    Copyright (c) 2025 - 2026 Haixing Hu.
 *
 *    SPDX-License-Identifier: Apache-2.0
 *
 *    Licensed under the Apache License, Version 2.0.
 *
 ******************************************************************************/

//! Default value adapters for value access APIs.

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

/// Converts ergonomic default arguments into the value type expected by a read
/// API.
pub trait IntoValueDefault<T> {
    /// Converts this argument into the default value.
    fn into_value_default(self) -> T;
}

impl<T> IntoValueDefault<T> for T {
    #[inline]
    fn into_value_default(self) -> T {
        self
    }
}

impl IntoValueDefault<String> for &str {
    #[inline]
    fn into_value_default(self) -> String {
        self.to_string()
    }
}

impl IntoValueDefault<String> for &String {
    #[inline]
    fn into_value_default(self) -> String {
        self.clone()
    }
}

impl<T> IntoValueDefault<Vec<T>> for &[T]
where
    T: Clone,
{
    #[inline]
    fn into_value_default(self) -> Vec<T> {
        self.to_vec()
    }
}

impl<T> IntoValueDefault<Vec<T>> for &Vec<T>
where
    T: Clone,
{
    #[inline]
    fn into_value_default(self) -> Vec<T> {
        self.clone()
    }
}

impl<T, const N: usize> IntoValueDefault<Vec<T>> for [T; N] {
    #[inline]
    fn into_value_default(self) -> Vec<T> {
        Vec::from(self)
    }
}

impl<T, const N: usize> IntoValueDefault<Vec<T>> for &[T; N]
where
    T: Clone,
{
    #[inline]
    fn into_value_default(self) -> Vec<T> {
        self.to_vec()
    }
}

impl IntoValueDefault<Vec<String>> for Vec<&str> {
    #[inline]
    fn into_value_default(self) -> Vec<String> {
        collect_strings(self)
    }
}

impl IntoValueDefault<Vec<String>> for &[&str] {
    #[inline]
    fn into_value_default(self) -> Vec<String> {
        collect_strings(self.iter().copied())
    }
}

impl IntoValueDefault<Vec<String>> for &Vec<&str> {
    #[inline]
    fn into_value_default(self) -> Vec<String> {
        collect_strings(self.iter().copied())
    }
}

impl<const N: usize> IntoValueDefault<Vec<String>> for [&str; N] {
    #[inline]
    fn into_value_default(self) -> Vec<String> {
        collect_strings(self)
    }
}

impl<const N: usize> IntoValueDefault<Vec<String>> for &[&str; N] {
    #[inline]
    fn into_value_default(self) -> Vec<String> {
        collect_strings(self.iter().copied())
    }
}
