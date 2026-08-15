// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

//! One-pass JSON value serializer with strict finite-float validation.

use serde::Serialize;
use serde_json::Value;

mod internal;

pub(crate) use self::internal::StrictJsonError;
use self::internal::StrictJsonSerializer;

/// Result returned by strict JSON serialization helpers.
type Result<T> = std::result::Result<T, StrictJsonError>;

/// Serializes a value to JSON while rejecting every non-finite float.
///
/// # Parameters
///
/// * `value` - Serializable value to project into strict JSON.
///
/// # Returns
///
/// The projected JSON value.
///
/// # Errors
///
/// Returns [`StrictJsonError::NonFinite`] when any nested float is non-finite,
/// or [`StrictJsonError::Serialization`] for unsupported Serde shapes.
pub(crate) fn to_value<T>(value: &T) -> Result<Value>
where
    T: ?Sized + Serialize,
{
    value.serialize(StrictJsonSerializer)
}
