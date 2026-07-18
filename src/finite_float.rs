// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! Serde adapters that reject non-finite floating-point values.

/// Stable Serde error message used to identify non-finite values through
/// nested serializers.
pub(crate) const NON_FINITE_FLOAT_MESSAGE: &str =
    "non-finite floating-point value";

use serde::de::Error as _;
use serde::{
    Deserialize,
    Deserializer,
    Serialize,
    Serializer,
};

trait FiniteFloat: Copy {
    fn is_finite(self) -> bool;
}

impl FiniteFloat for f32 {
    fn is_finite(self) -> bool {
        self.is_finite()
    }
}

impl FiniteFloat for f64 {
    fn is_finite(self) -> bool {
        self.is_finite()
    }
}

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

fn serialize_finite_vec<T, S>(
    values: &Vec<T>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    T: FiniteFloat + Serialize,
    S: Serializer,
{
    if values.iter().any(|value| !value.is_finite()) {
        return Err(serde::ser::Error::custom(NON_FINITE_FLOAT_MESSAGE));
    }
    values.serialize(serializer)
}

fn deserialize_finite_vec<'de, T, D>(
    deserializer: D,
) -> Result<Vec<T>, D::Error>
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

pub(crate) mod float32 {
    use serde::{
        Deserializer,
        Serializer,
    };

    pub(crate) fn serialize<S>(
        value: &f32,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite(value, serializer, Serializer::serialize_f32)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f32, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite(deserializer)
    }
}

pub(crate) mod float64 {
    use serde::{
        Deserializer,
        Serializer,
    };

    pub(crate) fn serialize<S>(
        value: &f64,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite(value, serializer, Serializer::serialize_f64)
    }

    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<f64, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite(deserializer)
    }
}

pub(crate) mod float32_vec {
    use serde::{
        Deserializer,
        Serializer,
    };

    pub(crate) fn serialize<S>(
        values: &Vec<f32>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite_vec(values, serializer)
    }

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<f32>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite_vec(deserializer)
    }
}

pub(crate) mod float64_vec {
    use serde::{
        Deserializer,
        Serializer,
    };

    pub(crate) fn serialize<S>(
        values: &Vec<f64>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        super::serialize_finite_vec(values, serializer)
    }

    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<Vec<f64>, D::Error>
    where
        D: Deserializer<'de>,
    {
        super::deserialize_finite_vec(deserializer)
    }
}
