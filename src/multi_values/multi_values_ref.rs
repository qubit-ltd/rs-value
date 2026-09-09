// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Borrowed semantic views for [`crate::MultiValues`].

use qubit_datatype::DataType;

use crate::ValueRef;

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
        pub enum MultiValuesRef<'a> {
            /// Unset storage retaining its declared type.
            Unset(#[doc = "Declared source type."] DataType),
            $(
                $(#[$cfg])*
                #[doc = $multi_doc]
                $variant(#[doc = "Borrowed or copied source payload."] &'a [$type]),
            )+
        }

        impl<'a> MultiValuesRef<'a> {
            /// Returns the element type, including for unset or empty storage.
            pub fn data_type(self) -> DataType {
                match self {
                    Self::Unset(data_type) => data_type,
                    $($(#[$cfg])* Self::$variant(_) => $data_type,)+
                }
            }

            /// Returns the number of stored elements; unset storage has length zero.
            pub fn len(self) -> usize {
                match self {
                    Self::Unset(_) => 0,
                    $($(#[$cfg])* Self::$variant(values) => values.len(),)+
                }
            }

            /// Reports whether no concrete elements are available.
            pub fn is_empty(self) -> bool { self.len() == 0 }

            /// Borrows the indexed element, returning None for unset or out-of-range reads.
            ///
            /// Numeric and boolean payloads are copied. Text and rich payloads
            /// retain the original storage lifetime and are never cloned.
            pub fn get(self, index: usize) -> Option<ValueRef<'a>> {
                match self {
                    Self::Unset(_) => None,
                    $($(#[$cfg])* Self::$variant(values) => values.get(index)
                        .map(|value| ValueRef::$variant(value_view_payload!($variant, $number_projection, value))),)+
                }
            }
        }

    };
}

for_each_value_type!(define_view);
