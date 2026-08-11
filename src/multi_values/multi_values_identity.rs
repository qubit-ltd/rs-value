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

#[cfg(feature = "json")]
use qubit_budget::BudgetError;
#[cfg(feature = "json")]
use qubit_budget::JsonBudget;

use super::multi_values::MultiValues;
use super::multi_values::MultiValuesRepr;
use crate::identity::canonical_f32_bits;
use crate::identity::canonical_f64_bits;
#[cfg(feature = "big-decimal")]
use crate::identity::hash_big_decimal;
#[cfg(feature = "json")]
use crate::identity::hash_json;
#[cfg(feature = "json")]
use crate::identity::hash_json_with_budget;
use crate::identity::hash_string_map;
#[cfg(feature = "json")]
use crate::identity::json_eq;

/// Compares ordered payloads using the identity rule for their element type.
macro_rules! payloads_eq {
    (Float32, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left.iter().zip($right).all(|(left, right)| {
                canonical_f32_bits(*left) == canonical_f32_bits(*right)
            })
    };
    (Float64, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left.iter().zip($right).all(|(left, right)| {
                canonical_f64_bits(*left) == canonical_f64_bits(*right)
            })
    };
    (Json, $left:expr, $right:expr) => {
        $left.len() == $right.len()
            && $left
                .iter()
                .zip($right)
                .all(|(left, right)| json_eq(left, right))
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

/// Hashes one multi-value payload while applying a budget to JSON elements.
#[cfg(feature = "json")]
pub(crate) fn hash_multi_values_payload_with_json_budget<H, R>(
    repr: &MultiValuesRepr,
    state: &mut H,
    budget: &mut JsonBudget<R, usize>,
) -> Result<(), BudgetError<R, usize>>
where
    H: Hasher,
    R: Clone,
{
    match repr {
        MultiValuesRepr::Unset(data_type) => data_type.hash(state),
        MultiValuesRepr::Bool(values) => hash_payloads!(Bool, values, state),
        MultiValuesRepr::Char(values) => hash_payloads!(Char, values, state),
        MultiValuesRepr::Int8(values) => hash_payloads!(Int8, values, state),
        MultiValuesRepr::Int16(values) => hash_payloads!(Int16, values, state),
        MultiValuesRepr::Int32(values) => hash_payloads!(Int32, values, state),
        MultiValuesRepr::Int64(values) => hash_payloads!(Int64, values, state),
        MultiValuesRepr::Int128(values) => {
            hash_payloads!(Int128, values, state)
        }
        MultiValuesRepr::UInt8(values) => hash_payloads!(UInt8, values, state),
        MultiValuesRepr::UInt16(values) => {
            hash_payloads!(UInt16, values, state)
        }
        MultiValuesRepr::UInt32(values) => {
            hash_payloads!(UInt32, values, state)
        }
        MultiValuesRepr::UInt64(values) => {
            hash_payloads!(UInt64, values, state)
        }
        MultiValuesRepr::UInt128(values) => {
            hash_payloads!(UInt128, values, state)
        }
        MultiValuesRepr::Float32(values) => {
            hash_payloads!(Float32, values, state)
        }
        MultiValuesRepr::Float64(values) => {
            hash_payloads!(Float64, values, state)
        }
        #[cfg(feature = "big-integer")]
        MultiValuesRepr::BigInteger(values) => {
            hash_payloads!(BigInteger, values, state)
        }
        #[cfg(feature = "big-decimal")]
        MultiValuesRepr::BigDecimal(values) => {
            hash_payloads!(BigDecimal, values, state)
        }
        MultiValuesRepr::String(values) => {
            hash_payloads!(String, values, state)
        }
        #[cfg(feature = "chrono")]
        MultiValuesRepr::Date(values) => hash_payloads!(Date, values, state),
        #[cfg(feature = "chrono")]
        MultiValuesRepr::Time(values) => hash_payloads!(Time, values, state),
        #[cfg(feature = "chrono")]
        MultiValuesRepr::DateTime(values) => {
            hash_payloads!(DateTime, values, state)
        }
        #[cfg(feature = "chrono")]
        MultiValuesRepr::Instant(values) => {
            hash_payloads!(Instant, values, state)
        }
        MultiValuesRepr::Duration(values) => {
            hash_payloads!(Duration, values, state)
        }
        #[cfg(feature = "url")]
        MultiValuesRepr::Url(values) => hash_payloads!(Url, values, state),
        MultiValuesRepr::StringMap(values) => {
            hash_payloads!(StringMap, values, state)
        }
        MultiValuesRepr::Json(values) => {
            values.len().hash(state);
            for value in values {
                hash_json_with_budget(value, state, budget)?;
            }
        }
    }
    Ok(())
}

/// Implements lawful equality and hashing for the complete value-type table.
macro_rules! impl_multi_values_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?
    ) => {
        impl PartialEq for MultiValues {
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
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::mem::discriminant(&self.repr).hash(state);
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
