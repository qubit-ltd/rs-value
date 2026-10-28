// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `From<T>` implementations for supported `MultiValues` input forms.

use super::multi_values::MultiValues;

/// Collects borrowed string values into owned strings.
///
/// # Type Parameters
///
/// * `I` - Iterator source yielding borrowed UTF-8 strings.
///
/// # Parameters
///
/// * `values` - Borrowed strings to copy into the returned collection.
///
/// # Returns
///
/// Owned strings in the source iteration order.
#[inline]
fn collect_strings<'a, I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    values.into_iter().map(str::to_owned).collect()
}

/// Implements owned collection conversions from the shared value table.
macro_rules! impl_multi_values_from_table {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
                $(, $_wire:tt)*
            )
        ),+ $(,)?
    ) => {
        $(
            $(#[$cfg])*
            impl From<$type> for MultiValues {
                /// Creates a one-item collection by taking ownership of `value`.
                #[inline]
                fn from(value: $type) -> Self {
                    MultiValues::$variant(vec![value])
                }
            }

            $(#[$cfg])*
            impl From<Vec<$type>> for MultiValues {
                /// Takes ownership of the vector and its element storage without cloning elements.
                /// An empty vector creates a concrete empty collection.
                #[inline]
                fn from(values: Vec<$type>) -> Self {
                    MultiValues::$variant(values)
                }
            }

            $(#[$cfg])*
            impl From<&[$type]> for MultiValues {
                /// Clones the elements into an owned collection, preserving order.
                /// An empty slice creates a concrete empty collection.
                #[inline]
                fn from(values: &[$type]) -> Self {
                    MultiValues::$variant(values.to_vec())
                }
            }

            $(#[$cfg])*
            impl From<&Vec<$type>> for MultiValues {
                /// Clones the elements into an owned collection, preserving order.
                /// An empty vector creates a concrete empty collection.
                #[inline]
                fn from(values: &Vec<$type>) -> Self {
                    MultiValues::$variant(values.clone())
                }
            }

            $(#[$cfg])*
            impl<const N: usize> From<[$type; N]> for MultiValues {
                /// Moves the array elements into a newly allocated vector, preserving order.
                /// An empty array creates a concrete empty collection.
                #[inline]
                fn from(values: [$type; N]) -> Self {
                    MultiValues::$variant(Vec::from(values))
                }
            }

            $(#[$cfg])*
            impl<const N: usize> From<&[$type; N]> for MultiValues {
                /// Clones the elements into an owned collection, preserving order.
                /// An empty array creates a concrete empty collection.
                #[inline]
                fn from(values: &[$type; N]) -> Self {
                    MultiValues::$variant(values.to_vec())
                }
            }
        )+
    };
}

for_each_value_type!(impl_multi_values_from_table);

impl From<&str> for MultiValues {
    /// Copies the text into a one-item owned string collection.
    #[inline]
    fn from(value: &str) -> Self {
        MultiValues::String(vec![value.to_string()])
    }
}

impl<'a> From<Vec<&'a str>> for MultiValues {
    /// Copies each string into an owned collection, preserving order.
    /// An empty vector creates a concrete empty collection.
    #[inline]
    fn from(values: Vec<&'a str>) -> Self {
        MultiValues::String(collect_strings(values))
    }
}

impl<'a, 'b> From<&'a [&'b str]> for MultiValues {
    /// Copies each string into an owned collection, preserving order.
    /// An empty slice creates a concrete empty collection.
    #[inline]
    fn from(values: &'a [&'b str]) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}

impl<'a, 'b> From<&'a Vec<&'b str>> for MultiValues {
    /// Copies each string into an owned collection, preserving order.
    /// An empty vector creates a concrete empty collection.
    #[inline]
    fn from(values: &'a Vec<&'b str>) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}

impl<'a, const N: usize> From<[&'a str; N]> for MultiValues {
    /// Copies each string into an owned collection, preserving order.
    /// An empty array creates a concrete empty collection.
    #[inline]
    fn from(values: [&'a str; N]) -> Self {
        MultiValues::String(collect_strings(values))
    }
}

impl<'a, 'b, const N: usize> From<&'a [&'b str; N]> for MultiValues {
    /// Copies each string into an owned collection, preserving order.
    /// An empty array creates a concrete empty collection.
    #[inline]
    fn from(values: &'a [&'b str; N]) -> Self {
        MultiValues::String(collect_strings(values.iter().copied()))
    }
}
