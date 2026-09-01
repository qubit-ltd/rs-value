// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! `From<T>` implementations for all supported `Value` input types.

use super::value::Value;

/// Implements owned scalar conversions from the shared value table.
macro_rules! impl_value_from_table {
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
            impl From<$type> for Value {
                #[inline(always)]
                fn from(value: $type) -> Self {
                    Value::$variant(value)
                }
            }
        )+
    };
}

for_each_value_type!(impl_value_from_table);

impl From<&str> for Value {
    #[inline]
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}
