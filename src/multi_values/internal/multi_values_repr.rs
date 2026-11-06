// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Private storage representation for `MultiValues`.

use qubit_datatype::DataType;

/// Defines the private storage representation for the public multi-value
/// container from the shared value-type table.
macro_rules! define_multi_values_enum {
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
        /// Owns the homogeneous payload of a multi-value container.
        ///
        /// Each populated variant owns a vector of one element type from the
        /// closed [`DataType`] family. `Unset` retains a declared element type
        /// without a vector, and is distinct from a populated but empty vector.
        ///
        /// This representation is private; downstream code uses
        /// [`crate::MultiValues`] constructors and [`crate::MultiValuesRef`]
        /// semantic views instead of matching storage details. Accessors,
        /// conversions, equality, and hashing belong to the public container.
        ///
        /// Cloning duplicates the vector and clones its elements; moving this
        /// representation transfers ownership of the payload without cloning it.
        #[derive(Debug, Clone)]
        pub(crate) enum MultiValuesRepr {
            /// Unset collection with a declared element data type.
            Unset(
                /// Declared element type retained while the collection is unset.
                DataType,
            ),
            $(
                #[doc = $multi_doc]
                $(#[$cfg])*
                $variant(
                    #[doc = concat!("Stored ", $multi_doc, " payload.")]
                    Vec<$type>,
                ),
            )+
        }
    };
}

for_each_value_type!(define_multi_values_enum);
