// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed semantic views for [`crate::Value`].

use qubit_datatype::DataType;

/// Generates semantic views from the same closed table as owned storage.
macro_rules! define_view {
    (; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        /// Borrowed semantic view preserving source types without owning payloads.
        ///
        /// The lifetime is that of the source storage; copying this view does
        /// not clone strings, maps, JSON, or other rich payloads.
        #[must_use]
        #[non_exhaustive]
        #[derive(Debug, Clone, Copy)]
        pub enum ValueRef<'a> {
            /// Unset storage retaining its declared type.
            Unset(#[doc = "Declared source type."] DataType),
            $(
                $(#[$cfg])*
                #[doc = $value_doc]
                $variant(#[doc = "Borrowed or copied source payload."] value_view_payload_type!($variant, $number_projection, 'a, $type)),
            )+
        }

    };
}

for_each_value_type!(define_view);
