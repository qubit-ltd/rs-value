// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde adapters that reject non-finite floating-point values.

mod internal;

use internal::FiniteFloat;

/// Stable Serde error message used to identify non-finite values through
/// nested serializers.
pub(crate) const NON_FINITE_FLOAT_MESSAGE: &str = "non-finite floating-point value";

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// Serializes one finite floating-point value with a caller-provided adapter.
///
/// # Type Parameters
///
/// * `T` - Floating-point type being serialized.
/// * `S` - Destination Serde serializer.
///
/// # Parameters
///
/// * `value` - Floating-point value to validate and serialize.
/// * `serializer` - Destination serializer.
/// * `serialize` - Adapter that emits the validated value.
///
/// # Returns
///
/// The result returned by `serialize` for a finite value.
///
/// # Errors
///
/// Returns `S::Error` when `value` is non-finite or the destination serializer
/// rejects it.
#[inline]
fn serialize_finite<T, S>(
    value: &T,
    serializer: S,
    serialize: impl FnOnce(S, T) -> Result<S::Ok, S::Error>,
) -> Result<S::Ok, S::Error>
where
    T: FiniteFloat,
    S: Serializer,
{
    if !value.is_finite() {
        return Err(serde::ser::Error::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    serialize(serializer, *value)
}

/// Deserializes and validates one finite floating-point value.
///
/// # Type Parameters
///
/// * `T` - Floating-point type being deserialized.
/// * `D` - Source Serde deserializer.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// The deserialized finite floating-point value.
///
/// # Errors
///
/// Returns `D::Error` when deserialization fails or the decoded value is
/// non-finite.
#[inline]
fn deserialize_finite<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FiniteFloat,
    D: Deserializer<'de>,
{
    let value = T::deserialize(deserializer)?;
    if !value.is_finite() {
        return Err(D::Error::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    Ok(value)
}

/// Serializes a slice after validating that every float is finite.
///
/// # Type Parameters
///
/// * `T` - Floating-point element type.
/// * `S` - Destination Serde serializer.
///
/// # Parameters
///
/// * `values` - Floating-point values to validate and serialize.
/// * `serializer` - Destination serializer.
///
/// # Returns
///
/// The destination serializer's sequence result.
///
/// # Errors
///
/// Returns `S::Error` when any element is non-finite or the destination
/// serializer rejects the sequence.
fn serialize_finite_vec<T, S>(values: &[T], serializer: S) -> Result<S::Ok, S::Error>
where
    T: FiniteFloat + Serialize,
    S: Serializer,
{
    if values.iter().any(|value| !value.is_finite()) {
        return Err(serde::ser::Error::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    values.serialize(serializer)
}

/// Deserializes a vector and validates that every float is finite.
///
/// # Type Parameters
///
/// * `T` - Floating-point element type.
/// * `D` - Source Serde deserializer.
///
/// # Parameters
///
/// * `deserializer` - Source deserializer.
///
/// # Returns
///
/// The deserialized vector of finite floating-point values.
///
/// # Errors
///
/// Returns `D::Error` when deserialization fails or any decoded element is
/// non-finite.
fn deserialize_finite_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FiniteFloat,
    D: Deserializer<'de>,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    if values.iter().any(|value| !value.is_finite()) {
        return Err(D::Error::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    Ok(values)
}

/// Serde adapter for one finite `f32` value.
pub(crate) mod float32 {
    use serde::{Deserializer, Serializer};

    /// Serializes one finite `f32` value.
    ///
    /// # Parameters
    ///
    /// * `value` - Floating-point value to serialize.
    /// * `serializer` - Destination serializer.
    ///
    /// # Returns
    ///
    /// The destination serializer's result.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` when `value` is non-finite or serialization fails.
    #[inline(always)]
    pub(crate) fn serialize<S>(value: &f32, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite(value, serializer, Serializer::serialize_f32)
    }

    /// Deserializes one finite `f32` value.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source deserializer.
    ///
    /// # Returns
    ///
    /// The decoded finite `f32` value.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` when deserialization fails or the value is
    /// non-finite.
    #[inline(always)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite(deserializer)
    }
}

/// Serde adapter for one finite `f64` value.
pub(crate) mod float64 {
    use serde::{Deserializer, Serializer};

    /// Serializes one finite `f64` value.
    ///
    /// # Parameters
    ///
    /// * `value` - Floating-point value to serialize.
    /// * `serializer` - Destination serializer.
    ///
    /// # Returns
    ///
    /// The destination serializer's result.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` when `value` is non-finite or serialization fails.
    #[inline(always)]
    pub(crate) fn serialize<S>(value: &f64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite(value, serializer, Serializer::serialize_f64)
    }

    /// Deserializes one finite `f64` value.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source deserializer.
    ///
    /// # Returns
    ///
    /// The decoded finite `f64` value.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` when deserialization fails or the value is
    /// non-finite.
    #[inline(always)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite(deserializer)
    }
}

/// Serde adapter for vectors of finite `f32` values.
pub(crate) mod float32_vec {
    use serde::{Deserializer, Serializer};

    /// Serializes a slice of finite `f32` values.
    ///
    /// # Parameters
    ///
    /// * `values` - Floating-point values to serialize.
    /// * `serializer` - Destination serializer.
    ///
    /// # Returns
    ///
    /// The destination serializer's sequence result.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` when any element is non-finite or serialization
    /// fails.
    #[inline(always)]
    pub(crate) fn serialize<S>(values: &[f32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite_vec(values, serializer)
    }

    /// Deserializes a vector of finite `f32` values.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source deserializer.
    ///
    /// # Returns
    ///
    /// The decoded vector of finite `f32` values.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` when deserialization fails or any element is
    /// non-finite.
    #[inline(always)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite_vec(deserializer)
    }
}

/// Serde adapter for vectors of finite `f64` values.
pub(crate) mod float64_vec {
    use serde::{Deserializer, Serializer};

    /// Serializes a slice of finite `f64` values.
    ///
    /// # Parameters
    ///
    /// * `values` - Floating-point values to serialize.
    /// * `serializer` - Destination serializer.
    ///
    /// # Returns
    ///
    /// The destination serializer's sequence result.
    ///
    /// # Errors
    ///
    /// Returns `S::Error` when any element is non-finite or serialization
    /// fails.
    #[inline(always)]
    pub(crate) fn serialize<S>(values: &[f64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite_vec(values, serializer)
    }

    /// Deserializes a vector of finite `f64` values.
    ///
    /// # Parameters
    ///
    /// * `deserializer` - Source deserializer.
    ///
    /// # Returns
    ///
    /// The decoded vector of finite `f64` values.
    ///
    /// # Errors
    ///
    /// Returns `D::Error` when deserialization fails or any element is
    /// non-finite.
    #[inline(always)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite_vec(deserializer)
    }
}
