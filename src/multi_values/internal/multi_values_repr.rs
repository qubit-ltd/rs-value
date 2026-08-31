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
        /// Internal multiple-values representation.
        ///
        /// Uses an enum to represent multiple values of different types,
        /// providing type-safe storage and access for multiple values.
        ///
        /// This representation is private; downstream code uses
        /// [`crate::MultiValues`] constructors and [`crate::MultiValuesRef`] semantic views
        /// instead of matching storage details.
        ///
        /// # Behavior
        ///
        /// - Stores a homogeneous collection from the closed [`DataType`]
        ///   family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from a concrete empty vector.
        ///
        /// # Equality and hashing
        ///
        /// Equality preserves the collection variant and element order. Float
        /// elements use canonical signed-zero and NaN identity, while map-like
        /// elements hash structurally. Standard hash output is suitable for in-memory
        /// keys but is not a stable persistent fingerprint.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::MultiValues;
        ///
        /// let mut values = MultiValues::Int32(vec![1, 2, 3]);
        /// assert_eq!(values.len(), 3);
        /// assert_eq!(values.get_first_int32().unwrap(), 1);
        ///
        /// let all = values.get_int32s().unwrap();
        /// assert_eq!(all, &[1, 2, 3]);
        ///
        /// values.add(4).unwrap();
        /// assert_eq!(values.len(), 4);
        /// ```
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
