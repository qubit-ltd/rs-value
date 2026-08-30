// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed collection payload used by V1 serialization.

use serde::Serialize;

use super::WireDataTypeV1;
use crate::MultiValues;
use crate::multi_values::MultiValuesRepr;

/// Defines the borrowed collection payload and its runtime conversion.
macro_rules! define_collection_wire_ref {
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
        /// Borrowed payload for one homogeneous typed collection.
        ///
        /// # Type Parameters
        ///
        /// * `'a` - Lifetime of the borrowed collection elements.
        #[derive(Clone, Copy, Serialize)]
        pub(in crate::value_wire) enum CollectionWireRef<'a> {
            /// Unset collection and its declared element data type.
            #[serde(rename = "unset")]
            Unset(
                /// Declared element type of the unset collection.
                WireDataTypeV1,
            ),
            $(
                $(#[$cfg])*
                $(#[$collection_attr])*
                #[doc = concat!("Borrowed `", $tag, "` collection payload.")]
                #[serde(rename = $tag)]
                $variant(
                    #[doc = concat!("Stored `", $tag, "` collection values.")]
                    &'a Vec<$type>,
                ),
            )+
        }

        impl<'a> From<&'a MultiValues> for CollectionWireRef<'a> {
            /// Borrows the exact runtime collection variant for V1 serialization.
            fn from(values: &'a MultiValues) -> Self {
                match &values.repr {
                    MultiValuesRepr::Unset(data_type) => Self::Unset((*data_type).into()),
                    $(
                        $(#[$cfg])*
                        MultiValuesRepr::$variant(values) => Self::$variant(values),
                    )+
                }
            }
        }
    };
}

for_each_wire_type!(define_collection_wire_ref);
