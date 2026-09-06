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
                type Error = ValueError;

                #[inline(always)]
                fn try_from(values: &MultiValues) -> ValueResult<$type> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => values
                            .first()
                            .map(|value| materialize_stored!($materialization, value))
                            .ok_or(ValueError::Missing(ValueMissing::EmptyCollection {
                                data_type: $data_type,
                            })),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetCollection {
                                data_type: *actual,
                            }))
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
                type Error = ValueError;

                #[inline(always)]
                fn try_from(values: &MultiValues) -> ValueResult<Vec<$type>> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values
                            .iter()
                            .map(|value| materialize_stored!($materialization, value))
                            .collect()),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetCollection {
                                data_type: *actual,
                            }))
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
                type Error = ValueError;

                #[inline(always)]
                fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => values.first().ok_or(
                            ValueError::Missing(ValueMissing::EmptyCollection { data_type: $data_type }),
                        ),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetCollection { data_type: *actual }))
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
                type Error = ValueError;

                #[inline(always)]
                fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
                    match &values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values.as_slice()),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetCollection { data_type: *actual }))
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
                type Error = ValueError;

                #[inline(always)]
                fn try_from(values: MultiValues) -> ValueResult<Self> {
                    match values.repr {
                        MultiValuesRepr::$variant(values) => Ok(values),
                        MultiValuesRepr::Unset(actual) if actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetCollection { data_type: actual }))
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
    type Error = ValueError;

    #[inline(always)]
    fn try_from(values: &'a MultiValues) -> ValueResult<Self> {
        match &values.repr {
            MultiValuesRepr::String(values) => {
                values
                    .first()
                    .map(String::as_str)
                    .ok_or(ValueError::Missing(ValueMissing::EmptyCollection {
                        data_type: DataType::String,
                    }))
            }
            MultiValuesRepr::Unset(actual) if *actual == DataType::String => {
                Err(ValueError::Missing(ValueMissing::UnsetCollection {
                    data_type: *actual,
                }))
            }
            _ => Err(ValueError::TypeMismatch {
                expected: DataType::String,
                actual: values.data_type(),
            }),
        }
    }
}
