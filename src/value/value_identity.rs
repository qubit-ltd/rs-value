// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Equality and hashing for [`super::Value`].

use std::hash::{
    Hash,
    Hasher,
};

use super::Value;
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

macro_rules! impl_value_identity {
    (
        ;
        $(([$($cfg:meta),*], $variant:ident, $type:ty, $data_type:expr, $materialization:ident, $json_class:ident, $value_doc:literal, $multi_doc:literal)),+ $(,)?
    ) => {
        impl PartialEq for Value {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    (Self::Unset(left), Self::Unset(right)) => left == right,
                    $($(#[$cfg])*
                    (Self::$variant(left), Self::$variant(right)) => {
                        payload_eq!($variant, left, right)
                    },)+
                    _ => false,
                }
            }
        }

        impl Eq for Value {}

        impl Hash for Value {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::mem::discriminant(self).hash(state);
                match self {
                    Self::Unset(data_type) => data_type.hash(state),
                    $($(#[$cfg])*
                    Self::$variant(value) => hash_payload!($variant, value, state),)+
                }
            }
        }
    };
}

for_each_value_type!(impl_value_identity);
