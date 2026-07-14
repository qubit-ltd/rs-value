// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&Value>` implementations for strict typed reads.

use super::value::Value;
use crate::value_error::{ValueError, ValueResult};

macro_rules! impl_value_try_from_table {
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
            impl TryFrom<&Value> for $type {
                type Error = ValueError;

                #[inline]
                fn try_from(value: &Value) -> ValueResult<$type> {
                    match value {
                        Value::$variant(value) => Ok(value.clone()),
                        Value::Empty(actual) if *actual == $data_type => {
                            Err(ValueError::NoValue)
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
