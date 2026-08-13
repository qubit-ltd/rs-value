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
#[cfg(feature = "json")]
use crate::identity::hash_json_with_budget;
use crate::identity::hash_string_map;
#[cfg(feature = "json")]
use crate::identity::json_eq;

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

/// Hashes one value payload while applying a budget to JSON payloads.
#[cfg(feature = "json")]
pub(crate) fn hash_value_payload_with_json_budget<H, R, Q>(
    repr: &ValueRepr,
    state: &mut H,
    budget: &mut JsonValueBudget<R, Q>,
) -> Result<(), MeasuredBudgetError<R, Q>>
where
    H: Hasher,
    R: Clone,
    Q: ResourceQuantity,
{
    match repr {
        ValueRepr::Unset(data_type) => data_type.hash(state),
        ValueRepr::Bool(value) => hash_payload!(Bool, value, state),
        ValueRepr::Char(value) => hash_payload!(Char, value, state),
        ValueRepr::Int8(value) => hash_payload!(Int8, value, state),
        ValueRepr::Int16(value) => hash_payload!(Int16, value, state),
        ValueRepr::Int32(value) => hash_payload!(Int32, value, state),
        ValueRepr::Int64(value) => hash_payload!(Int64, value, state),
        ValueRepr::Int128(value) => hash_payload!(Int128, value, state),
        ValueRepr::UInt8(value) => hash_payload!(UInt8, value, state),
        ValueRepr::UInt16(value) => hash_payload!(UInt16, value, state),
        ValueRepr::UInt32(value) => hash_payload!(UInt32, value, state),
        ValueRepr::UInt64(value) => hash_payload!(UInt64, value, state),
        ValueRepr::UInt128(value) => hash_payload!(UInt128, value, state),
        ValueRepr::Float32(value) => hash_payload!(Float32, value, state),
        ValueRepr::Float64(value) => hash_payload!(Float64, value, state),
        #[cfg(feature = "big-integer")]
        ValueRepr::BigInteger(value) => hash_payload!(BigInteger, value, state),
        #[cfg(feature = "big-decimal")]
        ValueRepr::BigDecimal(value) => hash_payload!(BigDecimal, value, state),
        ValueRepr::String(value) => hash_payload!(String, value, state),
        #[cfg(feature = "chrono")]
        ValueRepr::Date(value) => hash_payload!(Date, value, state),
        #[cfg(feature = "chrono")]
        ValueRepr::Time(value) => hash_payload!(Time, value, state),
        #[cfg(feature = "chrono")]
        ValueRepr::DateTime(value) => hash_payload!(DateTime, value, state),
        #[cfg(feature = "chrono")]
        ValueRepr::Instant(value) => hash_payload!(Instant, value, state),
        ValueRepr::Duration(value) => hash_payload!(Duration, value, state),
        #[cfg(feature = "url")]
        ValueRepr::Url(value) => hash_payload!(Url, value, state),
        ValueRepr::StringMap(value) => hash_payload!(StringMap, value, state),
        ValueRepr::Json(value) => {
            return hash_json_with_budget(value, state, budget);
        }
    }
    Ok(())
}

macro_rules! impl_value_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $number_projection:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?
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
