// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Owned collection payload used by V1 deserialization.

use serde::Deserialize;

use super::WireDataTypeV1;
use crate::MultiValues;

/// Defines the owned collection payload and its runtime conversion.
macro_rules! define_collection_wire_owned {
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
        /// Owned payload for one homogeneous typed collection.
        #[derive(Deserialize)]
        pub(in crate::value_wire) enum CollectionWireOwned {
            /// Unset collection and its declared element data type.
            #[serde(rename = "unset")]
            Unset(
                /// Declared element type of the unset collection.
                WireDataTypeV1,
            ),
            $(
                #[doc = concat!("Owned `", $tag, "` collection payload.")]
                $(#[$cfg])*
                $(#[$collection_attr])*
                #[serde(rename = $tag)]
                $variant(
                    #[doc = concat!("Stored `", $tag, "` collection values.")]
                    Vec<$type>,
                ),
            )+
        }

        impl From<CollectionWireOwned> for MultiValues {
            /// Restores the exact runtime collection variant.
            fn from(values: CollectionWireOwned) -> Self {
                match values {
                    CollectionWireOwned::Unset(data_type) => Self::new_unset(data_type.into()),
                    $(
                        $(#[$cfg])*
                        CollectionWireOwned::$variant(values) => Self::$variant(values),
                    )+
                }
            }
        }
    };
}

for_each_value_type!(define_collection_wire_owned);
