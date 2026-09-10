// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Frozen data type tags used by unset V1 payloads.

use qubit_datatype::DataType;
use serde::Deserialize;
use serde::Serialize;

/// Internal classification used by Wire V1 preflight measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(in crate::value_wire) enum WirePreflightStrategy {
    /// Frozen type tag text for an unset payload.
    UnsetText,
    /// Boolean node.
    Boolean,
    /// Borrowed text node.
    BorrowedText,
    /// JSON integer node.
    JsonInteger,
    /// Decimal integer encoded as text.
    DecimalText,
    /// JSON floating point node.
    JsonFloat,
    /// Arbitrary precision integer text.
    BigIntegerText,
    /// Arbitrary precision decimal object.
    DecimalObject,
    /// Temporal text node.
    TemporalText,
    /// Duration object.
    DurationObject,
    /// String map object.
    StringMap,
    /// Nested JSON tree.
    JsonTree,
}

macro_rules! wire_preflight_strategy {
    (boolean) => {
        WirePreflightStrategy::Boolean
    };
    (borrowed_text) => {
        WirePreflightStrategy::BorrowedText
    };
    (json_integer) => {
        WirePreflightStrategy::JsonInteger
    };
    (decimal_text) => {
        WirePreflightStrategy::DecimalText
    };
    (json_float) => {
        WirePreflightStrategy::JsonFloat
    };
    (big_integer_text) => {
        WirePreflightStrategy::BigIntegerText
    };
    (decimal_object) => {
        WirePreflightStrategy::DecimalObject
    };
    (temporal_text) => {
        WirePreflightStrategy::TemporalText
    };
    (duration_object) => {
        WirePreflightStrategy::DurationObject
    };
    (string_map) => {
        WirePreflightStrategy::StringMap
    };
    (json_tree) => {
        WirePreflightStrategy::JsonTree
    };
}

/// Defines the complete V1 data type tag set and runtime mappings.
macro_rules! define_wire_data_type_v1 {
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
                $wire_preflight:ident
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

        impl WireDataTypeV1 {
            /// Returns the canonical V1 tag used by unset payloads.
            pub(in crate::value_wire) const fn tag(self) -> &'static str {
                match self {
                    $(Self::$variant => $tag,)+
                }
            }

            /// Returns the table-owned preflight strategy for this tag.
            pub(in crate::value_wire) const fn preflight_strategy(self) -> WirePreflightStrategy {
                match self {
                    $(Self::$variant => wire_preflight_strategy!($wire_preflight),)+
                }
            }
        }
    };
}

for_each_value_type!(define_wire_data_type_v1);
