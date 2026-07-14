// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&MultiValues>` implementations for strict typed reads.

use super::multi_values::MultiValues;
use crate::value_error::{ValueError, ValueResult};

macro_rules! impl_multi_values_try_from_table {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                [$($value_attr:meta),*],
                [$($multi_attr:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $ownership:ident,
                $json_class:ident,
                $value_doc:literal,
                $multi_doc:literal
            )
        ),+ $(,)?
    ) => {
        $(
            $(#[$cfg])*
            impl TryFrom<&MultiValues> for $type {
                type Error = ValueError;

                #[inline]
                fn try_from(values: &MultiValues) -> ValueResult<$type> {
                    match values {
                        MultiValues::$variant(values) => {
                            values.first().cloned().ok_or(ValueError::NoValue)
                        }
                        MultiValues::Empty(actual) if *actual == $data_type => {
                            Err(ValueError::NoValue)
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

                #[inline]
                fn try_from(values: &MultiValues) -> ValueResult<Vec<$type>> {
                    match values {
                        MultiValues::$variant(values) => Ok(values.clone()),
                        MultiValues::Empty(actual) if *actual == $data_type => {
                            Err(ValueError::NoValue)
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
