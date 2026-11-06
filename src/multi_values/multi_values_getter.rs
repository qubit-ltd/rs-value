// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&MultiValues>` implementations for strict typed reads.

use qubit_datatype::DataType;

use super::MultiValuesRepr;
use super::multi_values::MultiValues;
use crate::ValueMissing;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

/// Implements strict borrowed collection reads from the shared value table.
macro_rules! impl_multi_values_try_from_table {
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
            impl TryFrom<&MultiValues> for $type {
                /// Reports a missing value or an exact runtime type mismatch.
                type Error = ValueError;

                /// Copies or clones the first element without converting its runtime type.
                ///
                /// # Parameters
                ///
                /// - `values`: The collection to read without modifying its storage.
                ///
                /// # Returns
                ///
                /// An owned copy or clone of the first element.
                ///
                /// # Errors
                ///
                /// Returns a missing-value error for an empty or matching unset collection,
                /// or a type-mismatch error when the runtime element type differs.
                #[inline(always)]
                fn try_from(values: &MultiValues) -> ValueResult<$type> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => values
                            .first()
                            .map(|value| materialize_stored!($materialization, value))
                            .ok_or(ValueError::Missing(ValueMissing::empty_collection($data_type, $data_type))),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_collection(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: values.data_type(),
                        }),
                    }
                }
            }

            $(#[$cfg])*
            impl TryFrom<&MultiValues> for Vec<$type> {
                /// Reports a missing value or an exact runtime type mismatch.
                type Error = ValueError;

                /// Copies or clones all elements while preserving their order.
                ///
                /// # Parameters
                ///
                /// - `values`: The collection to read without modifying its storage.
                ///
                /// # Returns
                ///
                /// An owned vector, including an empty vector for matching empty storage.
                ///
                /// # Errors
                ///
                /// Returns a missing-value error for a matching unset collection,
                /// or a type-mismatch error when the runtime element type differs.
                #[inline(always)]
                fn try_from(values: &MultiValues) -> ValueResult<Vec<$type>> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values
                            .iter()
                            .map(|value| materialize_stored!($materialization, value))
                            .collect()),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_collection(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: values.data_type(),
                        }),
                    }
                }
            }
        )+
    };
}

for_each_value_type!(impl_multi_values_try_from_table);

/// Implements borrowed and consuming collection reads from the shared table.
macro_rules! impl_multi_values_borrowed_and_owned_table {
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
            impl<'a> TryFrom<&'a MultiValues> for &'a $type {
                /// Reports a missing value or an exact runtime type mismatch.
                type Error = ValueError;

                /// Borrows the first element without allocating or converting its runtime type.
                ///
                /// # Parameters
                ///
                /// - `values`: The collection whose storage is borrowed for the returned lifetime.
                ///
                /// # Returns
                ///
                /// A reference to the first element, valid for the input borrow.
                ///
                /// # Errors
                ///
                /// Returns a missing-value error for an empty or matching unset collection,
                /// or a type-mismatch error when the runtime element type differs.
                #[inline(always)]
                fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => values.first().ok_or(
                            ValueError::Missing(ValueMissing::empty_collection($data_type, $data_type)),
                        ),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_collection(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: values.data_type(),
                        }),
                    }
                }
            }

            $(#[$cfg])*
            impl<'a> TryFrom<&'a MultiValues> for &'a [$type] {
                /// Reports a missing value or an exact runtime type mismatch.
                type Error = ValueError;

                /// Borrows the complete element slice without allocating.
                ///
                /// # Parameters
                ///
                /// - `values`: The collection whose storage is borrowed for the returned lifetime.
                ///
                /// # Returns
                ///
                /// The stored slice, possibly empty, valid for the input borrow.
                ///
                /// # Errors
                ///
                /// Returns a missing-value error for a matching unset collection,
                /// or a type-mismatch error when the runtime element type differs.
                #[inline(always)]
                fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values.as_slice()),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_collection(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: values.data_type(),
                        }),
                    }
                }
            }

            $(#[$cfg])*
            impl TryFrom<MultiValues> for Vec<$type> {
                /// Reports a missing value or an exact runtime type mismatch.
                type Error = ValueError;

                /// Moves the stored vector out of the collection without cloning its elements.
                ///
                /// # Parameters
                ///
                /// - `values`: The collection consumed by this conversion.
                ///
                /// # Returns
                ///
                /// The owned vector in its original order, possibly empty.
                ///
                /// # Errors
                ///
                /// Returns a missing-value error for a matching unset collection,
                /// or a type-mismatch error when the runtime element type differs.
                #[inline(always)]
                fn try_from(values: MultiValues) -> ValueResult<Self> {
                    match values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values),
                        MultiValuesRepr::Unset(actual) if actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_collection(actual, actual)))
                        }
                        other => {
                            let actual = MultiValues { repr: other }.data_type();
                            Err(ValueError::TypeMismatch { expected: $data_type, actual })
                        }
                    }
                }
            }
        )+
    };
}

for_each_value_type!(impl_multi_values_borrowed_and_owned_table);

impl<'a> TryFrom<&'a MultiValues> for &'a str {
    /// Reports a missing string or an exact runtime type mismatch.
    type Error = ValueError;

    /// Borrows the first stored string as a slice without allocating.
    ///
    /// # Parameters
    ///
    /// - `values`: The collection borrowed for the returned slice's lifetime.
    ///
    /// # Returns
    ///
    /// The first string slice, valid for the input borrow.
    ///
    /// # Errors
    ///
    /// Returns a missing-value error for an empty or matching unset collection,
    /// or a type-mismatch error when the runtime element type is not string.
    #[inline(always)]
    fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
        match &values.repr {
            MultiValuesRepr::String(values) => {
                values
                    .first()
                    .map(String::as_str)
                    .ok_or(ValueError::Missing(ValueMissing::empty_collection(
                        DataType::String,
                        DataType::String,
                    )))
            }
            MultiValuesRepr::Unset(actual) if *actual == DataType::String => {
                Err(ValueError::Missing(ValueMissing::unset_collection(*actual, *actual)))
            }
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::String,
                actual: values.data_type(),
            }),
        }
    }
}
