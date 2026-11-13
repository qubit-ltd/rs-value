// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Equality and hashing for [`super::MultiValues`].

use std::hash::Hash;
use std::hash::Hasher;
use std::mem::discriminant;

use super::MultiValuesRepr;
use super::multi_values::MultiValues;
use crate::identity::canonical_f32_bits;
use crate::identity::canonical_f64_bits;
#[cfg(feature = "big-decimal")]
use crate::identity::hash_big_decimal;
#[cfg(feature = "json")]
use crate::identity::hash_json;
use crate::identity::hash_string_map;
#[cfg(feature = "json")]
use crate::identity::json_eq;

/// Compares ordered payloads using the identity rule for their element type.
macro_rules! payloads_eq {
    (Float32, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left
                .iter()
                .zip($right)
                .all(|(left, right)| canonical_f32_bits(*left) == canonical_f32_bits(*right))
    };
    (Float64, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left
                .iter()
                .zip($right)
                .all(|(left, right)| canonical_f64_bits(*left) == canonical_f64_bits(*right))
    };
    (Json, $left:expr, $right:expr) => {
        $left.len() == $right.len() && $left.iter().zip($right).all(|(left, right)| json_eq(left, right))
    };
    ($variant:ident, $left:expr, $right:expr) => {
        $left == $right
    };
}

/// Hashes ordered payloads using the identity rule for their element type.
macro_rules! hash_payloads {
    (Float32, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            canonical_f32_bits(*value).hash($state);
        }
    }};
    (Float64, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            canonical_f64_bits(*value).hash($state);
        }
    }};
    (BigDecimal, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_big_decimal(value, $state);
        }
    }};
    (StringMap, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_string_map(value, $state);
        }
    }};
    (Json, $values:expr, $state:expr) => {{
        $values.len().hash($state);
        for value in $values {
            hash_json(value, $state);
        }
    }};
    ($variant:ident, $values:expr, $state:expr) => {
        $values.hash($state)
    };
}

/// Keeps JSON transaction handling in the caller while dispatching other types.
#[cfg(feature = "json")]
macro_rules! non_json_hash_payload {
    (Json, $value:expr, $state:expr) => {{
        let _ = $value;
        unreachable!("JSON payload hashing is handled by MultiValues::hash_with_json_budget")
    }};
    ($variant:ident, $value:expr, $state:expr) => {
        hash_payloads!($variant, $value, $state)
    };
}

/// Generates the non-JSON payload dispatch from the storage type table.
#[cfg(feature = "json")]
macro_rules! non_json_payload_match {
    ($repr:expr, $state:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $repr {
            MultiValuesRepr::Unset(data_type) => data_type.hash($state),
            $($(#[$cfg])* MultiValuesRepr::$variant(value) => {
                non_json_hash_payload!($variant, value, $state)
            },)+
        }
    };
}

/// Hashes a non-JSON collection payload. JSON is handled by
/// [`MultiValues::hash_with_json_budget`](crate::MultiValues::hash_with_json_budget).
///
/// # Type Parameters
///
/// * `H` - Hasher receiving the collection payload's semantic identity.
///
/// # Parameters
///
/// * `repr` - Non-JSON collection storage to hash, excluding its variant tag.
/// * `state` - Hasher updated with the payload's semantic identity.
///
/// # Panics
///
/// Panics if `repr` contains JSON storage; the caller must handle that variant
/// through the budget-aware hashing path instead.
#[cfg(feature = "json")]
pub(crate) fn hash_non_json_multi_values_payload<H: Hasher>(repr: &MultiValuesRepr, state: &mut H) {
    for_each_value_type!(non_json_payload_match, repr, state);
}

/// Implements lawful equality and hashing for the complete value-type table.
macro_rules! impl_multi_values_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?
    ) => {
        impl PartialEq for MultiValues {
            /// Compares variants and ordered payloads using semantic identity.
            ///
            /// # Parameters
            ///
            /// * `other` - Collection to compare with this borrowed collection.
            ///
            /// # Returns
            ///
            /// Whether both collections have the same variant and equal payloads.
            fn eq(&self, other: &Self) -> bool {
                match (&self.repr, &other.repr) {
                    (MultiValuesRepr::Unset(left), MultiValuesRepr::Unset(right)) => left == right,
                    $($(#[$cfg])*
                    (MultiValuesRepr::$variant(left), MultiValuesRepr::$variant(right)) => {
                        payloads_eq!($variant, left, right)
                    },)+
                    _ => false,
                }
            }
        }

        impl Eq for MultiValues {}

        impl Hash for MultiValues {
            /// Writes the variant tag and ordered semantic payload identity.
            ///
            /// # Type Parameters
            ///
            /// * `H` - Hasher receiving the collection identity.
            ///
            /// # Parameters
            ///
            /// * `state` - Hasher updated with the variant and payload identity.
            fn hash<H: Hasher>(&self, state: &mut H) {
                discriminant(&self.repr).hash(state);
                match &self.repr {
                    MultiValuesRepr::Unset(data_type) => data_type.hash(state),
                    $($(#[$cfg])*
                    MultiValuesRepr::$variant(values) => {
                        hash_payloads!($variant, values, state)
                    },)+
                }
            }
        }
    };
}

for_each_value_type!(impl_multi_values_identity);
