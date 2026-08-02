// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&MultiValues>` implementations for strict typed reads.

use super::multi_values::{
    MultiValues,
    MultiValuesRepr,
};
use crate::ValueAbsence;
use crate::value_error::{
    ValueError,
    ValueResult,
};

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
                            .ok_or(ValueError::NoValue(ValueAbsence::EmptyCollection {
                                data_type: $data_type,
                            })),
                        MultiValuesRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::NoValue(ValueAbsence::UnsetCollection {
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
                            Err(ValueError::NoValue(ValueAbsence::UnsetCollection {
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
