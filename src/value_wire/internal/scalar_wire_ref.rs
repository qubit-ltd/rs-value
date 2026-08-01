// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed scalar payload used by V1 serialization.

use serde::Serialize;

use crate::value::ValueRepr;
use crate::Value;

use super::WireDataTypeV1;

/// Defines the borrowed scalar payload and its exhaustive runtime conversion.
macro_rules! define_scalar_wire_ref {
    (
        $(
            (
                [$($cfg:meta),*],
                [$($scalar_attr:meta),*],
                [$($collection_attr:meta),*],
                $variant:ident,
                $type:ty,
                $tag:literal
            )
        ),+ $(,)?
    ) => {
        /// Borrowed payload for one typed scalar.
        #[derive(Clone, Copy, Serialize)]
        pub(in crate::value_wire) enum ScalarWireRef<'a> {
            /// Unset scalar and its declared data type.
            #[serde(rename = "unset")]
            Unset(
                /// Declared data type of the unset scalar.
                WireDataTypeV1,
            ),
            $(
                $(#[$cfg])*
                $(#[$scalar_attr])*
                #[doc = concat!("Borrowed `", $tag, "` scalar payload.")]
                #[serde(rename = $tag)]
                $variant(
                    #[doc = concat!("Stored `", $tag, "` scalar value.")]
                    &'a $type,
                ),
            )+
        }

        impl<'a> From<&'a Value> for ScalarWireRef<'a> {
            /// Borrows the exact runtime variant for V1 serialization.
            fn from(value: &'a Value) -> Self {
                match &value.repr {
                    ValueRepr::Unset(data_type) => Self::Unset((*data_type).into()),
                    $(
                        $(#[$cfg])*
                        ValueRepr::$variant(value) => {
                            Self::$variant(value_storage_ref!($variant, value))
                        },
                    )+
                }
            }
        }
    };
}

for_each_wire_type!(define_scalar_wire_ref);
