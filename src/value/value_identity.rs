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

#[cfg(feature = "json")]
use qubit_budget::MeasuredBudgetError;
#[cfg(feature = "json")]
use qubit_budget::ResourceQuantity;
#[cfg(feature = "json")]
use qubit_budget::json::JsonValueBudget;

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
macro_rules! budgeted_hash_payload {
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
macro_rules! budgeted_payload_match {
    ($repr:expr, $state:expr; $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?) => {
        match $repr {
            ValueRepr::Unset(data_type) => data_type.hash($state),
            $($(#[$cfg])* ValueRepr::$variant(value) => {
                budgeted_hash_payload!($variant, value, $state)
            },)+
        }
    };
}

/// Hashes one value payload while applying a budget to JSON payloads.
///
/// # Type Parameters
///
/// * `H` - Hasher receiving the semantic payload identity.
/// * `R` - Resource identifier used by the JSON budget.
/// * `Q` - Quantity type used by the JSON budget.
///
/// # Parameters
///
/// * `repr` - Private scalar representation whose payload is hashed.
/// * `state` - Destination hasher.
/// * `_budget` - Budget reserved for JSON payload accounting by the caller.
///
/// # Returns
///
/// `Ok(())` after the payload identity is hashed.
///
/// # Errors
///
/// This helper currently returns no error for non-JSON payloads; JSON payloads
/// are preflighted by
/// [`Value::hash_with_json_budget`](crate::Value::hash_with_json_budget).
#[cfg(feature = "json")]
pub(crate) fn hash_value_payload_with_json_budget<H, R, Q>(
    repr: &ValueRepr,
    state: &mut H,
    _budget: &mut JsonValueBudget<R, Q>,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    H: Hasher,
    R: Clone,
    Q: ResourceQuantity,
{
    for_each_value_type!(budgeted_payload_match, repr, state);
    Ok(())
}

/// Implements equality and hashing for the private value representation.
macro_rules! impl_value_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal $(, $_wire:tt)*)),+ $(,)?
    ) => {
        impl PartialEq for Value {
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
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::mem::discriminant(&self.repr).hash(state);
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
