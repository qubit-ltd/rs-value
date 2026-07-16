// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Equality and hashing for [`super::Value`].

use std::collections::HashMap;
use std::hash::{
    Hash,
    Hasher,
};

use super::Value;

/// Returns canonical identity bits for an `f32`.
///
/// # Arguments
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[inline(always)]
fn canonical_f32_bits(value: f32) -> u32 {
    if value == 0.0 {
        0.0_f32.to_bits()
    } else if value.is_nan() {
        f32::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Returns canonical identity bits for an `f64`.
///
/// # Arguments
///
/// * `value` - Floating-point payload to canonicalize.
///
/// # Returns
///
/// Positive-zero bits for either signed zero, one quiet-NaN representation for
/// every NaN, and the original bits for every other value.
#[inline(always)]
fn canonical_f64_bits(value: f64) -> u64 {
    if value == 0.0 {
        0.0_f64.to_bits()
    } else if value.is_nan() {
        f64::NAN.to_bits()
    } else {
        value.to_bits()
    }
}

/// Compares two JSON trees using structural JSON semantics.
///
/// # Arguments
///
/// * `left` - Left JSON tree.
/// * `right` - Right JSON tree.
///
/// # Returns
///
/// `true` when both trees are structurally equal. Object member order is not
/// significant and array element order is significant.
#[cfg(feature = "json")]
#[inline(always)]
fn json_eq(left: &serde_json::Value, right: &serde_json::Value) -> bool {
    left == right
}

/// Hashes a JSON tree using structural, object-order-independent semantics.
///
/// # Arguments
///
/// * `value` - JSON tree to hash.
/// * `state` - Destination hasher.
#[cfg(feature = "json")]
fn hash_json<H: Hasher>(value: &serde_json::Value, state: &mut H) {
    match value {
        serde_json::Value::Null => 0_u8.hash(state),
        serde_json::Value::Bool(value) => {
            1_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Number(value) => {
            2_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::String(value) => {
            3_u8.hash(state);
            value.hash(state);
        }
        serde_json::Value::Array(values) => {
            4_u8.hash(state);
            values.len().hash(state);
            for value in values {
                hash_json(value, state);
            }
        }
        serde_json::Value::Object(values) => {
            5_u8.hash(state);
            values.len().hash(state);
            let mut keys: Vec<_> = values.keys().collect();
            keys.sort_unstable();
            for key in keys {
                key.hash(state);
                hash_json(&values[key], state);
            }
        }
    }
}

/// Hashes a string map in sorted key order.
///
/// # Arguments
///
/// * `value` - Map to hash.
/// * `state` - Destination hasher.
fn hash_string_map<H: Hasher>(value: &HashMap<String, String>, state: &mut H) {
    value.len().hash(state);
    let mut entries: Vec<_> = value.iter().collect();
    entries.sort_unstable_by_key(|(left, _)| *left);
    for (key, value) in entries {
        key.hash(state);
        value.hash(state);
    }
}

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
            #[inline]
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
