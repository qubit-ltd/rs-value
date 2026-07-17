// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Equality and hashing for [`super::MultiValues`].

use std::hash::{
    Hash,
    Hasher,
};

use super::MultiValues;
#[cfg(feature = "big-number")]
use crate::identity::hash_big_decimal;
use crate::identity::{
    canonical_f32_bits,
    canonical_f64_bits,
    hash_string_map,
};
#[cfg(feature = "json")]
use crate::identity::{
    hash_json,
    json_eq,
};

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

/// Implements lawful equality and hashing for the complete value-type table.
macro_rules! impl_multi_values_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?
    ) => {
        impl PartialEq for MultiValues {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::Unset(left), Self::Unset(right)) => left == right,
                    $($(#[$cfg])*
                    (Self::$variant(left), Self::$variant(right)) => {
                        payloads_eq!($variant, left, right)
                    },)+
                    _ => false,
                }
            }
        }

        impl Eq for MultiValues {}

        impl Hash for MultiValues {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::mem::discriminant(self).hash(state);
                match self {
                    Self::Unset(data_type) => data_type.hash(state),
                    $($(#[$cfg])*
                    Self::$variant(values) => {
                        hash_payloads!($variant, values, state)
                    },)+
                }
            }
        }
    };
}

for_each_value_type!(impl_multi_values_identity);
