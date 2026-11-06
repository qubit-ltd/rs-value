// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Equality and hashing for [`super::Value`].

use std::hash::Hash;
use std::hash::Hasher;
use std::mem::discriminant;

use super::Value;
use super::ValueRepr;
use crate::identity::canonical_f32_bits;
use crate::identity::canonical_f64_bits;
#[cfg(feature = "big-decimal")]
use crate::identity::hash_big_decimal;
#[cfg(feature = "json")]
use crate::identity::hash_json;
use crate::identity::hash_string_map;
#[cfg(feature = "json")]
use crate::identity::json_eq;

/// Compares one pair of same-variant storage payloads by semantic identity.
macro_rules! payload_eq {
    (Float32, $left:expr, $right:expr) => {
        canonical_f32_bits(*$left) == canonical_f32_bits(*$right)
    };
    (Float64, $left:expr, $right:expr) => {
        canonical_f64_bits(*$left) == canonical_f64_bits(*$right)
    };
    (Json, $left:expr, $right:expr) => {
        json_eq($left, $right)
    };
    ($variant:ident, $left:expr, $right:expr) => {
        $left == $right
    };
}

/// Hashes one storage payload using the semantic identity contract.
macro_rules! hash_payload {
    (Float32, $value:expr, $state:expr) => {
        canonical_f32_bits(*$value).hash($state)
    };
    (Float64, $value:expr, $state:expr) => {
        canonical_f64_bits(*$value).hash($state)
    };
    (BigDecimal, $value:expr, $state:expr) => {
        hash_big_decimal($value, $state)
    };
    (StringMap, $value:expr, $state:expr) => {
        hash_string_map($value, $state)
    };
    (Json, $value:expr, $state:expr) => {
        hash_json($value, $state)
    };
    ($variant:ident, $value:expr, $state:expr) => {
        $value.hash($state)
    };
}

/// Keeps JSON transaction handling in the caller while dispatching other types.
#[cfg(feature = "json")]
macro_rules! non_json_hash_payload {
    (Json, $value:expr, $state:expr) => {{
        let _ = $value;
        unreachable!("JSON payload hashing is handled by Value::hash_with_json_budget")
    }};
    ($variant:ident, $value:expr, $state:expr) => {
        hash_payload!($variant, $value, $state)
    };
}

/// Generates the non-JSON payload dispatch from the storage type table.
#[cfg(feature = "json")]
macro_rules! non_json_payload_match {
    ($repr:expr, $state:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $repr {
            ValueRepr::Unset(data_type) => data_type.hash($state),
            $($(#[$cfg])* ValueRepr::$variant(value) => {
                non_json_hash_payload!($variant, value, $state)
            },)+
        }
    };
}

/// Hashes a non-JSON value payload. JSON is handled by
/// [`Value::hash_with_json_budget`](crate::Value::hash_with_json_budget).
///
/// # Type Parameters
///
/// - `H`: Hasher receiving the payload's semantic identity.
///
/// # Parameters
///
/// - `repr`: Borrowed non-JSON storage payload; its variant discriminant is
///   hashed separately by the caller.
/// - `state`: Hasher updated with the payload, or the declared type for an
///   unset value. The payload is neither cloned nor consumed.
///
/// # Panics
///
/// Panics if `repr` contains a JSON payload; callers must handle JSON through
/// the budget-aware path before calling this helper.
#[cfg(feature = "json")]
pub(crate) fn hash_non_json_value_payload<H: Hasher>(repr: &ValueRepr, state: &mut H) {
    for_each_value_type!(non_json_payload_match, repr, state);
}

/// Implements equality and hashing for the private value representation.
macro_rules! impl_value_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?
    ) => {
        impl PartialEq for Value {
            /// Compares variant identity and semantic payload identity.
            ///
            /// # Parameters
            ///
            /// - `other`: Value borrowed for comparison without consuming it.
            ///
            /// # Returns
            ///
            /// Whether both values have the same variant and equal payloads,
            /// with floating-point NaNs and signed zero canonicalized.
            fn eq(&self, other: &Self) -> bool {
                match (&self.repr, &other.repr) {
                    (ValueRepr::Unset(left), ValueRepr::Unset(right)) => left == right,
                    $($(#[$cfg])*
                    (ValueRepr::$variant(left), ValueRepr::$variant(right)) => {
                        payload_eq!($variant, left, right)
                    },)+
                    _ => false,
                }
            }
        }

        impl Eq for Value {}

        impl Hash for Value {
            /// Hashes the variant and payload consistently with equality.
            ///
            /// # Type Parameters
            ///
            /// - `H`: Hasher receiving the semantic identity.
            ///
            /// # Parameters
            ///
            /// - `state`: Hasher updated without cloning or consuming the value.
            fn hash<H: Hasher>(&self, state: &mut H) {
                discriminant(&self.repr).hash(state);
                match &self.repr {
                    ValueRepr::Unset(data_type) => data_type.hash(state),
                    $($(#[$cfg])*
                    ValueRepr::$variant(value) => hash_payload!($variant, value, state),)+
                }
            }
        }
    };
}

for_each_value_type!(impl_value_identity);
