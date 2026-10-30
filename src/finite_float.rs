// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde adapters that reject non-finite floating-point values.

use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use serde::Serializer;
use serde::de::Error as _;
use serde::ser::Error as SerializeError;

mod internal;

use self::internal::FiniteFloat;
use self::internal::FiniteFloatSequenceVisitor;

/// Stable Serde error message used to identify non-finite values through
/// nested serializers.
pub(crate) const NON_FINITE_FLOAT_MESSAGE: &str = "non-finite floating-point value";

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
        return Err(SerializeError::custom(NON_FINITE_FLOAT_MESSAGE));
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
        return Err(SerializeError::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    values.serialize(serializer)
}

/// Deserializes a vector, validating each float before requesting the next one.
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
/// Returns `D::Error` when deserialization fails or a decoded element is
/// non-finite. A non-finite element stops further typed element requests;
/// outer document preflight and already charged resources are unaffected.
fn deserialize_finite_vec<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de> + FiniteFloat,
    D: Deserializer<'de>,
{
    deserializer.deserialize_seq(FiniteFloatSequenceVisitor::<T>::new())
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use serde::de::value::Error;
    use serde::de::value::SeqDeserializer;

    use super::NON_FINITE_FLOAT_MESSAGE;
    use super::float32_vec;
    use super::float64_vec;

    #[test]
    fn test_float32_sequence_stops_at_first_non_finite_item() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let reads = Cell::new(0);
            let items = [invalid, 1.0, 2.0].into_iter().inspect(|_| {
                reads.set(reads.get() + 1);
            });
            let error = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(items))
                .expect_err("non-finite f32 must be rejected");
            assert_eq!(error.to_string(), NON_FINITE_FLOAT_MESSAGE);
            assert_eq!(reads.get(), 1, "typed tail must remain unread");
        }
    }

    #[test]
    fn test_float64_sequence_stops_at_first_non_finite_item() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let reads = Cell::new(0);
            let items = [invalid, 1.0, 2.0].into_iter().inspect(|_| {
                reads.set(reads.get() + 1);
            });
            let error = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(items))
                .expect_err("non-finite f64 must be rejected");
            assert_eq!(error.to_string(), NON_FINITE_FLOAT_MESSAGE);
            assert_eq!(reads.get(), 1, "typed tail must remain unread");
        }
    }

    #[test]
    fn test_float32_sequence_keeps_valid_prefix_and_stops_before_tail() {
        for invalid in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            let reads = Cell::new(0);
            let items = [1.0_f32, invalid, 3.0].into_iter().inspect(|_| {
                reads.set(reads.get() + 1);
            });
            let result = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(items));
            assert!(result.is_err());
            assert_eq!(reads.get(), 2, "only prefix and invalid item may be requested");
        }
    }

    #[test]
    fn test_float64_sequence_keeps_valid_prefix_and_stops_before_tail() {
        for invalid in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let reads = Cell::new(0);
            let items = [1.0_f64, invalid, 3.0].into_iter().inspect(|_| {
                reads.set(reads.get() + 1);
            });
            let result = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(items));
            assert!(result.is_err());
            assert_eq!(reads.get(), 2, "only prefix and invalid item may be requested");
        }
    }

    #[test]
    fn test_finite_sequences_preserve_empty_and_boundary_values() {
        let empty32 = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(std::iter::empty::<f32>()))
            .expect("empty f32 collection is valid");
        let empty64 = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(std::iter::empty::<f64>()))
            .expect("empty f64 collection is valid");
        assert!(empty32.is_empty());
        assert!(empty64.is_empty());
        let values32 = [f32::MIN, -0.0, 0.0, f32::from_bits(1), f32::MAX];
        let values64 = [f64::MIN, -0.0, 0.0, f64::from_bits(1), f64::MAX];
        let decoded32 = float32_vec::deserialize(SeqDeserializer::<_, Error>::new(values32.into_iter()))
            .expect("finite f32 values must keep their representation");
        let decoded64 = float64_vec::deserialize(SeqDeserializer::<_, Error>::new(values64.into_iter()))
            .expect("finite f64 values must keep their representation");
        assert_eq!(
            decoded32.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            values32.map(f32::to_bits).to_vec()
        );
        assert_eq!(
            decoded64.iter().map(|value| value.to_bits()).collect::<Vec<_>>(),
            values64.map(f64::to_bits).to_vec()
        );
    }
}

/// Serde adapter for one finite `f32` value.
pub(crate) mod float32 {
    use serde::Deserializer;
    use serde::Serializer;

    /// Serializes one finite `f32` value.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Destination serializer type.
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
    /// # Type Parameters
    ///
    /// * `D` - Source deserializer type.
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
    use serde::Deserializer;
    use serde::Serializer;

    /// Serializes one finite `f64` value.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Destination serializer type.
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
    /// # Type Parameters
    ///
    /// * `D` - Source deserializer type.
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
    use serde::Deserializer;
    use serde::Serializer;

    /// Serializes a slice of finite `f32` values.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Destination serializer type.
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

    /// Deserializes a vector of finite `f32` values, checking each element
    /// before requesting the next typed element.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Source deserializer type.
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
    /// Returns `D::Error` when deserialization fails or an element is
    /// non-finite. Failure stops subsequent typed element requests; outer
    /// document preflight and already charged resources are unaffected.
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
    use serde::Deserializer;
    use serde::Serializer;

    /// Serializes a slice of finite `f64` values.
    ///
    /// # Type Parameters
    ///
    /// * `S` - Destination serializer type.
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

    /// Deserializes a vector of finite `f64` values, checking each element
    /// before requesting the next typed element.
    ///
    /// # Type Parameters
    ///
    /// * `D` - Source deserializer type.
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
    /// Returns `D::Error` when deserialization fails or an element is
    /// non-finite. Failure stops subsequent typed element requests; outer
    /// document preflight and already charged resources are unaffected.
    #[inline(always)]
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite_vec(deserializer)
    }
}
