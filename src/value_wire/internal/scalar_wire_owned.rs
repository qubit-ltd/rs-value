// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned scalar payload used by V1 deserialization.

use serde::Deserialize;

use crate::Value;

use super::WireDataTypeV1;

/// Defines the owned scalar payload and its exhaustive runtime conversion.
macro_rules! define_scalar_wire_owned {
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
        /// Owned payload for one typed scalar.
        #[derive(Deserialize)]
        pub(in crate::value_wire) enum ScalarWireOwned {
            /// Unset scalar and its declared data type.
            #[serde(rename = "unset")]
            Unset(
                /// Declared data type of the unset scalar.
                WireDataTypeV1,
            ),
            $(
                $(#[$cfg])*
                $(#[$scalar_attr])*
                #[doc = concat!("Owned `", $tag, "` scalar payload.")]
                #[serde(rename = $tag)]
                $variant(
                    #[doc = concat!("Stored `", $tag, "` scalar value.")]
                    $type,
                ),
            )+
        }

        impl From<ScalarWireOwned> for Value {
            /// Restores the exact runtime scalar variant.
            fn from(value: ScalarWireOwned) -> Self {
                match value {
                    ScalarWireOwned::Unset(data_type) => Self::Unset(data_type.into()),
                    $(
                        $(#[$cfg])*
                        ScalarWireOwned::$variant(value) => {
                            Self::$variant(value_storage_new!($variant, value))
                        },
                    )+
                }
            }
        }
    };
}

for_each_wire_type!(define_scalar_wire_owned);
