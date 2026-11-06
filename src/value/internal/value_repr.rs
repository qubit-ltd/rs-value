// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private storage representation for `Value`.

use qubit_datatype::DataType;

/// Defines the private storage representation for the public single-value
/// container from the shared value-type table.
macro_rules! define_value_enum {
    (
        ;
        $(
            (
                [$($cfg:meta),*],
                $variant:ident,
                $type:ty,
                $data_type:expr,
                $materialization:ident,
                $json_class:ident,
                $number_projection:ident,
                $value_doc:literal,
                $multi_doc:literal
                $(, $_wire:tt)*
            )
        ),+ $(,)?
    ) => {
        /// Owned storage for one scalar value or an unset declared type.
        ///
        /// Each concrete variant owns its payload from the shared value-type table.
        /// The URL payload is boxed to keep the enum compact; other payloads are
        /// stored directly. Cloning follows each payload's `Clone` implementation.
        /// `Unset` retains a declared type without a concrete payload.
        ///
        /// This crate-private representation has no accessors or conversion methods.
        /// Public callers construct and inspect [`crate::Value`] through its typed
        /// constructors, getters, and [`crate::ValueRef`] views.
        #[derive(Debug, Clone)]
        pub(crate) enum ValueRepr {
            /// Unset value with a declared data type.
            Unset(
                /// Declared data type retained while the value is unset.
                DataType,
            ),
            $(
                #[doc = $value_doc]
                $(#[$cfg])*
                $variant(
                    #[doc = concat!("Stored ", $value_doc, " payload.")]
                    value_storage_type!($variant, $type),
                ),
            )+
        }
    };
}

for_each_value_type!(define_value_enum);
