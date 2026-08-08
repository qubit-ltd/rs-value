// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `TryFrom<&Value>` implementations for strict typed reads.

use super::value::Value;
use super::value::ValueRepr;
use crate::ValueMissing;
use crate::value_error::ValueError;
use crate::value_error::ValueResult;

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
            )
        ),+ $(,)?
    ) => {
        $(
            $(#[$cfg])*
            impl TryFrom<&Value> for $type {
                type Error = ValueError;

                #[inline(always)]
                fn try_from(value: &Value) -> ValueResult<$type> {
                    match &value.repr {
                        ValueRepr::$variant(value) => {
                            Ok(materialize_value_storage!($variant, $materialization, value))
                        }
                        ValueRepr::Unset(actual) if *actual == $data_type => {
                            Err(ValueError::Missing(ValueMissing::UnsetScalar {
                                data_type: *actual,
                            }))
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
