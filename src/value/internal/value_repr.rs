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
        /// Internal single-value representation.
        ///
        /// Uses an enum to represent different types of values, providing
        /// type-safe value storage and access.
        ///
        /// This representation is private; downstream code uses [`crate::Value`]
        /// constructors and [`crate::ValueRef`] semantic views instead of matching
        /// storage details.
        ///
        /// # Behavior
        ///
        /// - Stores one value from the closed [`DataType`] family.
        /// - Provides strict getters and, with `converter`, option-controlled
        ///   conversion methods.
        /// - Distinguishes an unset container from concrete inner values.
        /// - The URL variant uses boxed storage internally to keep the enum
        ///   compact; use [`crate::Value::new`] and typed getters instead of relying
        ///   on the storage representation of individual variants.
        ///
        /// # Equality and hashing
        ///
        /// Equality preserves enum-variant identity. Signed zero is canonicalized,
        /// every NaN payload within one float width is equal, and unordered payloads
        /// hash structurally. Standard hash output is suitable for in-memory keys but
        /// is not a stable persistent fingerprint.
        ///
        /// # Examples
        ///
        /// ```rust
        /// use qubit_value::Value;
        ///
        /// let value = Value::Int32(42);
        /// assert_eq!(value.get_int32().unwrap(), 42);
        ///
        /// let number: i32 = value.get().unwrap();
        /// assert_eq!(number, 42);
        ///
        /// let text = Value::String("hello".to_string());
        /// assert_eq!(text.get_string().unwrap(), "hello");
        /// ```
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
