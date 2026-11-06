// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&Value>` implementations for strict typed reads.

use qubit_datatype::DataType;

use super::ValueRepr;
use super::value::Value;
use crate::ValueMissing;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

/// Implements strict borrowed scalar reads from the shared value table.
macro_rules! impl_value_try_from_table {
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
            impl TryFrom<&Value> for $type {
                /// Reports an unset value of the requested type or a runtime type mismatch.
                type Error = ValueError;

                /// Copies or clones the matching payload without changing the container.
                ///
                /// # Parameters
                ///
                /// - `value` - Container borrowed for the duration of this read.
                ///
                /// # Returns
                ///
                /// An owned payload; heap-backed payloads are cloned.
                ///
                /// # Errors
                ///
                /// Returns [`ValueError::Missing`] for an unset value of the requested
                /// type, or [`ValueError::TypeMismatch`] for any other runtime type.
                #[inline(always)]
                fn try_from(value: &Value) -> ValueResult<$type> {
                    match &value.repr {
                        ValueRepr::$variant(value) => {
                            Ok(materialize_value_storage!($variant, $materialization, value))
                        }
                        ValueRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_scalar(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: value.data_type(),
                        }),
                    }
                }
            }
        )+
    };
}

for_each_value_type!(impl_value_try_from_table);

/// Implements zero-copy scalar reads from the shared value table.
macro_rules! impl_value_borrowed_try_from_table {
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
            impl<'a> TryFrom<&'a Value> for &'a $type {
                /// Reports an unset value of the requested type or a runtime type mismatch.
                type Error = ValueError;

                /// Borrows the matching payload without copying or allocating.
                ///
                /// # Parameters
                ///
                /// - `value` - Container that owns the payload and must outlive the returned reference.
                ///
                /// # Returns
                ///
                /// A payload reference tied to the lifetime of `value`.
                ///
                /// # Errors
                ///
                /// Returns [`ValueError::Missing`] for an unset value of the requested
                /// type, or [`ValueError::TypeMismatch`] for any other runtime type.
                #[inline(always)]
                fn try_from(value: &'a Value) -> ValueResult<Self> {
                    match &value.repr {
                        ValueRepr::$variant(value) => Ok(value_storage_ref!($variant, value)),
                        ValueRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_scalar(*actual, *actual)))
                        }
                        _ => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: value.data_type(),
                        }),
                    }
                }
            }

            $(#[$cfg])*
            impl TryFrom<Value> for $type {
                /// Reports an unset value of the requested type or a runtime type mismatch.
                type Error = ValueError;

                /// Consumes the container and moves out its matching payload without cloning.
                ///
                /// # Parameters
                ///
                /// - `value` - Container consumed even when its runtime type does not match.
                ///
                /// # Returns
                ///
                /// The owned payload moved from the container.
                ///
                /// # Errors
                ///
                /// Returns [`ValueError::Missing`] for an unset value of the requested
                /// type, or [`ValueError::TypeMismatch`] for any other runtime type.
                #[inline(always)]
                fn try_from(value: Value) -> ValueResult<Self> {
                    match value.repr {
                        ValueRepr::$variant(value) => Ok(move_value_storage!($variant, value)),
                        ValueRepr::Unset(actual) if actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::unset_scalar(actual, actual)))
                        }
                        other => Err(ValueError::TypeMismatch {
                            expected: $data_type,
                            actual: Value { repr: other }.data_type(),
                        }),
                    }
                }
            }
        )+
    };
}

for_each_value_type!(impl_value_borrowed_try_from_table);

impl<'a> TryFrom<&'a Value> for &'a str {
    /// Reports an unset string value or a runtime type mismatch.
    type Error = ValueError;

    /// Borrows the stored string as a slice without copying or allocating.
    ///
    /// # Parameters
    ///
    /// - `value` - Container that owns the string and must outlive the returned
    ///   slice.
    ///
    /// # Returns
    ///
    /// A string slice tied to the lifetime of `value`.
    ///
    /// # Errors
    ///
    /// Returns [`ValueError::Missing`] for an unset string value, or
    /// [`ValueError::TypeMismatch`] for any other runtime type.
    #[inline(always)]
    fn try_from(value: &'a Value) -> ValueResult<Self> {
        match &value.repr {
            ValueRepr::String(value) => Ok(value.as_str()),
            ValueRepr::Unset(actual) if *actual == DataType::String => {
                Err(ValueError::Missing(ValueMissing::unset_scalar(*actual, *actual)))
            }
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::String,
                actual: value.data_type(),
            }),
        }
    }
}
