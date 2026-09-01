// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned scalar payload used by V1 deserialization.

use serde::Deserialize;

use super::WireDataTypeV1;
use crate::Value;

/// Defines the owned scalar payload and its exhaustive runtime conversion.
macro_rules! define_scalar_wire_owned {
    (
        $($arg:expr),*;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $_data_type:expr,
                $_materialization:ident,
                $_json_class:ident,
                $_number_projection:ident,
                $_value_doc:literal,
                $_multi_doc:literal,
                [$($scalar_attr:meta),*],
                [$($collection_attr:meta),*],
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
                #[doc = concat!("Owned `", $tag, "` scalar payload.")]
                $(#[$cfg])*
                $(#[$scalar_attr])*
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
                    ScalarWireOwned::Unset(data_type) => Self::new_unset(data_type.into()),
                    $(
                        $(#[$cfg])*
                        ScalarWireOwned::$variant(value) => {
                            Self::$variant(value)
                        },
                    )+
                }
            }
        }
    };
}

for_each_value_type!(define_scalar_wire_owned);
