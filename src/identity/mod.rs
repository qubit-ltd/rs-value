// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================
//! Shared equality and hashing for runtime value payloads.

#[cfg(feature = "big-decimal")]
mod big_decimal_hash;
mod float_identity;
#[cfg(feature = "json")]
mod json_identity;
mod string_map_hash;

#[cfg(feature = "big-decimal")]
pub(crate) use big_decimal_hash::hash_big_decimal;
pub(crate) use float_identity::{canonical_f32_bits, canonical_f64_bits};
#[cfg(feature = "json")]
pub(crate) use json_identity::{hash_json, json_eq};
pub(crate) use string_map_hash::hash_string_map;
