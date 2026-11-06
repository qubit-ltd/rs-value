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
                $tag:literal,
                $_wire_preflight:ident
            )
        ),+ $(,)?
    ) => {
        /// Borrowed payload for one homogeneous typed collection.
        ///
        /// Populated variants borrow the runtime vector without allocating or
        /// cloning elements. `Unset` retains only the declared element type and
        /// remains distinct from a populated collection with no elements.
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
                #[doc = concat!("Borrowed `", $tag, "` collection payload.")]
                $(#[$cfg])*
                $(#[$collection_attr])*
                #[serde(rename = $tag)]
                $variant(
                    #[doc = concat!("Stored `", $tag, "` collection values.")]
                    &'a Vec<$type>,
                ),
            )+
        }

        impl<'a> From<&'a MultiValues> for CollectionWireRef<'a> {
            /// Borrows the exact runtime collection variant for V1 serialization.
            ///
            /// # Parameters
            ///
            /// * `values` - Collection whose payload remains borrowed for `'a`.
            ///
            /// # Returns
            ///
            /// A matching wire payload, preserving unset state or borrowing the
            /// populated vector without cloning or transferring its ownership.
            #[inline]
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

for_each_value_type!(define_collection_wire_ref);
