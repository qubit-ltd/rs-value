// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen data type tags used by unset V1 payloads.

use qubit_datatype::DataType;
use serde::{
    Deserialize,
    Serialize,
};

/// Defines the complete V1 data type tag set and runtime mappings.
macro_rules! define_wire_data_type_v1 {
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
        /// Frozen data type tag used by V1 unset scalar and collection payloads.
        ///
        /// Every variant is intentionally independent of crate features because
        /// unset runtime values can declare any supported [`DataType`].
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
        pub(in crate::value_wire) enum WireDataTypeV1 {
            $(
                #[doc = concat!("V1 `", $tag, "` data type tag.")]
                #[serde(rename = $tag)]
                $variant,
            )+
        }

        impl From<DataType> for WireDataTypeV1 {
            /// Maps a runtime data type to its frozen V1 tag.
            fn from(data_type: DataType) -> Self {
                match data_type {
                    $(DataType::$variant => Self::$variant,)+
                }
            }
        }

        impl From<WireDataTypeV1> for DataType {
            /// Restores a runtime data type from its frozen V1 tag.
            fn from(data_type: WireDataTypeV1) -> Self {
                match data_type {
                    $(WireDataTypeV1::$variant => Self::$variant,)+
                }
            }
        }
    };
}

for_each_wire_type!(define_wire_data_type_v1);
